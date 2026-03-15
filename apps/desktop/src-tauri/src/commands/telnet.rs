use api_contract::error::{AppError, CmdResult};
use serde::Deserialize;
use tauri::State;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct TelnetConnectRequest {
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize)]
pub struct TelnetWriteRequest {
    pub session_id: String,
    pub data: String,
}

#[derive(Deserialize)]
pub struct TelnetCloseRequest {
    pub session_id: String,
}

#[derive(Deserialize)]
pub struct TelnetTakeOutputBacklogRequest {
    pub session_id: String,
}

#[tauri::command]
pub async fn telnet_connect(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    req: TelnetConnectRequest,
) -> CmdResult<String> {
    state.auth.require_auth().await?;

    let host = req.host.trim();
    if host.is_empty() {
        return Err(AppError::Validation("host is required".into()));
    }
    if req.port == 0 {
        return Err(AppError::Validation("port is required".into()));
    }

    let session_id = uuid::Uuid::new_v4().to_string();
    state
        .telnet_manager
        .connect(session_id.clone(), host.to_string(), req.port, app_handle)
        .await
        .map_err(AppError::Internal)?;

    Ok(session_id)
}

#[tauri::command]
pub async fn telnet_write(state: State<'_, AppState>, req: TelnetWriteRequest) -> CmdResult<()> {
    state.auth.require_auth().await?;

    use base64::{engine::general_purpose::STANDARD as B64, Engine};
    let data = B64
        .decode(&req.data)
        .map_err(|e| AppError::Validation(format!("invalid base64: {e}")))?;

    state
        .telnet_manager
        .write(&req.session_id, data)
        .await
        .map_err(AppError::Internal)
}

#[tauri::command]
pub async fn telnet_close(state: State<'_, AppState>, req: TelnetCloseRequest) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    Ok(state.telnet_manager.close(&req.session_id).await)
}

#[tauri::command]
pub async fn telnet_session_list(
    state: State<'_, AppState>,
) -> CmdResult<Vec<crate::telnet::TelnetSessionInfo>> {
    state.auth.require_auth().await?;
    Ok(state.telnet_manager.list())
}

#[tauri::command]
pub async fn telnet_take_output_backlog(
    state: State<'_, AppState>,
    req: TelnetTakeOutputBacklogRequest,
) -> CmdResult<Vec<crate::telnet::TelnetOutputChunk>> {
    state.auth.require_auth().await?;
    Ok(state.telnet_manager.take_output_backlog(&req.session_id).await)
}

