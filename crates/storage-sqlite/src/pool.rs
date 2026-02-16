//! SQLite connection pool initialization.

use serde::Deserialize;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::collections::HashSet;
use std::path::Path;
use std::str::FromStr;
use tracing::{info, warn};

const BUILTIN_TERMINAL_THEME_PRESETS_JSON: &str =
    include_str!("../../../apps/desktop/frontend/src/assets/terminal-theme-presets.json");

/// Shared storage handle wrapping the SQLite pool.
#[derive(Debug, Clone)]
pub struct SqliteStorage {
    pub pool: SqlitePool,
}

#[derive(Debug, Deserialize)]
struct BuiltinTerminalThemePreset {
    name: String,
    #[serde(rename = "themeData")]
    theme_data: BuiltinTerminalThemeData,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BuiltinTerminalThemeData {
    background: Option<String>,
    foreground: Option<String>,
    cursor: Option<String>,
    cursor_accent: Option<String>,
    selection_background: Option<String>,
    selection_foreground: Option<String>,
    selection_inactive_background: Option<String>,
    black: Option<String>,
    red: Option<String>,
    green: Option<String>,
    yellow: Option<String>,
    blue: Option<String>,
    magenta: Option<String>,
    cyan: Option<String>,
    white: Option<String>,
    bright_black: Option<String>,
    bright_red: Option<String>,
    bright_green: Option<String>,
    bright_yellow: Option<String>,
    bright_blue: Option<String>,
    bright_magenta: Option<String>,
    bright_cyan: Option<String>,
    bright_white: Option<String>,
}

async fn ensure_builtin_terminal_theme_presets(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let presets: Vec<BuiltinTerminalThemePreset> = serde_json::from_str(BUILTIN_TERMINAL_THEME_PRESETS_JSON)
        .map_err(|error| {
            sqlx::Error::Protocol(
                format!("failed to parse builtin terminal theme presets: {error}").into(),
            )
        })?;

    if presets.is_empty() {
        return Ok(());
    }

    let existing_rows = sqlx::query_as::<_, (String,)>(
        "SELECT name FROM terminal_themes WHERE theme_type = 'preset'",
    )
    .fetch_all(pool)
    .await?;

    let mut existing_names: HashSet<String> = existing_rows.into_iter().map(|(name,)| name).collect();
    let mut inserted_count = 0usize;

    for preset in presets {
        if existing_names.contains(&preset.name) {
            continue;
        }

        let result = sqlx::query(
            "INSERT INTO terminal_themes (name, theme_type, background, foreground, cursor, cursor_accent, selection_background, selection_foreground, selection_inactive_background, black, red, green, yellow, blue, magenta, cyan, white, bright_black, bright_red, bright_green, bright_yellow, bright_blue, bright_magenta, bright_cyan, bright_white) VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        )
        .bind(&preset.name)
        .bind("preset")
        .bind(&preset.theme_data.background)
        .bind(&preset.theme_data.foreground)
        .bind(&preset.theme_data.cursor)
        .bind(&preset.theme_data.cursor_accent)
        .bind(&preset.theme_data.selection_background)
        .bind(&preset.theme_data.selection_foreground)
        .bind(&preset.theme_data.selection_inactive_background)
        .bind(&preset.theme_data.black)
        .bind(&preset.theme_data.red)
        .bind(&preset.theme_data.green)
        .bind(&preset.theme_data.yellow)
        .bind(&preset.theme_data.blue)
        .bind(&preset.theme_data.magenta)
        .bind(&preset.theme_data.cyan)
        .bind(&preset.theme_data.white)
        .bind(&preset.theme_data.bright_black)
        .bind(&preset.theme_data.bright_red)
        .bind(&preset.theme_data.bright_green)
        .bind(&preset.theme_data.bright_yellow)
        .bind(&preset.theme_data.bright_blue)
        .bind(&preset.theme_data.bright_magenta)
        .bind(&preset.theme_data.bright_cyan)
        .bind(&preset.theme_data.bright_white)
        .execute(pool)
        .await;

        match result {
            Ok(_) => {
                existing_names.insert(preset.name);
                inserted_count += 1;
            }
            Err(error) => {
                warn!(theme_name = %preset.name, %error, "failed to insert builtin terminal preset");
            }
        }
    }

    if inserted_count > 0 {
        info!(inserted_count, "seeded builtin terminal theme presets");
    }

    Ok(())
}

/// Initialize SQLite pool and run migrations.
pub async fn init_pool(db_path: &Path) -> Result<SqliteStorage, sqlx::Error> {
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
    info!("Initializing SQLite pool: {}", db_url);

    let options = SqliteConnectOptions::from_str(&db_url)?
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(30))
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    crate::migrations::run(&pool).await?;
    ensure_builtin_terminal_theme_presets(&pool).await?;

    info!("SQLite pool initialized successfully");
    Ok(SqliteStorage { pool })
}
