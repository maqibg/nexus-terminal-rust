//! Unified error type for Tauri commands.

use serde::Serialize;
use thiserror::Error;

/// Application-level error returned from Tauri commands.
#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    #[error("not authenticated")]
    Unauthorized,

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("database error: {0}")]
    Database(String),

    #[error("crypto error: {0}")]
    Crypto(String),

    #[error("ssh error: {0}")]
    Ssh(String),

    #[error("sftp error: {0}")]
    Sftp(String),

    #[error("setup required")]
    SetupRequired,
}

/// Result type alias for Tauri commands.
pub type CmdResult<T> = Result<T, AppError>;
