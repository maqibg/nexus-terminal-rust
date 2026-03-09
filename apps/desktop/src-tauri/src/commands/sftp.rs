//! SFTP Tauri commands.

use api_contract::error::{AppError, CmdResult};
use connection_core::model::AuthMethod;
use connection_core::repository::ConnectionRepository;
use futures_util::future::try_join_all;
use serde::{Deserialize, Serialize};
use shared_utils::path::safe_join;
use ssh_core::host_key::HostKeyStore;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tauri::State;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::state::AppState;

const TRANSFER_BUFFER_SIZE: usize = 1024 * 1024;
const DIRECTORY_UPLOAD_CONCURRENCY: usize = 4;
const SCP_FILE_MODE: u32 = 0o644;

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
pub struct SftpCopyEntryRequest {
    pub session_id: String,
    pub source_path: String,
    pub target_path: String,
}

#[derive(Deserialize)]
pub struct SftpCloseRequest {
    pub session_id: String,
}

/// Build SshCredentials from a stored connection, allowing username / auth-method
/// / password overrides supplied by the caller (used for jump-host scenarios).
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
        .map_err(AppError::from)?
        .ok_or(AppError::NotFound("connection not found".into()))?;

    let normalized_username = username
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| conn.username.clone());

    // 解析调用方传入的 auth_method 字符串；非空但无法识别时报错
    let normalized_auth_method: Option<AuthMethod> = auth_method
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(AuthMethod::try_parse)
        .transpose()
        .map_err(AppError::Validation)?;

    let auth = match normalized_auth_method {
        Some(AuthMethod::Password) => {
            let normalized_password = password
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .ok_or(AppError::Validation(
                    "password auth requires non-empty password".into(),
                ))?;
            ssh_core::session::SshAuth::Password(normalized_password)
        }
        Some(AuthMethod::Key) => {
            let key_id = conn
                .ssh_key_id
                .ok_or(AppError::Validation("no SSH key configured".into()))?;
            let key = state
                .conn_repo
                .get_ssh_key(key_id)
                .await
                .map_err(AppError::from)?
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
        None => match conn.auth_method_enum() {
            AuthMethod::Key => {
                let key_id = conn
                    .ssh_key_id
                    .ok_or(AppError::Validation("no SSH key configured".into()))?;
                let key = state
                    .conn_repo
                    .get_ssh_key(key_id)
                    .await
                    .map_err(AppError::from)?
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
            AuthMethod::Password => {
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
        },
    };

    Ok(ssh_core::session::SshCredentials {
        host: conn.host,
        port: conn.port as u16,
        username: normalized_username,
        auth,
    })
}

#[tauri::command]
pub async fn sftp_open(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    req: SftpOpenRequest,
) -> CmdResult<String> {
    state.auth.require_auth().await?;
    let creds = super::build_creds(&state, req.connection_id).await?;
    let session_id = uuid::Uuid::new_v4().to_string();
    let host_key_store: Arc<dyn HostKeyStore> = Arc::new(super::SettingsHostKeyStore {
        repo: state.host_key_repo.clone(),
    });
    state
        .ssh_manager
        .open_sftp(
            session_id.clone(),
            creds,
            req.connection_id,
            Some(host_key_store),
            Some(app_handle),
        )
        .await
        .map_err(AppError::Ssh)?;
    Ok(session_id)
}

#[tauri::command]
pub async fn sftp_open_override(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
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
    let host_key_store: Arc<dyn HostKeyStore> = Arc::new(super::SettingsHostKeyStore {
        repo: state.host_key_repo.clone(),
    });
    let open_result = state
        .ssh_manager
        .open_sftp(
            session_id.clone(),
            creds,
            connection_id,
            Some(host_key_store.clone()),
            Some(app_handle.clone()),
        )
        .await;

    if let Err(primary_err) = open_result {
        let normalized_method = auth_method.as_deref().and_then(AuthMethod::parse);
        let should_try_key_fallback =
            matches!(normalized_method, Some(AuthMethod::Password) | None);

        if primary_err.contains("authentication rejected") && should_try_key_fallback {
            if let Ok(key_creds) = build_creds_override(
                &state,
                connection_id,
                username.clone(),
                Some(AuthMethod::Key.as_str().to_string()),
                None,
            )
            .await
            {
                state
                    .ssh_manager
                    .open_sftp(
                        session_id.clone(),
                        key_creds,
                        connection_id,
                        Some(host_key_store),
                        Some(app_handle),
                    )
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
    sftp_core::service::rmdir_recursive(&sftp, &req.path)
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
pub async fn sftp_copy_entry(
    state: State<'_, AppState>,
    req: SftpCopyEntryRequest,
) -> CmdResult<()> {
    state.auth.require_auth().await?;
    let sftp = state
        .ssh_manager
        .get_sftp(&req.session_id)
        .await
        .map_err(AppError::Ssh)?;

    let source = sftp_core::service::stat(&sftp, &req.source_path)
        .await
        .map_err(AppError::Ssh)?;

    if !source.is_dir {
        return sftp_core::service::copy_file_streaming(&sftp, &req.source_path, &req.target_path)
            .await
            .map_err(AppError::Ssh);
    }

    let mut stack = vec![(req.source_path.clone(), req.target_path.clone())];
    while let Some((source_dir, target_dir)) = stack.pop() {
        sftp_core::service::mkdir(&sftp, &target_dir)
            .await
            .map_err(AppError::Ssh)?;

        let children = sftp_core::service::list_dir(&sftp, &source_dir)
            .await
            .map_err(AppError::Ssh)?;
        for child in children {
            let child_target_path = if target_dir.ends_with('/') {
                format!("{target_dir}{}", child.name)
            } else {
                format!("{target_dir}/{}", child.name)
            };

            if child.is_dir {
                stack.push((child.path, child_target_path));
                continue;
            }

            sftp_core::service::copy_file_streaming(&sftp, &child.path, &child_target_path)
                .await
                .map_err(AppError::Ssh)?;
        }
    }

    Ok(())
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

            let mut buf = vec![0u8; TRANSFER_BUFFER_SIZE];
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

#[derive(Deserialize)]
pub struct SftpUploadEntryFromDiskRequest {
    pub session_id: String,
    pub local_path: String,
    pub remote_path: String,
}

#[derive(Deserialize)]
pub struct SftpCollectLocalUploadEntriesRequest {
    pub paths: Vec<String>,
}

#[derive(Clone, Serialize)]
pub struct LocalUploadEntry {
    pub local_path: String,
    pub relative_path: String,
    pub display_path: String,
}

fn shell_escape_posix(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    format!("'{}'", value.replace('\'', r"'\''"))
}

fn split_remote_parent_and_name(remote_path: &str) -> Result<(String, String), String> {
    let normalized = remote_path.trim().replace('\\', "/");
    let Some((parent, name)) = normalized.rsplit_once('/') else {
        return Err(format!("invalid remote path: {remote_path}"));
    };
    if name.is_empty() {
        return Err(format!("invalid remote file name in path: {remote_path}"));
    }
    if name.contains('/') || name.contains('\n') || name.contains('\r') {
        return Err(format!("unsupported remote file name for scp: {name}"));
    }
    let parent = if parent.is_empty() { "/" } else { parent }.to_string();
    Ok((parent, name.to_string()))
}

async fn open_exec_channel(
    handle: Arc<russh::client::Handle<ssh_core::session::SshHandler>>,
    command: &str,
) -> Result<russh::Channel<russh::client::Msg>, String> {
    let channel = handle
        .channel_open_session()
        .await
        .map_err(|error| format!("open exec channel failed: {error}"))?;
    channel
        .exec(false, command)
        .await
        .map_err(|error| format!("exec '{command}' failed: {error}"))?;
    Ok(channel)
}

async fn open_upload_exec_handle(
    creds: &ssh_core::session::SshCredentials,
    host_key_store: Arc<dyn HostKeyStore>,
    app_handle: &tauri::AppHandle,
) -> Result<Arc<russh::client::Handle<ssh_core::session::SshHandler>>, String> {
    let handler = ssh_core::session::SshHandler::with_tofu(
        creds.host.clone(),
        creds.port,
        host_key_store,
        app_handle.clone(),
    );
    let handle = ssh_core::session::connect_and_authenticate(creds, handler).await?;
    Ok(Arc::new(handle))
}

async fn read_exec_result(
    mut channel: russh::Channel<russh::client::Msg>,
) -> Result<(u32, String, String), String> {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut exit_status = 0u32;

    while let Some(message) = channel.wait().await {
        match message {
            russh::ChannelMsg::Data { data } => stdout.extend_from_slice(data.as_ref()),
            russh::ChannelMsg::ExtendedData { data, .. } => stderr.extend_from_slice(data.as_ref()),
            russh::ChannelMsg::ExitStatus { exit_status: status } => exit_status = status,
            russh::ChannelMsg::Eof | russh::ChannelMsg::Close => break,
            _ => {}
        }
    }

    Ok((
        exit_status,
        String::from_utf8_lossy(&stdout).to_string(),
        String::from_utf8_lossy(&stderr).to_string(),
    ))
}

async fn ensure_remote_parent_dirs_with_exec(
    handle: Arc<russh::client::Handle<ssh_core::session::SshHandler>>,
    remote_path: &str,
) -> Result<(), String> {
    let normalized = remote_path.trim().replace('\\', "/");
    let Some((parent_dir, _)) = normalized.rsplit_once('/') else {
        return Ok(());
    };
    if parent_dir.is_empty() || parent_dir == "/" {
        return Ok(());
    }

    let command = format!("mkdir -p -- {}", shell_escape_posix(parent_dir));
    let channel = open_exec_channel(handle, &command).await?;
    let (exit_status, _stdout, stderr) = read_exec_result(channel).await?;
    if exit_status == 0 {
        return Ok(());
    }

    let message = stderr.trim();
    if message.is_empty() {
        Err(format!("mkdir -p failed for {parent_dir} with exit status {exit_status}"))
    } else {
        Err(format!("mkdir -p failed for {parent_dir}: {message}"))
    }
}

async fn scp_read_ack<R>(reader: &mut R) -> Result<(), String>
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut code = [0u8; 1];
    reader
        .read_exact(&mut code)
        .await
        .map_err(|error| format!("scp ack read failed: {error}"))?;

    match code[0] {
        0 => Ok(()),
        1 | 2 => {
            let mut message = Vec::new();
            loop {
                let mut byte = [0u8; 1];
                reader
                    .read_exact(&mut byte)
                    .await
                    .map_err(|error| format!("scp error read failed: {error}"))?;
                if byte[0] == b'\n' {
                    break;
                }
                message.push(byte[0]);
            }
            let text = String::from_utf8_lossy(&message).trim().to_string();
            if text.is_empty() {
                Err("scp remote error".to_string())
            } else {
                Err(text)
            }
        }
        other => Err(format!("unexpected scp ack byte: {other}")),
    }
}

fn join_remote_path(base: &str, name: &str) -> String {
    if base.ends_with('/') {
        format!("{base}{name}")
    } else {
        format!("{base}/{name}")
    }
}

async fn collect_local_entry_total_bytes(root_path: &Path) -> Result<u64, String> {
    let metadata = tokio::fs::symlink_metadata(root_path)
        .await
        .map_err(|error| format!("local path error: {error}"))?;

    if metadata.file_type().is_symlink() {
        return Err(format!(
            "symlink upload is not supported: {}",
            root_path.display()
        ));
    }

    if metadata.is_file() {
        return Ok(metadata.len());
    }

    let mut total_bytes = 0u64;
    let mut stack = vec![root_path.to_path_buf()];
    while let Some(dir_path) = stack.pop() {
        let mut dir = tokio::fs::read_dir(&dir_path)
            .await
            .map_err(|error| format!("read_dir failed: {error}"))?;

        while let Some(entry) = dir
            .next_entry()
            .await
            .map_err(|error| format!("read_dir entry failed: {error}"))?
        {
            let entry_type = entry
                .file_type()
                .await
                .map_err(|error| format!("file_type failed: {error}"))?;
            let entry_path = entry.path();

            if entry_type.is_symlink() {
                return Err(format!(
                    "symlink upload is not supported: {}",
                    entry_path.display()
                ));
            }

            if entry_type.is_dir() {
                stack.push(entry_path);
                continue;
            }

            if entry_type.is_file() {
                total_bytes = total_bytes.saturating_add(
                    entry
                        .metadata()
                        .await
                        .map_err(|error| format!("metadata failed: {error}"))?
                        .len(),
                );
            }
        }
    }

    Ok(total_bytes)
}

async fn collect_local_upload_jobs(
    root_path: &Path,
    remote_root: &str,
) -> Result<Vec<(PathBuf, String)>, String> {
    let metadata = tokio::fs::symlink_metadata(root_path)
        .await
        .map_err(|error| format!("local path error: {error}"))?;

    if metadata.file_type().is_symlink() {
        return Err(format!(
            "symlink upload is not supported: {}",
            root_path.display()
        ));
    }

    if metadata.is_file() {
        return Ok(vec![(root_path.to_path_buf(), remote_root.to_string())]);
    }

    let mut jobs = Vec::new();
    let mut stack = vec![(root_path.to_path_buf(), remote_root.to_string())];
    while let Some((local_dir, remote_dir)) = stack.pop() {
        let mut dir = tokio::fs::read_dir(&local_dir)
            .await
            .map_err(|error| format!("read_dir failed: {error}"))?;
        while let Some(entry) = dir
            .next_entry()
            .await
            .map_err(|error| format!("read_dir entry failed: {error}"))?
        {
            let entry_type = entry
                .file_type()
                .await
                .map_err(|error| format!("file_type failed: {error}"))?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            let child_local_path = entry.path();
            let child_remote_path = join_remote_path(&remote_dir, &file_name);

            if entry_type.is_symlink() {
                return Err(format!(
                    "symlink upload is not supported: {}",
                    child_local_path.display()
                ));
            }

            if entry_type.is_dir() {
                stack.push((child_local_path, child_remote_path));
                continue;
            }

            if entry_type.is_file() {
                jobs.push((child_local_path, child_remote_path));
            }
        }
    }

    Ok(jobs)
}

fn local_upload_entry(
    local_path: &Path,
    relative_path: String,
    file_name: &str,
) -> LocalUploadEntry {
    let display_path = if relative_path.is_empty() {
        file_name.to_string()
    } else {
        format!("{relative_path}{file_name}")
    };

    LocalUploadEntry {
        local_path: local_path.to_string_lossy().to_string(),
        relative_path,
        display_path,
    }
}

async fn collect_local_upload_entries_for_root(
    root_path: &Path,
) -> Result<Vec<LocalUploadEntry>, String> {
    let metadata = tokio::fs::symlink_metadata(root_path)
        .await
        .map_err(|error| format!("local path error: {error}"))?;

    if metadata.file_type().is_symlink() {
        return Err(format!(
            "symlink upload is not supported: {}",
            root_path.display()
        ));
    }

    let root_name = root_path
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("invalid local path: {}", root_path.display()))?;

    if metadata.is_file() {
        return Ok(vec![local_upload_entry(root_path, String::new(), root_name)]);
    }

    let mut uploads = Vec::new();
    let mut stack = vec![(root_path.to_path_buf(), format!("{root_name}/"))];
    while let Some((local_dir, relative_dir)) = stack.pop() {
        let mut dir = tokio::fs::read_dir(&local_dir)
            .await
            .map_err(|error| format!("read_dir failed: {error}"))?;

        while let Some(entry) = dir
            .next_entry()
            .await
            .map_err(|error| format!("read_dir entry failed: {error}"))?
        {
            let entry_type = entry
                .file_type()
                .await
                .map_err(|error| format!("file_type failed: {error}"))?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            let child_path = entry.path();

            if entry_type.is_symlink() {
                return Err(format!(
                    "symlink upload is not supported: {}",
                    child_path.display()
                ));
            }

            if entry_type.is_dir() {
                stack.push((child_path, format!("{relative_dir}{file_name}/")));
                continue;
            }

            if entry_type.is_file() {
                uploads.push(local_upload_entry(&child_path, relative_dir.clone(), &file_name));
            }
        }
    }

    Ok(uploads)
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
        tokio::time::sleep(std::time::Duration::from_millis(120)).await;
    }
    Ok(())
}

#[tauri::command]
pub async fn sftp_collect_local_upload_entries(
    state: State<'_, AppState>,
    req: SftpCollectLocalUploadEntriesRequest,
) -> CmdResult<Vec<LocalUploadEntry>> {
    state.auth.require_auth().await?;

    let mut uploads = Vec::new();
    for path in req.paths {
        let root_path = PathBuf::from(path);
        let mut root_entries = collect_local_upload_entries_for_root(&root_path)
            .await
            .map_err(AppError::Validation)?;
        uploads.append(&mut root_entries);
    }

    uploads.sort_by(|left, right| left.display_path.cmp(&right.display_path));
    Ok(uploads)
}

async fn upload_local_file_with_progress(
    handle: Arc<russh::client::Handle<ssh_core::session::SshHandler>>,
    local_path: &Path,
    remote_path: &str,
    transferred: Arc<AtomicU64>,
    total: u64,
    task_id: &str,
    file_name: &str,
    tm: &transfer_core::TransferManager,
    app_handle: &tauri::AppHandle,
    cancel_rx: &tokio::sync::watch::Receiver<bool>,
    upload_limiter: &Arc<tokio::sync::Semaphore>,
) -> Result<(), String> {
    use tauri::Emitter;
    let _permit = upload_limiter
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| "upload limiter closed".to_string())?;
    let (remote_parent, remote_name) = split_remote_parent_and_name(remote_path)?;
    ensure_remote_parent_dirs_with_exec(handle.clone(), remote_path).await?;

    let command = format!("scp -t -- {}", shell_escape_posix(&remote_parent));
    let channel = open_exec_channel(handle, &command).await?;
    let stream = channel.into_stream();
    let (mut reader, mut writer) = tokio::io::split(stream);

    scp_read_ack(&mut reader).await?;

    let file_size = tokio::fs::metadata(local_path)
        .await
        .map_err(|error| format!("stat local failed: {error}"))?
        .len();
    let header = format!("C{:04o} {file_size} {remote_name}\n", SCP_FILE_MODE);
    writer
        .write_all(header.as_bytes())
        .await
        .map_err(|error| format!("send scp header failed: {error}"))?;
    writer
        .flush()
        .await
        .map_err(|error| format!("flush scp header failed: {error}"))?;
    scp_read_ack(&mut reader).await?;

    let mut local = tokio::fs::File::open(local_path)
        .await
        .map_err(|error| format!("open local failed: {error}"))?;
    let mut buffer = vec![0u8; TRANSFER_BUFFER_SIZE];

    loop {
        if *cancel_rx.borrow() {
            return Err("cancelled".into());
        }
        wait_if_paused(tm, task_id, cancel_rx).await?;

        let read = local
            .read(&mut buffer)
            .await
            .map_err(|error| format!("read local failed: {error}"))?;
        if read == 0 {
            break;
        }

        writer
            .write_all(&buffer[..read])
            .await
            .map_err(|error| format!("write remote failed: {error}"))?;

        let total_transferred = transferred.fetch_add(read as u64, Ordering::Relaxed) + read as u64;
        tm.update_progress(task_id, total_transferred);
        let percent = if total > 0 {
            (total_transferred.saturating_mul(100) / total) as u32
        } else {
            0
        };
        let _ = app_handle.emit(
            "transfers/progress",
            serde_json::json!({
                "task_id": task_id,
                "file_name": file_name,
                "kind": "upload",
                "bytes_transferred": total_transferred,
                "total_bytes": total,
                "percent": percent,
            }),
        );
    }

    writer
        .write_all(&[0])
        .await
        .map_err(|error| format!("send scp file terminator failed: {error}"))?;
    writer
        .flush()
        .await
        .map_err(|error| format!("flush scp payload failed: {error}"))?;
    scp_read_ack(&mut reader).await?;
    writer
        .shutdown()
        .await
        .or_else(|error| {
            if matches!(
                error.kind(),
                std::io::ErrorKind::BrokenPipe | std::io::ErrorKind::ConnectionReset
            ) {
                Ok(())
            } else {
                Err(error)
            }
        })
        .map_err(|error| format!("shutdown scp stream failed: {error}"))?;
    Ok(())
}

#[tauri::command]
pub async fn sftp_upload_from_disk(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    req: SftpUploadFromDiskRequest,
) -> CmdResult<String> {
    state.auth.require_auth().await?;
    use tauri::Emitter;

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
    let local_path = PathBuf::from(req.local_path.clone());
    let remote_path = req.remote_path.clone();
    let upload_limiter = state.sftp_upload_limiter.clone();
    let creds = state
        .ssh_manager
        .get_sftp_credentials(&req.session_id)
        .await
        .map_err(AppError::Ssh)?;
    let host_key_store: Arc<dyn HostKeyStore> = Arc::new(super::SettingsHostKeyStore {
        repo: state.host_key_repo.clone(),
    });

    tokio::spawn(async move {
        let _ = ah.emit(
            "transfers/status",
            serde_json::json!({
                "task_id": tid,
                "file_name": file_name,
                "kind": "upload",
                "status": "active",
            }),
        );

        let handle = match open_upload_exec_handle(&creds, host_key_store, &ah).await {
            Ok(handle) => handle,
            Err(error) => {
                tm2.fail(&tid, &error);
                let _ = ah.emit(
                    "transfers/status",
                    serde_json::json!({
                        "task_id": tid,
                        "file_name": file_name,
                        "kind": "upload",
                        "status": "failed",
                        "error": error,
                    }),
                );
                return;
            }
        };

        let transferred = Arc::new(AtomicU64::new(0));
        let result = upload_local_file_with_progress(
            handle,
            &local_path,
            &remote_path,
            transferred,
            total,
            &tid,
            &file_name,
            &tm2,
            &ah,
            &cancel_rx,
            &upload_limiter,
        )
        .await;

        let (status, error) = match result {
            Ok(()) => {
                tm2.complete(&tid);
                ("completed", None)
            }
            Err(error) if error == "cancelled" => {
                let _ = tm2.cancel(&tid);
                ("cancelled", None)
            }
            Err(error) => {
                tm2.fail(&tid, &error);
                ("failed", Some(error))
            }
        };

        let _ = ah.emit(
            "transfers/status",
            serde_json::json!({
                "task_id": tid,
                "file_name": file_name,
                "kind": "upload",
                "status": status,
                "error": error,
            }),
        );
    });

    Ok(task_id)
}

#[tauri::command]
pub async fn sftp_upload_entry_from_disk(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    req: SftpUploadEntryFromDiskRequest,
) -> CmdResult<String> {
    state.auth.require_auth().await?;
    let local_path = PathBuf::from(&req.local_path);
    let entry_name = local_path
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::Validation("invalid local path".into()))?
        .to_string();

    let total = collect_local_entry_total_bytes(&local_path)
        .await
        .map_err(AppError::Validation)?;

    let tm = &state.transfer_manager;
    let (task_id, cancel_rx) =
        tm.create_task(transfer_core::TransferKind::Upload, &entry_name, total);

    let tid = task_id.clone();
    let tm2 = tm.clone();
    let ah = app_handle.clone();
    let remote_root = req.remote_path.clone();
    let upload_limiter = state.sftp_upload_limiter.clone();
    let creds = state
        .ssh_manager
        .get_sftp_credentials(&req.session_id)
        .await
        .map_err(AppError::Ssh)?;
    let host_key_store: Arc<dyn HostKeyStore> = Arc::new(super::SettingsHostKeyStore {
        repo: state.host_key_repo.clone(),
    });

    tokio::spawn(async move {
        use tauri::Emitter;

        let _ = ah.emit(
            "transfers/status",
            serde_json::json!({
                "task_id": tid,
                "file_name": entry_name,
                "kind": "upload",
                "status": "active",
            }),
        );

        let handle = match open_upload_exec_handle(&creds, host_key_store, &ah).await {
            Ok(handle) => handle,
            Err(error) => {
                tm2.fail(&tid, &error);
                let _ = ah.emit(
                    "transfers/status",
                    serde_json::json!({
                        "task_id": tid,
                        "file_name": entry_name,
                        "kind": "upload",
                        "status": "failed",
                        "error": error,
                    }),
                );
                return;
            }
        };

        let result: Result<(), String> = async {
            let metadata = tokio::fs::symlink_metadata(&local_path)
                .await
                .map_err(|error| format!("local path error: {error}"))?;
            if metadata.file_type().is_symlink() {
                return Err(format!(
                    "symlink upload is not supported: {}",
                    local_path.display()
                ));
            }

            let transferred = Arc::new(AtomicU64::new(0));
            if metadata.is_file() {
                upload_local_file_with_progress(
                    handle.clone(),
                    &local_path,
                    &remote_root,
                    transferred.clone(),
                    total,
                    &tid,
                    &entry_name,
                    &tm2,
                    &ah,
                    &cancel_rx,
                    &upload_limiter,
                )
                .await?;
                return Ok(());
            }

            let jobs = collect_local_upload_jobs(&local_path, &remote_root).await?;
            for job_group in jobs.chunks(DIRECTORY_UPLOAD_CONCURRENCY) {
                if *cancel_rx.borrow() {
                    return Err("cancelled".into());
                }

                let futures = job_group.iter().map(|(child_local_path, child_remote_path)| {
                    let handle = handle.clone();
                    let tm = tm2.clone();
                    let app_handle = ah.clone();
                    let cancel_rx = cancel_rx.clone();
                    let task_id = tid.clone();
                    let entry_name = entry_name.clone();
                    let transferred = transferred.clone();
                    let local_path = child_local_path.clone();
                    let remote_path = child_remote_path.clone();
                    let upload_limiter = upload_limiter.clone();

                    async move {
                        upload_local_file_with_progress(
                            handle,
                            &local_path,
                            &remote_path,
                            transferred,
                            total,
                            &task_id,
                            &entry_name,
                            &tm,
                            &app_handle,
                            &cancel_rx,
                            &upload_limiter,
                        )
                        .await
                    }
                });

                try_join_all(futures).await?;
            }

            Ok(())
        }
        .await;

        let (status, error) = match result {
            Ok(()) => {
                tm2.complete(&tid);
                ("completed", None)
            }
            Err(error) if error == "cancelled" => {
                let _ = tm2.cancel(&tid);
                ("cancelled", None)
            }
            Err(error) => {
                tm2.fail(&tid, &error);
                ("failed", Some(error))
            }
        };

        let _ = ah.emit(
            "transfers/status",
            serde_json::json!({
                "task_id": tid,
                "file_name": entry_name,
                "kind": "upload",
                "status": status,
                "error": error,
            }),
        );
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
        .map_err(|_| AppError::NotFound(format!("transfer task '{}' not found", req.task_id)))
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
                    // 跳过 symlink（防止符号链接逃逸到下载目录之外）
                    let is_symlink = entry
                        .permissions
                        .map(|p| (p & 0o170000) == 0o120000)
                        .unwrap_or(false);
                    if is_symlink {
                        continue;
                    }

                    let local_path = safe_join(&local_dir, &entry.name)
                        .map_err(|e| format!("unsafe filename rejected: {e}"))?;

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
