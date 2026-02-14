//! Settings domain models.

use serde::{Deserialize, Serialize};

/// Key-value setting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

/// Terminal theme entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalTheme {
    pub id: i64,
    pub name: String,
    pub theme_type: String,
    pub background: Option<String>,
    pub foreground: Option<String>,
    pub cursor: Option<String>,
    pub cursor_accent: Option<String>,
    pub selection_background: Option<String>,
    pub selection_foreground: Option<String>,
    pub selection_inactive_background: Option<String>,
    pub black: Option<String>,
    pub red: Option<String>,
    pub green: Option<String>,
    pub yellow: Option<String>,
    pub blue: Option<String>,
    pub magenta: Option<String>,
    pub cyan: Option<String>,
    pub white: Option<String>,
    pub bright_black: Option<String>,
    pub bright_red: Option<String>,
    pub bright_green: Option<String>,
    pub bright_yellow: Option<String>,
    pub bright_blue: Option<String>,
    pub bright_magenta: Option<String>,
    pub bright_cyan: Option<String>,
    pub bright_white: Option<String>,
}

/// Notification channel config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    pub id: i64,
    pub channel_type: String,
    pub name: String,
    pub enabled: bool,
    pub config: String,
    pub enabled_events: String,
}
