mod common;

use sqlx::Row;

#[tokio::test]
async fn new_db_has_sign_count_column() {
    let pool = common::setup_test_db().await;

    // 新库迁移完成后 passkeys 表应有 sign_count 列
    let has_sign_count: bool = sqlx::query_scalar(
        "SELECT COUNT(*) > 0 FROM pragma_table_info('passkeys') WHERE name = 'sign_count'",
    )
    .fetch_one(&pool)
    .await
    .expect("pragma query failed");

    assert!(has_sign_count, "new db should have sign_count column");
}

#[tokio::test]
async fn new_db_has_no_counter_column() {
    let pool = common::setup_test_db().await;

    let has_counter: bool = sqlx::query_scalar(
        "SELECT COUNT(*) > 0 FROM pragma_table_info('passkeys') WHERE name = 'counter'",
    )
    .fetch_one(&pool)
    .await
    .expect("pragma query failed");

    assert!(!has_counter, "new db must not have legacy counter column");
}

#[tokio::test]
async fn v7_migration_renames_counter_column_and_migrates_data() {
    // 模拟旧库：手动创建带 counter 列的 passkeys 表
    let pool = sqlx::SqlitePool::connect(":memory:")
        .await
        .expect("failed to open in-memory SQLite");

    // 运行 v1-v6 基础结构（不含 v7）
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS _migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    // 模拟旧版 passkeys 表（使用 counter 列名）
    sqlx::query(
        "CREATE TABLE users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            hashed_password TEXT NOT NULL,
            two_factor_secret TEXT,
            two_factor_enabled INTEGER NOT NULL DEFAULT 0
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "CREATE TABLE passkeys (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            credential_id TEXT NOT NULL UNIQUE,
            public_key TEXT NOT NULL,
            counter INTEGER NOT NULL DEFAULT 0,
            transports TEXT,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            last_used_at TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    // 插入测试数据
    sqlx::query("INSERT INTO users (username, hashed_password) VALUES ('test', 'hash')")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(
        "INSERT INTO passkeys (user_id, credential_id, public_key, counter, name) VALUES (1, 'cred1', 'pk1', 42, 'key1')",
    )
    .execute(&pool)
    .await
    .unwrap();

    // 标记 v1-v4 为已应用（使应用程序迁移逻辑跳过它们）
    for (v, n) in [
        (1i64, "create_users"),
        (2, "create_connections"),
        (3, "create_settings"),
        (4, "add_passkeys"),
    ] {
        sqlx::query("INSERT INTO _migrations (version, name) VALUES (?, ?)")
            .bind(v)
            .bind(n)
            .execute(&pool)
            .await
            .unwrap();
    }

    // 还需要 settings 和 connection 表才能让后续迁移不报错
    sqlx::query("CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("CREATE TABLE IF NOT EXISTS connections (id INTEGER PRIMARY KEY)")
        .execute(&pool)
        .await
        .unwrap();

    // 标记 v5, v6 已应用（跳过需要真实表结构的迁移）
    for (v, n) in [
        (5i64, "add_desktop_options"),
        (6, "add_server_management_fields"),
    ] {
        sqlx::query("INSERT INTO _migrations (version, name) VALUES (?, ?)")
            .bind(v)
            .bind(n)
            .execute(&pool)
            .await
            .unwrap();
    }

    // 现在运行完整迁移（仅 v7 会被执行）
    storage_sqlite::migrations::run(&pool)
        .await
        .expect("migration v7 should succeed");

    // 验证 counter 已改名为 sign_count
    let has_sign_count: bool = sqlx::query_scalar(
        "SELECT COUNT(*) > 0 FROM pragma_table_info('passkeys') WHERE name = 'sign_count'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(has_sign_count, "after v7, sign_count column must exist");

    let has_counter: bool = sqlx::query_scalar(
        "SELECT COUNT(*) > 0 FROM pragma_table_info('passkeys') WHERE name = 'counter'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(!has_counter, "after v7, counter column must be gone");

    // 验证数据已迁移
    let row = sqlx::query("SELECT sign_count FROM passkeys WHERE credential_id = 'cred1'")
        .fetch_one(&pool)
        .await
        .unwrap();
    let sign_count: i64 = row.get(0);
    assert_eq!(
        sign_count, 42,
        "sign_count data must be preserved after rename"
    );
}
