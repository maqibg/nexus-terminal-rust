//! SQLite implementations for audit, history, quick-command repositories.

use async_trait::async_trait;
use audit_core::model::AuditLog;
use audit_core::repository::AuditRepository;
use history_core::model::*;
use history_core::repository::HistoryRepository;
use quick_command_core::model::*;
use quick_command_core::repository::QuickCommandRepository;
use shared_utils::StorageError;
use sqlx::SqlitePool;

// ── Audit ──

pub struct SqliteAuditRepo {
    pool: SqlitePool,
}

impl SqliteAuditRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditRepository for SqliteAuditRepo {
    async fn list_logs(&self, limit: i64, offset: i64) -> Result<Vec<AuditLog>, StorageError> {
        sqlx::query_as::<_, (i64, String, String, Option<String>)>(
            "SELECT id, timestamp, action_type, details FROM audit_logs ORDER BY id DESC LIMIT ? OFFSET ?",
        ).bind(limit).bind(offset)
        .fetch_all(&self.pool).await
        .map(|rows| rows.into_iter().map(|(id, timestamp, action_type, details)| AuditLog { id, timestamp, action_type, details }).collect())
        .map_err(|e| StorageError(e.to_string()))
    }

    async fn count_logs(&self) -> Result<i64, StorageError> {
        let (c,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM audit_logs")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        Ok(c)
    }

    async fn create_log(
        &self,
        action_type: &str,
        details: Option<&str>,
    ) -> Result<i64, StorageError> {
        let r = sqlx::query("INSERT INTO audit_logs (action_type, details) VALUES (?, ?)")
            .bind(action_type)
            .bind(details)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        Ok(r.last_insert_rowid())
    }

    async fn clear_logs(&self) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM audit_logs")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        Ok(())
    }
}

// ── History ──

pub struct SqliteHistoryRepo {
    pool: SqlitePool,
}

impl SqliteHistoryRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HistoryRepository for SqliteHistoryRepo {
    async fn list_command_history(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<CommandHistory>, StorageError> {
        sqlx::query_as::<_, (i64, String, Option<String>, Option<i64>, String)>(
            "SELECT id, command, session_id, connection_id, timestamp FROM command_history ORDER BY id DESC LIMIT ? OFFSET ?",
        ).bind(limit).bind(offset)
        .fetch_all(&self.pool).await
        .map(|rows| rows.into_iter().map(|(id, command, session_id, connection_id, timestamp)| CommandHistory { id, command, session_id, connection_id, timestamp }).collect())
        .map_err(|e| StorageError(e.to_string()))
    }

    async fn add_command(
        &self,
        command: &str,
        session_id: Option<&str>,
        connection_id: Option<i64>,
    ) -> Result<i64, StorageError> {
        let r = sqlx::query(
            "INSERT INTO command_history (command, session_id, connection_id) VALUES (?, ?, ?)",
        )
        .bind(command)
        .bind(session_id)
        .bind(connection_id)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError(e.to_string()))?;
        Ok(r.last_insert_rowid())
    }

    async fn clear_command_history(&self) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM command_history")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        Ok(())
    }

    async fn delete_command_history_entry(&self, id: i64) -> Result<bool, StorageError> {
        let r = sqlx::query("DELETE FROM command_history WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        Ok(r.rows_affected() > 0)
    }

    async fn list_path_history(
        &self,
        connection_id: Option<i64>,
        limit: i64,
    ) -> Result<Vec<PathHistory>, StorageError> {
        let (sql, bind_conn) = match connection_id {
            Some(_) => ("SELECT id, path, connection_id, timestamp FROM path_history WHERE connection_id = ? ORDER BY id DESC LIMIT ?", true),
            None => ("SELECT id, path, connection_id, timestamp FROM path_history ORDER BY id DESC LIMIT ?", false),
        };
        let mut q = sqlx::query_as::<_, (i64, String, Option<i64>, String)>(sql);
        if bind_conn {
            q = q.bind(connection_id);
        }
        q = q.bind(limit);
        q.fetch_all(&self.pool)
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|(id, path, connection_id, timestamp)| PathHistory {
                        id,
                        path,
                        connection_id,
                        timestamp,
                    })
                    .collect()
            })
            .map_err(|e| StorageError(e.to_string()))
    }

    async fn add_path(&self, path: &str, connection_id: Option<i64>) -> Result<i64, StorageError> {
        let r = sqlx::query("INSERT INTO path_history (path, connection_id) VALUES (?, ?)")
            .bind(path)
            .bind(connection_id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        Ok(r.last_insert_rowid())
    }

    async fn clear_path_history(&self) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM path_history")
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        Ok(())
    }

    async fn list_favorite_paths(
        &self,
        connection_id: Option<i64>,
    ) -> Result<Vec<FavoritePath>, StorageError> {
        let (sql, bind_conn) = match connection_id {
            Some(_) => ("SELECT id, name, path, connection_id, last_used_at FROM favorite_paths WHERE connection_id = ? ORDER BY name", true),
            None => ("SELECT id, name, path, connection_id, last_used_at FROM favorite_paths ORDER BY name", false),
        };
        let mut q = sqlx::query_as::<_, (i64, String, String, Option<i64>, Option<String>)>(sql);
        if bind_conn {
            q = q.bind(connection_id);
        }
        q.fetch_all(&self.pool)
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(
                        |(id, name, path, connection_id, last_used_at)| FavoritePath {
                            id,
                            name,
                            path,
                            connection_id,
                            last_used_at,
                        },
                    )
                    .collect()
            })
            .map_err(|e| StorageError(e.to_string()))
    }

    async fn create_favorite_path(
        &self,
        name: &str,
        path: &str,
        connection_id: Option<i64>,
    ) -> Result<i64, StorageError> {
        let r =
            sqlx::query("INSERT INTO favorite_paths (name, path, connection_id) VALUES (?, ?, ?)")
                .bind(name)
                .bind(path)
                .bind(connection_id)
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError(e.to_string()))?;
        Ok(r.last_insert_rowid())
    }

    async fn update_favorite_path(
        &self,
        id: i64,
        name: &str,
        path: &str,
        connection_id: Option<i64>,
    ) -> Result<bool, StorageError> {
        let r = sqlx::query("UPDATE favorite_paths SET name=?, path=?, connection_id=? WHERE id=?")
            .bind(name)
            .bind(path)
            .bind(connection_id)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        Ok(r.rows_affected() > 0)
    }

    async fn delete_favorite_path(&self, id: i64) -> Result<bool, StorageError> {
        let r = sqlx::query("DELETE FROM favorite_paths WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        Ok(r.rows_affected() > 0)
    }

    async fn mark_favorite_path_used(&self, id: i64) -> Result<bool, StorageError> {
        let r =
            sqlx::query("UPDATE favorite_paths SET last_used_at = datetime('now') WHERE id = ?")
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError(e.to_string()))?;
        Ok(r.rows_affected() > 0)
    }
}

// ── Quick Commands ──

pub struct SqliteQuickCommandRepo {
    pool: SqlitePool,
}

async fn sync_qc_tags_tx(
    conn: &mut sqlx::SqliteConnection,
    qc_id: i64,
    tags: &[String],
) -> Result<(), StorageError> {
    sqlx::query("DELETE FROM quick_command_tag_associations WHERE quick_command_id = ?")
        .bind(qc_id)
        .execute(&mut *conn)
        .await
        .map_err(|e| StorageError(e.to_string()))?;
    for tag in tags {
        sqlx::query("INSERT OR IGNORE INTO quick_command_tags (name) VALUES (?)")
            .bind(tag)
            .execute(&mut *conn)
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        sqlx::query("INSERT INTO quick_command_tag_associations (quick_command_id, tag_id) SELECT ?, id FROM quick_command_tags WHERE name = ?")
            .bind(qc_id).bind(tag).execute(&mut *conn).await.map_err(|e| StorageError(e.to_string()))?;
    }
    Ok(())
}

impl SqliteQuickCommandRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    async fn get_tags_for_qc(&self, qc_id: i64) -> Result<Vec<String>, StorageError> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT t.name FROM quick_command_tags t JOIN quick_command_tag_associations a ON t.id = a.tag_id WHERE a.quick_command_id = ?",
        ).bind(qc_id).fetch_all(&self.pool).await.map_err(|e| StorageError(e.to_string()))?;
        Ok(rows.into_iter().map(|(n,)| n).collect())
    }
}

#[async_trait]
impl QuickCommandRepository for SqliteQuickCommandRepo {
    async fn list(&self) -> Result<Vec<QuickCommand>, StorageError> {
        let rows: Vec<(i64, String, String, i64, Option<String>)> = sqlx::query_as(
            "SELECT id, name, command, usage_count, variables FROM quick_commands ORDER BY usage_count DESC, id",
        ).fetch_all(&self.pool).await.map_err(|e| StorageError(e.to_string()))?;

        // 批量查询所有 tag 关联，避免 N+1
        let tag_rows: Vec<(i64, String)> = sqlx::query_as(
            "SELECT a.quick_command_id, t.name FROM quick_command_tag_associations a JOIN quick_command_tags t ON t.id = a.tag_id",
        ).fetch_all(&self.pool).await.map_err(|e| StorageError(e.to_string()))?;
        let mut tag_map: std::collections::HashMap<i64, Vec<String>> =
            std::collections::HashMap::new();
        for (qc_id, tag_name) in tag_rows {
            tag_map.entry(qc_id).or_default().push(tag_name);
        }

        let result = rows
            .into_iter()
            .map(|(id, name, command, usage_count, variables)| QuickCommand {
                id,
                name,
                command,
                usage_count,
                variables,
                tags: tag_map.remove(&id).unwrap_or_default(),
            })
            .collect();
        Ok(result)
    }

    async fn get(&self, id: i64) -> Result<Option<QuickCommand>, StorageError> {
        let row: Option<(i64, String, String, i64, Option<String>)> = sqlx::query_as(
            "SELECT id, name, command, usage_count, variables FROM quick_commands WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError(e.to_string()))?;
        match row {
            None => Ok(None),
            Some((id, name, command, usage_count, variables)) => {
                let tags = self.get_tags_for_qc(id).await?;
                Ok(Some(QuickCommand {
                    id,
                    name,
                    command,
                    usage_count,
                    variables,
                    tags,
                }))
            }
        }
    }

    async fn create(&self, input: &QuickCommandInput) -> Result<i64, StorageError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        let r =
            sqlx::query("INSERT INTO quick_commands (name, command, variables) VALUES (?, ?, ?)")
                .bind(&input.name)
                .bind(&input.command)
                .bind(&input.variables)
                .execute(&mut *tx)
                .await
                .map_err(|e| StorageError(e.to_string()))?;
        let id = r.last_insert_rowid();
        if let Some(tags) = &input.tags {
            sync_qc_tags_tx(tx.as_mut(), id, tags).await?;
        }
        tx.commit().await.map_err(|e| StorageError(e.to_string()))?;
        Ok(id)
    }

    async fn update(&self, id: i64, input: &QuickCommandInput) -> Result<bool, StorageError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        let r = sqlx::query("UPDATE quick_commands SET name=?, command=?, variables=?, updated_at=datetime('now') WHERE id=?")
            .bind(&input.name).bind(&input.command).bind(&input.variables).bind(id)
            .execute(&mut *tx).await.map_err(|e| StorageError(e.to_string()))?;
        if let Some(tags) = &input.tags {
            sync_qc_tags_tx(tx.as_mut(), id, tags).await?;
        }
        tx.commit().await.map_err(|e| StorageError(e.to_string()))?;
        Ok(r.rows_affected() > 0)
    }

    async fn delete(&self, id: i64) -> Result<bool, StorageError> {
        let r = sqlx::query("DELETE FROM quick_commands WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        Ok(r.rows_affected() > 0)
    }

    async fn increment_usage(&self, id: i64) -> Result<(), StorageError> {
        sqlx::query("UPDATE quick_commands SET usage_count = usage_count + 1 WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        Ok(())
    }

    async fn list_tags(&self) -> Result<Vec<QuickCommandTag>, StorageError> {
        sqlx::query_as::<_, (i64, String)>("SELECT id, name FROM quick_command_tags ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|(id, name)| QuickCommandTag { id, name })
                    .collect()
            })
            .map_err(|e| StorageError(e.to_string()))
    }

    async fn create_tag(&self, name: &str) -> Result<i64, StorageError> {
        let r = sqlx::query("INSERT INTO quick_command_tags (name) VALUES (?)")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        Ok(r.last_insert_rowid())
    }

    async fn delete_tag(&self, id: i64) -> Result<bool, StorageError> {
        let r = sqlx::query("DELETE FROM quick_command_tags WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        Ok(r.rows_affected() > 0)
    }

    async fn bulk_assign_tag(
        &self,
        tag_id: i64,
        quick_command_ids: &[i64],
    ) -> Result<(), StorageError> {
        for qc_id in quick_command_ids {
            sqlx::query("INSERT OR IGNORE INTO quick_command_tag_associations (quick_command_id, tag_id) VALUES (?, ?)")
                .bind(qc_id).bind(tag_id)
                .execute(&self.pool).await.map_err(|e| StorageError(e.to_string()))?;
        }
        Ok(())
    }
}
