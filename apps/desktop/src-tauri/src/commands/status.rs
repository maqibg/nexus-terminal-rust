use api_contract::error::AppError;
use serde::Serialize;
use tauri::State;

use crate::state::AppState;
use crate::status_monitor::{collect_status_once, StatusUpdatePayload};

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[tauri::command]
pub async fn get_backend_health() -> Result<HealthResponse, String> {
    Ok(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[tauri::command]
pub async fn get_connection_runtime_status(
    state: State<'_, AppState>,
    connection_id: Option<i64>,
    session_id: Option<String>,
) -> Result<StatusUpdatePayload, AppError> {
    state.auth.require_auth().await?;

    let target_session_id = if let Some(sid) = session_id {
        sid
    } else {
        let cid = connection_id.ok_or_else(|| {
            AppError::Validation("session_id or connection_id is required".into())
        })?;

        state
            .ssh_manager
            .find_session_by_connection_id(cid)
            .await
            .ok_or_else(|| {
                AppError::Validation("no active SSH session for the specified connection".into())
            })?
    };

    collect_status_once(&state.ssh_manager, &target_session_id)
        .await
        .map_err(AppError::Ssh)
}
