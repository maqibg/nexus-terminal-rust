use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use dashmap::DashMap;
use serde::Serialize;
use std::{
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tauri::Emitter;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{mpsc, Mutex, Notify},
};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TelnetOutputChunk {
    pub seq: u64,
    pub data: Arc<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TelnetSessionInfo {
    pub session_id: String,
    pub host: String,
    pub port: u16,
}

#[derive(Default)]
struct OutputBacklogState {
    next_seq: u64,
    chunks: VecDeque<TelnetOutputChunk>,
}

const OUTPUT_BACKLOG_LIMIT: usize = 512;

enum WriteCmd {
    Data(Vec<u8>),
}

struct ActiveTelnetSession {
    info: TelnetSessionInfo,
    write_tx: mpsc::Sender<WriteCmd>,
    stop_flag: Arc<AtomicBool>,
    stop_notify: Arc<Notify>,
}

#[derive(Clone, Default)]
pub struct TelnetSessionManager {
    sessions: Arc<DashMap<String, Arc<ActiveTelnetSession>>>,
    output_backlogs: Arc<Mutex<HashMap<String, OutputBacklogState>>>,
}

impl TelnetSessionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn list(&self) -> Vec<TelnetSessionInfo> {
        self.sessions.iter().map(|e| e.value().info.clone()).collect()
    }

    pub async fn take_output_backlog(&self, session_id: &str) -> Vec<TelnetOutputChunk> {
        let mut guard = self.output_backlogs.lock().await;
        let Some(state) = guard.get_mut(session_id) else {
            return vec![];
        };
        state.chunks.drain(..).collect()
    }

    pub async fn connect(
        &self,
        session_id: String,
        host: String,
        port: u16,
        app_handle: tauri::AppHandle,
    ) -> Result<String, String> {
        if self.sessions.contains_key(&session_id) {
            return Err(format!("session '{session_id}' already exists"));
        }

        let addr = format!("{host}:{port}");
        let socket = TcpStream::connect(&addr)
            .await
            .map_err(|e| format!("telnet connect failed: {addr}: {e}"))?;

        let (write_tx, write_rx) = mpsc::channel(128);
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_notify = Arc::new(Notify::new());
        let info = TelnetSessionInfo {
            session_id: session_id.clone(),
            host: host.clone(),
            port,
        };

        let active = Arc::new(ActiveTelnetSession {
            info: info.clone(),
            write_tx,
            stop_flag: stop_flag.clone(),
            stop_notify: stop_notify.clone(),
        });

        self.sessions.insert(session_id.clone(), active);
        self.output_backlogs
            .lock()
            .await
            .insert(session_id.clone(), OutputBacklogState::default());

        let sessions = self.sessions.clone();
        let output_backlogs = self.output_backlogs.clone();
        let sid = session_id.clone();
        tokio::spawn(async move {
            read_loop(
                sessions,
                output_backlogs,
                sid,
                app_handle,
                socket,
                write_rx,
                stop_flag,
                stop_notify,
            )
            .await;
        });

        info!(session_id, host, port, "Telnet session connected");
        Ok(session_id)
    }

    pub async fn write(&self, session_id: &str, data: Vec<u8>) -> Result<(), String> {
        let Some(s) = self.sessions.get(session_id) else {
            return Err(format!("session '{session_id}' not found"));
        };

        s.write_tx
            .send(WriteCmd::Data(data))
            .await
            .map_err(|_| "write channel closed".to_string())
    }

    pub async fn close(&self, session_id: &str) -> bool {
        let Some((_id, s)) = self.sessions.remove(session_id) else {
            return false;
        };
        s.stop_flag.store(true, Ordering::SeqCst);
        s.stop_notify.notify_waiters();
        true
    }
}

async fn append_output_chunk(
    output_backlogs: &Arc<Mutex<HashMap<String, OutputBacklogState>>>,
    session_id: &str,
    base64: String,
) -> TelnetOutputChunk {
    let mut guard = output_backlogs.lock().await;
    let state = guard
        .entry(session_id.to_string())
        .or_insert_with(OutputBacklogState::default);
    let seq = state.next_seq;
    state.next_seq += 1;

    let chunk = TelnetOutputChunk {
        seq,
        data: Arc::new(base64),
    };
    state.chunks.push_back(chunk.clone());
    if state.chunks.len() > OUTPUT_BACKLOG_LIMIT {
        state.chunks.pop_front();
    }
    chunk
}

async fn read_loop(
    sessions: Arc<DashMap<String, Arc<ActiveTelnetSession>>>,
    output_backlogs: Arc<Mutex<HashMap<String, OutputBacklogState>>>,
    session_id: String,
    app_handle: tauri::AppHandle,
    mut socket: TcpStream,
    mut write_rx: mpsc::Receiver<WriteCmd>,
    stop_flag: Arc<AtomicBool>,
    stop_notify: Arc<Notify>,
) {
    let out_event = format!("telnet:output:{session_id}");
    let close_event = format!("telnet:close:{session_id}");
    let mut parser = TelnetParser::new();
    let mut buf = vec![0u8; 16 * 1024];

    loop {
        if stop_flag.load(Ordering::SeqCst) {
            break;
        }

        tokio::select! {
            _ = stop_notify.notified() => break,
            cmd = write_rx.recv() => {
                let Some(cmd) = cmd else { break; };
                match cmd {
                    WriteCmd::Data(data) => {
                        if let Err(err) = socket.write_all(&data).await {
                            warn!("telnet write failed: {err}");
                            break;
                        }
                    }
                }
            }
            read = socket.read(&mut buf) => {
                let Ok(n) = read else { break; };
                if n == 0 { break; }

                let (out, reply) = parser.consume(&buf[..n]);
                if !reply.is_empty() {
                    let _ = socket.write_all(&reply).await;
                }
                if !out.is_empty() {
                    let chunk = append_output_chunk(&output_backlogs, &session_id, B64.encode(&out)).await;
                    let _ = app_handle.emit(&out_event, &chunk);
                }
            }
        }
    }

    sessions.remove(&session_id);
    let _ = app_handle.emit(&close_event, &());
    info!(session_id, "Telnet session closed");
}

#[derive(Clone, Copy)]
enum ParseState {
    Data,
    Iac,
    IacOption(u8),
    Sub,
    SubIac,
}

struct TelnetParser {
    state: ParseState,
    pending: Vec<u8>,
}

impl TelnetParser {
    fn new() -> Self {
        Self {
            state: ParseState::Data,
            pending: vec![],
        }
    }

    fn consume(&mut self, input: &[u8]) -> (Vec<u8>, Vec<u8>) {
        self.pending.extend_from_slice(input);
        let mut out = Vec::with_capacity(self.pending.len());
        let mut reply = Vec::new();
        let mut idx = 0usize;

        while idx < self.pending.len() {
            let b = self.pending[idx];
            match self.state {
                ParseState::Data => {
                    if b == 255 {
                        self.state = ParseState::Iac;
                    } else {
                        out.push(b);
                    }
                }
                ParseState::Iac => {
                    match b {
                        255 => {
                            out.push(255);
                            self.state = ParseState::Data;
                        }
                        251 | 252 | 253 | 254 => self.state = ParseState::IacOption(b),
                        250 => self.state = ParseState::Sub,
                        _ => self.state = ParseState::Data,
                    }
                }
                ParseState::IacOption(cmd) => {
                    match cmd {
                        253 => reply.extend_from_slice(&[255, 252, b]), // DO -> WONT
                        251 => reply.extend_from_slice(&[255, 254, b]), // WILL -> DONT
                        _ => {}
                    }
                    self.state = ParseState::Data;
                }
                ParseState::Sub => {
                    if b == 255 {
                        self.state = ParseState::SubIac;
                    }
                }
                ParseState::SubIac => {
                    if b == 240 {
                        self.state = ParseState::Data; // IAC SE
                    } else if b != 255 {
                        self.state = ParseState::Sub;
                    }
                }
            }
            idx += 1;
        }

        self.pending.clear();
        (out, reply)
    }
}
