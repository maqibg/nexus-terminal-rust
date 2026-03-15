use api_contract::error::{AppError, CmdResult};
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn crypto_encrypt(state: State<'_, AppState>, plaintext: String) -> CmdResult<String> {
    state.auth.require_auth().await?;
    let trimmed = plaintext.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("plaintext is empty".into()));
    }
    state
        .crypto
        .encrypt(trimmed)
        .map_err(|e| AppError::Crypto(e.to_string()))
}

