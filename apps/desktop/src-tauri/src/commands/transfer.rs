//! Cross-connection transfer commands.

use api_contract::error::AppError;
use connection_core::repository::ConnectionRepository;
use serde::Deserialize;
use tauri::State;

use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

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

async fn build_creds(
    state: &AppState,
    connection_id: i64,
) -> Result<ssh_core::session::SshCredentials, AppError> {
    let conn = state
        .conn_repo
        .get_connection(connection_id)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("connection not found".into()))?;

    if !conn.conn_type.eq_ignore_ascii_case("SSH") {
        return Err(AppError::Validation(
            "only SSH connection can be used as transfer target".into(),
        ));
    }

    let auth = if conn.auth_method == "key" {
        let key_id = conn
            .ssh_key_id
            .ok_or(AppError::Validation("no SSH key configured".into()))?;
        let key = state
            .conn_repo
            .get_ssh_key(key_id)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::NotFound("SSH key not found".into()))?;
        let private_key = state
            .crypto
            .decrypt(&key.encrypted_private_key)
            .map_err(|e| AppError::Crypto(e.to_string()))?;
        let passphrase = key
            .encrypted_passphrase
            .as_deref()
            .map(|p| state.crypto.decrypt(p))
            .transpose()
            .map_err(|e| AppError::Crypto(e.to_string()))?;
        ssh_core::session::SshAuth::Key {
            private_key_pem: private_key,
            passphrase,
        }
    } else {
        let password = conn
            .encrypted_password
            .as_deref()
            .ok_or(AppError::Validation("no password configured".into()))?;
        let decrypted = state
            .crypto
            .decrypt(password)
            .map_err(|e| AppError::Crypto(e.to_string()))?;
        ssh_core::session::SshAuth::Password(decrypted)
    };

    Ok(ssh_core::session::SshCredentials {
        host: conn.host,
        port: conn.port as u16,
        username: conn.username,
        auth,
    })
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
        .map_err(AppError::Ssh)?;

    let stat = sftp_core::service::stat(&source_sftp, &req.source_path)
        .await
        .map_err(AppError::Ssh)?;
    if stat.is_dir {
        return Err(AppError::Validation(
            "directory transfer is not supported in send-file mode".into(),
        ));
    }

    let creds = build_creds(&state, req.target_connection_id).await?;
    let target_session_id = format!("transfer-target-{}", uuid::Uuid::new_v4());
    state
        .ssh_manager
        .open_sftp(target_session_id.clone(), creds, req.target_connection_id)
        .await
        .map_err(AppError::Ssh)?;

    let target_sftp = state
        .ssh_manager
        .get_sftp(&target_session_id)
        .await
        .map_err(AppError::Ssh)?;

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

            let mut buf = vec![0u8; 64 * 1024];
            let mut transferred = 0u64;

            loop {
                if *cancel_rx.borrow() {
                    return Err("cancelled".into());
                }

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
        .map_err(AppError::Ssh)
}
