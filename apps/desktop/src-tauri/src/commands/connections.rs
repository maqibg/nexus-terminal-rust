//! Connection Tauri commands.

use api_contract::error::AppError;
use connection_core::model::{Connection, ConnectionInput, Proxy, SshKey, Tag};
use connection_core::repository::ConnectionRepository;
use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

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
    state
        .conn_repo
        .list_connections()
        .await
        .map_err(AppError::Database)
}

#[tauri::command]
pub async fn connection_get(state: State<'_, AppState>, id: i64) -> CmdResult<Connection> {
    state.auth.require_auth().await?;
    state
        .conn_repo
        .get_connection(id)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("connection not found".into()))
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
        .map_err(AppError::Database)
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
        .map_err(AppError::Database)
}

#[tauri::command]
pub async fn connection_delete(state: State<'_, AppState>, id: i64) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    state
        .conn_repo
        .delete_connection(id)
        .await
        .map_err(AppError::Database)
}

#[tauri::command]
pub async fn connection_reorder(state: State<'_, AppState>, req: ReorderRequest) -> CmdResult<()> {
    state.auth.require_auth().await?;
    state
        .conn_repo
        .reorder_connections(&req.ids)
        .await
        .map_err(AppError::Database)
}

#[tauri::command]
pub async fn tag_list(state: State<'_, AppState>) -> CmdResult<Vec<Tag>> {
    state.auth.require_auth().await?;
    state
        .conn_repo
        .list_tags()
        .await
        .map_err(AppError::Database)
}

#[tauri::command]
pub async fn tag_create(state: State<'_, AppState>, req: TagCreateRequest) -> CmdResult<i64> {
    state.auth.require_auth().await?;
    state
        .conn_repo
        .create_tag(&req.name)
        .await
        .map_err(AppError::Database)
}

#[tauri::command]
pub async fn tag_delete(state: State<'_, AppState>, id: i64) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    state
        .conn_repo
        .delete_tag(id)
        .await
        .map_err(AppError::Database)
}

#[tauri::command]
pub async fn ssh_key_list(state: State<'_, AppState>) -> CmdResult<Vec<SshKey>> {
    state.auth.require_auth().await?;
    state
        .conn_repo
        .list_ssh_keys()
        .await
        .map_err(AppError::Database)
}

#[tauri::command]
pub async fn ssh_key_delete(state: State<'_, AppState>, id: i64) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    state
        .conn_repo
        .delete_ssh_key(id)
        .await
        .map_err(AppError::Database)
}

#[tauri::command]
pub async fn proxy_list(state: State<'_, AppState>) -> CmdResult<Vec<Proxy>> {
    state.auth.require_auth().await?;
    state
        .conn_repo
        .list_proxies()
        .await
        .map_err(AppError::Database)
}

#[tauri::command]
pub async fn proxy_delete(state: State<'_, AppState>, id: i64) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    state
        .conn_repo
        .delete_proxy(id)
        .await
        .map_err(AppError::Database)
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
        .map_err(AppError::Database)
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
        .map_err(AppError::Database)
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
        .map_err(AppError::Database)
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
        .map_err(AppError::Database)
}

// ── Connection Test / Clone / Export / Import ──

#[tauri::command]
pub async fn connection_test(state: State<'_, AppState>, id: i64) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    let conn = state
        .conn_repo
        .get_connection(id)
        .await
        .map_err(AppError::Database)?
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
            .map_err(AppError::Database)?
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

    match ssh_core::session::connect_and_authenticate(&creds).await {
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
            .map_err(AppError::Database)?
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

    match ssh_core::session::connect_and_authenticate(&creds).await {
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
        .map_err(AppError::Database)?
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
        sort_order: Some(conn.sort_order),
        tags: Some(conn.tags),
    };
    // Pass encrypted_password directly (already encrypted)
    state
        .conn_repo
        .create_connection(&input, conn.encrypted_password.as_deref())
        .await
        .map_err(AppError::Database)
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
        .map_err(AppError::Database)?;
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
            sort_order: None,
            tags: Some(item.tags),
        };
        let id = state
            .conn_repo
            .create_connection(&input, None)
            .await
            .map_err(AppError::Database)?;
        ids.push(id);
    }
    Ok(ids)
}
