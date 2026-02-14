//! Auth repository trait — implemented by storage-sqlite.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// User row from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub hashed_password: String,
    pub two_factor_secret: Option<String>,
    pub two_factor_enabled: bool,
}

/// Passkey row from the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyRow {
    pub id: i64,
    pub user_id: i64,
    pub credential_id: String,
    pub public_key: String,
    pub counter: u32,
    pub transports: Option<String>,
    pub name: String,
}

/// Abstract auth data access.
#[async_trait]
pub trait AuthRepository: Send + Sync {
    async fn user_count(&self) -> Result<i64, String>;
    async fn find_user_by_username(&self, username: &str) -> Result<Option<UserRow>, String>;
    async fn find_user_by_id(&self, id: i64) -> Result<Option<UserRow>, String>;
    async fn create_user(&self, username: &str, hashed_password: &str) -> Result<i64, String>;
    async fn update_password(&self, user_id: i64, hashed_password: &str) -> Result<(), String>;
    async fn set_two_factor_secret(&self, user_id: i64, secret: Option<&str>)
        -> Result<(), String>;
    async fn get_persisted_login_user_id(&self) -> Result<Option<i64>, String>;
    async fn set_persisted_login_user_id(&self, user_id: Option<i64>) -> Result<(), String>;

    // Passkey operations
    async fn list_passkeys(&self, user_id: i64) -> Result<Vec<PasskeyRow>, String>;
    async fn find_passkey_by_credential_id(
        &self,
        credential_id: &str,
    ) -> Result<Option<PasskeyRow>, String>;
    async fn create_passkey(
        &self,
        user_id: i64,
        credential_id: &str,
        public_key: &str,
        counter: u32,
        transports: Option<&str>,
        name: &str,
    ) -> Result<i64, String>;
    async fn update_passkey_counter(&self, credential_id: &str, counter: u32)
        -> Result<(), String>;
    async fn delete_passkey(&self, user_id: i64, credential_id: &str) -> Result<bool, String>;
    async fn rename_passkey(
        &self,
        user_id: i64,
        credential_id: &str,
        name: &str,
    ) -> Result<bool, String>;
}
