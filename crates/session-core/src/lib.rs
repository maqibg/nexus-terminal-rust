//! Session management and auth state machine.

use dashmap::DashMap;
use serde::Serialize;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Authentication state for the local desktop app.
#[derive(Debug, Clone)]
pub enum AuthState {
    /// No user exists yet, setup required.
    NeedsSetup,
    /// User exists but not authenticated.
    Locked,
    /// Authenticated at the given instant.
    Unlocked {
        user_id: i64,
        username: String,
        at: Instant,
    },
}

/// Thread-safe auth state store.
#[derive(Debug, Clone)]
pub struct AuthStateStore(pub Arc<RwLock<AuthState>>);

impl AuthStateStore {
    pub fn new(initial: AuthState) -> Self {
        Self(Arc::new(RwLock::new(initial)))
    }

    pub async fn get(&self) -> AuthState {
        self.0.read().await.clone()
    }

    pub async fn set(&self, state: AuthState) {
        *self.0.write().await = state;
    }

    pub async fn is_unlocked(&self) -> bool {
        matches!(*self.0.read().await, AuthState::Unlocked { .. })
    }
}

/// Runtime session info for an active SSH connection.
#[derive(Debug, Clone, Serialize)]
pub struct RuntimeSession {
    pub session_id: String,
    pub connection_id: i64,
    pub connection_name: String,
}

/// Registry of active sessions.
#[derive(Debug, Clone, Default)]
pub struct SessionRegistry {
    pub sessions: Arc<DashMap<String, RuntimeSession>>,
}

impl SessionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&self, session: RuntimeSession) {
        self.sessions.insert(session.session_id.clone(), session);
    }

    pub fn remove(&self, session_id: &str) -> Option<RuntimeSession> {
        self.sessions.remove(session_id).map(|(_, v)| v)
    }

    pub fn get(&self, session_id: &str) -> Option<RuntimeSession> {
        self.sessions.get(session_id).map(|v| v.clone())
    }
}
