use api_contract::error::AppError;
use dashmap::DashMap;
use serde::Serialize;
use std::{
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::{
    process::Command,
    sync::{mpsc, Mutex, Notify},
};
use tracing::info;

mod runner;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalTerminalOutputChunk {
    pub seq: u64,
    pub stream: &'static str,
    pub data: Arc<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalTerminalSessionInfo {
    pub session_id: String,
    pub shell: String,
}

#[derive(Default)]
pub(super) struct OutputBacklogState {
    pub(super) next_seq: u64,
    pub(super) chunks: VecDeque<LocalTerminalOutputChunk>,
}

pub(super) const OUTPUT_BACKLOG_LIMIT: usize = 512;

pub(super) enum WriteCmd {
    Data(Vec<u8>),
}

pub(super) struct ActiveSession {
    pub(super) info: LocalTerminalSessionInfo,
    pub(super) write_tx: mpsc::Sender<WriteCmd>,
    pub(super) stop_flag: Arc<AtomicBool>,
    pub(super) stop_notify: Arc<Notify>,
}

#[derive(Clone, Default)]
pub struct LocalTerminalManager {
    sessions: Arc<DashMap<String, Arc<ActiveSession>>>,
    output_backlogs: Arc<Mutex<HashMap<String, OutputBacklogState>>>,
}

impl LocalTerminalManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn list(&self) -> Vec<LocalTerminalSessionInfo> {
        self.sessions.iter().map(|e| e.value().info.clone()).collect()
    }

    pub async fn take_output_backlog(&self, session_id: &str) -> Vec<LocalTerminalOutputChunk> {
        let mut guard = self.output_backlogs.lock().await;
        let Some(state) = guard.get_mut(session_id) else {
            return vec![];
        };
        state.chunks.drain(..).collect()
    }

    pub async fn open_session(
        &self,
        session_id: String,
        shell: Option<String>,
        app_handle: tauri::AppHandle,
    ) -> Result<String, AppError> {
        if self.sessions.contains_key(&session_id) {
            return Err(AppError::Conflict(format!(
                "session '{session_id}' already exists"
            )));
        }

        let (program, args) = runner::resolve_shell_command(shell.as_deref());
        let mut cmd = Command::new(&program);
        cmd.args(&args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| AppError::Internal(format!("spawn shell failed: {program}: {e}")))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| AppError::Internal("failed to open stdin".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AppError::Internal("failed to open stdout".into()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| AppError::Internal("failed to open stderr".into()))?;

        let (write_tx, write_rx) = mpsc::channel(128);
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_notify = Arc::new(Notify::new());
        let info = LocalTerminalSessionInfo {
            session_id: session_id.clone(),
            shell: program.clone(),
        };

        self.sessions.insert(
            session_id.clone(),
            Arc::new(ActiveSession {
                info,
                write_tx,
                stop_flag: stop_flag.clone(),
                stop_notify: stop_notify.clone(),
            }),
        );
        self.output_backlogs
            .lock()
            .await
            .insert(session_id.clone(), OutputBacklogState::default());

        let sessions = self.sessions.clone();
        let output_backlogs = self.output_backlogs.clone();
        let sid = session_id.clone();
        tokio::spawn(async move {
            runner::supervisor_loop(
                sessions,
                output_backlogs,
                sid,
                app_handle,
                child,
                stdin,
                stdout,
                stderr,
                write_rx,
                stop_flag,
                stop_notify,
            )
            .await;
        });

        info!(session_id, program, "Local terminal session opened");
        Ok(session_id)
    }

    pub async fn write(&self, session_id: &str, data: Vec<u8>) -> Result<(), AppError> {
        let Some(s) = self.sessions.get(session_id) else {
            return Err(AppError::NotFound("session not found".into()));
        };
        s.write_tx
            .send(WriteCmd::Data(data))
            .await
            .map_err(|_| AppError::Internal("write channel closed".into()))
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

