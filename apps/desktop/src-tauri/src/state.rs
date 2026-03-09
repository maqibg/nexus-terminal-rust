//! Tauri AppState - wires storage + auth + session + ssh into managed state.

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
};

use auth_core::service::AuthService;
use session_core::{AuthState, AuthStateStore, SessionRegistry};
use shared_utils::crypto::CryptoService;
use ssh_core::{host_key::HostKeyRepository, manager::SshSessionManager};
use ssh_suspend_core::SuspendManager;
use storage_sqlite::{
    SqliteAuditRepo, SqliteAuthRepo, SqliteConnectionRepo, SqliteHistoryRepo, SqliteHostKeyRepo,
    SqliteQuickCommandRepo, SqliteSettingsRepo, SqliteStorage,
};
use tokio::sync::{Mutex, Semaphore};
use transfer_core::TransferManager;

use crate::status_monitor::StatusMonitorService;

#[derive(Clone, Debug)]
pub struct RuntimePaths {
    pub exe_dir: PathBuf,
    pub data_dir: PathBuf,
    pub download_dir: PathBuf,
    pub temp_dir: PathBuf,
}

impl RuntimePaths {
    pub fn new(exe_dir: PathBuf) -> Self {
        let data_dir = exe_dir.join("data");
        let download_dir = data_dir.join("download");
        let temp_dir = data_dir.join("temp");
        Self {
            exe_dir,
            data_dir,
            download_dir,
            temp_dir,
        }
    }

    pub fn ensure_dirs(&self) -> Result<(), String> {
        std::fs::create_dir_all(&self.data_dir)
            .map_err(|error| format!("failed to create data dir: {error}"))?;
        std::fs::create_dir_all(&self.download_dir)
            .map_err(|error| format!("failed to create download dir: {error}"))?;
        std::fs::create_dir_all(&self.temp_dir)
            .map_err(|error| format!("failed to create temp dir: {error}"))?;
        Ok(())
    }
}

/// Application state managed by Tauri.
pub struct AppState {
    pub auth: Arc<AuthService>,
    pub auth_state: AuthStateStore,
    #[allow(dead_code)]
    pub sessions: SessionRegistry,
    pub conn_repo: Arc<SqliteConnectionRepo>,
    pub settings_repo: Arc<SqliteSettingsRepo>,
    pub host_key_repo: Arc<dyn HostKeyRepository>,
    pub audit_repo: Arc<SqliteAuditRepo>,
    pub history_repo: Arc<SqliteHistoryRepo>,
    pub qc_repo: Arc<SqliteQuickCommandRepo>,
    pub ssh_manager: SshSessionManager,
    pub transfer_manager: TransferManager,
    pub status_monitor: StatusMonitorService,
    pub suspend_manager: SuspendManager,
    pub crypto: CryptoService,
    pub storage: SqliteStorage,
    pub runtime_paths: RuntimePaths,
    pub ai_cancel_flags: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
    pub sftp_upload_limiter: Arc<Semaphore>,
}

impl AppState {
    pub fn new(storage: SqliteStorage, crypto: CryptoService, runtime_paths: RuntimePaths) -> Self {
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
            host_key_repo: Arc::new(SqliteHostKeyRepo::new(pool.clone())),
            audit_repo: Arc::new(SqliteAuditRepo::new(pool.clone())),
            history_repo: Arc::new(SqliteHistoryRepo::new(pool.clone())),
            qc_repo: Arc::new(SqliteQuickCommandRepo::new(pool)),
            ssh_manager: SshSessionManager::new(),
            transfer_manager: TransferManager::new(),
            status_monitor: StatusMonitorService::new(),
            suspend_manager: SuspendManager::new(),
            crypto,
            storage,
            runtime_paths,
            ai_cancel_flags: Arc::new(Mutex::new(HashMap::new())),
            sftp_upload_limiter: Arc::new(Semaphore::new(4)),
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
