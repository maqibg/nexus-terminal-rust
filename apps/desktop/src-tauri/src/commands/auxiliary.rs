//! Audit, History, QuickCommand Tauri commands.

use api_contract::error::{AppError, CmdResult};
use audit_core::model::AuditLog;
use audit_core::repository::AuditRepository;
use history_core::model::*;
use history_core::repository::HistoryRepository;
use quick_command_core::model::*;
use quick_command_core::repository::QuickCommandRepository;
use serde::Deserialize;
use tauri::State;

use crate::state::AppState;

// ── Audit ──

#[derive(Deserialize)]
pub struct PaginationRequest {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[tauri::command]
pub async fn audit_log_list(
    state: State<'_, AppState>,
    req: PaginationRequest,
) -> CmdResult<Vec<AuditLog>> {
    state.auth.require_auth().await?;
    state
        .audit_repo
        .list_logs(req.limit.unwrap_or(100), req.offset.unwrap_or(0))
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn audit_log_count(state: State<'_, AppState>) -> CmdResult<i64> {
    state.auth.require_auth().await?;
    state.audit_repo.count_logs().await.map_err(AppError::from)
}

#[tauri::command]
pub async fn audit_log_clear(state: State<'_, AppState>) -> CmdResult<()> {
    state.auth.require_auth().await?;
    state.audit_repo.clear_logs().await.map_err(AppError::from)
}

// ── Command History ──

#[tauri::command]
pub async fn command_history_list(
    state: State<'_, AppState>,
    req: PaginationRequest,
) -> CmdResult<Vec<CommandHistory>> {
    state.auth.require_auth().await?;
    state
        .history_repo
        .list_command_history(req.limit.unwrap_or(100), req.offset.unwrap_or(0))
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn command_history_clear(state: State<'_, AppState>) -> CmdResult<()> {
    state.auth.require_auth().await?;
    state
        .history_repo
        .clear_command_history()
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn command_history_delete(state: State<'_, AppState>, id: i64) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    state
        .history_repo
        .delete_command_history_entry(id)
        .await
        .map_err(AppError::from)
}

// ── Command History Add ──

#[derive(Deserialize)]
pub struct CommandHistoryAddRequest {
    pub command: String,
    pub session_id: Option<String>,
    pub connection_id: Option<i64>,
}

#[tauri::command]
pub async fn command_history_add(
    state: State<'_, AppState>,
    req: CommandHistoryAddRequest,
) -> CmdResult<i64> {
    state.auth.require_auth().await?;
    state
        .history_repo
        .add_command(&req.command, req.session_id.as_deref(), req.connection_id)
        .await
        .map_err(AppError::from)
}

// ── Path History ──

#[derive(Deserialize)]
pub struct PathHistoryListRequest {
    pub connection_id: Option<i64>,
    pub limit: Option<i64>,
}

#[tauri::command]
pub async fn path_history_list(
    state: State<'_, AppState>,
    req: PathHistoryListRequest,
) -> CmdResult<Vec<PathHistory>> {
    state.auth.require_auth().await?;
    state
        .history_repo
        .list_path_history(req.connection_id, req.limit.unwrap_or(50))
        .await
        .map_err(AppError::from)
}

#[derive(Deserialize)]
pub struct PathHistoryAddRequest {
    pub path: String,
    pub connection_id: Option<i64>,
}

#[tauri::command]
pub async fn path_history_add(
    state: State<'_, AppState>,
    req: PathHistoryAddRequest,
) -> CmdResult<i64> {
    state.auth.require_auth().await?;
    state
        .history_repo
        .add_path(&req.path, req.connection_id)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn path_history_clear(state: State<'_, AppState>) -> CmdResult<()> {
    state.auth.require_auth().await?;
    state
        .history_repo
        .clear_path_history()
        .await
        .map_err(AppError::from)
}

// ── Favorite Paths ──

#[derive(Deserialize)]
pub struct FavoritePathRequest {
    pub name: String,
    pub path: String,
    pub connection_id: Option<i64>,
}

#[tauri::command]
pub async fn favorite_path_list(
    state: State<'_, AppState>,
    connection_id: Option<i64>,
) -> CmdResult<Vec<FavoritePath>> {
    state.auth.require_auth().await?;
    state
        .history_repo
        .list_favorite_paths(connection_id)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn favorite_path_create(
    state: State<'_, AppState>,
    req: FavoritePathRequest,
) -> CmdResult<i64> {
    state.auth.require_auth().await?;
    state
        .history_repo
        .create_favorite_path(&req.name, &req.path, req.connection_id)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn favorite_path_delete(state: State<'_, AppState>, id: i64) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    state
        .history_repo
        .delete_favorite_path(id)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn favorite_path_mark_used(state: State<'_, AppState>, id: i64) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    state
        .history_repo
        .mark_favorite_path_used(id)
        .await
        .map_err(AppError::from)
}

// ── Favorite Path Update ──

#[derive(Deserialize)]
pub struct FavoritePathUpdateRequest {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub connection_id: Option<i64>,
}

#[tauri::command]
pub async fn favorite_path_update(
    state: State<'_, AppState>,
    req: FavoritePathUpdateRequest,
) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    state
        .history_repo
        .update_favorite_path(req.id, &req.name, &req.path, req.connection_id)
        .await
        .map_err(AppError::from)
}

// ── Quick Commands ──

#[tauri::command]
pub async fn quick_command_list(state: State<'_, AppState>) -> CmdResult<Vec<QuickCommand>> {
    state.auth.require_auth().await?;
    state.qc_repo.list().await.map_err(AppError::from)
}

#[tauri::command]
pub async fn quick_command_get(state: State<'_, AppState>, id: i64) -> CmdResult<QuickCommand> {
    state.auth.require_auth().await?;
    state
        .qc_repo
        .get(id)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::NotFound("quick command not found".into()))
}

#[tauri::command]
pub async fn quick_command_create(
    state: State<'_, AppState>,
    input: QuickCommandInput,
) -> CmdResult<i64> {
    state.auth.require_auth().await?;
    state.qc_repo.create(&input).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn quick_command_update(
    state: State<'_, AppState>,
    id: i64,
    input: QuickCommandInput,
) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    state
        .qc_repo
        .update(id, &input)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn quick_command_delete(state: State<'_, AppState>, id: i64) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    state.qc_repo.delete(id).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn quick_command_use(state: State<'_, AppState>, id: i64) -> CmdResult<()> {
    state.auth.require_auth().await?;
    state
        .qc_repo
        .increment_usage(id)
        .await
        .map_err(AppError::from)
}

// ── Quick Command Tags ──

#[tauri::command]
pub async fn quick_command_tag_list(state: State<'_, AppState>) -> CmdResult<Vec<QuickCommandTag>> {
    state.auth.require_auth().await?;
    state.qc_repo.list_tags().await.map_err(AppError::from)
}

#[derive(Deserialize)]
pub struct QcTagCreateRequest {
    pub name: String,
}

#[tauri::command]
pub async fn quick_command_tag_create(
    state: State<'_, AppState>,
    req: QcTagCreateRequest,
) -> CmdResult<i64> {
    state.auth.require_auth().await?;
    state
        .qc_repo
        .create_tag(&req.name)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn quick_command_tag_delete(state: State<'_, AppState>, id: i64) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    state.qc_repo.delete_tag(id).await.map_err(AppError::from)
}

#[derive(Deserialize)]
pub struct QcBulkAssignTagRequest {
    pub tag_id: i64,
    pub quick_command_ids: Vec<i64>,
}

#[tauri::command]
pub async fn quick_command_bulk_assign_tag(
    state: State<'_, AppState>,
    req: QcBulkAssignTagRequest,
) -> CmdResult<()> {
    state.auth.require_auth().await?;
    state
        .qc_repo
        .bulk_assign_tag(req.tag_id, &req.quick_command_ids)
        .await
        .map_err(AppError::from)
}
