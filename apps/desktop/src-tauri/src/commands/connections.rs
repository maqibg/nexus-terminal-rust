//! Connection Tauri commands.

use api_contract::error::{AppError, CmdResult};
use connection_core::model::{Connection, ConnectionInput, Proxy, SshKey, Tag};
use connection_core::repository::ConnectionRepository;
use history_core::repository::HistoryRepository;
use quick_command_core::repository::QuickCommandRepository;
use serde::{Deserialize, Serialize};
use settings_core::repository::SettingsRepository;
use std::collections::HashMap;
use tauri::State;
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

use crate::state::AppState;

/// 返回连接给前端前抹去加密密钥材料，避免扩大密文暴露面。
fn mask_connection(mut conn: Connection) -> Connection {
    conn.encrypted_password = None;
    conn
}

/// 返回 SSH Key 前清除私钥密文与口令密文。
fn mask_ssh_key(mut key: SshKey) -> SshKey {
    key.encrypted_private_key = String::new();
    key.encrypted_passphrase = None;
    key
}

/// 返回代理记录前清除所有加密凭据字段。
fn mask_proxy(mut proxy: Proxy) -> Proxy {
    proxy.encrypted_password = None;
    proxy.encrypted_private_key = None;
    proxy
}

#[derive(Deserialize)]
pub struct ReorderRequest {
    pub ids: Vec<i64>,
}

#[derive(Deserialize)]
pub struct TagCreateRequest {
    pub name: String,
}

async fn test_tcp_endpoint(host: &str, port: u16) -> CmdResult<bool> {
    let addr = format!("{host}:{port}");
    let conn = timeout(Duration::from_secs(5), TcpStream::connect(addr))
        .await
        .map_err(|_| AppError::Ssh("connection timeout".into()))?;
    match conn {
        Ok(_stream) => Ok(true),
        Err(e) => Err(AppError::Ssh(format!("tcp connect failed: {e}"))),
    }
}

fn resolve_port(port: Option<i32>, default_port: u16) -> u16 {
    port.and_then(|value| u16::try_from(value).ok())
        .filter(|value| *value > 0)
        .unwrap_or(default_port)
}

#[tauri::command]
pub async fn connection_list(state: State<'_, AppState>) -> CmdResult<Vec<Connection>> {
    state.auth.require_auth().await?;
    let list = state
        .conn_repo
        .list_connections()
        .await
        .map_err(AppError::from)?;
    Ok(list.into_iter().map(mask_connection).collect())
}

#[tauri::command]
pub async fn connection_get(state: State<'_, AppState>, id: i64) -> CmdResult<Connection> {
    state.auth.require_auth().await?;
    let conn = state
        .conn_repo
        .get_connection(id)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::NotFound("connection not found".into()))?;
    Ok(mask_connection(conn))
}

#[tauri::command]
pub async fn connection_create(
    state: State<'_, AppState>,
    input: connection_core::model::ConnectionInput,
) -> CmdResult<i64> {
    state.auth.require_auth().await?;
    let encrypted = input
        .password
        .as_deref()
        .map(|p| state.crypto.encrypt(p))
        .transpose()
        .map_err(|e| AppError::Crypto(e.to_string()))?;
    state
        .conn_repo
        .create_connection(&input, encrypted.as_deref())
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn connection_update(
    state: State<'_, AppState>,
    id: i64,
    input: connection_core::model::ConnectionInput,
) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    let encrypted = input
        .password
        .as_deref()
        .map(|p| state.crypto.encrypt(p))
        .transpose()
        .map_err(|e| AppError::Crypto(e.to_string()))?;
    state
        .conn_repo
        .update_connection(id, &input, encrypted.as_deref())
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn connection_delete(state: State<'_, AppState>, id: i64) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    state
        .conn_repo
        .delete_connection(id)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn connection_reorder(state: State<'_, AppState>, req: ReorderRequest) -> CmdResult<()> {
    state.auth.require_auth().await?;
    state
        .conn_repo
        .reorder_connections(&req.ids)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn tag_list(state: State<'_, AppState>) -> CmdResult<Vec<Tag>> {
    state.auth.require_auth().await?;
    state.conn_repo.list_tags().await.map_err(AppError::from)
}

#[tauri::command]
pub async fn tag_create(state: State<'_, AppState>, req: TagCreateRequest) -> CmdResult<i64> {
    state.auth.require_auth().await?;
    state
        .conn_repo
        .create_tag(&req.name)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn tag_delete(state: State<'_, AppState>, id: i64) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    state.conn_repo.delete_tag(id).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn ssh_key_list(state: State<'_, AppState>) -> CmdResult<Vec<SshKey>> {
    state.auth.require_auth().await?;
    state
        .conn_repo
        .list_ssh_keys()
        .await
        .map_err(AppError::from)
        .map(|keys| keys.into_iter().map(mask_ssh_key).collect())
}

#[tauri::command]
pub async fn ssh_key_delete(state: State<'_, AppState>, id: i64) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    state
        .conn_repo
        .delete_ssh_key(id)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn proxy_list(state: State<'_, AppState>) -> CmdResult<Vec<Proxy>> {
    state.auth.require_auth().await?;
    state
        .conn_repo
        .list_proxies()
        .await
        .map_err(AppError::from)
        .map(|proxies| proxies.into_iter().map(mask_proxy).collect())
}

#[tauri::command]
pub async fn proxy_delete(state: State<'_, AppState>, id: i64) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    state
        .conn_repo
        .delete_proxy(id)
        .await
        .map_err(AppError::from)
}

// ── SSH Key Create / Update ──

#[derive(Deserialize)]
pub struct SshKeyCreateRequest {
    pub name: String,
    pub private_key_pem: String,
    pub passphrase: Option<String>,
}

#[tauri::command]
pub async fn ssh_key_create(
    state: State<'_, AppState>,
    req: SshKeyCreateRequest,
) -> CmdResult<i64> {
    state.auth.require_auth().await?;
    let encrypted_key = state
        .crypto
        .encrypt(&req.private_key_pem)
        .map_err(|e| AppError::Crypto(e.to_string()))?;
    let encrypted_pass = req
        .passphrase
        .as_deref()
        .map(|p| state.crypto.encrypt(p))
        .transpose()
        .map_err(|e| AppError::Crypto(e.to_string()))?;
    state
        .conn_repo
        .create_ssh_key(&req.name, &encrypted_key, encrypted_pass.as_deref())
        .await
        .map_err(AppError::from)
}

#[derive(Deserialize)]
pub struct SshKeyUpdateRequest {
    pub id: i64,
    pub name: String,
    pub private_key_pem: Option<String>,
    pub passphrase: Option<String>,
}

#[tauri::command]
pub async fn ssh_key_update(
    state: State<'_, AppState>,
    req: SshKeyUpdateRequest,
) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    let encrypted_key = req
        .private_key_pem
        .as_deref()
        .map(|k| state.crypto.encrypt(k))
        .transpose()
        .map_err(|e| AppError::Crypto(e.to_string()))?;
    let encrypted_pass = req
        .passphrase
        .as_deref()
        .map(|p| state.crypto.encrypt(p))
        .transpose()
        .map_err(|e| AppError::Crypto(e.to_string()))?;
    state
        .conn_repo
        .update_ssh_key(
            req.id,
            &req.name,
            encrypted_key.as_deref(),
            encrypted_pass.as_deref(),
        )
        .await
        .map_err(AppError::from)
}

// ── Proxy Create / Update ──

#[tauri::command]
pub async fn proxy_create(state: State<'_, AppState>, input: Proxy) -> CmdResult<i64> {
    state.auth.require_auth().await?;
    let mut p = input;
    if let Some(ref pw) = p.encrypted_password {
        p.encrypted_password = Some(
            state
                .crypto
                .encrypt(pw)
                .map_err(|e| AppError::Crypto(e.to_string()))?,
        );
    }
    state
        .conn_repo
        .create_proxy(&p)
        .await
        .map_err(AppError::from)
}

#[tauri::command]
pub async fn proxy_update(state: State<'_, AppState>, input: Proxy) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    let mut p = input;
    if let Some(ref pw) = p.encrypted_password {
        p.encrypted_password = Some(
            state
                .crypto
                .encrypt(pw)
                .map_err(|e| AppError::Crypto(e.to_string()))?,
        );
    }
    state
        .conn_repo
        .update_proxy(&p)
        .await
        .map_err(AppError::from)
}

// ── Connection Test / Clone / Export / Import ──

#[tauri::command]
pub async fn connection_test(state: State<'_, AppState>, id: i64) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    let conn = state
        .conn_repo
        .get_connection(id)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::NotFound("connection not found".into()))?;

    if conn.conn_type.eq_ignore_ascii_case("RDP") {
        return test_tcp_endpoint(&conn.host, resolve_port(Some(conn.port), 3389)).await;
    }

    if conn.conn_type.eq_ignore_ascii_case("VNC") {
        return test_tcp_endpoint(&conn.host, resolve_port(Some(conn.port), 5900)).await;
    }

    let auth = if conn.auth_method == "key" {
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
    } else {
        let password = conn
            .encrypted_password
            .as_deref()
            .ok_or(AppError::Validation("no password configured".into()))?;
        let decrypted = state
            .crypto
            .decrypt(password)
            .map_err(|e| AppError::Crypto(e.to_string()))?;
        ssh_core::session::SshAuth::Password(decrypted)
    };

    let creds = ssh_core::session::SshCredentials {
        host: conn.host,
        port: resolve_port(Some(conn.port), 22),
        username: conn.username,
        auth,
    };

    match ssh_core::session::connect_and_authenticate(
        &creds,
        ssh_core::session::SshHandler::permissive(creds.host.clone(), creds.port),
    )
    .await
    {
        Ok(_handle) => Ok(true),
        Err(e) => Err(AppError::Ssh(e)),
    }
}

#[tauri::command]
pub async fn connection_test_unsaved(
    state: State<'_, AppState>,
    input: connection_core::model::ConnectionInput,
) -> CmdResult<bool> {
    state.auth.require_auth().await?;

    let conn_type = input.conn_type.as_deref().unwrap_or("SSH");
    let host = input.host.trim();
    if host.is_empty() {
        return Err(AppError::Validation("host is required".into()));
    }

    if conn_type.eq_ignore_ascii_case("RDP") {
        return test_tcp_endpoint(host, resolve_port(input.port, 3389)).await;
    }

    if conn_type.eq_ignore_ascii_case("VNC") {
        return test_tcp_endpoint(host, resolve_port(input.port, 5900)).await;
    }

    let auth_method = input.auth_method.as_deref().unwrap_or("password");
    let auth = if auth_method == "key" {
        let key_id = input
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
    } else {
        let password = input
            .password
            .ok_or(AppError::Validation("password is required for test".into()))?;
        ssh_core::session::SshAuth::Password(password)
    };

    let creds = ssh_core::session::SshCredentials {
        host: host.to_string(),
        port: resolve_port(input.port, 22),
        username: input.username.unwrap_or_else(|| "root".into()),
        auth,
    };

    match ssh_core::session::connect_and_authenticate(
        &creds,
        ssh_core::session::SshHandler::permissive(creds.host.clone(), creds.port),
    )
    .await
    {
        Ok(_handle) => Ok(true),
        Err(e) => Err(AppError::Ssh(e)),
    }
}

#[tauri::command]
pub async fn connection_clone(state: State<'_, AppState>, id: i64) -> CmdResult<i64> {
    state.auth.require_auth().await?;
    let conn = state
        .conn_repo
        .get_connection(id)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::NotFound("connection not found".into()))?;
    let input = ConnectionInput {
        name: format!("{} (copy)", conn.name),
        conn_type: Some(conn.conn_type),
        host: conn.host,
        port: Some(conn.port),
        username: Some(conn.username),
        auth_method: Some(conn.auth_method),
        password: None,
        ssh_key_id: conn.ssh_key_id,
        proxy_id: conn.proxy_id,
        jump_chain: conn.jump_chain,
        notes: conn.notes,
        rdp_options: conn.rdp_options,
        vnc_options: conn.vnc_options,
        provider: conn.provider,
        region: conn.region,
        expiry_date: conn.expiry_date,
        billing_cycle: conn.billing_cycle,
        billing_amount: conn.billing_amount,
        billing_currency: conn.billing_currency,
        sort_order: Some(conn.sort_order),
        tags: Some(conn.tags),
    };
    // Pass encrypted_password directly (already encrypted)
    state
        .conn_repo
        .create_connection(&input, conn.encrypted_password.as_deref())
        .await
        .map_err(AppError::from)
}

/// Exportable connection (no encrypted fields).
#[derive(Serialize, Deserialize)]
pub struct ExportConnection {
    pub name: String,
    #[serde(rename = "type")]
    pub conn_type: String,
    pub host: String,
    pub port: i32,
    pub username: String,
    pub auth_method: String,
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
    pub tags: Vec<String>,
}

#[tauri::command]
pub async fn connection_export(
    state: State<'_, AppState>,
    ids: Option<Vec<i64>>,
) -> CmdResult<String> {
    state.auth.require_auth().await?;
    let all = state
        .conn_repo
        .list_connections()
        .await
        .map_err(AppError::from)?;
    let filtered: Vec<ExportConnection> = all
        .into_iter()
        .filter(|c| ids.as_ref().map_or(true, |list| list.contains(&c.id)))
        .map(|c| ExportConnection {
            name: c.name,
            conn_type: c.conn_type,
            host: c.host,
            port: c.port,
            username: c.username,
            auth_method: c.auth_method,
            ssh_key_id: c.ssh_key_id,
            proxy_id: c.proxy_id,
            jump_chain: c.jump_chain,
            notes: c.notes,
            rdp_options: c.rdp_options,
            vnc_options: c.vnc_options,
            provider: c.provider,
            region: c.region,
            expiry_date: c.expiry_date,
            billing_cycle: c.billing_cycle,
            billing_amount: c.billing_amount,
            billing_currency: c.billing_currency,
            tags: c.tags,
        })
        .collect();
    serde_json::to_string_pretty(&filtered).map_err(|e| AppError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn connection_import(state: State<'_, AppState>, json: String) -> CmdResult<Vec<i64>> {
    state.auth.require_auth().await?;
    let items: Vec<ExportConnection> = serde_json::from_str(&json)
        .map_err(|e| AppError::Validation(format!("invalid JSON: {e}")))?;
    let mut ids = Vec::with_capacity(items.len());
    for item in items {
        let input = ConnectionInput {
            name: item.name,
            conn_type: Some(item.conn_type),
            host: item.host,
            port: Some(item.port),
            username: Some(item.username),
            auth_method: Some(item.auth_method),
            password: None,
            ssh_key_id: item.ssh_key_id,
            proxy_id: item.proxy_id,
            jump_chain: item.jump_chain,
            notes: item.notes,
            rdp_options: item.rdp_options,
            vnc_options: item.vnc_options,
            provider: item.provider,
            region: item.region,
            expiry_date: item.expiry_date,
            billing_cycle: item.billing_cycle,
            billing_amount: item.billing_amount,
            billing_currency: item.billing_currency,
            sort_order: None,
            tags: Some(item.tags),
        };
        let id = state
            .conn_repo
            .create_connection(&input, None)
            .await
            .map_err(AppError::from)?;
        ids.push(id);
    }
    Ok(ids)
}

// ── Full App Backup Export / Import ──

/// SSH Key stub exported without private key material.
#[derive(Serialize, Deserialize)]
pub struct ExportSshKeyStub {
    pub name: String,
    pub private_key_pem: Option<String>,
    pub passphrase: Option<String>,
}

/// Proxy exported without encrypted credential fields.
#[derive(Serialize, Deserialize)]
pub struct ExportProxyFull {
    pub name: String,
    pub proxy_type: String,
    pub host: String,
    pub port: i32,
    pub username: Option<String>,
    pub auth_method: String,
    pub password: Option<String>,
    pub private_key_pem: Option<String>,
}

/// Connection exported with name-based references (no raw IDs).
#[derive(Serialize, Deserialize)]
pub struct ExportConnectionV2 {
    pub name: String,
    #[serde(rename = "type")]
    pub conn_type: String,
    pub host: String,
    pub port: i32,
    pub username: String,
    pub auth_method: String,
    pub password: Option<String>,
    /// Name of the SSH key to use (when auth_method == "key").
    pub ssh_key_name: Option<String>,
    /// Name of the proxy to use.
    pub proxy_name: Option<String>,
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
    pub tags: Vec<String>,
}

/// Quick command exported with tag names instead of IDs.
#[derive(Serialize, Deserialize)]
pub struct ExportQuickCommand {
    pub name: String,
    pub command: String,
    pub variables: Option<String>,
    pub tags: Vec<String>,
}

/// Favorite path exported with connection name instead of ID.
#[derive(Serialize, Deserialize)]
pub struct ExportFavoritePath {
    pub name: String,
    pub path: String,
    /// Name of the linked connection (if any).
    pub connection_name: Option<String>,
}

/// Terminal theme exported (user-created only).
#[derive(Serialize, Deserialize)]
pub struct ExportTerminalTheme {
    pub name: String,
    pub background: Option<String>,
    pub foreground: Option<String>,
    pub cursor: Option<String>,
    pub cursor_accent: Option<String>,
    pub selection_background: Option<String>,
    pub selection_foreground: Option<String>,
    pub selection_inactive_background: Option<String>,
    pub black: Option<String>,
    pub red: Option<String>,
    pub green: Option<String>,
    pub yellow: Option<String>,
    pub blue: Option<String>,
    pub magenta: Option<String>,
    pub cyan: Option<String>,
    pub white: Option<String>,
    pub bright_black: Option<String>,
    pub bright_red: Option<String>,
    pub bright_green: Option<String>,
    pub bright_yellow: Option<String>,
    pub bright_blue: Option<String>,
    pub bright_magenta: Option<String>,
    pub bright_cyan: Option<String>,
    pub bright_white: Option<String>,
}

/// Key-value setting pair (reuses same shape as Setting model).
#[derive(Serialize, Deserialize)]
pub struct ExportSetting {
    pub key: String,
    pub value: String,
}

/// Notification channel exported.
#[derive(Serialize, Deserialize)]
pub struct ExportNotificationChannel {
    pub channel_type: String,
    pub name: String,
    pub enabled: bool,
    pub config: String,
    pub enabled_events: String,
}

/// Top-level unified backup format (v1).
#[derive(Serialize, Deserialize)]
pub struct AppExportData {
    pub version: u32,
    pub ssh_keys: Vec<ExportSshKeyStub>,
    pub proxies: Vec<ExportProxyFull>,
    pub connections: Vec<ExportConnectionV2>,
    pub quick_command_tags: Vec<String>,
    pub quick_commands: Vec<ExportQuickCommand>,
    pub favorite_paths: Vec<ExportFavoritePath>,
    pub terminal_themes: Vec<ExportTerminalTheme>,
    pub settings: Vec<ExportSetting>,
    pub appearance: Vec<ExportSetting>,
    pub notification_channels: Vec<ExportNotificationChannel>,
}

/// Settings key prefixes that must never be exported / imported.
const PROTECTED_KEY_PREFIXES: &[&str] = &["auth.", "ai."];

fn is_protected_key(key: &str) -> bool {
    PROTECTED_KEY_PREFIXES
        .iter()
        .any(|prefix| key.starts_with(prefix))
}

/// Result counts returned after a successful import.
#[derive(Serialize)]
pub struct ImportResult {
    pub connections: usize,
    pub proxies: usize,
    pub ssh_keys: usize,
    pub quick_commands: usize,
    pub quick_command_tags: usize,
    pub favorite_paths: usize,
    pub terminal_themes: usize,
    pub settings: usize,
    pub appearance: usize,
    pub notification_channels: usize,
}

#[tauri::command]
pub async fn app_export(state: State<'_, AppState>) -> CmdResult<String> {
    state.auth.require_auth().await?;

    // Build name lookup maps
    let ssh_keys = state
        .conn_repo
        .list_ssh_keys()
        .await
        .map_err(AppError::from)?;
    let key_id_to_name: HashMap<i64, String> =
        ssh_keys.iter().map(|k| (k.id, k.name.clone())).collect();

    let proxies = state
        .conn_repo
        .list_proxies()
        .await
        .map_err(AppError::from)?;
    let proxy_id_to_name: HashMap<i64, String> =
        proxies.iter().map(|p| (p.id, p.name.clone())).collect();

    let connections = state
        .conn_repo
        .list_connections()
        .await
        .map_err(AppError::from)?;
    let conn_id_to_name: HashMap<i64, String> =
        connections.iter().map(|c| (c.id, c.name.clone())).collect();

    let export_ssh_keys: Vec<ExportSshKeyStub> = ssh_keys
        .into_iter()
        .map(|k| {
            let private_key_pem = state.crypto.decrypt(&k.encrypted_private_key).ok();
            let passphrase = k
                .encrypted_passphrase
                .as_deref()
                .and_then(|value| state.crypto.decrypt(value).ok());
            ExportSshKeyStub {
                name: k.name,
                private_key_pem,
                passphrase,
            }
        })
        .collect();

    let export_proxies: Vec<ExportProxyFull> = proxies
        .into_iter()
        .map(|p| ExportProxyFull {
            name: p.name,
            proxy_type: p.proxy_type,
            host: p.host,
            port: p.port,
            username: p.username,
            auth_method: p.auth_method,
            password: p
                .encrypted_password
                .as_deref()
                .and_then(|value| state.crypto.decrypt(value).ok()),
            private_key_pem: p
                .encrypted_private_key
                .as_deref()
                .and_then(|value| state.crypto.decrypt(value).ok()),
        })
        .collect();

    let export_connections: Vec<ExportConnectionV2> = connections
        .into_iter()
        .map(|c| ExportConnectionV2 {
            name: c.name,
            conn_type: c.conn_type,
            host: c.host,
            port: c.port,
            username: c.username,
            auth_method: c.auth_method,
            password: c
                .encrypted_password
                .as_deref()
                .and_then(|value| state.crypto.decrypt(value).ok()),
            ssh_key_name: c.ssh_key_id.and_then(|id| key_id_to_name.get(&id).cloned()),
            proxy_name: c.proxy_id.and_then(|id| proxy_id_to_name.get(&id).cloned()),
            jump_chain: c.jump_chain,
            notes: c.notes,
            rdp_options: c.rdp_options,
            vnc_options: c.vnc_options,
            provider: c.provider,
            region: c.region,
            expiry_date: c.expiry_date,
            billing_cycle: c.billing_cycle,
            billing_amount: c.billing_amount,
            billing_currency: c.billing_currency,
            tags: c.tags,
        })
        .collect();

    // Quick commands
    let qc_tags = state.qc_repo.list_tags().await.map_err(AppError::from)?;
    let export_qc_tags: Vec<String> = qc_tags.into_iter().map(|t| t.name).collect();

    let qc_list = state.qc_repo.list().await.map_err(AppError::from)?;
    let export_qc: Vec<ExportQuickCommand> = qc_list
        .into_iter()
        .map(|q| ExportQuickCommand {
            name: q.name,
            command: q.command,
            variables: q.variables,
            tags: q.tags,
        })
        .collect();

    // Favorite paths
    let fav_paths = state
        .history_repo
        .list_favorite_paths(None)
        .await
        .map_err(AppError::from)?;
    let export_fav: Vec<ExportFavoritePath> = fav_paths
        .into_iter()
        .map(|f| ExportFavoritePath {
            name: f.name,
            path: f.path,
            connection_name: f
                .connection_id
                .and_then(|id| conn_id_to_name.get(&id).cloned()),
        })
        .collect();

    // Terminal themes (user-created only)
    let all_themes = state
        .settings_repo
        .list_themes()
        .await
        .map_err(AppError::from)?;
    let export_themes: Vec<ExportTerminalTheme> = all_themes
        .into_iter()
        .filter(|t| t.theme_type == "user")
        .map(|t| ExportTerminalTheme {
            name: t.name,
            background: t.background,
            foreground: t.foreground,
            cursor: t.cursor,
            cursor_accent: t.cursor_accent,
            selection_background: t.selection_background,
            selection_foreground: t.selection_foreground,
            selection_inactive_background: t.selection_inactive_background,
            black: t.black,
            red: t.red,
            green: t.green,
            yellow: t.yellow,
            blue: t.blue,
            magenta: t.magenta,
            cyan: t.cyan,
            white: t.white,
            bright_black: t.bright_black,
            bright_red: t.bright_red,
            bright_green: t.bright_green,
            bright_yellow: t.bright_yellow,
            bright_blue: t.bright_blue,
            bright_magenta: t.bright_magenta,
            bright_cyan: t.bright_cyan,
            bright_white: t.bright_white,
        })
        .collect();

    // Settings (exclude protected prefixes)
    let all_settings = state
        .settings_repo
        .get_all_settings()
        .await
        .map_err(AppError::from)?;
    let export_settings: Vec<ExportSetting> = all_settings
        .into_iter()
        .filter(|s| !is_protected_key(&s.key))
        .map(|s| ExportSetting {
            key: s.key,
            value: s.value,
        })
        .collect();

    // Appearance settings (exclude protected prefixes)
    let all_appearance = state
        .settings_repo
        .get_all_appearance()
        .await
        .map_err(AppError::from)?;
    let export_appearance: Vec<ExportSetting> = all_appearance
        .into_iter()
        .filter(|s| !is_protected_key(&s.key))
        .map(|s| ExportSetting {
            key: s.key,
            value: s.value,
        })
        .collect();

    // Notification channels
    let channels = state
        .settings_repo
        .list_notification_channels()
        .await
        .map_err(AppError::from)?;
    let export_channels: Vec<ExportNotificationChannel> = channels
        .into_iter()
        .map(|c| ExportNotificationChannel {
            channel_type: c.channel_type,
            name: c.name,
            enabled: c.enabled,
            config: c.config,
            enabled_events: c.enabled_events,
        })
        .collect();

    let payload = AppExportData {
        version: 2,
        ssh_keys: export_ssh_keys,
        proxies: export_proxies,
        connections: export_connections,
        quick_command_tags: export_qc_tags,
        quick_commands: export_qc,
        favorite_paths: export_fav,
        terminal_themes: export_themes,
        settings: export_settings,
        appearance: export_appearance,
        notification_channels: export_channels,
    };

    serde_json::to_string_pretty(&payload).map_err(|e| AppError::Internal(e.to_string()))
}

#[tauri::command]
pub async fn app_import(state: State<'_, AppState>, json: String) -> CmdResult<ImportResult> {
    state.auth.require_auth().await?;

    let data: AppExportData = serde_json::from_str(&json)
        .map_err(|e| AppError::Validation(format!("无效的备份文件: {e}")))?;

    // Begin transaction directly on the pool for atomic import
    let mut tx = state
        .storage
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("transaction begin failed: {e}")))?;

    let mut ssh_key_name_to_id: HashMap<String, i64> = HashMap::new();
    let mut qc_tag_name_to_id: HashMap<String, i64> = HashMap::new();
    let mut proxy_name_to_id: HashMap<String, i64> = HashMap::new();
    let mut conn_name_to_id: HashMap<String, i64> = HashMap::new();

    // 1. SSH Keys
    let mut ssh_keys_count = 0usize;
    for key in &data.ssh_keys {
        let encrypted_private_key = key
            .private_key_pem
            .as_deref()
            .map(|value| state.crypto.encrypt(value))
            .transpose()
            .map_err(|e| AppError::Crypto(e.to_string()))?
            .unwrap_or_default();
        let encrypted_passphrase = key
            .passphrase
            .as_deref()
            .map(|value| state.crypto.encrypt(value))
            .transpose()
            .map_err(|e| AppError::Crypto(e.to_string()))?;
        let result = sqlx::query(
            "INSERT INTO ssh_keys (name, encrypted_private_key, encrypted_passphrase) VALUES (?, ?, ?)",
        )
        .bind(&key.name)
        .bind(encrypted_private_key)
        .bind(encrypted_passphrase)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(format!("insert ssh_key '{}': {e}", key.name)))?;
        // Use entry().or_insert() so first occurrence wins on duplicate names
        ssh_key_name_to_id
            .entry(key.name.clone())
            .or_insert(result.last_insert_rowid());
        ssh_keys_count += 1;
    }

    // 2. Quick command tags
    let mut qc_tags_count = 0usize;
    for tag_name in &data.quick_command_tags {
        sqlx::query("INSERT OR IGNORE INTO quick_command_tags (name) VALUES (?)")
            .bind(tag_name)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(format!("insert qc_tag '{tag_name}': {e}")))?;
        let row: (i64,) = sqlx::query_as("SELECT id FROM quick_command_tags WHERE name = ?")
            .bind(tag_name)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(format!("select qc_tag '{tag_name}': {e}")))?;
        qc_tag_name_to_id.entry(tag_name.clone()).or_insert(row.0);
        qc_tags_count += 1;
    }

    // 3. Proxies
    let mut proxies_count = 0usize;
    for proxy in &data.proxies {
        let encrypted_password = proxy
            .password
            .as_deref()
            .map(|value| state.crypto.encrypt(value))
            .transpose()
            .map_err(|e| AppError::Crypto(e.to_string()))?;
        let encrypted_private_key = proxy
            .private_key_pem
            .as_deref()
            .map(|value| state.crypto.encrypt(value))
            .transpose()
            .map_err(|e| AppError::Crypto(e.to_string()))?;
        let result = sqlx::query(
            "INSERT INTO proxies (name, type, host, port, username, auth_method, encrypted_password, encrypted_private_key) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&proxy.name)
        .bind(&proxy.proxy_type)
        .bind(&proxy.host)
        .bind(proxy.port)
        .bind(&proxy.username)
        .bind(&proxy.auth_method)
        .bind(encrypted_password)
        .bind(encrypted_private_key)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(format!("insert proxy '{}': {e}", proxy.name)))?;
        proxy_name_to_id
            .entry(proxy.name.clone())
            .or_insert(result.last_insert_rowid());
        proxies_count += 1;
    }

    // 4. Connections (with name-based SSH key / proxy resolution)
    let mut connections_count = 0usize;
    for conn in &data.connections {
        // Validate ssh_key reference when auth_method == "key"
        let resolved_ssh_key_id = if conn.auth_method == "key" {
            match &conn.ssh_key_name {
                Some(key_name) => {
                    let id = ssh_key_name_to_id.get(key_name.as_str()).copied();
                    if id.is_none() {
                        return Err(AppError::Validation(format!(
                            "连接 '{}': auth_method=key 但 SSH 密钥 '{}' 不在导入数据中",
                            conn.name, key_name
                        )));
                    }
                    id
                }
                None => {
                    return Err(AppError::Validation(format!(
                        "连接 '{}': auth_method=key 但未指定 ssh_key_name",
                        conn.name
                    )));
                }
            }
        } else {
            None
        };

        let resolved_proxy_id = conn
            .proxy_name
            .as_deref()
            .and_then(|name| proxy_name_to_id.get(name).copied());
        let encrypted_password = conn
            .password
            .as_deref()
            .map(|value| state.crypto.encrypt(value))
            .transpose()
            .map_err(|e| AppError::Crypto(e.to_string()))?;

        let result = sqlx::query(
            "INSERT INTO connections \
             (name, type, host, port, username, auth_method, encrypted_password, ssh_key_id, proxy_id, \
              jump_chain, notes, rdp_options, vnc_options, \
              provider, region, expiry_date, billing_cycle, billing_amount, billing_currency) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&conn.name)
        .bind(&conn.conn_type)
        .bind(&conn.host)
        .bind(conn.port)
        .bind(&conn.username)
        .bind(&conn.auth_method)
        .bind(encrypted_password)
        .bind(resolved_ssh_key_id)
        .bind(resolved_proxy_id)
        .bind(&conn.jump_chain)
        .bind(&conn.notes)
        .bind(&conn.rdp_options)
        .bind(&conn.vnc_options)
        .bind(&conn.provider)
        .bind(&conn.region)
        .bind(&conn.expiry_date)
        .bind(&conn.billing_cycle)
        .bind(conn.billing_amount)
        .bind(&conn.billing_currency)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(format!("insert connection '{}': {e}", conn.name)))?;

        let connection_id = result.last_insert_rowid();
        // First occurrence wins on duplicate connection names
        conn_name_to_id
            .entry(conn.name.clone())
            .or_insert(connection_id);

        // Insert tags (create if missing, then associate)
        for tag_name in &conn.tags {
            sqlx::query("INSERT OR IGNORE INTO tags (name) VALUES (?)")
                .bind(tag_name)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::Internal(format!("insert tag '{tag_name}': {e}")))?;

            sqlx::query(
                "INSERT OR IGNORE INTO connection_tags (connection_id, tag_id) \
                 SELECT ?, id FROM tags WHERE name = ?",
            )
            .bind(connection_id)
            .bind(tag_name)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(format!("link tag '{tag_name}': {e}")))?;
        }

        connections_count += 1;
    }

    // 5. Quick commands
    let mut qc_count = 0usize;
    for qc in &data.quick_commands {
        let result =
            sqlx::query("INSERT INTO quick_commands (name, command, variables) VALUES (?, ?, ?)")
                .bind(&qc.name)
                .bind(&qc.command)
                .bind(&qc.variables)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    AppError::Internal(format!("insert quick_command '{}': {e}", qc.name))
                })?;

        let qc_id = result.last_insert_rowid();
        for tag_name in &qc.tags {
            if let Some(&tag_id) = qc_tag_name_to_id.get(tag_name.as_str()) {
                sqlx::query(
                    "INSERT OR IGNORE INTO quick_command_tag_associations \
                     (quick_command_id, tag_id) VALUES (?, ?)",
                )
                .bind(qc_id)
                .bind(tag_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::Internal(format!("link qc tag '{tag_name}': {e}")))?;
            }
        }
        qc_count += 1;
    }

    // 6. Favorite paths
    let mut fav_count = 0usize;
    for fav in &data.favorite_paths {
        let resolved_conn_id: Option<i64> = fav
            .connection_name
            .as_deref()
            .and_then(|name| conn_name_to_id.get(name).copied());

        sqlx::query("INSERT INTO favorite_paths (name, path, connection_id) VALUES (?, ?, ?)")
            .bind(&fav.name)
            .bind(&fav.path)
            .bind(resolved_conn_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(format!("insert favorite_path '{}': {e}", fav.name)))?;
        fav_count += 1;
    }

    // 7. Terminal themes (user type only)
    let mut themes_count = 0usize;
    for theme in &data.terminal_themes {
        sqlx::query(
            "INSERT INTO terminal_themes \
             (name, theme_type, background, foreground, cursor, cursor_accent, \
              selection_background, selection_foreground, selection_inactive_background, \
              black, red, green, yellow, blue, magenta, cyan, white, \
              bright_black, bright_red, bright_green, bright_yellow, \
              bright_blue, bright_magenta, bright_cyan, bright_white) \
             VALUES (?, 'user', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, \
                     ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&theme.name)
        .bind(&theme.background)
        .bind(&theme.foreground)
        .bind(&theme.cursor)
        .bind(&theme.cursor_accent)
        .bind(&theme.selection_background)
        .bind(&theme.selection_foreground)
        .bind(&theme.selection_inactive_background)
        .bind(&theme.black)
        .bind(&theme.red)
        .bind(&theme.green)
        .bind(&theme.yellow)
        .bind(&theme.blue)
        .bind(&theme.magenta)
        .bind(&theme.cyan)
        .bind(&theme.white)
        .bind(&theme.bright_black)
        .bind(&theme.bright_red)
        .bind(&theme.bright_green)
        .bind(&theme.bright_yellow)
        .bind(&theme.bright_blue)
        .bind(&theme.bright_magenta)
        .bind(&theme.bright_cyan)
        .bind(&theme.bright_white)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(format!("insert theme '{}': {e}", theme.name)))?;
        themes_count += 1;
    }

    // 8. Settings (skip protected keys)
    let mut settings_count = 0usize;
    for s in &data.settings {
        if is_protected_key(&s.key) {
            continue;
        }
        sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)")
            .bind(&s.key)
            .bind(&s.value)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(format!("insert setting '{}': {e}", s.key)))?;
        settings_count += 1;
    }

    // 9. Appearance settings (skip protected keys)
    let mut appearance_count = 0usize;
    for s in &data.appearance {
        if is_protected_key(&s.key) {
            continue;
        }
        sqlx::query("INSERT OR REPLACE INTO appearance_settings (key, value) VALUES (?, ?)")
            .bind(&s.key)
            .bind(&s.value)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Internal(format!("insert appearance '{}': {e}", s.key)))?;
        appearance_count += 1;
    }

    // 10. Notification channels
    let mut channels_count = 0usize;
    for ch in &data.notification_channels {
        sqlx::query(
            "INSERT INTO notification_settings \
             (channel_type, name, enabled, config, enabled_events) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&ch.channel_type)
        .bind(&ch.name)
        .bind(ch.enabled)
        .bind(&ch.config)
        .bind(&ch.enabled_events)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal(format!("insert channel '{}': {e}", ch.name)))?;
        channels_count += 1;
    }

    // Commit the transaction — any error above causes automatic rollback
    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("transaction commit failed: {e}")))?;

    Ok(ImportResult {
        connections: connections_count,
        proxies: proxies_count,
        ssh_keys: ssh_keys_count,
        quick_commands: qc_count,
        quick_command_tags: qc_tags_count,
        favorite_paths: fav_count,
        terminal_themes: themes_count,
        settings: settings_count,
        appearance: appearance_count,
        notification_channels: channels_count,
    })
}
