//! Cross-connection transfer commands.

use api_contract::error::{AppError, CmdResult};
use connection_core::repository::ConnectionRepository;
use serde::Deserialize;
use ssh_core::host_key::HostKeyStore;
use std::sync::Arc;
use tauri::State;
use tokio::time::{sleep, Duration};

use crate::state::AppState;

const TRANSFER_BUFFER_SIZE: usize = 1024 * 1024;

#[derive(Deserialize)]
pub struct TransferSendRequest {
    pub source_session_id: String,
    pub target_connection_id: i64,
    pub source_path: String,
    pub target_path: String,
}

#[derive(Deserialize)]
pub struct TransferTaskRequest {
    pub task_id: String,
}

async fn wait_if_paused(
    manager: &transfer_core::TransferManager,
    task_id: &str,
    cancel_rx: &tokio::sync::watch::Receiver<bool>,
) -> Result<(), String> {
    while manager.is_paused(task_id) {
        if *cancel_rx.borrow() {
            return Err("cancelled".into());
        }
        sleep(Duration::from_millis(120)).await;
    }
    Ok(())
}

/// 校验目标连接是 SSH 类型，transfer_send 专用。
async fn require_ssh_target(state: &AppState, connection_id: i64) -> Result<(), AppError> {
    let conn = state
        .conn_repo
        .get_connection(connection_id)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::NotFound("connection not found".into()))?;
    if !conn.conn_type.eq_ignore_ascii_case("SSH") {
        return Err(AppError::Validation(
            "only SSH connection can be used as transfer target".into(),
        ));
    }
    Ok(())
}

#[tauri::command]
pub async fn transfer_send(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    req: TransferSendRequest,
) -> CmdResult<String> {
    state.auth.require_auth().await?;

    let source_sftp = state
        .ssh_manager
        .get_sftp(&req.source_session_id)
        .await
        .map_err(AppError::Sftp)?;

    let stat = sftp_core::service::stat(&source_sftp, &req.source_path)
        .await
        .map_err(AppError::Sftp)?;
    if stat.is_dir {
        return Err(AppError::Validation(
            "directory transfer is not supported in send-file mode".into(),
        ));
    }

    let creds = {
        require_ssh_target(&state, req.target_connection_id).await?;
        super::build_creds(&state, req.target_connection_id).await?
    };
    let target_session_id = format!("transfer-target-{}", uuid::Uuid::new_v4());
    let host_key_store: Arc<dyn HostKeyStore> = Arc::new(super::SettingsHostKeyStore {
        repo: state.host_key_repo.clone(),
    });
    state
        .ssh_manager
        .open_sftp(
            target_session_id.clone(),
            creds,
            req.target_connection_id,
            Some(host_key_store),
            Some(app_handle.clone()),
        )
        .await
        .map_err(AppError::Ssh)?;

    let target_sftp = state
        .ssh_manager
        .get_sftp(&target_session_id)
        .await
        .map_err(AppError::Sftp)?;

    let file_name = req
        .source_path
        .rsplit('/')
        .next()
        .unwrap_or("file")
        .to_string();
    let total = stat.size;
    let (task_id, cancel_rx) =
        state
            .transfer_manager
            .create_task(transfer_core::TransferKind::Upload, &file_name, total);

    let tm = state.transfer_manager.clone();
    let tid = task_id.clone();
    let ah = app_handle.clone();
    let ssh_manager = state.ssh_manager.clone();
    let source_path = req.source_path.clone();
    let target_path = req.target_path.clone();

    tokio::spawn(async move {
        use tauri::Emitter;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let _ = ah.emit(
            "transfers/status",
            serde_json::json!({
                "task_id": tid,
                "file_name": file_name,
                "kind": "upload",
                "status": "active",
            }),
        );

        let task_id = tid.clone();
        let result: Result<(), String> = async {
            let mut source = source_sftp
                .open(&source_path)
                .await
                .map_err(|e| format!("open source failed: {e}"))?;
            let mut target = target_sftp
                .create(&target_path)
                .await
                .map_err(|e| format!("create target failed: {e}"))?;

            let mut buf = vec![0u8; TRANSFER_BUFFER_SIZE];
            let mut transferred = 0u64;

            loop {
                if *cancel_rx.borrow() {
                    return Err("cancelled".into());
                }
                wait_if_paused(&tm, &task_id, &cancel_rx).await?;

                let n = source
                    .read(&mut buf)
                    .await
                    .map_err(|e| format!("read source failed: {e}"))?;
                if n == 0 {
                    break;
                }

                target
                    .write_all(&buf[..n])
                    .await
                    .map_err(|e| format!("write target failed: {e}"))?;

                transferred += n as u64;
                tm.update_progress(&task_id, transferred);

                let percent = if total > 0 {
                    (transferred.saturating_mul(100) / total) as u32
                } else {
                    0
                };

                let _ = ah.emit(
                    "transfers/progress",
                    serde_json::json!({
                        "task_id": task_id,
                        "file_name": file_name,
                        "kind": "upload",
                        "bytes_transferred": transferred,
                        "total_bytes": total,
                        "percent": percent,
                    }),
                );
            }

            target
                .shutdown()
                .await
                .map_err(|e| format!("flush target failed: {e}"))?;
            Ok(())
        }
        .await;

        let (status, error_msg) = match result {
            Ok(()) => {
                tm.complete(&tid);
                ("completed", None)
            }
            Err(e) if e == "cancelled" => {
                let _ = tm.cancel(&tid);
                ("cancelled", None)
            }
            Err(e) => {
                tm.fail(&tid, &e);
                ("failed", Some(e))
            }
        };

        let _ = ah.emit(
            "transfers/status",
            serde_json::json!({
                "task_id": tid,
                "file_name": file_name,
                "kind": "upload",
                "status": status,
                "error": error_msg,
            }),
        );

        let _ = ssh_manager.close_sftp(&target_session_id).await;
    });

    Ok(task_id)
}

#[tauri::command]
pub async fn transfer_list(
    state: State<'_, AppState>,
) -> CmdResult<Vec<transfer_core::TransferTask>> {
    state.auth.require_auth().await?;
    Ok(state.transfer_manager.list_tasks())
}

#[tauri::command]
pub async fn transfer_get(
    state: State<'_, AppState>,
    req: TransferTaskRequest,
) -> CmdResult<Option<transfer_core::TransferTask>> {
    state.auth.require_auth().await?;
    Ok(state.transfer_manager.get_task(&req.task_id))
}

#[tauri::command]
pub async fn transfer_cancel(
    state: State<'_, AppState>,
    req: TransferTaskRequest,
) -> CmdResult<()> {
    state.auth.require_auth().await?;
    state
        .transfer_manager
        .cancel(&req.task_id)
        .map_err(|_| AppError::NotFound(format!("transfer task '{}' not found", req.task_id)))
}

#[tauri::command]
pub async fn transfer_pause(state: State<'_, AppState>, req: TransferTaskRequest) -> CmdResult<()> {
    state.auth.require_auth().await?;
    state
        .transfer_manager
        .pause(&req.task_id)
        .map_err(|_| AppError::NotFound(format!("transfer task '{}' not found", req.task_id)))
}

#[tauri::command]
pub async fn transfer_resume(
    state: State<'_, AppState>,
    req: TransferTaskRequest,
) -> CmdResult<()> {
    state.auth.require_auth().await?;
    state
        .transfer_manager
        .resume(&req.task_id)
        .map_err(|_| AppError::NotFound(format!("transfer task '{}' not found", req.task_id)))
}

#[tauri::command]
pub async fn transfer_pause_all(state: State<'_, AppState>) -> CmdResult<()> {
    state.auth.require_auth().await?;
    state.transfer_manager.pause_all();
    Ok(())
}

#[tauri::command]
pub async fn transfer_resume_all(state: State<'_, AppState>) -> CmdResult<()> {
    state.auth.require_auth().await?;
    state.transfer_manager.resume_all();
    Ok(())
}

#[tauri::command]
pub async fn transfer_cancel_all(state: State<'_, AppState>) -> CmdResult<()> {
    state.auth.require_auth().await?;
    state.transfer_manager.cancel_all();
    Ok(())
}

#[tauri::command]
pub async fn transfer_cleanup_completed(state: State<'_, AppState>) -> CmdResult<()> {
    state.auth.require_auth().await?;
    state.transfer_manager.cleanup();
    Ok(())
}
