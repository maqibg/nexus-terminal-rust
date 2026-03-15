use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use dashmap::DashMap;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tauri::Emitter;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::{Child, ChildStderr, ChildStdin, ChildStdout},
    sync::{mpsc, Mutex, Notify},
    time::{sleep, Duration},
};
use tracing::{info, warn};

use super::{ActiveSession, OutputBacklogState, WriteCmd};

pub(super) fn resolve_shell_command(override_shell: Option<&str>) -> (String, Vec<String>) {
    if let Some(s) = override_shell {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            return (trimmed.to_string(), vec![]);
        }
    }

    #[cfg(windows)]
    {
        ("powershell.exe".to_string(), vec!["-NoLogo".to_string()])
    }

    #[cfg(not(windows))]
    {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        (shell, vec!["-l".to_string()])
    }
}

pub(super) async fn supervisor_loop(
    sessions: Arc<DashMap<String, Arc<ActiveSession>>>,
    output_backlogs: Arc<Mutex<HashMap<String, OutputBacklogState>>>,
    session_id: String,
    app_handle: tauri::AppHandle,
    mut child: Child,
    stdin: ChildStdin,
    stdout: ChildStdout,
    stderr: ChildStderr,
    write_rx: mpsc::Receiver<WriteCmd>,
    stop_flag: Arc<AtomicBool>,
    stop_notify: Arc<Notify>,
) {
    let out_event = format!("local:output:{session_id}");
    let err_event = format!("local:stderr:{session_id}");
    let exit_event = format!("local:exit:{session_id}");
    let close_event = format!("local:close:{session_id}");

    let writer = tokio::spawn(write_loop(stdin, write_rx, stop_flag.clone(), stop_notify.clone()));
    let out_reader = tokio::spawn(read_loop(
        output_backlogs.clone(),
        session_id.clone(),
        "stdout",
        out_event.clone(),
        app_handle.clone(),
        stdout,
        stop_flag.clone(),
        stop_notify.clone(),
    ));
    let err_reader = tokio::spawn(read_loop(
        output_backlogs.clone(),
        session_id.clone(),
        "stderr",
        err_event.clone(),
        app_handle.clone(),
        stderr,
        stop_flag.clone(),
        stop_notify.clone(),
    ));

    let exit_code = wait_or_kill(&mut child, stop_flag.clone(), stop_notify.clone()).await;
    let _ = app_handle.emit(&exit_event, &exit_code);
    stop_flag.store(true, Ordering::SeqCst);
    stop_notify.notify_waiters();

    let _ = writer.await;
    let _ = out_reader.await;
    let _ = err_reader.await;

    sessions.remove(&session_id);
    let _ = app_handle.emit(&close_event, &());
    info!(session_id, exit_code, "Local terminal session closed");
}

async fn wait_or_kill(child: &mut Child, stop_flag: Arc<AtomicBool>, stop: Arc<Notify>) -> i32 {
    loop {
        if stop_flag.load(Ordering::SeqCst) {
            let _ = child.kill().await;
        }

        match child.try_wait() {
            Ok(Some(status)) => return status.code().unwrap_or(-1),
            Ok(None) => {}
            Err(err) => {
                warn!("local terminal try_wait failed: {err}");
                return -1;
            }
        }

        tokio::select! {
            _ = stop.notified() => {},
            _ = sleep(Duration::from_millis(50)) => {}
        }
    }
}

async fn write_loop(
    mut stdin: ChildStdin,
    mut write_rx: mpsc::Receiver<WriteCmd>,
    stop_flag: Arc<AtomicBool>,
    stop: Arc<Notify>,
) {
    loop {
        if stop_flag.load(Ordering::SeqCst) {
            break;
        }

        tokio::select! {
            _ = stop.notified() => break,
            cmd = write_rx.recv() => {
                let Some(cmd) = cmd else { break; };
                match cmd {
                    WriteCmd::Data(data) => {
                        if stdin.write_all(&data).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    }
}

async fn read_loop<R: tokio::io::AsyncRead + Unpin>(
    output_backlogs: Arc<Mutex<HashMap<String, OutputBacklogState>>>,
    session_id: String,
    stream: &'static str,
    event: String,
    app_handle: tauri::AppHandle,
    mut reader: R,
    stop_flag: Arc<AtomicBool>,
    stop: Arc<Notify>,
) {
    let mut buf = vec![0u8; 16 * 1024];
    loop {
        if stop_flag.load(Ordering::SeqCst) {
            break;
        }

        let read = tokio::select! {
            _ = stop.notified() => return,
            n = reader.read(&mut buf) => n
        };
        let Ok(n) = read else { break; };
        if n == 0 {
            break;
        }

        let chunk = append_output_chunk(&output_backlogs, &session_id, stream, B64.encode(&buf[..n])).await;
        let _ = app_handle.emit(&event, &chunk);
    }
}

async fn append_output_chunk(
    output_backlogs: &Arc<Mutex<HashMap<String, OutputBacklogState>>>,
    session_id: &str,
    stream: &'static str,
    base64: String,
) -> super::LocalTerminalOutputChunk {
    use super::OUTPUT_BACKLOG_LIMIT;

    let mut guard = output_backlogs.lock().await;
    let state = guard
        .entry(session_id.to_string())
        .or_insert_with(OutputBacklogState::default);
    let seq = state.next_seq;
    state.next_seq += 1;

    let chunk = super::LocalTerminalOutputChunk {
        seq,
        stream,
        data: Arc::new(base64),
    };
    state.chunks.push_back(chunk.clone());
    if state.chunks.len() > OUTPUT_BACKLOG_LIMIT {
        state.chunks.pop_front();
    }
    chunk
}
