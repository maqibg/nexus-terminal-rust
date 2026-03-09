//! SSH session manager - owns active SSH channels, bridges Tauri events.

use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;

use dashmap::DashMap;
use russh::ChannelMsg;
use russh_sftp::client::SftpSession;
use serde::Serialize;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{timeout, Duration};
use tracing::{info, warn};

use crate::host_key::HostKeyStore;
use crate::session::{self, SshCredentials, SshHandler};

/// Output from command execution on an active SSH session.
#[derive(Debug, Clone)]
pub struct ExecOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: u32,
    /// True when stdout or stderr was cut at the 4 MB per-stream limit.
    pub truncated: bool,
}

enum WriteCmd {
    Data(Vec<u8>),
    Resize(u32, u32),
}

/// Active SSH shell session.
struct ActiveSession {
    handle: Arc<russh::client::Handle<session::SshHandler>>,
    write_tx: mpsc::Sender<WriteCmd>,
    connection_id: i64,
    connection_name: String,
}

/// Active SFTP session.
#[allow(dead_code)]
struct ActiveSftpSession {
    sftp: Arc<SftpSession>,
    connection_id: i64,
    creds: SshCredentials,
}

#[derive(Debug, Clone, Serialize)]
pub struct SshOutputChunk {
    pub seq: u64,
    pub stream: String,
    pub data: String,
}

#[derive(Default)]
struct OutputBacklogState {
    next_seq: u64,
    chunks: VecDeque<SshOutputChunk>,
}

const OUTPUT_BACKLOG_LIMIT: usize = 512;

/// Manages all active SSH sessions.
#[derive(Clone)]
pub struct SshSessionManager {
    /// Per-session DashMap — no global lock; writes are funneled via per-session mpsc.
    sessions: Arc<DashMap<String, Arc<ActiveSession>>>,
    sftp_sessions: Arc<DashMap<String, ActiveSftpSession>>,
    output_backlogs: Arc<Mutex<HashMap<String, OutputBacklogState>>>,
}

impl SshSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            sftp_sessions: Arc::new(DashMap::new()),
            output_backlogs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Open a new SSH shell session. Returns session_id.
    pub async fn open_session(
        &self,
        session_id: String,
        creds: SshCredentials,
        connection_id: i64,
        connection_name: String,
        cols: u32,
        rows: u32,
        app_handle: tauri::AppHandle,
        host_key_store: Option<Arc<dyn HostKeyStore>>,
    ) -> Result<String, String> {
        let handler = match host_key_store {
            Some(store) => {
                SshHandler::with_tofu(creds.host.clone(), creds.port, store, app_handle.clone())
            }
            None => SshHandler::permissive(creds.host.clone(), creds.port),
        };
        let handle = Arc::new(session::connect_and_authenticate(&creds, handler).await?);
        let channel =
            session::open_shell_channel(handle.as_ref(), cols, rows, "xterm-256color").await?;
        let (write_tx, write_rx) = mpsc::channel(128);

        let active = Arc::new(ActiveSession {
            handle,
            write_tx,
            connection_id,
            connection_name: connection_name.clone(),
        });

        if self.sessions.contains_key(&session_id) {
            return Err(format!("session '{session_id}' already exists"));
        }
        self.sessions.insert(session_id.clone(), active);
        self.output_backlogs
            .lock()
            .await
            .insert(session_id.clone(), OutputBacklogState::default());

        // Spawn reader task to forward SSH output -> Tauri event
        let sessions = self.sessions.clone();
        let output_backlogs = self.output_backlogs.clone();
        let sid = session_id.clone();
        tokio::spawn(async move {
            Self::read_loop(
                sessions,
                output_backlogs,
                sid,
                app_handle,
                channel,
                write_rx,
            )
            .await;
        });

        info!(session_id, connection_name, "SSH session opened");
        Ok(session_id)
    }

    /// Read loop with micro-batching: accumulates output over a short window
    /// before emitting to reduce IPC overhead on high-throughput streams.
    ///
    /// Pattern: the read loop exclusively owns the SSH channel; write/resize calls
    /// send commands through mpsc so no per-channel mutex is held across await.
    async fn read_loop(
        sessions: Arc<DashMap<String, Arc<ActiveSession>>>,
        output_backlogs: Arc<Mutex<HashMap<String, OutputBacklogState>>>,
        session_id: String,
        app_handle: tauri::AppHandle,
        mut channel: russh::Channel<russh::client::Msg>,
        mut write_rx: mpsc::Receiver<WriteCmd>,
    ) {
        use base64::{engine::general_purpose::STANDARD as B64, Engine};
        use tauri::Emitter;
        use tokio::time::interval;

        const FLUSH_INTERVAL: Duration = Duration::from_millis(16); // ~60fps
        const MAX_BATCH_BYTES: usize = 64 * 1024;

        let mut stdout_buf: Vec<u8> = Vec::with_capacity(8192);
        let mut stderr_buf: Vec<u8> = Vec::with_capacity(1024);
        let mut flush_timer = interval(FLUSH_INTERVAL);
        let mut ended = false;

        let out_event = format!("ssh:output:{session_id}");
        let err_event = format!("ssh:stderr:{session_id}");
        let mut pending_cmd: Option<WriteCmd> = None;
        let mut close_channel = false;

        loop {
            tokio::select! {
                biased;
                _ = flush_timer.tick() => {
                    if !stdout_buf.is_empty() {
                        let chunk = Self::append_output_chunk(
                            &output_backlogs,
                            &session_id,
                            "stdout",
                            B64.encode(&stdout_buf),
                        )
                        .await;
                        let _ = app_handle.emit(&out_event, &chunk);
                        stdout_buf.clear();
                    }
                    if !stderr_buf.is_empty() {
                        let chunk = Self::append_output_chunk(
                            &output_backlogs,
                            &session_id,
                            "stderr",
                            B64.encode(&stderr_buf),
                        )
                        .await;
                        let _ = app_handle.emit(&err_event, &chunk);
                        stderr_buf.clear();
                    }
                    if ended { break; }
                }
                msg = channel.wait(), if !ended => {
                    match msg {
                        Some(ChannelMsg::Data { ref data }) => {
                            stdout_buf.extend_from_slice(data.as_ref());
                            if stdout_buf.len() >= MAX_BATCH_BYTES {
                                let chunk = Self::append_output_chunk(
                                    &output_backlogs,
                                    &session_id,
                                    "stdout",
                                    B64.encode(&stdout_buf),
                                )
                                .await;
                                let _ = app_handle.emit(&out_event, &chunk);
                                stdout_buf.clear();
                            }
                        }
                        Some(ChannelMsg::ExtendedData { ref data, .. }) => {
                            stderr_buf.extend_from_slice(data.as_ref());
                            if stderr_buf.len() >= MAX_BATCH_BYTES {
                                let chunk = Self::append_output_chunk(
                                    &output_backlogs,
                                    &session_id,
                                    "stderr",
                                    B64.encode(&stderr_buf),
                                )
                                .await;
                                let _ = app_handle.emit(&err_event, &chunk);
                                stderr_buf.clear();
                            }
                        }
                        Some(ChannelMsg::ExitStatus { exit_status }) => {
                            let _ = app_handle.emit(&format!("ssh:exit:{session_id}"), &exit_status);
                        }
                        Some(ChannelMsg::Eof | ChannelMsg::Close) | None => {
                            if !stdout_buf.is_empty() {
                                let chunk = Self::append_output_chunk(
                                    &output_backlogs,
                                    &session_id,
                                    "stdout",
                                    B64.encode(&stdout_buf),
                                )
                                .await;
                                let _ = app_handle.emit(&out_event, &chunk);
                                stdout_buf.clear();
                            }
                            if !stderr_buf.is_empty() {
                                let chunk = Self::append_output_chunk(
                                    &output_backlogs,
                                    &session_id,
                                    "stderr",
                                    B64.encode(&stderr_buf),
                                )
                                .await;
                                let _ = app_handle.emit(&err_event, &chunk);
                                stderr_buf.clear();
                            }
                            let _ = app_handle.emit(&format!("ssh:close:{session_id}"), &());
                            ended = true;
                        }
                        _ => {}
                    }
                }
                cmd = write_rx.recv(), if !ended => {
                    match cmd {
                        Some(cmd) => pending_cmd = Some(cmd),
                        None => close_channel = true,
                    }
                }
            }

            if close_channel {
                let _ = channel.close().await;
                ended = true;
                close_channel = false;
            }

            if ended {
                continue;
            }

            if let Some(cmd) = pending_cmd.take() {
                match cmd {
                    WriteCmd::Data(data) => {
                        if let Err(error) = channel.data(&data[..]).await {
                            warn!(session_id, %error, "SSH channel write command failed");
                        }
                    }
                    WriteCmd::Resize(cols, rows) => {
                        if let Err(error) = channel.window_change(cols, rows, 0, 0).await {
                            warn!(session_id, %error, "SSH channel resize command failed");
                        }
                    }
                }
            }
        }

        sessions.remove(&session_id);
        info!(session_id, "SSH read loop ended");
    }

    async fn append_output_chunk(
        output_backlogs: &Arc<Mutex<HashMap<String, OutputBacklogState>>>,
        session_id: &str,
        stream: &str,
        data: String,
    ) -> SshOutputChunk {
        let mut map = output_backlogs.lock().await;
        let state = map.entry(session_id.to_string()).or_default();
        let chunk = SshOutputChunk {
            seq: state.next_seq,
            stream: stream.to_string(),
            data,
        };
        state.next_seq = state.next_seq.saturating_add(1);
        state.chunks.push_back(chunk.clone());
        while state.chunks.len() > OUTPUT_BACKLOG_LIMIT {
            state.chunks.pop_front();
        }
        chunk
    }

    /// Write data to session stdin.
    pub async fn write(&self, session_id: &str, data: &[u8]) -> Result<(), String> {
        let write_tx = self
            .sessions
            .get(session_id)
            .ok_or("session not found")?
            .value()
            .write_tx
            .clone();
        write_tx
            .send(WriteCmd::Data(data.to_vec()))
            .await
            .map_err(|e| format!("write failed: {e}"))
    }

    /// Resize PTY.
    pub async fn resize(&self, session_id: &str, cols: u32, rows: u32) -> Result<(), String> {
        let write_tx = self
            .sessions
            .get(session_id)
            .ok_or("session not found")?
            .value()
            .write_tx
            .clone();
        write_tx
            .send(WriteCmd::Resize(cols, rows))
            .await
            .map_err(|e| format!("resize failed: {e}"))
    }

    /// Execute command using the active SSH handle for a session.
    pub async fn exec_command(
        &self,
        session_id: &str,
        command: &str,
        stdin: Option<Vec<u8>>,
        request_pty: bool,
        timeout_duration: Duration,
    ) -> Result<ExecOutput, String> {
        // Short scope: clone handle from DashMap, drop entry before async work
        let handle = {
            let entry = self.sessions.get(session_id).ok_or("session not found")?;
            entry.value().handle.clone()
        };

        let exec_task = async move {
            let mut channel = handle
                .channel_open_session()
                .await
                .map_err(|e| format!("open channel failed: {e}"))?;

            if request_pty {
                channel
                    .request_pty(false, "xterm-256color", 80, 24, 0, 0, &[])
                    .await
                    .map_err(|e| format!("pty request failed: {e}"))?;
            }

            channel
                .exec(false, command)
                .await
                .map_err(|e| format!("exec command failed: {e}"))?;

            if let Some(stdin_bytes) = stdin {
                use tokio::io::AsyncWriteExt;

                let mut writer = channel.make_writer();
                writer
                    .write_all(&stdin_bytes)
                    .await
                    .map_err(|e| format!("write stdin failed: {e}"))?;
                writer
                    .shutdown()
                    .await
                    .or_else(|error| {
                        if matches!(
                            error.kind(),
                            std::io::ErrorKind::BrokenPipe | std::io::ErrorKind::ConnectionReset
                        ) {
                            Ok(())
                        } else {
                            Err(error)
                        }
                    })
                    .map_err(|e| format!("shutdown stdin failed: {e}"))?;
            }

            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let mut exit_code = 0;
            let mut truncated = false;

            const MAX_STREAM_BYTES: usize = 4 * 1024 * 1024; // 4 MB per stream

            'read: loop {
                match channel.wait().await {
                    Some(ChannelMsg::Data { data }) => {
                        let remaining = MAX_STREAM_BYTES.saturating_sub(stdout.len());
                        let to_copy = data.len().min(remaining);
                        stdout.extend_from_slice(&data[..to_copy]);
                        if to_copy < data.len() {
                            truncated = true;
                            break 'read;
                        }
                    }
                    Some(ChannelMsg::ExtendedData { data, .. }) => {
                        let remaining = MAX_STREAM_BYTES.saturating_sub(stderr.len());
                        let to_copy = data.len().min(remaining);
                        stderr.extend_from_slice(&data[..to_copy]);
                        if to_copy < data.len() {
                            truncated = true;
                            break 'read;
                        }
                    }
                    Some(ChannelMsg::ExitStatus { exit_status }) => exit_code = exit_status,
                    Some(ChannelMsg::Eof | ChannelMsg::Close) | None => break,
                    _ => {}
                }
            }

            Ok::<ExecOutput, String>(ExecOutput {
                stdout: String::from_utf8_lossy(&stdout).to_string(),
                stderr: String::from_utf8_lossy(&stderr).to_string(),
                exit_code,
                truncated,
            })
        };

        timeout(timeout_duration, exec_task)
            .await
            .map_err(|_| format!("command timeout after {}s", timeout_duration.as_secs()))?
    }

    /// Whether a shell session is active.
    pub async fn has_session(&self, session_id: &str) -> bool {
        self.sessions.contains_key(session_id)
    }

    /// Return active session id for a connection id.
    pub async fn find_session_by_connection_id(&self, connection_id: i64) -> Option<String> {
        self.sessions
            .iter()
            .find(|entry| entry.value().connection_id == connection_id)
            .map(|entry| entry.key().clone())
    }

    /// Return connection id for a session id.
    pub async fn get_connection_id(&self, session_id: &str) -> Option<i64> {
        self.sessions
            .get(session_id)
            .map(|entry| entry.value().connection_id)
    }

    /// Close a session.
    pub async fn close(&self, session_id: &str) -> Result<(), String> {
        if self.sessions.remove(session_id).is_some() {
            info!(session_id, "SSH session closed");
        }
        self.output_backlogs.lock().await.remove(session_id);
        Ok(())
    }

    /// List active session IDs.
    pub async fn list_sessions(&self) -> Vec<(String, i64, String)> {
        self.sessions
            .iter()
            .map(|entry| {
                let s = entry.value();
                (
                    entry.key().clone(),
                    s.connection_id,
                    s.connection_name.clone(),
                )
            })
            .collect()
    }

    // -- SFTP ---------------------------------------------------------

    /// Open a new SFTP session over a dedicated SSH connection.
    pub async fn open_sftp(
        &self,
        session_id: String,
        creds: SshCredentials,
        connection_id: i64,
        host_key_store: Option<Arc<dyn HostKeyStore>>,
        app_handle: Option<tauri::AppHandle>,
    ) -> Result<String, String> {
        if self.sftp_sessions.contains_key(&session_id) {
            return Err(format!("SFTP session '{session_id}' already exists"));
        }
        let handler = match host_key_store {
            Some(store) => SshHandler::with_tofu(
                creds.host.clone(),
                creds.port,
                store,
                app_handle.expect("app_handle required when host_key_store is Some"),
            ),
            None => SshHandler::permissive(creds.host.clone(), creds.port),
        };
        let handle = session::connect_and_authenticate(&creds, handler).await?;
        let channel = session::open_sftp_channel(&handle).await?;
        let sftp = SftpSession::new(channel.into_stream())
            .await
            .map_err(|e| format!("SFTP init failed: {e}"))?;

        self.sftp_sessions.insert(
            session_id.clone(),
            ActiveSftpSession {
                sftp: Arc::new(sftp),
                connection_id,
                creds,
            },
        );
        info!(session_id, "SFTP session opened");
        Ok(session_id)
    }

    /// Close an SFTP session.
    pub async fn close_sftp(&self, session_id: &str) -> Result<(), String> {
        self.sftp_sessions.remove(session_id);
        info!(session_id, "SFTP session closed");
        Ok(())
    }

    /// Get a reference to an active SFTP session.
    pub async fn get_sftp(&self, session_id: &str) -> Result<Arc<SftpSession>, String> {
        self.sftp_sessions
            .get(session_id)
            .map(|entry| entry.sftp.clone())
            .ok_or_else(|| "SFTP session not found".to_string())
    }

    pub async fn get_sftp_credentials(&self, session_id: &str) -> Result<SshCredentials, String> {
        self.sftp_sessions
            .get(session_id)
            .map(|entry| entry.creds.clone())
            .ok_or_else(|| "SFTP session not found".to_string())
    }

    /// Return and clear buffered output chunks for a session.
    pub async fn take_output_backlog(&self, session_id: &str) -> Vec<SshOutputChunk> {
        let mut map = self.output_backlogs.lock().await;
        if let Some(state) = map.get_mut(session_id) {
            let chunks: Vec<SshOutputChunk> = state.chunks.drain(..).collect();
            return chunks;
        }
        Vec::new()
    }
}
