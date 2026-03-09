//! HostKeyRepository implementation for SQLite.

use async_trait::async_trait;
use shared_utils::StorageError;
use sqlx::SqlitePool;
use ssh_core::host_key::{HostKeyEntry, HostKeyRepository};

pub struct SqliteHostKeyRepo {
    pool: SqlitePool,
}

impl SqliteHostKeyRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HostKeyRepository for SqliteHostKeyRepo {
    async fn get(&self, host: &str, port: u16) -> Result<Option<String>, StorageError> {
        sqlx::query_as::<_, (String,)>(
            "SELECT fingerprint FROM ssh_known_hosts WHERE host = ? AND port = ?",
        )
        .bind(host)
        .bind(i64::from(port))
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(|(fingerprint,)| fingerprint))
        .map_err(|e| StorageError(e.to_string()))
    }

    async fn set(&self, host: &str, port: u16, fingerprint: &str) -> Result<(), StorageError> {
        sqlx::query(
            "INSERT INTO ssh_known_hosts (host, port, fingerprint) VALUES (?, ?, ?)
             ON CONFLICT(host, port)
             DO UPDATE SET fingerprint = excluded.fingerprint, updated_at = datetime('now')",
        )
        .bind(host)
        .bind(i64::from(port))
        .bind(fingerprint)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(|e| StorageError(e.to_string()))
    }

    async fn delete(&self, host: &str, port: u16) -> Result<bool, StorageError> {
        let result = sqlx::query("DELETE FROM ssh_known_hosts WHERE host = ? AND port = ?")
            .bind(host)
            .bind(i64::from(port))
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        Ok(result.rows_affected() > 0)
    }

    async fn list(&self) -> Result<Vec<HostKeyEntry>, StorageError> {
        let rows = sqlx::query_as::<_, (String, i64, String)>(
            "SELECT host, port, fingerprint FROM ssh_known_hosts ORDER BY host, port",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError(e.to_string()))?;

        rows.into_iter()
            .map(|(host, port, fingerprint)| {
                let port = u16::try_from(port).map_err(|_| {
                    StorageError(format!(
                        "invalid port {port} in ssh_known_hosts for host {host}"
                    ))
                })?;
                Ok(HostKeyEntry {
                    host,
                    port,
                    fingerprint,
                })
            })
            .collect()
    }
}
