//! SSH session manager - owns active SSH channels, bridges Tauri events.

use std::collections::HashMap;
use std::sync::Arc;

use russh::ChannelMsg;
use russh_sftp::client::SftpSession;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};
use tracing::info;

use crate::session::{self, SshCredentials};

/// Output from command execution on an active SSH session.
#[derive(Debug, Clone)]
pub struct ExecOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: u32,
}

/// Active SSH shell session.
struct ActiveSession {
    handle: Arc<russh::client::Handle<session::SshHandler>>,
    channel: russh::Channel<russh::client::Msg>,
    connection_id: i64,
    connection_name: String,
}

/// Active SFTP session.
#[allow(dead_code)]
struct ActiveSftpSession {
    sftp: Arc<SftpSession>,
    connection_id: i64,
}

/// Manages all active SSH sessions.
#[derive(Clone)]
pub struct SshSessionManager {
    sessions: Arc<Mutex<HashMap<String, ActiveSession>>>,
    sftp_sessions: Arc<Mutex<HashMap<String, ActiveSftpSession>>>,
}

impl SshSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            sftp_sessions: Arc::new(Mutex::new(HashMap::new())),
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
    ) -> Result<String, String> {
        let handle = Arc::new(session::connect_and_authenticate(&creds).await?);
        let channel =
            session::open_shell_channel(handle.as_ref(), cols, rows, "xterm-256color").await?;

        let active = ActiveSession {
            handle,
            channel,
            connection_id,
            connection_name: connection_name.clone(),
        };
        self.sessions
            .lock()
            .await
            .insert(session_id.clone(), active);

        // Spawn reader task to forward SSH output -> Tauri event
        let sessions = self.sessions.clone();
        let sid = session_id.clone();
        tokio::spawn(async move {
            Self::read_loop(sessions, sid, app_handle).await;
        });

        info!(session_id, connection_name, "SSH session opened");
        Ok(session_id)
    }

    /// Read loop with micro-batching: accumulates output over a short window
    /// before emitting to reduce IPC overhead on high-throughput streams.
    async fn read_loop(
        sessions: Arc<Mutex<HashMap<String, ActiveSession>>>,
        session_id: String,
        app_handle: tauri::AppHandle,
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

        let flush = |buf: &mut Vec<u8>, event: &str, handle: &tauri::AppHandle| {
            if !buf.is_empty() {
                let b64 = B64.encode(&buf);
                let _ = handle.emit(event, &b64);
                buf.clear();
            }
        };

        let out_event = format!("ssh:output:{session_id}");
        let err_event = format!("ssh:stderr:{session_id}");

        loop {
            tokio::select! {
                biased;
                _ = flush_timer.tick() => {
                    flush(&mut stdout_buf, &out_event, &app_handle);
                    flush(&mut stderr_buf, &err_event, &app_handle);
                    if ended { break; }
                }
                msg = async {
                    let mut map = sessions.lock().await;
                    let Some(session) = map.get_mut(&session_id) else { return None };
                    session.channel.wait().await
                } => {
                    match msg {
                        Some(ChannelMsg::Data { ref data }) => {
                            stdout_buf.extend_from_slice(data.as_ref());
                            if stdout_buf.len() >= MAX_BATCH_BYTES {
                                flush(&mut stdout_buf, &out_event, &app_handle);
                            }
                        }
                        Some(ChannelMsg::ExtendedData { ref data, .. }) => {
                            stderr_buf.extend_from_slice(data.as_ref());
                            if stderr_buf.len() >= MAX_BATCH_BYTES {
                                flush(&mut stderr_buf, &err_event, &app_handle);
                            }
                        }
                        Some(ChannelMsg::ExitStatus { exit_status }) => {
                            let _ = app_handle.emit(&format!("ssh:exit:{session_id}"), &exit_status);
                        }
                        Some(ChannelMsg::Eof | ChannelMsg::Close) | None => {
                            flush(&mut stdout_buf, &out_event, &app_handle);
                            flush(&mut stderr_buf, &err_event, &app_handle);
                            let _ = app_handle.emit(&format!("ssh:close:{session_id}"), &());
                            ended = true;
                        }
                        _ => {}
                    }
                }
            }
        }

        sessions.lock().await.remove(&session_id);
        info!(session_id, "SSH read loop ended");
    }

    /// Write data to session stdin.
    pub async fn write(&self, session_id: &str, data: &[u8]) -> Result<(), String> {
        let mut map = self.sessions.lock().await;
        let session = map.get_mut(session_id).ok_or("session not found")?;
        session
            .channel
            .data(&data[..])
            .await
            .map_err(|e| format!("write failed: {e}"))
    }

    /// Resize PTY.
    pub async fn resize(&self, session_id: &str, cols: u32, rows: u32) -> Result<(), String> {
        let map = self.sessions.lock().await;
        let session = map.get(session_id).ok_or("session not found")?;
        session
            .channel
            .window_change(cols, rows, 0, 0)
            .await
            .map_err(|e| format!("resize failed: {e}"))
    }

    /// Execute command using the active SSH handle for a session.
    pub async fn exec_command(
        &self,
        session_id: &str,
        command: &str,
        timeout_duration: Duration,
    ) -> Result<ExecOutput, String> {
        let handle = {
            let map = self.sessions.lock().await;
            let session = map.get(session_id).ok_or("session not found")?;
            session.handle.clone()
        };

        let exec_task = async move {
            let mut channel = handle
                .channel_open_session()
                .await
                .map_err(|e| format!("open channel failed: {e}"))?;

            channel
                .exec(false, command)
                .await
                .map_err(|e| format!("exec command failed: {e}"))?;

            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let mut exit_code = 0;

            loop {
                match channel.wait().await {
                    Some(ChannelMsg::Data { data }) => stdout.extend_from_slice(data.as_ref()),
                    Some(ChannelMsg::ExtendedData { data, .. }) => {
                        stderr.extend_from_slice(data.as_ref())
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
            })
        };

        timeout(timeout_duration, exec_task)
            .await
            .map_err(|_| format!("command timeout after {}s", timeout_duration.as_secs()))?
    }

    /// Whether a shell session is active.
    pub async fn has_session(&self, session_id: &str) -> bool {
        self.sessions.lock().await.contains_key(session_id)
    }

    /// Return active session id for a connection id.
    pub async fn find_session_by_connection_id(&self, connection_id: i64) -> Option<String> {
        self.sessions
            .lock()
            .await
            .iter()
            .find(|(_, session)| session.connection_id == connection_id)
            .map(|(id, _)| id.clone())
    }

    /// Return connection id for a session id.
    pub async fn get_connection_id(&self, session_id: &str) -> Option<i64> {
        self.sessions
            .lock()
            .await
            .get(session_id)
            .map(|session| session.connection_id)
    }

    /// Close a session.
    pub async fn close(&self, session_id: &str) -> Result<(), String> {
        let mut map = self.sessions.lock().await;
        if let Some(session) = map.remove(session_id) {
            let _ = session.channel.close().await;
            info!(session_id, "SSH session closed");
        }
        Ok(())
    }

    /// List active session IDs.
    pub async fn list_sessions(&self) -> Vec<(String, i64, String)> {
        self.sessions
            .lock()
            .await
            .iter()
            .map(|(id, s)| (id.clone(), s.connection_id, s.connection_name.clone()))
            .collect()
    }

    // -- SFTP ---------------------------------------------------------

    /// Open a new SFTP session over a dedicated SSH connection.
    pub async fn open_sftp(
        &self,
        session_id: String,
        creds: SshCredentials,
        connection_id: i64,
    ) -> Result<String, String> {
        let channel = session::connect_and_open_sftp(&creds).await?;
        let sftp = SftpSession::new(channel.into_stream())
            .await
            .map_err(|e| format!("SFTP init failed: {e}"))?;

        self.sftp_sessions.lock().await.insert(
            session_id.clone(),
            ActiveSftpSession {
                sftp: Arc::new(sftp),
                connection_id,
            },
        );
        info!(session_id, "SFTP session opened");
        Ok(session_id)
    }

    /// Close an SFTP session.
    pub async fn close_sftp(&self, session_id: &str) -> Result<(), String> {
        self.sftp_sessions.lock().await.remove(session_id);
        info!(session_id, "SFTP session closed");
        Ok(())
    }

    /// Get a reference to an active SFTP session.
    pub async fn get_sftp(&self, session_id: &str) -> Result<Arc<SftpSession>, String> {
        let map = self.sftp_sessions.lock().await;
        let session = map.get(session_id).ok_or("SFTP session not found")?;
        Ok(session.sftp.clone())
    }
}
