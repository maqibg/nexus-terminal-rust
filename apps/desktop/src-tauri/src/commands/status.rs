use api_contract::error::AppError;
use serde::Serialize;
use settings_core::repository::SettingsRepository;
use sqlx::Row;
use tauri::State;
use tokio::time::Duration;

use crate::state::AppState;
use crate::status_monitor::{collect_status_once, StatusUpdatePayload};

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimePathsResponse {
    pub exe_dir: String,
    pub data_dir: String,
    pub download_dir: String,
    pub temp_dir: String,
}

#[tauri::command]
pub async fn get_backend_health(state: State<'_, AppState>) -> Result<HealthResponse, String> {
    if !state.runtime_paths.data_dir.exists() {
        return Err(format!(
            "data directory not found: {}",
            state.runtime_paths.data_dir.display()
        ));
    }

    if !state.runtime_paths.download_dir.exists() {
        return Err(format!(
            "download directory not found: {}",
            state.runtime_paths.download_dir.display()
        ));
    }

    sqlx::query("SELECT 1")
        .fetch_one(&state.storage.pool)
        .await
        .map_err(|error| format!("database health check failed: {error}"))?
        .try_get::<i64, _>(0)
        .map_err(|error| format!("database health check parse failed: {error}"))?;

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

    let status_monitor_enabled = match state
        .settings_repo
        .get_setting("statusMonitorEnabled")
        .await
        .map_err(AppError::from)?
    {
        Some(raw) => raw.trim().eq_ignore_ascii_case("true") || raw.trim() == "1",
        None => true,
    };

    if !status_monitor_enabled {
        return Err(AppError::Validation("status monitor is disabled".into()));
    }

    collect_status_once(&state.ssh_manager, &target_session_id)
        .await
        .map_err(AppError::Ssh)
}

#[tauri::command]
pub async fn set_status_monitor_enabled(
    app_handle: tauri::AppHandle,
    state: State<'_, AppState>,
    enabled: bool,
) -> Result<(), AppError> {
    state.auth.require_auth().await?;

    if !enabled {
        state.status_monitor.stop_all().await;
        return Ok(());
    }

    let poll_interval_override = state
        .settings_repo
        .get_setting("statusMonitorIntervalSeconds")
        .await
        .map_err(AppError::from)?
        .and_then(|raw| raw.trim().parse::<u64>().ok())
        .filter(|seconds| *seconds >= 1)
        .map(Duration::from_secs);

    let sessions = state.ssh_manager.list_sessions().await;
    for (session_id, _, _) in sessions {
        state
            .status_monitor
            .start_session(
                session_id,
                state.ssh_manager.clone(),
                app_handle.clone(),
                poll_interval_override,
            )
            .await;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_runtime_paths(state: State<'_, AppState>) -> Result<RuntimePathsResponse, String> {
    Ok(RuntimePathsResponse {
        exe_dir: state.runtime_paths.exe_dir.to_string_lossy().to_string(),
        data_dir: state.runtime_paths.data_dir.to_string_lossy().to_string(),
        download_dir: state
            .runtime_paths
            .download_dir
            .to_string_lossy()
            .to_string(),
        temp_dir: state.runtime_paths.temp_dir.to_string_lossy().to_string(),
    })
}
