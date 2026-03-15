pub mod ai;
pub mod auth;
pub mod auxiliary;
pub mod connections;
pub mod crypto;
pub mod database;
pub mod db_clickhouse;
pub mod db_mssql;
pub mod db_mysql;
pub mod db_oracle;
pub mod db_postgres;
pub mod db_redis;
pub mod db_types;
pub mod desktop;
pub mod local_terminal;
pub mod settings;
pub mod sftp;
pub mod ssh;
pub mod ssh_suspend;
pub mod status;
pub mod telnet;
pub mod transfer;

// ── 内部共用：SettingsHostKeyStore + build_creds ──
//
// ssh.rs、sftp.rs 和 transfer.rs 都需要通过 settings 表持久化 SSH 主机指纹，
// 并共用相同的凭据构建逻辑。统一定义于此避免重复。

use api_contract::error::AppError;
use async_trait::async_trait;
use connection_core::model::AuthMethod;
use connection_core::repository::ConnectionRepository;
use ssh_core::host_key::{HostKeyRepository, HostKeyStore};
use std::sync::Arc;

use crate::state::AppState;

/// 从数据库连接记录构建 SSH 凭据，供 sftp.rs / transfer.rs 共用。
/// 不含连接类型校验（调用方按需处理）。
pub(super) async fn build_creds(
    state: &AppState,
    connection_id: i64,
) -> Result<ssh_core::session::SshCredentials, AppError> {
    let conn = state
        .conn_repo
        .get_connection(connection_id)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::NotFound("connection not found".into()))?;

    let auth = match conn.auth_method_enum() {
        AuthMethod::Key => {
            let key_id = conn
                .ssh_key_id
                .ok_or(AppError::Validation("no SSH key configured".into()))?;
            let key = state
                .conn_repo
                .get_ssh_key(key_id)
                .await
                .map_err(AppError::from)?
                .ok_or(AppError::NotFound("SSH key not found".into()))?;
            let private_key = state
                .crypto
                .decrypt(&key.encrypted_private_key)
                .map_err(|e| AppError::Crypto(e.to_string()))?;
            let passphrase = key
                .encrypted_passphrase
                .as_deref()
                .map(|p| state.crypto.decrypt(p))
                .transpose()
                .map_err(|e| AppError::Crypto(e.to_string()))?;
            ssh_core::session::SshAuth::Key {
                private_key_pem: private_key,
                passphrase,
            }
        }
        AuthMethod::Password => {
            let password = conn
                .encrypted_password
                .as_deref()
                .ok_or(AppError::Validation("no password configured".into()))?;
            let decrypted = state
                .crypto
                .decrypt(password)
                .map_err(|e| AppError::Crypto(e.to_string()))?;
            ssh_core::session::SshAuth::Password(decrypted)
        }
    };

    Ok(ssh_core::session::SshCredentials {
        host: conn.host,
        port: conn.port as u16,
        username: conn.username,
        auth,
    })
}

/// HostKeyStore 适配器：通过专用 host_key_repo 持久化 SSH 主机指纹。
pub(super) struct SettingsHostKeyStore {
    pub(super) repo: Arc<dyn HostKeyRepository>,
}

#[async_trait]
impl HostKeyStore for SettingsHostKeyStore {
    async fn get_fingerprint(&self, host: &str, port: u16) -> Result<Option<String>, String> {
        self.repo.get(host, port).await.map_err(|e| e.0)
    }

    async fn set_fingerprint(
        &self,
        host: &str,
        port: u16,
        fingerprint: &str,
    ) -> Result<(), String> {
        self.repo
            .set(host, port, fingerprint)
            .await
            .map_err(|e| e.0)
    }
}
