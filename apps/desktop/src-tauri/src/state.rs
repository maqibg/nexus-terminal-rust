//! Tauri AppState - wires storage + auth + session + ssh into managed state.

use std::sync::Arc;

use auth_core::service::AuthService;
use session_core::{AuthState, AuthStateStore, SessionRegistry};
use shared_utils::crypto::CryptoService;
use ssh_core::manager::SshSessionManager;
use storage_sqlite::{
    SqliteAuditRepo, SqliteAuthRepo, SqliteConnectionRepo, SqliteHistoryRepo,
    SqliteQuickCommandRepo, SqliteSettingsRepo, SqliteStorage,
};
use transfer_core::TransferManager;

use crate::status_monitor::StatusMonitorService;

/// Application state managed by Tauri.
pub struct AppState {
    pub auth: Arc<AuthService>,
    pub auth_state: AuthStateStore,
    #[allow(dead_code)]
    pub sessions: SessionRegistry,
    pub conn_repo: Arc<SqliteConnectionRepo>,
    pub settings_repo: Arc<SqliteSettingsRepo>,
    pub audit_repo: Arc<SqliteAuditRepo>,
    pub history_repo: Arc<SqliteHistoryRepo>,
    pub qc_repo: Arc<SqliteQuickCommandRepo>,
    pub ssh_manager: SshSessionManager,
    pub transfer_manager: TransferManager,
    pub status_monitor: StatusMonitorService,
    pub crypto: CryptoService,
    pub storage: SqliteStorage,
}

impl AppState {
    pub fn new(storage: SqliteStorage, crypto: CryptoService) -> Self {
        let pool = storage.pool.clone();
        let auth_repo = Arc::new(SqliteAuthRepo::new(pool.clone()));
        let auth_state = AuthStateStore::new(AuthState::NeedsSetup);
        let auth = Arc::new(AuthService::new(auth_repo, auth_state.clone()));

        Self {
            auth,
            auth_state,
            sessions: SessionRegistry::new(),
            conn_repo: Arc::new(SqliteConnectionRepo::new(pool.clone())),
            settings_repo: Arc::new(SqliteSettingsRepo::new(pool.clone())),
            audit_repo: Arc::new(SqliteAuditRepo::new(pool.clone())),
            history_repo: Arc::new(SqliteHistoryRepo::new(pool.clone())),
            qc_repo: Arc::new(SqliteQuickCommandRepo::new(pool)),
            ssh_manager: SshSessionManager::new(),
            transfer_manager: TransferManager::new(),
            status_monitor: StatusMonitorService::new(),
            crypto,
            storage,
        }
    }

    /// Determine initial auth state from DB.
    pub async fn init_auth_state(&self) {
        use auth_core::repository::AuthRepository;

        let repo = SqliteAuthRepo::new(self.storage.pool.clone());
        match repo.user_count().await {
            Ok(0) => self.auth_state.set(AuthState::NeedsSetup).await,
            Ok(_) => match repo.get_persisted_login_user_id().await {
                Ok(Some(user_id)) => match repo.find_user_by_id(user_id).await {
                    Ok(Some(user)) => {
                        self.auth_state
                            .set(AuthState::Unlocked {
                                user_id,
                                username: user.username,
                                at: std::time::Instant::now(),
                            })
                            .await;
                    }
                    Ok(None) => {
                        let _ = repo.set_persisted_login_user_id(None).await;
                        self.auth_state.set(AuthState::Locked).await;
                    }
                    Err(err) => {
                        tracing::error!(
                            "Failed to fetch persisted auth user by id {user_id}: {err}"
                        );
                        self.auth_state.set(AuthState::Locked).await;
                    }
                },
                Ok(None) => self.auth_state.set(AuthState::Locked).await,
                Err(err) => {
                    tracing::error!("Failed to load persisted auth session: {err}");
                    self.auth_state.set(AuthState::Locked).await;
                }
            },
            Err(err) => tracing::error!("Failed to check user count: {err}"),
        }
    }
}
