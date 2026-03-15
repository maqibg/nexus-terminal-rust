use api_contract::error::{AppError, CmdResult};
use serde::Deserialize;
use tauri::State;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct LocalTerminalOpenRequest {
    pub shell: Option<String>,
}

#[derive(Deserialize)]
pub struct LocalTerminalWriteRequest {
    pub session_id: String,
    pub data: String,
}

#[derive(Deserialize)]
pub struct LocalTerminalCloseRequest {
    pub session_id: String,
}

#[derive(Deserialize)]
pub struct LocalTerminalTakeOutputBacklogRequest {
    pub session_id: String,
}

#[derive(Deserialize)]
pub struct LocalTerminalResizeRequest {
    pub session_id: String,
    pub cols: u32,
    pub rows: u32,
}

#[tauri::command]
pub async fn local_terminal_open(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    req: LocalTerminalOpenRequest,
) -> CmdResult<String> {
    state.auth.require_auth().await?;

    let session_id = uuid::Uuid::new_v4().to_string();
    state
        .local_terminal_manager
        .open_session(session_id.clone(), req.shell, app_handle)
        .await?;

    Ok(session_id)
}

#[tauri::command]
pub async fn local_terminal_write(
    state: State<'_, AppState>,
    req: LocalTerminalWriteRequest,
) -> CmdResult<()> {
    state.auth.require_auth().await?;

    use base64::{engine::general_purpose::STANDARD as B64, Engine};
    let data = B64
        .decode(&req.data)
        .map_err(|e| AppError::Validation(format!("invalid base64: {e}")))?;

    state
        .local_terminal_manager
        .write(&req.session_id, data)
        .await
}

#[tauri::command]
pub async fn local_terminal_resize(
    state: State<'_, AppState>,
    req: LocalTerminalResizeRequest,
) -> CmdResult<()> {
    state.auth.require_auth().await?;
    let _ = (req.session_id, req.cols, req.rows);
    Ok(())
}

#[tauri::command]
pub async fn local_terminal_close(
    state: State<'_, AppState>,
    req: LocalTerminalCloseRequest,
) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    Ok(state.local_terminal_manager.close(&req.session_id).await)
}

#[tauri::command]
pub async fn local_terminal_session_list(
    state: State<'_, AppState>,
) -> CmdResult<Vec<crate::local_terminal::LocalTerminalSessionInfo>> {
    state.auth.require_auth().await?;
    Ok(state.local_terminal_manager.list())
}

#[tauri::command]
pub async fn local_terminal_take_output_backlog(
    state: State<'_, AppState>,
    req: LocalTerminalTakeOutputBacklogRequest,
) -> CmdResult<Vec<crate::local_terminal::LocalTerminalOutputChunk>> {
    state.auth.require_auth().await?;
    Ok(state
        .local_terminal_manager
        .take_output_backlog(&req.session_id)
        .await)
}
