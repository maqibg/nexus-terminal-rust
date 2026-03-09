//! Auth Tauri commands.

use api_contract::error::AppError;
use serde::Deserialize;
use tauri::State;

use crate::state::AppState;
use auth_core::service::{
    AuthStatusResponse, LoginResponse, PasskeyInfo, TwoFactorSetupResponse, UserInfo,
};

type CmdResult<T> = Result<T, AppError>;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct SetupRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct TokenRequest {
    pub token: String,
}

#[derive(Deserialize)]
pub struct PasswordRequest {
    pub password: String,
}

#[tauri::command]
pub async fn auth_status(state: State<'_, AppState>) -> CmdResult<AuthStatusResponse> {
    state.auth.status().await
}

#[tauri::command]
pub async fn auth_setup(state: State<'_, AppState>, req: SetupRequest) -> CmdResult<UserInfo> {
    state.auth.setup_admin(&req.username, &req.password).await
}

#[tauri::command]
pub async fn auth_login(state: State<'_, AppState>, req: LoginRequest) -> CmdResult<LoginResponse> {
    state.auth.login(&req.username, &req.password).await
}

#[tauri::command]
pub async fn auth_verify_2fa(
    state: State<'_, AppState>,
    req: TokenRequest,
) -> CmdResult<LoginResponse> {
    state.auth.verify_login_2fa(&req.token).await
}

#[tauri::command]
pub async fn auth_logout(state: State<'_, AppState>) -> CmdResult<()> {
    state.auth.logout().await
}

#[tauri::command]
pub async fn auth_change_password(
    state: State<'_, AppState>,
    req: ChangePasswordRequest,
) -> CmdResult<()> {
    state
        .auth
        .change_password(&req.current_password, &req.new_password)
        .await
}

#[tauri::command]
pub async fn auth_setup_2fa(state: State<'_, AppState>) -> CmdResult<TwoFactorSetupResponse> {
    state.auth.setup_2fa().await
}

#[tauri::command]
pub async fn auth_verify_activate_2fa(
    state: State<'_, AppState>,
    req: TokenRequest,
) -> CmdResult<()> {
    state.auth.verify_and_activate_2fa(&req.token).await
}

#[tauri::command]
pub async fn auth_disable_2fa(state: State<'_, AppState>, req: PasswordRequest) -> CmdResult<()> {
    state.auth.disable_2fa(&req.password).await
}

// ── Passkey Commands ──

/// Finish passkey registration — accepts the full credential JSON from navigator.credentials.create()
#[derive(Deserialize)]
pub struct PasskeyRegisterFinishRequest {
    pub credential_json: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct PasskeyActionRequest {
    pub credential_id: String,
}

#[derive(Deserialize)]
pub struct PasskeyRenameRequest {
    pub credential_id: String,
    pub name: String,
}

#[tauri::command]
pub async fn passkey_list(state: State<'_, AppState>) -> CmdResult<Vec<PasskeyInfo>> {
    state.auth.passkey_list().await
}

#[tauri::command]
pub async fn passkey_register_start(state: State<'_, AppState>) -> CmdResult<String> {
    state.auth.passkey_register_start().await
}

#[tauri::command]
pub async fn passkey_register_finish(
    state: State<'_, AppState>,
    req: PasskeyRegisterFinishRequest,
) -> CmdResult<i64> {
    state
        .auth
        .passkey_register_finish(&req.credential_json, &req.name)
        .await
}

#[tauri::command]
pub async fn passkey_delete(
    state: State<'_, AppState>,
    req: PasskeyActionRequest,
) -> CmdResult<bool> {
    state.auth.passkey_delete(&req.credential_id).await
}

#[tauri::command]
pub async fn passkey_rename(
    state: State<'_, AppState>,
    req: PasskeyRenameRequest,
) -> CmdResult<bool> {
    state
        .auth
        .passkey_rename(&req.credential_id, &req.name)
        .await
}
