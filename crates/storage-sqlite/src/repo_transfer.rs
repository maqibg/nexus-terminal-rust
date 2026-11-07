//! Persistence for transfer tasks, so an interrupted upload can be resumed
//! after the application restarts.

use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct StoredTransferTask {
    pub id: String,
    pub kind: String,
    pub file_name: String,
    pub total_bytes: i64,
    pub transferred_bytes: i64,
    pub status: String,
    pub error: Option<String>,
    pub session_id: Option<String>,
    pub source_path: Option<String>,
    pub target_path: Option<String>,
}

#[derive(Clone)]
pub struct SqliteTransferRepo {
    pool: SqlitePool,
}

impl SqliteTransferRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn upsert(&self, task: &StoredTransferTask) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO transfer_tasks
                (id, kind, file_name, total_bytes, transferred_bytes, status, error,
                 session_id, source_path, target_path, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
             ON CONFLICT(id) DO UPDATE SET
                file_name = excluded.file_name,
                total_bytes = excluded.total_bytes,
                transferred_bytes = excluded.transferred_bytes,
                status = excluded.status,
                error = excluded.error,
                session_id = excluded.session_id,
                source_path = excluded.source_path,
                target_path = excluded.target_path,
                updated_at = datetime('now')",
        )
        .bind(&task.id)
        .bind(&task.kind)
        .bind(&task.file_name)
        .bind(task.total_bytes)
        .bind(task.transferred_bytes)
        .bind(&task.status)
        .bind(&task.error)
        .bind(&task.session_id)
        .bind(&task.source_path)
        .bind(&task.target_path)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Tasks that were interrupted: still unfinished, with a usable source and
    /// target so the user can pick them up again.
    pub async fn list_resumable(&self) -> Result<Vec<StoredTransferTask>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (String, String, String, i64, i64, String, Option<String>, Option<String>, Option<String>, Option<String>)>(
            "SELECT id, kind, file_name, total_bytes, transferred_bytes, status, error,
                    session_id, source_path, target_path
             FROM transfer_tasks
             WHERE status IN ('pending', 'inprogress', 'failed', 'cancelled')
               AND source_path IS NOT NULL
               AND target_path IS NOT NULL
             ORDER BY updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    id,
                    kind,
                    file_name,
                    total_bytes,
                    transferred_bytes,
                    status,
                    error,
                    session_id,
                    source_path,
                    target_path,
                )| StoredTransferTask {
                    id,
                    kind,
                    file_name,
                    total_bytes,
                    transferred_bytes,
                    status,
                    error,
                    session_id,
                    source_path,
                    target_path,
                },
            )
            .collect())
    }

    pub async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM transfer_tasks WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
