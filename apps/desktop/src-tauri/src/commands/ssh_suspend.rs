use api_contract::error::{AppError, CmdResult};
use std::sync::Arc;
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn ssh_suspend_list(
    state: State<'_, AppState>,
) -> CmdResult<Vec<ssh_suspend_core::SuspendedSession>> {
    state.auth.require_auth().await?;
    Ok(state.suspend_manager.list().await)
}

#[tauri::command]
pub async fn ssh_suspend(state: State<'_, AppState>, session_id: String) -> CmdResult<()> {
    state.auth.require_auth().await?;

    let active = state
        .ssh_manager
        .list_sessions()
        .await
        .into_iter()
        .find(|(id, _, _)| id == &session_id)
        .ok_or_else(|| AppError::NotFound(format!("session '{}' not found", session_id)))?;

    state
        .suspend_manager
        .suspend(&active.0, active.1, &active.2)
        .await
        .map_err(AppError::Ssh)?;
    state.status_monitor.stop_session(&active.0).await;
    state
        .ssh_manager
        .close(&active.0)
        .await
        .map_err(AppError::Ssh)
}

#[tauri::command]
pub async fn ssh_resume(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    id: String,
) -> CmdResult<String> {
    state.auth.require_auth().await?;

    let suspended = state
        .suspend_manager
        .list()
        .await
        .into_iter()
        .find(|item| item.id == id)
        .ok_or_else(|| AppError::NotFound(format!("suspended session '{}' not found", id)))?;

    let cols = 80;
    let rows = 24;
    let session_id = uuid::Uuid::new_v4().to_string();
    let creds = super::build_creds(&state, suspended.connection_id).await?;
    let host_key_store = std::sync::Arc::new(super::SettingsHostKeyStore {
        repo: state.host_key_repo.clone(),
    });

    state
        .ssh_manager
        .open_session(
            session_id.clone(),
            creds,
            suspended.connection_id,
            suspended.connection_name.clone(),
            cols,
            rows,
            app_handle.clone(),
            Some(host_key_store),
        )
        .await
        .map_err(AppError::Ssh)?;

    let buffered = state
        .suspend_manager
        .resume(&id)
        .await
        .map_err(AppError::Ssh)?;
    if !buffered.is_empty() {
        use base64::{engine::general_purpose::STANDARD as B64, Engine};
        use tauri::Emitter;

        let chunk = ssh_core::manager::SshOutputChunk {
            seq: 0,
            stream: "stdout",
            data: Arc::new(B64.encode(buffered)),
        };
        app_handle
            .emit(&format!("ssh:output:{session_id}"), &chunk)
            .map_err(|error| AppError::Internal(error.to_string()))?;
    }

    Ok(session_id)
}

#[tauri::command]
pub async fn ssh_suspend_terminate(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    state.auth.require_auth().await?;
    state
        .suspend_manager
        .terminate(&id)
        .await
        .map_err(AppError::Ssh)
}
