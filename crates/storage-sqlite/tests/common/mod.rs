use sqlx::SqlitePool;

/// 创建内存 SQLite 连接池并运行全量迁移，用于集成测试。
pub async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:")
        .await
        .expect("failed to open in-memory SQLite");

    storage_sqlite::migrations::run(&pool)
        .await
        .expect("failed to run migrations on test db");

    pool
}
