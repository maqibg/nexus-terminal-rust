//! SFTP Tauri commands.

use api_contract::error::AppError;
use connection_core::repository::ConnectionRepository;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tauri::State;

use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

#[derive(Deserialize)]
pub struct SftpOpenRequest {
    pub connection_id: i64,
}

#[derive(Deserialize)]
pub struct SftpOpenOverrideRequest {
    pub connection_id: i64,
    pub username: Option<String>,
    pub auth_method: Option<String>,
    pub password: Option<String>,
}

#[derive(Deserialize)]
pub struct SftpPathRequest {
    pub session_id: String,
    pub path: String,
}

#[derive(Deserialize)]
pub struct SftpWriteFileRequest {
    pub session_id: String,
    pub path: String,
    pub data: String, // base64 encoded
}

#[derive(Deserialize)]
pub struct SftpRenameRequest {
    pub session_id: String,
    pub old_path: String,
    pub new_path: String,
}

#[derive(Deserialize)]
pub struct SftpCloseRequest {
    pub session_id: String,
}

/// Build SshCredentials from a connection record.
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

async fn build_creds_override(
    state: &AppState,
    connection_id: i64,
    username: Option<String>,
    auth_method: Option<String>,
    password: Option<String>,
) -> Result<ssh_core::session::SshCredentials, AppError> {
    let conn = state
        .conn_repo
        .get_connection(connection_id)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("connection not found".into()))?;

    let normalized_username = username
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| conn.username.clone());

    let normalized_auth_method = auth_method
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());

    let auth = match normalized_auth_method.as_deref() {
        Some("password") => {
            let normalized_password = password
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .ok_or(AppError::Validation(
                    "password auth requires non-empty password".into(),
                ))?;
            ssh_core::session::SshAuth::Password(normalized_password)
        }
        Some("key") => {
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
        }
        Some(other) => {
            return Err(AppError::Validation(format!(
                "unsupported auth method: {other}"
            )))
        }
        None => {
            if conn.auth_method == "key" {
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
                let stored_password = conn
                    .encrypted_password
                    .as_deref()
                    .ok_or(AppError::Validation("no password configured".into()))?;
                let decrypted = state
                    .crypto
                    .decrypt(stored_password)
                    .map_err(|e| AppError::Crypto(e.to_string()))?;
                ssh_core::session::SshAuth::Password(decrypted)
            }
        }
    };

    Ok(ssh_core::session::SshCredentials {
        host: conn.host,
        port: conn.port as u16,
        username: normalized_username,
        auth,
    })
}

#[tauri::command]
pub async fn sftp_open(state: State<'_, AppState>, req: SftpOpenRequest) -> CmdResult<String> {
    state.auth.require_auth().await?;
    let creds = build_creds(&state, req.connection_id).await?;
    let session_id = uuid::Uuid::new_v4().to_string();
    state
        .ssh_manager
        .open_sftp(session_id.clone(), creds, req.connection_id)
        .await
        .map_err(AppError::Ssh)?;
    Ok(session_id)
}

#[tauri::command]
pub async fn sftp_open_override(
    state: State<'_, AppState>,
    req: SftpOpenOverrideRequest,
) -> CmdResult<String> {
    state.auth.require_auth().await?;
    let connection_id = req.connection_id;
    let username = req.username.clone();
    let auth_method = req.auth_method.clone();
    let password = req.password.clone();

    let creds = build_creds_override(
        &state,
        connection_id,
        username.clone(),
        auth_method.clone(),
        password,
    )
    .await?;
    let session_id = uuid::Uuid::new_v4().to_string();
    let open_result = state
        .ssh_manager
        .open_sftp(session_id.clone(), creds, connection_id)
        .await;

    if let Err(primary_err) = open_result {
        let normalized_method = auth_method
            .as_deref()
            .map(|value| value.trim().to_ascii_lowercase());
        let should_try_key_fallback =
            matches!(normalized_method.as_deref(), Some("password") | None);

        if primary_err.contains("authentication rejected") && should_try_key_fallback {
            if let Ok(key_creds) = build_creds_override(
                &state,
                connection_id,
                username.clone(),
                Some("key".to_string()),
                None,
            )
            .await
            {
                state
                    .ssh_manager
                    .open_sftp(session_id.clone(), key_creds, connection_id)
                    .await
                    .map_err(AppError::Ssh)?;
                return Ok(session_id);
            }
        }

        let detailed = if primary_err.contains("authentication rejected") {
            "认证被拒绝：目标主机可能禁用 root SSH 密码登录。请检查 PermitRootLogin / PasswordAuthentication，或改用密钥认证。".to_string()
        } else {
            primary_err
        };
        return Err(AppError::Ssh(detailed));
    }

    Ok(session_id)
}

#[tauri::command]
pub async fn sftp_close(state: State<'_, AppState>, req: SftpCloseRequest) -> CmdResult<()> {
    state.auth.require_auth().await?;
    state
        .ssh_manager
        .close_sftp(&req.session_id)
        .await
        .map_err(AppError::Ssh)
}

#[tauri::command]
pub async fn sftp_list_dir(
    state: State<'_, AppState>,
    req: SftpPathRequest,
) -> CmdResult<Vec<sftp_core::service::FileEntry>> {
    state.auth.require_auth().await?;
    let sftp = state
        .ssh_manager
        .get_sftp(&req.session_id)
        .await
        .map_err(AppError::Ssh)?;
    sftp_core::service::list_dir(&sftp, &req.path)
        .await
        .map_err(AppError::Ssh)
}

#[tauri::command]
pub async fn sftp_read_file(state: State<'_, AppState>, req: SftpPathRequest) -> CmdResult<String> {
    state.auth.require_auth().await?;
    use base64::{engine::general_purpose::STANDARD as B64, Engine};
    let sftp = state
        .ssh_manager
        .get_sftp(&req.session_id)
        .await
        .map_err(AppError::Ssh)?;
    let bytes = sftp_core::service::read_file(&sftp, &req.path)
        .await
        .map_err(AppError::Ssh)?;
    Ok(B64.encode(&bytes))
}

#[tauri::command]
pub async fn sftp_write_file(
    state: State<'_, AppState>,
    req: SftpWriteFileRequest,
) -> CmdResult<()> {
    state.auth.require_auth().await?;
    use base64::{engine::general_purpose::STANDARD as B64, Engine};
    let data = B64
        .decode(&req.data)
        .map_err(|e| AppError::Validation(format!("invalid base64: {e}")))?;
    let sftp = state
        .ssh_manager
        .get_sftp(&req.session_id)
        .await
        .map_err(AppError::Ssh)?;
    sftp_core::service::write_file(&sftp, &req.path, &data)
        .await
        .map_err(AppError::Ssh)
}

#[tauri::command]
pub async fn sftp_mkdir(state: State<'_, AppState>, req: SftpPathRequest) -> CmdResult<()> {
    state.auth.require_auth().await?;
    let sftp = state
        .ssh_manager
        .get_sftp(&req.session_id)
        .await
        .map_err(AppError::Ssh)?;
    sftp_core::service::mkdir(&sftp, &req.path)
        .await
        .map_err(AppError::Ssh)
}

#[tauri::command]
pub async fn sftp_rmdir(state: State<'_, AppState>, req: SftpPathRequest) -> CmdResult<()> {
    state.auth.require_auth().await?;
    let sftp = state
        .ssh_manager
        .get_sftp(&req.session_id)
        .await
        .map_err(AppError::Ssh)?;
    sftp_core::service::rmdir(&sftp, &req.path)
        .await
        .map_err(AppError::Ssh)
}

#[tauri::command]
pub async fn sftp_remove_file(state: State<'_, AppState>, req: SftpPathRequest) -> CmdResult<()> {
    state.auth.require_auth().await?;
    let sftp = state
        .ssh_manager
        .get_sftp(&req.session_id)
        .await
        .map_err(AppError::Ssh)?;
    sftp_core::service::remove_file(&sftp, &req.path)
        .await
        .map_err(AppError::Ssh)
}

#[tauri::command]
pub async fn sftp_rename(state: State<'_, AppState>, req: SftpRenameRequest) -> CmdResult<()> {
    state.auth.require_auth().await?;
    let sftp = state
        .ssh_manager
        .get_sftp(&req.session_id)
        .await
        .map_err(AppError::Ssh)?;
    sftp_core::service::rename(&sftp, &req.old_path, &req.new_path)
        .await
        .map_err(AppError::Ssh)
}

#[tauri::command]
pub async fn sftp_stat(
    state: State<'_, AppState>,
    req: SftpPathRequest,
) -> CmdResult<sftp_core::service::FileEntry> {
    state.auth.require_auth().await?;
    let sftp = state
        .ssh_manager
        .get_sftp(&req.session_id)
        .await
        .map_err(AppError::Ssh)?;
    sftp_core::service::stat(&sftp, &req.path)
        .await
        .map_err(AppError::Ssh)
}

// ── SFTP Extensions (chmod, upload_chunk, download_file) ──

#[derive(Deserialize)]
pub struct SftpChmodRequest {
    pub session_id: String,
    pub path: String,
    pub mode: u32,
}

#[tauri::command]
pub async fn sftp_chmod(state: State<'_, AppState>, req: SftpChmodRequest) -> CmdResult<()> {
    state.auth.require_auth().await?;
    let sftp = state
        .ssh_manager
        .get_sftp(&req.session_id)
        .await
        .map_err(AppError::Ssh)?;
    sftp_core::service::chmod(&sftp, &req.path, req.mode)
        .await
        .map_err(AppError::Ssh)
}

#[derive(Deserialize)]
pub struct SftpUploadChunkRequest {
    pub session_id: String,
    pub path: String,
    pub chunk_index: u32,
    pub data_base64: String,
}

#[tauri::command]
pub async fn sftp_upload_chunk(
    state: State<'_, AppState>,
    req: SftpUploadChunkRequest,
) -> CmdResult<()> {
    state.auth.require_auth().await?;
    use base64::{engine::general_purpose::STANDARD as B64, Engine};
    let data = B64
        .decode(&req.data_base64)
        .map_err(|e| AppError::Validation(format!("invalid base64: {e}")))?;
    let sftp = state
        .ssh_manager
        .get_sftp(&req.session_id)
        .await
        .map_err(AppError::Ssh)?;
    if req.chunk_index == 0 {
        sftp_core::service::write_file(&sftp, &req.path, &data)
            .await
            .map_err(AppError::Ssh)?;
    } else {
        sftp_core::service::append_file(&sftp, &req.path, &data)
            .await
            .map_err(AppError::Ssh)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn sftp_download_file(
    state: State<'_, AppState>,
    req: SftpPathRequest,
) -> CmdResult<String> {
    state.auth.require_auth().await?;
    use base64::{engine::general_purpose::STANDARD as B64, Engine};
    let sftp = state
        .ssh_manager
        .get_sftp(&req.session_id)
        .await
        .map_err(AppError::Ssh)?;
    let bytes = sftp_core::service::read_file(&sftp, &req.path)
        .await
        .map_err(AppError::Ssh)?;
    Ok(B64.encode(&bytes))
}

/// Download large file directly to local disk (no base64 overhead).
#[derive(Deserialize)]
pub struct SftpDownloadToDiskRequest {
    pub session_id: String,
    pub remote_path: String,
    pub local_path: String,
}

#[tauri::command]
pub async fn sftp_download_to_disk(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    req: SftpDownloadToDiskRequest,
) -> CmdResult<String> {
    state.auth.require_auth().await?;
    use tauri::Emitter;
    use tokio::io::AsyncReadExt;

    let sftp = state
        .ssh_manager
        .get_sftp(&req.session_id)
        .await
        .map_err(AppError::Ssh)?;
    let file_name = req
        .remote_path
        .rsplit('/')
        .next()
        .unwrap_or("file")
        .to_string();

    // Get file size for progress
    let stat = sftp_core::service::stat(&sftp, &req.remote_path)
        .await
        .map_err(AppError::Ssh)?;
    let total = stat.size;

    // Create transfer task
    let tm = &state.transfer_manager;
    let (task_id, cancel_rx) =
        tm.create_task(transfer_core::TransferKind::Download, &file_name, total);

    let local_path = req.local_path.clone();
    let tid = task_id.clone();
    let tm2 = tm.clone();
    let ah = app_handle.clone();

    tokio::spawn(async move {
        let result: Result<(), String> = async {
            let mut remote = sftp
                .open(&req.remote_path)
                .await
                .map_err(|e| format!("open failed: {e}"))?;
            let mut local = tokio::fs::File::create(&local_path)
                .await
                .map_err(|e| format!("create local file failed: {e}"))?;

            let mut buf = vec![0u8; 64 * 1024];
            let mut transferred: u64 = 0;

            loop {
                if *cancel_rx.borrow() {
                    return Err("cancelled".into());
                }

                let n = remote
                    .read(&mut buf)
                    .await
                    .map_err(|e| format!("read: {e}"))?;
                if n == 0 {
                    break;
                }

                use tokio::io::AsyncWriteExt;
                local
                    .write_all(&buf[..n])
                    .await
                    .map_err(|e| format!("write: {e}"))?;
                transferred += n as u64;
                tm2.update_progress(&tid, transferred);

                let pct = if total > 0 {
                    (transferred * 100 / total) as u32
                } else {
                    0
                };
                let _ = ah.emit(
                    "transfers/progress",
                    serde_json::json!({
                        "task_id": tid, "file_name": file_name,
                        "kind": "download",
                        "bytes_transferred": transferred, "total_bytes": total, "percent": pct,
                    }),
                );
            }
            Ok(())
        }
        .await;

        match result {
            Ok(()) => tm2.complete(&tid),
            Err(e) => tm2.fail(&tid, &e),
        }
    });

    Ok(task_id)
}

/// Upload file from local disk with progress tracking.
#[derive(Deserialize)]
pub struct SftpUploadFromDiskRequest {
    pub session_id: String,
    pub local_path: String,
    pub remote_path: String,
}

#[tauri::command]
pub async fn sftp_upload_from_disk(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    req: SftpUploadFromDiskRequest,
) -> CmdResult<String> {
    state.auth.require_auth().await?;
    use tauri::Emitter;

    let sftp = state
        .ssh_manager
        .get_sftp(&req.session_id)
        .await
        .map_err(AppError::Ssh)?;
    let file_name = req
        .local_path
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or("file")
        .to_string();

    let meta = tokio::fs::metadata(&req.local_path)
        .await
        .map_err(|e| AppError::Validation(format!("local file error: {e}")))?;
    let total = meta.len();

    let tm = &state.transfer_manager;
    let (task_id, cancel_rx) =
        tm.create_task(transfer_core::TransferKind::Upload, &file_name, total);

    let tid = task_id.clone();
    let tm2 = tm.clone();
    let ah = app_handle.clone();
    let local_path = req.local_path.clone();
    let remote_path = req.remote_path.clone();

    tokio::spawn(async move {
        let result: Result<(), String> = async {
            let mut local = tokio::fs::File::open(&local_path)
                .await
                .map_err(|e| format!("open local: {e}"))?;
            let mut remote = sftp
                .create(&remote_path)
                .await
                .map_err(|e| format!("create remote: {e}"))?;

            let mut buf = vec![0u8; 64 * 1024];
            let mut transferred: u64 = 0;

            loop {
                if *cancel_rx.borrow() {
                    return Err("cancelled".into());
                }

                use tokio::io::AsyncReadExt;
                let n = local
                    .read(&mut buf)
                    .await
                    .map_err(|e| format!("read: {e}"))?;
                if n == 0 {
                    break;
                }

                use tokio::io::AsyncWriteExt;
                remote
                    .write_all(&buf[..n])
                    .await
                    .map_err(|e| format!("write: {e}"))?;
                transferred += n as u64;
                tm2.update_progress(&tid, transferred);

                let pct = if total > 0 {
                    (transferred * 100 / total) as u32
                } else {
                    0
                };
                let _ = ah.emit(
                    "transfers/progress",
                    serde_json::json!({
                        "task_id": tid, "file_name": file_name,
                        "kind": "upload",
                        "bytes_transferred": transferred, "total_bytes": total, "percent": pct,
                    }),
                );
            }

            use tokio::io::AsyncWriteExt;
            remote.shutdown().await.map_err(|e| format!("flush: {e}"))?;
            Ok(())
        }
        .await;

        match result {
            Ok(()) => tm2.complete(&tid),
            Err(e) => tm2.fail(&tid, &e),
        }
    });

    Ok(task_id)
}

/// Cancel an active transfer task.
#[derive(Deserialize)]
pub struct SftpCancelTaskRequest {
    pub task_id: String,
}

#[tauri::command]
pub async fn sftp_cancel_task(
    state: State<'_, AppState>,
    req: SftpCancelTaskRequest,
) -> CmdResult<()> {
    state.auth.require_auth().await?;
    state
        .transfer_manager
        .cancel(&req.task_id)
        .map_err(AppError::Ssh)
}

/// Download a remote directory, compress to ZIP, and save to local disk.
#[derive(Deserialize)]
pub struct SftpDownloadDirectoryToDiskRequest {
    pub session_id: String,
    pub remote_path: String,
    pub local_zip_path: String,
}

#[tauri::command]
pub async fn sftp_download_directory_to_disk(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    req: SftpDownloadDirectoryToDiskRequest,
) -> CmdResult<String> {
    state.auth.require_auth().await?;

    let sftp = state
        .ssh_manager
        .get_sftp(&req.session_id)
        .await
        .map_err(AppError::Ssh)?;

    let root_stat = sftp_core::service::stat(&sftp, &req.remote_path)
        .await
        .map_err(AppError::Ssh)?;
    if !root_stat.is_dir {
        return Err(AppError::Validation(
            "remote path is not a directory".into(),
        ));
    }

    let mut total_bytes = 0u64;
    let mut stat_stack = vec![req.remote_path.clone()];
    while let Some(dir) = stat_stack.pop() {
        let entries = sftp_core::service::list_dir(&sftp, &dir)
            .await
            .map_err(AppError::Ssh)?;
        for entry in entries {
            if entry.is_dir {
                stat_stack.push(entry.path);
            } else {
                total_bytes = total_bytes.saturating_add(entry.size);
            }
        }
    }

    let file_name = req
        .remote_path
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|v| !v.is_empty())
        .unwrap_or("directory")
        .to_string();

    let (task_id, cancel_rx) = state.transfer_manager.create_task(
        transfer_core::TransferKind::Download,
        &format!("{file_name}.zip"),
        total_bytes,
    );

    let tm = state.transfer_manager.clone();
    let tid = task_id.clone();
    let ah = app_handle.clone();
    let remote_root = req.remote_path.clone();
    let zip_path = PathBuf::from(req.local_zip_path.clone());
    let temp_dir = state
        .runtime_paths
        .temp_dir
        .join(format!("nexus-dir-download-{}", uuid::Uuid::new_v4()));

    tokio::spawn(async move {
        use tauri::Emitter;

        let _ = ah.emit(
            "transfers/status",
            serde_json::json!({
                "task_id": tid,
                "file_name": format!("{file_name}.zip"),
                "kind": "download",
                "status": "active",
            }),
        );

        let result: Result<(), String> = async {
            if let Some(parent) = zip_path.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(|e| format!("create output dir failed: {e}"))?;
            }

            tokio::fs::create_dir_all(&temp_dir)
                .await
                .map_err(|e| format!("create temp dir failed: {e}"))?;

            let mut transferred = 0u64;
            let mut stack: Vec<(String, PathBuf)> = vec![(remote_root.clone(), temp_dir.clone())];

            while let Some((remote_dir, local_dir)) = stack.pop() {
                if *cancel_rx.borrow() {
                    return Err("cancelled".into());
                }

                tokio::fs::create_dir_all(&local_dir)
                    .await
                    .map_err(|e| format!("create local dir failed: {e}"))?;

                let entries = sftp_core::service::list_dir(&sftp, &remote_dir)
                    .await
                    .map_err(|e| format!("list remote dir failed: {e}"))?;

                for entry in entries {
                    let local_path = local_dir.join(&entry.name);
                    if entry.is_dir {
                        stack.push((entry.path.clone(), local_path));
                        continue;
                    }

                    let bytes = sftp_core::service::read_file(&sftp, &entry.path)
                        .await
                        .map_err(|e| format!("read remote file failed: {e}"))?;

                    tokio::fs::write(&local_path, &bytes)
                        .await
                        .map_err(|e| format!("write local file failed: {e}"))?;

                    transferred = transferred.saturating_add(bytes.len() as u64);
                    tm.update_progress(&tid, transferred);

                    let pct = if total_bytes > 0 {
                        (transferred.saturating_mul(100) / total_bytes) as u32
                    } else {
                        0
                    };
                    let _ = ah.emit(
                        "transfers/progress",
                        serde_json::json!({
                            "task_id": tid,
                            "file_name": format!("{file_name}.zip"),
                            "kind": "download",
                            "bytes_transferred": transferred,
                            "total_bytes": total_bytes,
                            "percent": pct,
                        }),
                    );
                }
            }

            let temp_dir_clone = temp_dir.clone();
            let zip_path_clone = zip_path.clone();
            tokio::task::spawn_blocking(move || zip_directory(&temp_dir_clone, &zip_path_clone))
                .await
                .map_err(|e| format!("zip task join failed: {e}"))??;

            Ok(())
        }
        .await;

        let _ = tokio::fs::remove_dir_all(&temp_dir).await;

        let (status, err) = match result {
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
                "file_name": format!("{file_name}.zip"),
                "kind": "download",
                "status": status,
                "error": err,
            }),
        );
    });

    Ok(task_id)
}

fn zip_directory(source_dir: &Path, zip_path: &Path) -> Result<(), String> {
    use std::fs::File;
    use std::io::{Read, Write};
    use walkdir::WalkDir;
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    let file = File::create(zip_path).map_err(|e| format!("create zip failed: {e}"))?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    for entry in WalkDir::new(source_dir) {
        let entry = entry.map_err(|e| format!("walkdir failed: {e}"))?;
        let path = entry.path();
        let rel = path
            .strip_prefix(source_dir)
            .map_err(|e| format!("strip prefix failed: {e}"))?;

        if rel.as_os_str().is_empty() {
            continue;
        }

        let rel_name = rel.to_string_lossy().replace('\\', "/");
        if entry.file_type().is_dir() {
            zip.add_directory(format!("{rel_name}/"), options)
                .map_err(|e| format!("zip add dir failed: {e}"))?;
            continue;
        }

        zip.start_file(rel_name, options)
            .map_err(|e| format!("zip start file failed: {e}"))?;

        let mut src = File::open(path).map_err(|e| format!("open file failed: {e}"))?;
        let mut buf = Vec::new();
        src.read_to_end(&mut buf)
            .map_err(|e| format!("read file failed: {e}"))?;
        zip.write_all(&buf)
            .map_err(|e| format!("zip write failed: {e}"))?;
    }

    zip.finish()
        .map_err(|e| format!("zip finalize failed: {e}"))?;
    Ok(())
}
