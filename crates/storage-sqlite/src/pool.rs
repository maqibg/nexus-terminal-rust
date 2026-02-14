//! SQLite connection pool initialization.

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::Path;
use std::str::FromStr;
use tracing::info;

/// Shared storage handle wrapping the SQLite pool.
#[derive(Debug, Clone)]
pub struct SqliteStorage {
    pub pool: SqlitePool,
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

    info!("SQLite pool initialized successfully");
    Ok(SqliteStorage { pool })
}
