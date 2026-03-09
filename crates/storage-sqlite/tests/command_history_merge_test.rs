mod common;

use history_core::repository::HistoryRepository;
use storage_sqlite::SqliteHistoryRepo;

#[tokio::test]
async fn add_command_merges_same_command_and_updates_latest_context() {
    let pool = common::setup_test_db().await;
    let repo = SqliteHistoryRepo::new(pool.clone());

    let first_id = repo
        .add_command("ls -la", Some("session-a"), Some(1))
        .await
        .expect("first insert should succeed");

    let second_id = repo
        .add_command("pwd", Some("session-b"), Some(2))
        .await
        .expect("second insert should succeed");

    let merged_id = repo
        .add_command("ls -la", Some("session-c"), Some(3))
        .await
        .expect("duplicate command should merge");

    assert_eq!(merged_id, first_id, "duplicate command should reuse the latest row");

    let ls_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM command_history WHERE command = ?")
        .bind("ls -la")
        .fetch_one(&pool)
        .await
        .expect("count query should succeed");
    assert_eq!(ls_count, 1, "same command should only keep one row");

    let items = repo
        .list_command_history(20, 0)
        .await
        .expect("history list should succeed");

    assert_eq!(items.len(), 2, "history list should be deduplicated");

    let merged_entry = items
        .iter()
        .find(|item| item.command == "ls -la")
        .expect("merged command should exist");
    assert_eq!(merged_entry.id, first_id);
    assert_eq!(merged_entry.session_id.as_deref(), Some("session-c"));
    assert_eq!(merged_entry.connection_id, Some(3));

    let pwd_entry = items
        .iter()
        .find(|item| item.command == "pwd")
        .expect("other command should still exist");
    assert_eq!(pwd_entry.id, second_id);
}
