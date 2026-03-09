//! Connection repository trait — implemented by storage-sqlite.

use crate::model::*;
use async_trait::async_trait;
use shared_utils::StorageError;

#[async_trait]
pub trait ConnectionRepository: Send + Sync {
    // Connections
    async fn list_connections(&self) -> Result<Vec<Connection>, StorageError>;
    async fn get_connection(&self, id: i64) -> Result<Option<Connection>, StorageError>;
    async fn create_connection(
        &self,
        input: &ConnectionInput,
        encrypted_password: Option<&str>,
    ) -> Result<i64, StorageError>;
    async fn update_connection(
        &self,
        id: i64,
        input: &ConnectionInput,
        encrypted_password: Option<&str>,
    ) -> Result<bool, StorageError>;
    async fn delete_connection(&self, id: i64) -> Result<bool, StorageError>;
    async fn reorder_connections(&self, ids: &[i64]) -> Result<(), StorageError>;

    // Tags
    async fn list_tags(&self) -> Result<Vec<Tag>, StorageError>;
    async fn create_tag(&self, name: &str) -> Result<i64, StorageError>;
    async fn delete_tag(&self, id: i64) -> Result<bool, StorageError>;

    // SSH Keys
    async fn list_ssh_keys(&self) -> Result<Vec<SshKey>, StorageError>;
    async fn get_ssh_key(&self, id: i64) -> Result<Option<SshKey>, StorageError>;
    async fn create_ssh_key(
        &self,
        name: &str,
        encrypted_private_key: &str,
        encrypted_passphrase: Option<&str>,
    ) -> Result<i64, StorageError>;
    async fn update_ssh_key(
        &self,
        id: i64,
        name: &str,
        encrypted_private_key: Option<&str>,
        encrypted_passphrase: Option<&str>,
    ) -> Result<bool, StorageError>;
    async fn delete_ssh_key(&self, id: i64) -> Result<bool, StorageError>;

    // Proxies
    async fn list_proxies(&self) -> Result<Vec<Proxy>, StorageError>;
    async fn get_proxy(&self, id: i64) -> Result<Option<Proxy>, StorageError>;
    async fn create_proxy(&self, proxy: &Proxy) -> Result<i64, StorageError>;
    async fn update_proxy(&self, proxy: &Proxy) -> Result<bool, StorageError>;
    async fn delete_proxy(&self, id: i64) -> Result<bool, StorageError>;
}
