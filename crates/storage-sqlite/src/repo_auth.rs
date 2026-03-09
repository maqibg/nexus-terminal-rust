//! AuthRepository implementation for SQLite.

use async_trait::async_trait;
use auth_core::repository::{AuthRepository, PasskeyRow, UserRow};
use shared_utils::StorageError;
use sqlx::SqlitePool;

const PERSISTED_AUTH_USER_ID_KEY: &str = "auth.persisted_user_id";

pub struct SqliteAuthRepo {
    pool: SqlitePool,
}

impl SqliteAuthRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthRepository for SqliteAuthRepo {
    async fn user_count(&self) -> Result<i64, StorageError> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        Ok(count)
    }

    async fn find_user_by_username(&self, username: &str) -> Result<Option<UserRow>, StorageError> {
        sqlx::query_as::<_, (i64, String, String, Option<String>, bool)>(
            "SELECT id, username, hashed_password, two_factor_secret, two_factor_enabled != 0 FROM users WHERE username = ?",
        )
        .bind(username)
        .fetch_optional(&self.pool).await
        .map(|r| r.map(|(id, username, hashed_password, two_factor_secret, two_factor_enabled)| UserRow {
            id, username, hashed_password, two_factor_secret, two_factor_enabled,
        }))
        .map_err(|e| StorageError(e.to_string()))
    }

    async fn find_user_by_id(&self, id: i64) -> Result<Option<UserRow>, StorageError> {
        sqlx::query_as::<_, (i64, String, String, Option<String>, bool)>(
            "SELECT id, username, hashed_password, two_factor_secret, two_factor_enabled != 0 FROM users WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool).await
        .map(|r| r.map(|(id, username, hashed_password, two_factor_secret, two_factor_enabled)| UserRow {
            id, username, hashed_password, two_factor_secret, two_factor_enabled,
        }))
        .map_err(|e| StorageError(e.to_string()))
    }

    async fn create_user(
        &self,
        username: &str,
        hashed_password: &str,
    ) -> Result<i64, StorageError> {
        let result = sqlx::query("INSERT INTO users (username, hashed_password) VALUES (?, ?)")
            .bind(username)
            .bind(hashed_password)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        Ok(result.last_insert_rowid())
    }

    async fn update_password(
        &self,
        user_id: i64,
        hashed_password: &str,
    ) -> Result<(), StorageError> {
        sqlx::query(
            "UPDATE users SET hashed_password = ?, updated_at = datetime('now') WHERE id = ?",
        )
        .bind(hashed_password)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError(e.to_string()))?;
        Ok(())
    }

    async fn set_two_factor_secret(
        &self,
        user_id: i64,
        secret: Option<&str>,
    ) -> Result<(), StorageError> {
        let enabled: i32 = if secret.is_some() { 1 } else { 0 };
        sqlx::query("UPDATE users SET two_factor_secret = ?, two_factor_enabled = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(secret).bind(enabled).bind(user_id)
            .execute(&self.pool).await.map_err(|e| StorageError(e.to_string()))?;
        Ok(())
    }

    async fn get_persisted_login_user_id(&self) -> Result<Option<i64>, StorageError> {
        let value = sqlx::query_scalar::<_, String>("SELECT value FROM settings WHERE key = ?")
            .bind(PERSISTED_AUTH_USER_ID_KEY)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| StorageError(e.to_string()))?;

        Ok(value.and_then(|s| s.parse::<i64>().ok()))
    }

    async fn set_persisted_login_user_id(&self, user_id: Option<i64>) -> Result<(), StorageError> {
        if let Some(user_id) = user_id {
            sqlx::query(
                "INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            )
            .bind(PERSISTED_AUTH_USER_ID_KEY)
            .bind(user_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        } else {
            sqlx::query("DELETE FROM settings WHERE key = ?")
                .bind(PERSISTED_AUTH_USER_ID_KEY)
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError(e.to_string()))?;
        }

        Ok(())
    }

    async fn list_passkeys(&self, user_id: i64) -> Result<Vec<PasskeyRow>, StorageError> {
        sqlx::query_as::<_, (i64, i64, String, String, i64, Option<String>, String)>(
            "SELECT id, user_id, credential_id, public_key, sign_count, transports, name FROM passkeys WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_all(&self.pool).await
        .map(|rows| rows.into_iter().map(|(id, user_id, credential_id, public_key, counter, transports, name)| PasskeyRow {
            id, user_id, credential_id, public_key, counter: counter as u32, transports, name,
        }).collect())
        .map_err(|e| StorageError(e.to_string()))
    }

    async fn find_passkey_by_credential_id(
        &self,
        credential_id: &str,
    ) -> Result<Option<PasskeyRow>, StorageError> {
        sqlx::query_as::<_, (i64, i64, String, String, i64, Option<String>, String)>(
            "SELECT id, user_id, credential_id, public_key, sign_count, transports, name FROM passkeys WHERE credential_id = ?",
        )
        .bind(credential_id)
        .fetch_optional(&self.pool).await
        .map(|r| r.map(|(id, user_id, credential_id, public_key, counter, transports, name)| PasskeyRow {
            id, user_id, credential_id, public_key, counter: counter as u32, transports, name,
        }))
        .map_err(|e| StorageError(e.to_string()))
    }

    async fn create_passkey(
        &self,
        user_id: i64,
        credential_id: &str,
        public_key: &str,
        counter: u32,
        transports: Option<&str>,
        name: &str,
    ) -> Result<i64, StorageError> {
        let result = sqlx::query(
            "INSERT INTO passkeys (user_id, credential_id, public_key, sign_count, transports, name) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(user_id).bind(credential_id).bind(public_key).bind(counter as i64).bind(transports).bind(name)
        .execute(&self.pool).await.map_err(|e| StorageError(e.to_string()))?;
        Ok(result.last_insert_rowid())
    }

    async fn update_passkey_counter(
        &self,
        credential_id: &str,
        counter: u32,
    ) -> Result<(), StorageError> {
        sqlx::query("UPDATE passkeys SET sign_count = ? WHERE credential_id = ?")
            .bind(counter as i64)
            .bind(credential_id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        Ok(())
    }

    async fn delete_passkey(
        &self,
        user_id: i64,
        credential_id: &str,
    ) -> Result<bool, StorageError> {
        let result = sqlx::query("DELETE FROM passkeys WHERE user_id = ? AND credential_id = ?")
            .bind(user_id)
            .bind(credential_id)
            .execute(&self.pool)
            .await
            .map_err(|e| StorageError(e.to_string()))?;
        Ok(result.rows_affected() > 0)
    }

    async fn rename_passkey(
        &self,
        user_id: i64,
        credential_id: &str,
        name: &str,
    ) -> Result<bool, StorageError> {
        let result =
            sqlx::query("UPDATE passkeys SET name = ? WHERE user_id = ? AND credential_id = ?")
                .bind(name)
                .bind(user_id)
                .bind(credential_id)
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError(e.to_string()))?;
        Ok(result.rows_affected() > 0)
    }
}
