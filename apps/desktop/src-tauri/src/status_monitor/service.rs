use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use ssh_core::manager::SshSessionManager;
use tauri::Emitter;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

use crate::status_monitor::metrics::{collect_status_with_prev, now_timestamp_ms};
use crate::status_monitor::policy::{assess_failure, FailureState};
use crate::status_monitor::types::StatusErrorPayload;

const DEFAULT_POLL_INTERVAL: Duration = Duration::from_secs(3);

#[derive(Default)]
struct SessionRegistration {
    subscribers: HashSet<String>,
    task: Option<JoinHandle<()>>,
}

#[derive(Clone)]
pub struct StatusMonitorService {
    sessions: Arc<Mutex<HashMap<String, SessionRegistration>>>,
    poll_interval: Duration,
}

impl StatusMonitorService {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            poll_interval: DEFAULT_POLL_INTERVAL,
        }
    }

    pub async fn subscribe(
        &self,
        session_id: String,
        consumer_id: String,
        ssh_manager: SshSessionManager,
        app_handle: tauri::AppHandle,
        poll_interval_override: Option<Duration>,
        failure_backoff_enabled: bool,
    ) {
        self.prune_finished().await;
        let mut sessions = self.sessions.lock().await;
        let entry = sessions.entry(session_id.clone()).or_default();
        entry.subscribers.insert(consumer_id);
        if let Some(task) = entry.task.take() {
            task.abort();
        }
        entry.task = Some(Self::spawn_session_task(
            self.sessions.clone(),
            session_id,
            ssh_manager,
            app_handle,
            poll_interval_override.unwrap_or(self.poll_interval),
            failure_backoff_enabled,
        ));
    }

    pub async fn unsubscribe(&self, session_id: &str, consumer_id: &str) {
        let mut sessions = self.sessions.lock().await;
        if let Some(entry) = sessions.get_mut(session_id) {
            entry.subscribers.remove(consumer_id);
            if entry.subscribers.is_empty() {
                if let Some(task) = entry.task.take() {
                    task.abort();
                }
                sessions.remove(session_id);
            }
        }
    }

    pub async fn stop_session(&self, session_id: &str) {
        if let Some(entry) = self.sessions.lock().await.remove(session_id) {
            if let Some(task) = entry.task {
                task.abort();
            }
        }
    }

    pub async fn stop_all(&self) {
        for (_, entry) in self.sessions.lock().await.drain() {
            if let Some(task) = entry.task {
                task.abort();
            }
        }
    }

    async fn prune_finished(&self) {
        self.sessions
            .lock()
            .await
            .retain(|_, entry| entry.task.as_ref().map(|task| !task.is_finished()).unwrap_or(true));
    }

    fn spawn_session_task(
        sessions: Arc<Mutex<HashMap<String, SessionRegistration>>>,
        session_id: String,
        ssh_manager: SshSessionManager,
        app_handle: tauri::AppHandle,
        poll_interval: Duration,
        failure_backoff_enabled: bool,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut previous_cpu = None;
            let mut previous_net = None;
            let mut failure_state = FailureState::default();

            loop {
                if !ssh_manager.has_session(&session_id).await {
                    break;
                }
                if !has_subscribers(&sessions, &session_id).await {
                    break;
                }

                match collect_status_with_prev(
                    &ssh_manager,
                    &session_id,
                    &mut previous_cpu,
                    &mut previous_net,
                )
                .await
                {
                    Ok(status) => {
                        failure_state.reset();
                        let _ = app_handle.emit(&format!("status:update:{session_id}"), &status);
                    }
                    Err(error) => {
                        let assessment =
                            assess_failure(&error, &mut failure_state, failure_backoff_enabled);
                        let payload = StatusErrorPayload {
                            session_id: session_id.clone(),
                            message: assessment.message,
                            timestamp: now_timestamp_ms(),
                            degraded: assessment.degraded,
                            unsupported: assessment.unsupported,
                        };
                        let _ = app_handle.emit(&format!("status:error:{session_id}"), &payload);
                        if assessment.stop_sampling {
                            break;
                        }
                    }
                }

                sleep(failure_state.current_interval(poll_interval)).await;
            }

            clear_task_slot(&sessions, &session_id).await;
        })
    }
}

async fn has_subscribers(
    sessions: &Arc<Mutex<HashMap<String, SessionRegistration>>>,
    session_id: &str,
) -> bool {
    sessions
        .lock()
        .await
        .get(session_id)
        .map(|entry| !entry.subscribers.is_empty())
        .unwrap_or(false)
}

async fn clear_task_slot(
    sessions: &Arc<Mutex<HashMap<String, SessionRegistration>>>,
    session_id: &str,
) {
    let mut sessions = sessions.lock().await;
    if let Some(entry) = sessions.get_mut(session_id) {
        entry.task = None;
        if entry.subscribers.is_empty() {
            sessions.remove(session_id);
        }
    }
}
