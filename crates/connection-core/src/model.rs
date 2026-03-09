//! Connection domain models.

use serde::{Deserialize, Serialize};

/// SSH 认证方式。存储层持久化为小写字符串（"key" / "password"）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMethod {
    Password,
    Key,
}

impl AuthMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Password => "password",
            Self::Key => "key",
        }
    }

    /// 大小写不敏感解析，返回 None 表示未知值。
    pub fn parse(value: &str) -> Option<Self> {
        let trimmed = value.trim();
        if trimmed.eq_ignore_ascii_case("key") {
            Some(Self::Key)
        } else if trimmed.eq_ignore_ascii_case("password") {
            Some(Self::Password)
        } else {
            None
        }
    }

    /// 解析失败时默认返回 Password（向后兼容旧数据）。
    pub fn parse_or_default(value: &str) -> Self {
        Self::parse(value).unwrap_or(Self::Password)
    }

    /// 解析失败返回 Err，用于用户输入校验。
    pub fn try_parse(value: &str) -> Result<Self, String> {
        let trimmed = value.trim();
        Self::parse(trimmed).ok_or_else(|| format!("unsupported auth method: {trimmed}"))
    }
}

/// SSH connection entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub conn_type: String,
    pub host: String,
    pub port: i32,
    pub username: String,
    pub auth_method: String,
    pub encrypted_password: Option<String>,
    pub ssh_key_id: Option<i64>,
    pub proxy_id: Option<i64>,
    pub jump_chain: Option<String>,
    pub notes: Option<String>,
    pub rdp_options: Option<String>,
    pub vnc_options: Option<String>,
    pub provider: Option<String>,
    pub region: Option<String>,
    pub expiry_date: Option<String>,
    pub billing_cycle: Option<String>,
    pub billing_amount: Option<f64>,
    pub billing_currency: Option<String>,
    pub sort_order: i32,
    pub tags: Vec<String>,
}

impl Connection {
    /// 将字符串字段解析为强类型 AuthMethod，未知值默认 Password。
    pub fn auth_method_enum(&self) -> AuthMethod {
        AuthMethod::parse_or_default(&self.auth_method)
    }
}

/// Request to create/update a connection.
#[derive(Debug, Deserialize)]
pub struct ConnectionInput {
    pub name: String,
    #[serde(rename = "type")]
    pub conn_type: Option<String>,
    pub host: String,
    pub port: Option<i32>,
    pub username: Option<String>,
    pub auth_method: Option<String>,
    pub password: Option<String>,
    pub ssh_key_id: Option<i64>,
    pub proxy_id: Option<i64>,
    pub jump_chain: Option<String>,
    pub notes: Option<String>,
    pub rdp_options: Option<String>,
    pub vnc_options: Option<String>,
    pub provider: Option<String>,
    pub region: Option<String>,
    pub expiry_date: Option<String>,
    pub billing_cycle: Option<String>,
    pub billing_amount: Option<f64>,
    pub billing_currency: Option<String>,
    pub sort_order: Option<i32>,
    pub tags: Option<Vec<String>>,
}

impl ConnectionInput {
    /// 将 auth_method 字符串解析为枚举，缺失/未知时默认 Password。
    pub fn auth_method_enum_or_default(&self) -> AuthMethod {
        self.auth_method
            .as_deref()
            .and_then(AuthMethod::parse)
            .unwrap_or(AuthMethod::Password)
    }
}

/// SSH key entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKey {
    pub id: i64,
    pub name: String,
    pub encrypted_private_key: String,
    pub encrypted_passphrase: Option<String>,
}

/// Proxy entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proxy {
    pub id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub proxy_type: String,
    pub host: String,
    pub port: i32,
    pub username: Option<String>,
    pub auth_method: String,
    pub encrypted_password: Option<String>,
    pub encrypted_private_key: Option<String>,
}

/// Tag entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}
