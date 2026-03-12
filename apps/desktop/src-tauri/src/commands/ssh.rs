//! SSH/SFTP Tauri commands.

use api_contract::error::{AppError, CmdResult};
use connection_core::model::AuthMethod;
use connection_core::repository::ConnectionRepository;
use serde::{Deserialize, Serialize};
use settings_core::repository::SettingsRepository;
use ssh_core::host_key::{HostKeyEntry, HostKeyStore};
use std::sync::Arc;
use tauri::State;
use tokio::time::Duration;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct SshConnectRequest {
    pub connection_id: i64,
    pub cols: Option<u32>,
    pub rows: Option<u32>,
}

#[derive(Deserialize)]
pub struct SshWriteRequest {
    pub session_id: String,
    pub data: String, // base64 encoded
}

#[derive(Deserialize)]
pub struct SshResizeRequest {
    pub session_id: String,
    pub cols: u32,
    pub rows: u32,
}

#[derive(Deserialize)]
pub struct SshCloseRequest {
    pub session_id: String,
}

#[derive(Deserialize)]
pub struct SshTakeOutputBacklogRequest {
    pub session_id: String,
}

#[derive(Deserialize)]
pub struct SshExecRequest {
    pub session_id: String,
    pub command: String,
    pub timeout_ms: Option<u64>,
    pub stdin_base64: Option<String>,
    pub request_pty: Option<bool>,
}

#[derive(Serialize)]
pub struct SshExecResponse {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: u32,
    pub truncated: bool,
}

#[tauri::command]
pub async fn ssh_connect(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    req: SshConnectRequest,
) -> CmdResult<String> {
    state.auth.require_auth().await?;

    let conn = state
        .conn_repo
        .get_connection(req.connection_id)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::NotFound("connection not found".into()))?;

    let auth = match conn.auth_method_enum() {
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
            let password = conn
                .encrypted_password
                .as_deref()
                .ok_or(AppError::Validation("no password configured".into()))?;
            let decrypted = state
                .crypto
                .decrypt(password)
                .map_err(|e| AppError::Crypto(e.to_string()))?;

            ssh_core::session::SshAuth::Password(decrypted)
        }
    };

    let creds = ssh_core::session::SshCredentials {
        host: conn.host,
        port: conn.port as u16,
        username: conn.username,
        auth,
    };

    let host_key_store: Arc<dyn HostKeyStore> = Arc::new(super::SettingsHostKeyStore {
        repo: state.host_key_repo.clone(),
    });

    let session_id = uuid::Uuid::new_v4().to_string();
    state
        .ssh_manager
        .open_session(
            session_id.clone(),
            creds,
            req.connection_id,
            conn.name,
            req.cols.unwrap_or(80),
            req.rows.unwrap_or(24),
            app_handle.clone(),
            Some(host_key_store),
        )
        .await
        .map_err(AppError::Ssh)?;

    let status_monitor_enabled = match state
        .settings_repo
        .get_setting("statusMonitorEnabled")
        .await
        .map_err(AppError::from)?
    {
        Some(raw) => raw.trim().eq_ignore_ascii_case("true") || raw.trim() == "1",
        None => true,
    };

    if status_monitor_enabled {
        let status_monitor_interval = state
            .settings_repo
            .get_setting("statusMonitorIntervalSeconds")
            .await
            .map_err(AppError::from)?
            .and_then(|raw| raw.trim().parse::<u64>().ok())
            .filter(|seconds| *seconds >= 1)
            .map(Duration::from_secs);

        state
            .status_monitor
            .start_session(
                session_id.clone(),
                state.ssh_manager.clone(),
                app_handle,
                status_monitor_interval,
            )
            .await;
    }

    Ok(session_id)
}

#[tauri::command]
pub async fn ssh_write(state: State<'_, AppState>, req: SshWriteRequest) -> CmdResult<()> {
    state.auth.require_auth().await?;

    use base64::{engine::general_purpose::STANDARD as B64, Engine};
    let data = B64
        .decode(&req.data)
        .map_err(|e| AppError::Validation(format!("invalid base64: {e}")))?;

    state
        .ssh_manager
        .write(&req.session_id, &data)
        .await
        .map_err(AppError::Ssh)
}

#[tauri::command]
pub async fn ssh_resize(state: State<'_, AppState>, req: SshResizeRequest) -> CmdResult<()> {
    state.auth.require_auth().await?;

    state
        .ssh_manager
        .resize(&req.session_id, req.cols, req.rows)
        .await
        .map_err(AppError::Ssh)
}

#[tauri::command]
pub async fn ssh_close(state: State<'_, AppState>, req: SshCloseRequest) -> CmdResult<()> {
    state.auth.require_auth().await?;

    state.status_monitor.stop_session(&req.session_id).await;
    state
        .ssh_manager
        .close(&req.session_id)
        .await
        .map_err(AppError::Ssh)
}

#[tauri::command]
pub async fn ssh_session_list(state: State<'_, AppState>) -> CmdResult<Vec<serde_json::Value>> {
    state.auth.require_auth().await?;

    let sessions = state.ssh_manager.list_sessions().await;
    Ok(sessions
        .into_iter()
        .map(|(id, conn_id, name)| {
            serde_json::json!({ "session_id": id, "connection_id": conn_id, "connection_name": name })
        })
        .collect())
}

#[tauri::command]
pub async fn ssh_take_output_backlog(
    state: State<'_, AppState>,
    req: SshTakeOutputBacklogRequest,
) -> CmdResult<Vec<ssh_core::manager::SshOutputChunk>> {
    state.auth.require_auth().await?;
    Ok(state.ssh_manager.take_output_backlog(&req.session_id).await)
}

#[tauri::command]
pub async fn ssh_exec_command(
    state: State<'_, AppState>,
    req: SshExecRequest,
) -> CmdResult<SshExecResponse> {
    state.auth.require_auth().await?;

    let timeout_ms = req.timeout_ms.unwrap_or(3000).clamp(200, 120_000);
    let stdin = match req.stdin_base64.as_deref() {
        Some(base64) => {
            use base64::{engine::general_purpose::STANDARD as B64, Engine};
            Some(
                B64.decode(base64)
                    .map_err(|e| AppError::Validation(format!("invalid stdin base64: {e}")))?,
            )
        }
        None => None,
    };
    let output = state
        .ssh_manager
        .exec_command(
            &req.session_id,
            &req.command,
            stdin,
            req.request_pty.unwrap_or(false),
            Duration::from_millis(timeout_ms),
        )
        .await
        .map_err(AppError::Ssh)?;

    Ok(SshExecResponse {
        stdout: output.stdout,
        stderr: output.stderr,
        exit_code: output.exit_code,
        truncated: output.truncated,
    })
}

/// Accept a changed SSH host key — overwrites the stored fingerprint.
#[tauri::command]
pub async fn ssh_accept_host_key(
    state: State<'_, AppState>,
    host: String,
    port: u16,
    fingerprint: String,
) -> CmdResult<()> {
    state.auth.require_auth().await?;
    let store = super::SettingsHostKeyStore {
        repo: state.host_key_repo.clone(),
    };
    store
        .set_fingerprint(&host, port, &fingerprint)
        .await
        .map_err(AppError::Ssh)
}

/// 列出所有已知 SSH 主机指纹（known_hosts 管理）。
#[tauri::command]
pub async fn ssh_host_key_list(state: State<'_, AppState>) -> CmdResult<Vec<HostKeyEntry>> {
    state.auth.require_auth().await?;
    state.host_key_repo.list().await.map_err(AppError::from)
}

/// 删除指定主机的 SSH 指纹记录。
#[tauri::command]
pub async fn ssh_host_key_delete(
    state: State<'_, AppState>,
    host: String,
    port: u16,
) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    state
        .host_key_repo
        .delete(&host, port)
        .await
        .map_err(AppError::from)
}

/// 查询指定主机的 SSH 指纹。
#[tauri::command]
pub async fn ssh_host_key_get(
    state: State<'_, AppState>,
    host: String,
    port: u16,
) -> CmdResult<Option<String>> {
    state.auth.require_auth().await?;
    state
        .host_key_repo
        .get(&host, port)
        .await
        .map_err(AppError::from)
}
