//! Connection domain models.

use serde::{Deserialize, Serialize};

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
    pub sort_order: i32,
    pub tags: Vec<String>,
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
    pub sort_order: Option<i32>,
    pub tags: Option<Vec<String>>,
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