//! Database migrations.

use sqlx::SqlitePool;
use tracing::{info, warn};

/// 解析旧 settings 键 "ssh.known_hosts.HOST:PORT" 为 (host, port)。
fn parse_legacy_known_hosts_key(key: &str) -> Option<(String, u16)> {
    const PREFIX: &str = "ssh.known_hosts.";
    let endpoint = key.strip_prefix(PREFIX)?;
    let (host, port_raw) = endpoint.rsplit_once(':')?;
    if host.is_empty() {
        return None;
    }
    let port = port_raw.parse::<u16>().ok()?;
    Some((host.to_string(), port))
}

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
        (
            6,
            "add_server_management_fields",
            include_str!("sql/006_add_server_management_fields.sql"),
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

    // v7: 修复 passkeys 表列名漂移（counter → sign_count）
    // 仅当表中实际存在 counter 列时才重建，否则仅标记为已应用
    if !applied.contains(&7i64) {
        let has_counter: bool = sqlx::query_scalar(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('passkeys') WHERE name = 'counter'",
        )
        .fetch_one(pool)
        .await?;

        if has_counter {
            info!("Applying migration 7: fix_passkeys_sign_count (table rebuild)");
            let mut tx = pool.begin().await?;
            let sql = include_str!("sql/007_fix_passkeys.sql");
            for statement in sql.split(';').filter(|s| !s.trim().is_empty()) {
                sqlx::query(statement.trim()).execute(&mut *tx).await?;
            }
            tx.commit().await?;
        } else {
            info!("Migration 7: passkeys already uses sign_count, no rebuild needed");
        }

        sqlx::query("INSERT INTO _migrations (version, name) VALUES (?, ?)")
            .bind(7i64)
            .bind("fix_passkeys_sign_count")
            .execute(pool)
            .await?;
    }

    info!("Migrations complete. {} total.", migrations.len() + 1);

    // v8: 新建 ssh_known_hosts 专用表，并从 settings 回填历史数据
    if !applied.contains(&8i64) {
        info!("Applying migration 8: create_ssh_known_hosts");
        let mut tx = pool.begin().await?;

        let sql = include_str!("sql/008_create_ssh_known_hosts.sql");
        for statement in sql.split(';').filter(|s| !s.trim().is_empty()) {
            sqlx::query(statement.trim()).execute(&mut *tx).await?;
        }

        // 从 settings 回填旧的 known_hosts 数据
        let legacy_rows: Vec<(String, String)> =
            sqlx::query_as("SELECT key, value FROM settings WHERE key LIKE 'ssh.known_hosts.%'")
                .fetch_all(&mut *tx)
                .await?;

        for (legacy_key, fingerprint) in legacy_rows {
            let Some((host, port)) = parse_legacy_known_hosts_key(&legacy_key) else {
                warn!(
                    key = %legacy_key,
                    "Skipping malformed ssh known_hosts key during migration v8"
                );
                continue;
            };

            sqlx::query(
                "INSERT INTO ssh_known_hosts (host, port, fingerprint) VALUES (?, ?, ?)
                 ON CONFLICT(host, port)
                 DO UPDATE SET fingerprint = excluded.fingerprint, updated_at = datetime('now')",
            )
            .bind(&host)
            .bind(i64::from(port))
            .bind(&fingerprint)
            .execute(&mut *tx)
            .await?;

            sqlx::query("DELETE FROM settings WHERE key = ?")
                .bind(&legacy_key)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        sqlx::query("INSERT INTO _migrations (version, name) VALUES (?, ?)")
            .bind(8i64)
            .bind("create_ssh_known_hosts")
            .execute(pool)
            .await?;
    }

    Ok(())
}
