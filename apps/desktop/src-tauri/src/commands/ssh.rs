//! SSH/SFTP Tauri commands.

use api_contract::error::AppError;
use connection_core::repository::ConnectionRepository;
use serde::Deserialize;
use tauri::State;

use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

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

    let creds = ssh_core::session::SshCredentials {
        host: conn.host,
        port: conn.port as u16,
        username: conn.username,
        auth,
    };

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
        )
        .await
        .map_err(AppError::Ssh)?;

    state
        .status_monitor
        .start_session(session_id.clone(), state.ssh_manager.clone(), app_handle)
        .await;

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
