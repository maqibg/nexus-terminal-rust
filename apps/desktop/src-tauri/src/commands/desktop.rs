//! Desktop-only system integration commands (RDP / VNC).

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock};

use api_contract::error::AppError;
use connection_core::repository::ConnectionRepository;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct DesktopRdpGatewayOptions {
    pub enabled: bool,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct DesktopRdpOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fullscreen: Option<bool>,
    pub multimon: Option<bool>,
    pub admin: Option<bool>,
    pub restricted_admin: Option<bool>,
    pub remote_guard: Option<bool>,
    pub drives: Option<bool>,
    pub printers: Option<bool>,
    pub clipboard: Option<bool>,
    pub audio: Option<String>,
    pub color_depth: Option<u8>,
    pub compression: Option<bool>,
    pub gateway: Option<DesktopRdpGatewayOptions>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct DesktopVncOptions {
    pub view_only: Option<bool>,
    pub quality: Option<u8>,
    pub compression: Option<u8>,
    pub local_cursor: Option<bool>,
    pub shared_connection: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DesktopOpenRdpRequest {
    pub host: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub options: Option<DesktopRdpOptions>,
    pub connection_id: Option<i64>,
    pub connection_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DesktopOpenVncRequest {
    pub host: String,
    pub port: Option<u16>,
    pub password: Option<String>,
    pub options: Option<DesktopVncOptions>,
    pub connection_id: Option<i64>,
    pub connection_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopRdpSessionStatus {
    pub connection_id: i64,
    pub connection_name: Option<String>,
    pub host: String,
    pub port: u16,
    pub status: String,
    pub process_id: Option<u32>,
    pub started_at_ms: i64,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopVncSessionInfo {
    pub session_id: String,
    pub connection_id: Option<i64>,
    pub connection_name: Option<String>,
    pub host: String,
    pub port: u16,
    pub ws_port: u16,
    pub status: String,
    pub created_at_ms: i64,
    pub last_error: Option<String>,
    pub password: Option<String>,
    pub options: DesktopVncOptions,
}

#[derive(Debug, Clone)]
struct RdpSessionEntry {
    connection_id: i64,
    connection_name: Option<String>,
    host: String,
    port: u16,
    status: String,
    process_id: Option<u32>,
    started_at_ms: i64,
    last_error: Option<String>,
}

#[derive(Default)]
struct RdpSessionManager {
    sessions: Mutex<HashMap<i64, RdpSessionEntry>>,
}

impl RdpSessionManager {
    fn upsert(&self, entry: RdpSessionEntry) {
        if let Ok(mut guard) = self.sessions.lock() {
            guard.insert(entry.connection_id, entry);
        }
    }

    fn update_status(
        &self,
        connection_id: i64,
        status: &str,
        process_id: Option<u32>,
        last_error: Option<String>,
    ) {
        if let Ok(mut guard) = self.sessions.lock() {
            if let Some(entry) = guard.get_mut(&connection_id) {
                entry.status = status.to_string();
                entry.process_id = process_id;
                entry.last_error = last_error;
            }
        }
    }

    fn get(&self, connection_id: i64) -> Option<DesktopRdpSessionStatus> {
        let guard = self.sessions.lock().ok()?;
        let entry = guard.get(&connection_id)?.clone();
        Some(DesktopRdpSessionStatus {
            connection_id: entry.connection_id,
            connection_name: entry.connection_name,
            host: entry.host,
            port: entry.port,
            status: entry.status,
            process_id: entry.process_id,
            started_at_ms: entry.started_at_ms,
            last_error: entry.last_error,
        })
    }

    fn list(&self) -> Vec<DesktopRdpSessionStatus> {
        let guard = match self.sessions.lock() {
            Ok(guard) => guard,
            Err(_) => return Vec::new(),
        };

        guard
            .values()
            .cloned()
            .map(|entry| DesktopRdpSessionStatus {
                connection_id: entry.connection_id,
                connection_name: entry.connection_name,
                host: entry.host,
                port: entry.port,
                status: entry.status,
                process_id: entry.process_id,
                started_at_ms: entry.started_at_ms,
                last_error: entry.last_error,
            })
            .collect()
    }
}

struct VncSessionEntry {
    session_id: String,
    connection_id: Option<i64>,
    connection_name: Option<String>,
    host: String,
    port: u16,
    ws_port: u16,
    status: String,
    created_at_ms: i64,
    last_error: Option<String>,
    password: Option<String>,
    options: DesktopVncOptions,
    stop_tx: Option<oneshot::Sender<()>>,
}

#[derive(Default)]
struct VncSessionManager {
    sessions: Mutex<HashMap<String, VncSessionEntry>>,
    used_ports: Mutex<HashSet<u16>>,
}

impl VncSessionManager {
    async fn reserve_listener(&self) -> CmdResult<(u16, TcpListener)> {
        const MIN_PORT: u16 = 15900;
        const MAX_PORT: u16 = 15999;

        for port in MIN_PORT..=MAX_PORT {
            let reserved = {
                let mut used = self
                    .used_ports
                    .lock()
                    .map_err(|_| AppError::Internal("vnc used port lock poisoned".into()))?;
                if used.contains(&port) {
                    false
                } else {
                    used.insert(port);
                    true
                }
            };

            if !reserved {
                continue;
            }

            match TcpListener::bind(("127.0.0.1", port)).await {
                Ok(listener) => return Ok((port, listener)),
                Err(_) => {
                    self.release_port(port);
                }
            }
        }

        Err(AppError::Internal(
            "no available port for VNC websocket proxy".into(),
        ))
    }

    fn release_port(&self, port: u16) {
        if let Ok(mut used) = self.used_ports.lock() {
            used.remove(&port);
        }
    }

    fn insert(&self, session: VncSessionEntry) {
        if let Ok(mut guard) = self.sessions.lock() {
            guard.insert(session.session_id.clone(), session);
        }
    }

    fn update_status(&self, session_id: &str, status: &str, last_error: Option<String>) {
        if let Ok(mut guard) = self.sessions.lock() {
            if let Some(entry) = guard.get_mut(session_id) {
                entry.status = status.to_string();
                entry.last_error = last_error;
            }
        }
    }

    fn get(&self, session_id: &str) -> Option<DesktopVncSessionInfo> {
        let guard = self.sessions.lock().ok()?;
        let entry = guard.get(session_id)?;
        Some(vnc_entry_to_info(entry))
    }

    fn list(&self) -> Vec<DesktopVncSessionInfo> {
        let guard = match self.sessions.lock() {
            Ok(guard) => guard,
            Err(_) => return Vec::new(),
        };
        guard.values().map(vnc_entry_to_info).collect()
    }

    fn take(&self, session_id: &str) -> Option<VncSessionEntry> {
        let mut guard = self.sessions.lock().ok()?;
        guard.remove(session_id)
    }

    fn ensure_exists(&self, session_id: &str) -> bool {
        self.sessions
            .lock()
            .ok()
            .map(|guard| guard.contains_key(session_id))
            .unwrap_or(false)
    }
}

fn vnc_entry_to_info(entry: &VncSessionEntry) -> DesktopVncSessionInfo {
    DesktopVncSessionInfo {
        session_id: entry.session_id.clone(),
        connection_id: entry.connection_id,
        connection_name: entry.connection_name.clone(),
        host: entry.host.clone(),
        port: entry.port,
        ws_port: entry.ws_port,
        status: entry.status.clone(),
        created_at_ms: entry.created_at_ms,
        last_error: entry.last_error.clone(),
        password: entry.password.clone(),
        options: entry.options.clone(),
    }
}

fn now_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as i64,
        Err(_) => 0,
    }
}

fn parse_json_option<T>(raw: Option<&str>) -> CmdResult<T>
where
    T: serde::de::DeserializeOwned + Default,
{
    match raw {
        Some(text) if !text.trim().is_empty() => serde_json::from_str(text)
            .map_err(|e| AppError::Validation(format!("invalid desktop options JSON: {e}"))),
        _ => Ok(T::default()),
    }
}

fn rdp_manager() -> &'static Arc<RdpSessionManager> {
    static MANAGER: OnceLock<Arc<RdpSessionManager>> = OnceLock::new();
    MANAGER.get_or_init(|| Arc::new(RdpSessionManager::default()))
}

fn vnc_manager() -> &'static Arc<VncSessionManager> {
    static MANAGER: OnceLock<Arc<VncSessionManager>> = OnceLock::new();
    MANAGER.get_or_init(|| Arc::new(VncSessionManager::default()))
}

#[tauri::command]
pub async fn desktop_open_rdp(
    state: State<'_, AppState>,
    req: DesktopOpenRdpRequest,
) -> CmdResult<()> {
    state.auth.require_auth().await?;
    let options = req.options.unwrap_or_default();
    open_rdp_on_platform(
        &state,
        req.host.trim(),
        req.port,
        req.username.as_deref(),
        req.password.as_deref(),
        &options,
        req.connection_id,
        req.connection_name.as_deref(),
    )
}

#[tauri::command]
pub async fn desktop_open_rdp_connection(
    state: State<'_, AppState>,
    connection_id: i64,
) -> CmdResult<()> {
    state.auth.require_auth().await?;
    let conn = state
        .conn_repo
        .get_connection(connection_id)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("connection not found".into()))?;

    if !conn.conn_type.eq_ignore_ascii_case("RDP") {
        return Err(AppError::Validation(
            "only RDP connections can be opened with mstsc".into(),
        ));
    }

    let password = conn
        .encrypted_password
        .as_deref()
        .map(|encrypted| state.crypto.decrypt(encrypted))
        .transpose()
        .map_err(|e| AppError::Crypto(e.to_string()))?;

    let options = parse_json_option::<DesktopRdpOptions>(conn.rdp_options.as_deref())?;
    let port = u16::try_from(conn.port).ok().filter(|v| *v > 0);

    open_rdp_on_platform(
        &state,
        conn.host.trim(),
        port,
        Some(conn.username.as_str()),
        password.as_deref(),
        &options,
        Some(connection_id),
        Some(conn.name.as_str()),
    )
}

#[tauri::command]
pub async fn desktop_rdp_status(
    state: State<'_, AppState>,
    connection_id: i64,
) -> CmdResult<Option<DesktopRdpSessionStatus>> {
    state.auth.require_auth().await?;
    Ok(rdp_manager().get(connection_id))
}

#[tauri::command]
pub async fn desktop_rdp_list_sessions(
    state: State<'_, AppState>,
) -> CmdResult<Vec<DesktopRdpSessionStatus>> {
    state.auth.require_auth().await?;
    Ok(rdp_manager().list())
}

#[cfg(not(windows))]
#[tauri::command]
pub async fn desktop_rdp_disconnect_connection(
    state: State<'_, AppState>,
    _connection_id: i64,
) -> CmdResult<bool> {
    state.auth.require_auth().await?;
    Ok(false)
}

#[cfg(windows)]
#[tauri::command]
pub async fn desktop_rdp_disconnect_connection(
    state: State<'_, AppState>,
    connection_id: i64,
) -> CmdResult<bool> {
    use std::process::Command;

    state.auth.require_auth().await?;

    let status = rdp_manager().get(connection_id);
    let pid = status.and_then(|item| item.process_id);
    let Some(pid) = pid else {
        return Ok(false);
    };

    let result = Command::new("taskkill")
        .arg("/PID")
        .arg(pid.to_string())
        .arg("/T")
        .arg("/F")
        .status()
        .map_err(|e| AppError::Internal(format!("failed to execute taskkill: {e}")))?;

    if result.success() {
        rdp_manager().update_status(connection_id, "disconnected", None, None);
        Ok(true)
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub async fn desktop_open_vnc(
    state: State<'_, AppState>,
    req: DesktopOpenVncRequest,
) -> CmdResult<DesktopVncSessionInfo> {
    state.auth.require_auth().await?;

    let options = req.options.unwrap_or_default();
    start_vnc_session(
        req.host.trim(),
        req.port,
        req.password,
        options,
        req.connection_id,
        req.connection_name,
    )
    .await
}

#[tauri::command]
pub async fn desktop_open_vnc_connection(
    state: State<'_, AppState>,
    connection_id: i64,
) -> CmdResult<DesktopVncSessionInfo> {
    state.auth.require_auth().await?;
    let conn = state
        .conn_repo
        .get_connection(connection_id)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("connection not found".into()))?;

    if !conn.conn_type.eq_ignore_ascii_case("VNC") {
        return Err(AppError::Validation(
            "only VNC connections can be opened with VNC proxy".into(),
        ));
    }

    let password = conn
        .encrypted_password
        .as_deref()
        .map(|encrypted| state.crypto.decrypt(encrypted))
        .transpose()
        .map_err(|e| AppError::Crypto(e.to_string()))?;

    let options = parse_json_option::<DesktopVncOptions>(conn.vnc_options.as_deref())?;
    let port = u16::try_from(conn.port).ok().filter(|v| *v > 0);

    start_vnc_session(
        conn.host.trim(),
        port,
        password,
        options,
        Some(connection_id),
        Some(conn.name),
    )
    .await
}

#[tauri::command]
pub async fn desktop_vnc_status(
    state: State<'_, AppState>,
    session_id: String,
) -> CmdResult<Option<DesktopVncSessionInfo>> {
    state.auth.require_auth().await?;
    Ok(vnc_manager().get(session_id.trim()))
}

#[tauri::command]
pub async fn desktop_vnc_list_sessions(
    state: State<'_, AppState>,
) -> CmdResult<Vec<DesktopVncSessionInfo>> {
    state.auth.require_auth().await?;
    Ok(vnc_manager().list())
}

#[tauri::command]
pub async fn desktop_vnc_disconnect(
    state: State<'_, AppState>,
    session_id: String,
) -> CmdResult<bool> {
    state.auth.require_auth().await?;

    let trimmed = session_id.trim();
    if trimmed.is_empty() {
        return Ok(false);
    }

    if let Some(mut entry) = vnc_manager().take(trimmed) {
        if let Some(stop_tx) = entry.stop_tx.take() {
            let _ = stop_tx.send(());
        }
        vnc_manager().release_port(entry.ws_port);
        return Ok(true);
    }

    Ok(false)
}

async fn start_vnc_session(
    host: &str,
    port: Option<u16>,
    password: Option<String>,
    options: DesktopVncOptions,
    connection_id: Option<i64>,
    connection_name: Option<String>,
) -> CmdResult<DesktopVncSessionInfo> {
    if host.is_empty() {
        return Err(AppError::Validation("host is required".into()));
    }

    let port = port.filter(|value| *value > 0).unwrap_or(5900);
    let (ws_port, listener) = vnc_manager().reserve_listener().await?;
    let session_id = format!("vnc-{}", uuid::Uuid::new_v4());
    let (stop_tx, stop_rx) = oneshot::channel::<()>();

    let entry = VncSessionEntry {
        session_id: session_id.clone(),
        connection_id,
        connection_name,
        host: host.to_string(),
        port,
        ws_port,
        status: "connecting".into(),
        created_at_ms: now_ms(),
        last_error: None,
        password: password.clone(),
        options: options.clone(),
        stop_tx: Some(stop_tx),
    };
    vnc_manager().insert(entry);

    let manager = Arc::clone(vnc_manager());
    let task_session_id = session_id.clone();
    let task_host = host.to_string();
    tauri::async_runtime::spawn(async move {
        run_vnc_proxy(manager, task_session_id, task_host, port, listener, stop_rx).await;
    });

    vnc_manager()
        .get(&session_id)
        .ok_or(AppError::Internal("failed to create VNC session".into()))
}

async fn run_vnc_proxy(
    manager: Arc<VncSessionManager>,
    session_id: String,
    host: String,
    port: u16,
    listener: TcpListener,
    mut stop_rx: oneshot::Receiver<()>,
) {
    loop {
        tokio::select! {
            _ = &mut stop_rx => {
                manager.update_status(&session_id, "disconnected", None);
                break;
            }
            accepted = listener.accept() => {
                let Ok((socket, _addr)) = accepted else {
                    manager.update_status(&session_id, "error", Some("VNC websocket accept failed".into()));
                    break;
                };

                if !manager.ensure_exists(&session_id) {
                    break;
                }

                match handle_vnc_client(&session_id, &host, port, socket).await {
                    Ok(_) => {
                        manager.update_status(&session_id, "disconnected", None);
                    }
                    Err(error_message) => {
                        manager.update_status(&session_id, "error", Some(error_message));
                    }
                }
            }
        }
    }
}

async fn handle_vnc_client(
    session_id: &str,
    host: &str,
    port: u16,
    socket: tokio::net::TcpStream,
) -> Result<(), String> {
    let ws_stream = accept_async(socket)
        .await
        .map_err(|e| format!("VNC websocket handshake failed: {e}"))?;

    let server_stream = TcpStream::connect((host, port))
        .await
        .map_err(|e| format!("failed to connect VNC server: {e}"))?;

    vnc_manager().update_status(session_id, "connected", None);

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let (mut server_reader, mut server_writer) = server_stream.into_split();

    let tcp_to_ws = async {
        let mut buffer = vec![0_u8; 8192];
        loop {
            let read_size = server_reader
                .read(&mut buffer)
                .await
                .map_err(|e| format!("failed reading from VNC server: {e}"))?;
            if read_size == 0 {
                break;
            }
            ws_sender
                .send(Message::Binary(buffer[..read_size].to_vec().into()))
                .await
                .map_err(|e| format!("failed writing to websocket: {e}"))?;
        }
        Ok::<(), String>(())
    };

    let ws_to_tcp = async {
        while let Some(message) = ws_receiver.next().await {
            let message = message.map_err(|e| format!("failed reading websocket: {e}"))?;
            match message {
                Message::Binary(binary) => server_writer
                    .write_all(&binary)
                    .await
                    .map_err(|e| format!("failed writing to VNC server: {e}"))?,
                Message::Text(text) => server_writer
                    .write_all(text.as_bytes())
                    .await
                    .map_err(|e| format!("failed writing text to VNC server: {e}"))?,
                Message::Close(_) => break,
                _ => {}
            }
        }
        Ok::<(), String>(())
    };

    tokio::select! {
        result = tcp_to_ws => result?,
        result = ws_to_tcp => result?,
    }

    Ok(())
}

#[cfg(not(windows))]
fn open_rdp_on_platform(
    _state: &AppState,
    _host: &str,
    _port: Option<u16>,
    _username: Option<&str>,
    _password: Option<&str>,
    _options: &DesktopRdpOptions,
    _connection_id: Option<i64>,
    _connection_name: Option<&str>,
) -> CmdResult<()> {
    Err(AppError::Validation(
        "RDP launch is only supported on Windows desktop".into(),
    ))
}

#[cfg(windows)]
fn open_rdp_on_platform(
    state: &AppState,
    host: &str,
    port: Option<u16>,
    username: Option<&str>,
    password: Option<&str>,
    options: &DesktopRdpOptions,
    connection_id: Option<i64>,
    connection_name: Option<&str>,
) -> CmdResult<()> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    const DETACHED_PROCESS: u32 = 0x00000008;

    if host.is_empty() {
        return Err(AppError::Validation("host is required".into()));
    }

    let server_port = port.filter(|value| *value > 0).unwrap_or(3389);
    let server = if server_port == 3389 {
        host.to_string()
    } else {
        format!("{host}:{server_port}")
    };

    let mut credentials_added = false;
    if let (Some(user), Some(pass)) = (username, password) {
        let status = Command::new("cmdkey.exe")
            .arg(format!("/generic:TERMSRV/{host}"))
            .arg(format!("/user:{user}"))
            .arg(format!("/pass:{pass}"))
            .status()
            .map_err(|e| AppError::Internal(format!("failed to execute cmdkey: {e}")))?;

        if !status.success() {
            return Err(AppError::Internal(
                "cmdkey rejected stored credentials".into(),
            ));
        }
        credentials_added = true;
    }

    let temp_key = connection_id
        .map(|id| format!("conn-{id}"))
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let rdp_file_path = write_rdp_temp_file(
        state,
        &temp_key,
        &build_rdp_file_content(&server, username, options),
    )?;

    let mut args = vec![rdp_file_path.to_string_lossy().to_string()];
    if options.admin.unwrap_or(false) {
        args.push("/admin".into());
    }
    if options.restricted_admin.unwrap_or(false) {
        args.push("/restrictedAdmin".into());
    }
    if options.remote_guard.unwrap_or(false) {
        args.push("/remoteGuard".into());
    }

    let mut child = Command::new("mstsc.exe")
        .args(args)
        .creation_flags(DETACHED_PROCESS)
        .spawn()
        .map_err(|e| AppError::Internal(format!("failed to launch mstsc: {e}")))?;

    let process_id = Some(child.id());

    if let Some(cid) = connection_id {
        rdp_manager().upsert(RdpSessionEntry {
            connection_id: cid,
            connection_name: connection_name.map(|v| v.to_string()),
            host: host.to_string(),
            port: server_port,
            status: "connected".into(),
            process_id,
            started_at_ms: now_ms(),
            last_error: None,
        });
    }

    let wait_connection_id = connection_id;
    let wait_file = rdp_file_path.clone();
    std::thread::spawn(move || {
        let wait_result = child.wait();

        if let Some(cid) = wait_connection_id {
            match wait_result {
                Ok(exit) => {
                    if exit.success() {
                        rdp_manager().update_status(cid, "disconnected", None, None);
                    } else {
                        rdp_manager().update_status(
                            cid,
                            "error",
                            None,
                            Some(format!("mstsc exited with code {:?}", exit.code())),
                        );
                    }
                }
                Err(error) => {
                    rdp_manager().update_status(
                        cid,
                        "error",
                        None,
                        Some(format!("mstsc wait failed: {error}")),
                    );
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(5));
        let _ = std::fs::remove_file(&wait_file);
    });

    if credentials_added {
        let target = host.to_string();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(3));
            let _ = delete_cmdkey(&target);
        });
    }

    Ok(())
}

#[cfg(windows)]
fn write_rdp_temp_file(
    state: &AppState,
    key: &str,
    content: &str,
) -> CmdResult<std::path::PathBuf> {
    let dir = state.runtime_paths.temp_dir.join("nexus-terminal-rdp");
    std::fs::create_dir_all(&dir)
        .map_err(|e| AppError::Internal(format!("failed to create rdp temp dir: {e}")))?;

    let safe_key = key
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>();

    let file_path = dir.join(format!("{safe_key}.rdp"));
    std::fs::write(&file_path, content.as_bytes())
        .map_err(|e| AppError::Internal(format!("failed to write rdp temp file: {e}")))?;
    Ok(file_path)
}

#[cfg(windows)]
fn build_rdp_file_content(
    full_address: &str,
    username: Option<&str>,
    options: &DesktopRdpOptions,
) -> String {
    let mut lines = Vec::new();

    lines.push(format!("full address:s:{full_address}"));

    if let Some(user) = username {
        if !user.trim().is_empty() {
            lines.push(format!("username:s:{}", user.trim()));
        }
    }

    if let (Some(width), Some(height)) = (options.width, options.height) {
        if width > 0 && height > 0 {
            lines.push(format!("desktopwidth:i:{width}"));
            lines.push(format!("desktopheight:i:{height}"));
        }
    }

    if options.fullscreen.unwrap_or(false) {
        lines.push("screen mode id:i:2".into());
    } else {
        lines.push("screen mode id:i:1".into());
    }

    if options.multimon.unwrap_or(false) {
        lines.push("use multimon:i:1".into());
    }

    if let Some(depth) = options.color_depth {
        if [15, 16, 24, 32].contains(&depth) {
            lines.push(format!("session bpp:i:{depth}"));
        }
    }

    if options.drives.unwrap_or(false) {
        lines.push("drivestoredirect:s:*".into());
    } else {
        lines.push("drivestoredirect:s:".into());
    }

    lines.push(format!(
        "redirectprinters:i:{}",
        if options.printers.unwrap_or(true) {
            1
        } else {
            0
        }
    ));

    lines.push(format!(
        "redirectclipboard:i:{}",
        if options.clipboard.unwrap_or(true) {
            1
        } else {
            0
        }
    ));

    match options.audio.as_deref().unwrap_or("local") {
        "remote" => lines.push("audiomode:i:1".into()),
        "none" => lines.push("audiomode:i:2".into()),
        _ => lines.push("audiomode:i:0".into()),
    }

    lines.push(format!(
        "compression:i:{}",
        if options.compression.unwrap_or(true) {
            1
        } else {
            0
        }
    ));

    if let Some(gateway) = &options.gateway {
        if gateway.enabled {
            if let Some(host) = gateway.host.as_deref() {
                if !host.trim().is_empty() {
                    let gateway_host = if let Some(port) = gateway.port {
                        format!("{}:{port}", host.trim())
                    } else {
                        host.trim().to_string()
                    };
                    lines.push(format!("gatewayhostname:s:{gateway_host}"));
                    lines.push("gatewayusagemethod:i:1".into());
                    lines.push("gatewayprofileusagemethod:i:1".into());

                    if let Some(user) = gateway.username.as_deref() {
                        if !user.trim().is_empty() {
                            lines.push(format!("gatewayusername:s:{}", user.trim()));
                        }
                    }
                }
            }
        }
    }

    lines.push("autoreconnection enabled:i:1".into());
    lines.push("authentication level:i:2".into());
    lines.push("prompt for credentials:i:0".into());
    lines.push("negotiate security layer:i:1".into());
    lines.push("remoteapplicationmode:i:0".into());
    lines.push("alternate shell:s:".into());
    lines.push("shell working directory:s:".into());
    lines.push("disable wallpaper:i:0".into());
    lines.push("disable full window drag:i:0".into());
    lines.push("disable menu anims:i:0".into());
    lines.push("disable themes:i:0".into());
    lines.push("disable cursor setting:i:0".into());
    lines.push("bitmapcachepersistenable:i:1".into());

    lines.join("\r\n")
}

#[cfg(windows)]
fn delete_cmdkey(host: &str) -> Result<(), String> {
    let status = std::process::Command::new("cmdkey.exe")
        .arg(format!("/delete:TERMSRV/{host}"))
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("cmdkey delete failed".into())
    }
}
