//! SSH 主机密钥存储 trait。

use async_trait::async_trait;
use serde::Serialize;
use shared_utils::StorageError;

/// known_hosts 表中的单条记录。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HostKeyEntry {
    pub host: String,
    pub port: u16,
    pub fingerprint: String,
}

/// 专用 known_hosts 仓储抽象，面向持久化层 CRUD。
#[async_trait]
pub trait HostKeyRepository: Send + Sync {
    async fn get(&self, host: &str, port: u16) -> Result<Option<String>, StorageError>;
    async fn set(&self, host: &str, port: u16, fingerprint: &str) -> Result<(), StorageError>;
    async fn delete(&self, host: &str, port: u16) -> Result<bool, StorageError>;
    async fn list(&self) -> Result<Vec<HostKeyEntry>, StorageError>;
}

/// 持久化 SSH 主机公钥指纹，用于 TOFU（首次信任）验证。
#[async_trait]
pub trait HostKeyStore: Send + Sync {
    /// 获取已知的主机指纹，返回 None 表示首次连接。
    async fn get_fingerprint(&self, host: &str, port: u16) -> Result<Option<String>, String>;

    /// 存储主机指纹。
    async fn set_fingerprint(&self, host: &str, port: u16, fingerprint: &str)
        -> Result<(), String>;
}
