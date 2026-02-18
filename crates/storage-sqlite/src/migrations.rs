//! Database migrations.

use sqlx::SqlitePool;
use tracing::info;

/// Run all migrations in order.
pub async fn run(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    info!("Running database migrations...");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS _migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(pool)
    .await?;

    let applied: Vec<i64> = sqlx::query_scalar("SELECT version FROM _migrations ORDER BY version")
        .fetch_all(pool)
        .await?;

    let migrations: Vec<(i64, &str, &str)> = vec![
        (1, "create_users", include_str!("sql/001_create_users.sql")),
        (
            2,
            "create_connections",
            include_str!("sql/002_create_connections.sql"),
        ),
        (
            3,
            "create_settings",
            include_str!("sql/003_create_settings.sql"),
        ),
        (4, "add_passkeys", include_str!("sql/004_add_passkeys.sql")),
        (
            5,
            "add_desktop_options",
            include_str!("sql/005_add_desktop_options.sql"),
        ),
    ];

    for (version, name, sql) in &migrations {
        if !applied.contains(version) {
            info!("Applying migration {}: {}", version, name);
            for statement in sql.split(';').filter(|s| !s.trim().is_empty()) {
                sqlx::query(statement.trim()).execute(pool).await?;
            }
            sqlx::query("INSERT INTO _migrations (version, name) VALUES (?, ?)")
                .bind(version)
                .bind(name)
                .execute(pool)
                .await?;
        }
    }

    info!("Migrations complete. {} total.", migrations.len());
    Ok(())
}
