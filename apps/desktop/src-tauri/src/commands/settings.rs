//! Settings Tauri commands.

use api_contract::error::{AppError, CmdResult};
use serde::Deserialize;
use settings_core::model::{NotificationChannel, Setting, TerminalTheme};
use settings_core::repository::SettingsRepository;
use tauri::State;

use crate::state::AppState;

/// 内部保护键前缀：禁止通过 settings_set 直接写入。
/// known_hosts 已迁移到专用表，不再需要此前缀保护。
const INTERNAL_KEY_PREFIXES: &[&str] = &["auth.", "ai."];

#[derive(Deserialize)]
pub struct SetSettingRequest {
    pub key: String,
    pub value: String,
}

#[tauri::command]
pub async fn settings_get_all(state: State<'_, AppState>) -> CmdResult<Vec<Setting>> {
    state.auth.require_auth().await?;
    state
        .settings_repo
        .get_all_settings()
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn settings_set(state: State<'_, AppState>, req: SetSettingRequest) -> CmdResult<()> {
    state.auth.require_auth().await?;
    if INTERNAL_KEY_PREFIXES
        .iter()
        .any(|prefix| req.key.starts_with(prefix))
    {
        return Err(AppError::Forbidden(format!(
            "key '{}' is reserved for internal use",
            req.key
        )));
    }
    state
        .settings_repo
        .set_setting(&req.key, &req.value)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn appearance_get_all(state: State<'_, AppState>) -> CmdResult<Vec<Setting>> {
    state.auth.require_auth().await?;
    state
        .settings_repo
        .get_all_appearance()
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn appearance_set(state: State<'_, AppState>, req: SetSettingRequest) -> CmdResult<()> {
    state.auth.require_auth().await?;
    state
        .settings_repo
        .set_appearance(&req.key, &req.value)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn theme_list(state: State<'_, AppState>) -> CmdResult<Vec<TerminalTheme>> {
    state.auth.require_auth().await?;
    state
        .settings_repo
        .list_themes()
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn theme_get(state: State<'_, AppState>, id: i64) -> CmdResult<TerminalTheme> {
    state.auth.require_auth().await?;
    state
        .settings_repo
        .get_theme(id)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::NotFound("theme not found".into()))
}

#[tauri::command]
pub async fn theme_create(state: State<'_, AppState>, theme: TerminalTheme) -> CmdResult<i64> {
    state.auth.require_auth().await?;
    state
        .settings_repo
        .create_theme(&theme)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn theme_update(state: State<'_, AppState>, theme: TerminalTheme) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    state
        .settings_repo
        .update_theme(&theme)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn theme_delete(state: State<'_, AppState>, id: i64) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    state
        .settings_repo
        .delete_theme(id)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn notification_channel_list(
    state: State<'_, AppState>,
) -> CmdResult<Vec<NotificationChannel>> {
    state.auth.require_auth().await?;
    state
        .settings_repo
        .list_notification_channels()
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn notification_channel_create(
    state: State<'_, AppState>,
    channel: NotificationChannel,
) -> CmdResult<i64> {
    state.auth.require_auth().await?;
    state
        .settings_repo
        .create_notification_channel(&channel)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn notification_channel_update(
    state: State<'_, AppState>,
    channel: NotificationChannel,
) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    state
        .settings_repo
        .update_notification_channel(&channel)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn notification_channel_delete(state: State<'_, AppState>, id: i64) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    state
        .settings_repo
        .delete_notification_channel(id)
        .await
        .map_err(AppError::from)
}
