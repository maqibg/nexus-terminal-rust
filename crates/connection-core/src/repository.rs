//! Connection repository trait — implemented by storage-sqlite.

use crate::model::*;
use async_trait::async_trait;

#[async_trait]
pub trait ConnectionRepository: Send + Sync {
    // Connections
    async fn list_connections(&self) -> Result<Vec<Connection>, String>;
    async fn get_connection(&self, id: i64) -> Result<Option<Connection>, String>;
    async fn create_connection(
        &self,
        input: &ConnectionInput,
        encrypted_password: Option<&str>,
    ) -> Result<i64, String>;
    async fn update_connection(
        &self,
        id: i64,
        input: &ConnectionInput,
        encrypted_password: Option<&str>,
    ) -> Result<bool, String>;
    async fn delete_connection(&self, id: i64) -> Result<bool, String>;
    async fn reorder_connections(&self, ids: &[i64]) -> Result<(), String>;

    // Tags
    async fn list_tags(&self) -> Result<Vec<Tag>, String>;
    async fn create_tag(&self, name: &str) -> Result<i64, String>;
    async fn delete_tag(&self, id: i64) -> Result<bool, String>;

    // SSH Keys
    async fn list_ssh_keys(&self) -> Result<Vec<SshKey>, String>;
    async fn get_ssh_key(&self, id: i64) -> Result<Option<SshKey>, String>;
    async fn create_ssh_key(
        &self,
        name: &str,
        encrypted_private_key: &str,
        encrypted_passphrase: Option<&str>,
    ) -> Result<i64, String>;
    async fn update_ssh_key(
        &self,
        id: i64,
        name: &str,
        encrypted_private_key: Option<&str>,
        encrypted_passphrase: Option<&str>,
    ) -> Result<bool, String>;
    async fn delete_ssh_key(&self, id: i64) -> Result<bool, String>;

    // Proxies
    async fn list_proxies(&self) -> Result<Vec<Proxy>, String>;
    async fn get_proxy(&self, id: i64) -> Result<Option<Proxy>, String>;
    async fn create_proxy(&self, proxy: &Proxy) -> Result<i64, String>;
    async fn update_proxy(&self, proxy: &Proxy) -> Result<bool, String>;
    async fn delete_proxy(&self, id: i64) -> Result<bool, String>;
}
