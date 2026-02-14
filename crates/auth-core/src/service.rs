//! AuthService — orchestrates login, setup, 2FA, password change.

use std::sync::{Arc, Mutex};

use api_contract::error::AppError;
use serde::{Deserialize, Serialize};
use session_core::{AuthState, AuthStateStore};
use tracing::{info, warn};

use crate::password;
use crate::repository::AuthRepository;
use crate::totp;

/// Public user info returned to frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub is_two_factor_enabled: bool,
}

/// Auth status response.
#[derive(Debug, Serialize)]
pub struct AuthStatusResponse {
    pub is_authenticated: bool,
    pub user: Option<UserInfo>,
}

/// Login response.
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub requires_two_factor: bool,
    pub user: Option<UserInfo>,
}

/// 2FA setup response.
#[derive(Debug, Serialize)]
pub struct TwoFactorSetupResponse {
    pub secret: String,
    pub url: String,
}

/// Passkey info returned to frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyInfo {
    pub id: i64,
    pub credential_id: String,
    pub name: String,
}

pub struct AuthService {
    repo: Arc<dyn AuthRepository>,
    state: AuthStateStore,
    /// Temporary 2FA secret during setup (desktop = single user, no session needed).
    temp_2fa_secret: Mutex<Option<String>>,
    /// Pending user_id during 2FA login verification.
    pending_2fa_user: Mutex<Option<i64>>,
}

impl AuthService {
    pub fn new(repo: Arc<dyn AuthRepository>, state: AuthStateStore) -> Self {
        Self {
            repo,
            state,
            temp_2fa_secret: Mutex::new(None),
            pending_2fa_user: Mutex::new(None),
        }
    }

    // ── Status ──

    pub async fn status(&self) -> Result<AuthStatusResponse, AppError> {
        match self.state.get().await {
            AuthState::NeedsSetup => Err(AppError::SetupRequired),
            AuthState::Locked => Ok(AuthStatusResponse {
                is_authenticated: false,
                user: None,
            }),
            AuthState::Unlocked { user_id, .. } => {
                let user = self
                    .repo
                    .find_user_by_id(user_id)
                    .await
                    .map_err(AppError::Database)?;
                Ok(AuthStatusResponse {
                    is_authenticated: true,
                    user: user.map(|u| UserInfo {
                        id: u.id,
                        username: u.username,
                        is_two_factor_enabled: u.two_factor_enabled,
                    }),
                })
            }
        }
    }

    // ── Setup ──

    pub async fn setup_admin(&self, username: &str, pw: &str) -> Result<UserInfo, AppError> {
        let count = self.repo.user_count().await.map_err(AppError::Database)?;
        if count > 0 {
            return Err(AppError::Conflict("admin already exists".into()));
        }
        if pw.len() < 8 {
            return Err(AppError::Validation(
                "password must be at least 8 characters".into(),
            ));
        }
        let hash = password::hash_password(pw).map_err(AppError::Internal)?;
        let id = self
            .repo
            .create_user(username, &hash)
            .await
            .map_err(AppError::Database)?;
        info!(username, "admin user created");
        self.state.set(AuthState::Locked).await;
        if let Err(err) = self.repo.set_persisted_login_user_id(None).await {
            warn!("failed to reset persisted auth session after setup: {err}");
        }
        Ok(UserInfo {
            id,
            username: username.to_string(),
            is_two_factor_enabled: false,
        })
    }

    // ── Login ──

    pub async fn login(&self, username: &str, pw: &str) -> Result<LoginResponse, AppError> {
        let user = self
            .repo
            .find_user_by_username(username)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Unauthorized)?;

        let result =
            password::verify_password(pw, &user.hashed_password).map_err(AppError::Internal)?;
        if !result.valid {
            return Err(AppError::Unauthorized);
        }

        // Auto-rehash bcrypt → Argon2id
        if result.needs_rehash {
            if let Ok(new_hash) = password::hash_password(pw) {
                let _ = self.repo.update_password(user.id, &new_hash).await;
            }
        }

        // Check 2FA
        if user.two_factor_enabled && user.two_factor_secret.is_some() {
            *self.pending_2fa_user.lock().unwrap() = Some(user.id);
            return Ok(LoginResponse {
                requires_two_factor: true,
                user: None,
            });
        }

        self.unlock(user.id, &user.username).await;
        Ok(LoginResponse {
            requires_two_factor: false,
            user: Some(UserInfo {
                id: user.id,
                username: user.username,
                is_two_factor_enabled: user.two_factor_enabled,
            }),
        })
    }

    /// Verify TOTP during login.
    pub async fn verify_login_2fa(&self, token: &str) -> Result<LoginResponse, AppError> {
        let user_id = self
            .pending_2fa_user
            .lock()
            .unwrap()
            .ok_or(AppError::Validation("no pending 2FA login".into()))?;

        let user = self
            .repo
            .find_user_by_id(user_id)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Unauthorized)?;

        let secret = user
            .two_factor_secret
            .as_deref()
            .ok_or(AppError::Internal("2FA secret missing".into()))?;

        if !totp::verify_token(secret, token).map_err(AppError::Internal)? {
            return Err(AppError::Validation("invalid 2FA token".into()));
        }

        *self.pending_2fa_user.lock().unwrap() = None;
        self.unlock(user.id, &user.username).await;
        Ok(LoginResponse {
            requires_two_factor: false,
            user: Some(UserInfo {
                id: user.id,
                username: user.username,
                is_two_factor_enabled: true,
            }),
        })
    }

    // ── Logout ──

    pub async fn logout(&self) -> Result<(), AppError> {
        self.state.set(AuthState::Locked).await;
        self.repo
            .set_persisted_login_user_id(None)
            .await
            .map_err(AppError::Database)?;
        Ok(())
    }

    // ── Password Change ──

    pub async fn change_password(&self, current_pw: &str, new_pw: &str) -> Result<(), AppError> {
        let (user_id, _) = self.require_auth().await?;
        if new_pw.len() < 8 {
            return Err(AppError::Validation(
                "password must be at least 8 characters".into(),
            ));
        }
        let user = self
            .repo
            .find_user_by_id(user_id)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Unauthorized)?;

        let result = password::verify_password(current_pw, &user.hashed_password)
            .map_err(AppError::Internal)?;
        if !result.valid {
            return Err(AppError::Validation("current password is incorrect".into()));
        }

        let new_hash = password::hash_password(new_pw).map_err(AppError::Internal)?;
        self.repo
            .update_password(user_id, &new_hash)
            .await
            .map_err(AppError::Database)?;
        info!(user_id, "password changed");
        Ok(())
    }

    // ── 2FA Setup ──

    pub async fn setup_2fa(&self) -> Result<TwoFactorSetupResponse, AppError> {
        let (_, username) = self.require_auth().await?;
        let (secret, url) = totp::generate_secret(&username).map_err(AppError::Internal)?;
        *self.temp_2fa_secret.lock().unwrap() = Some(secret.clone());
        Ok(TwoFactorSetupResponse { secret, url })
    }

    pub async fn verify_and_activate_2fa(&self, token: &str) -> Result<(), AppError> {
        let (user_id, _) = self.require_auth().await?;
        let secret = self
            .temp_2fa_secret
            .lock()
            .unwrap()
            .clone()
            .ok_or(AppError::Validation("no pending 2FA setup".into()))?;

        if !totp::verify_token(&secret, token).map_err(AppError::Internal)? {
            return Err(AppError::Validation("invalid 2FA token".into()));
        }

        self.repo
            .set_two_factor_secret(user_id, Some(&secret))
            .await
            .map_err(AppError::Database)?;
        *self.temp_2fa_secret.lock().unwrap() = None;
        info!(user_id, "2FA enabled");
        Ok(())
    }

    pub async fn disable_2fa(&self, pw: &str) -> Result<(), AppError> {
        let (user_id, _) = self.require_auth().await?;
        let user = self
            .repo
            .find_user_by_id(user_id)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::Unauthorized)?;

        let result =
            password::verify_password(pw, &user.hashed_password).map_err(AppError::Internal)?;
        if !result.valid {
            return Err(AppError::Validation("password is incorrect".into()));
        }

        self.repo
            .set_two_factor_secret(user_id, None)
            .await
            .map_err(AppError::Database)?;
        info!(user_id, "2FA disabled");
        Ok(())
    }

    // ── Helpers ──

    async fn unlock(&self, user_id: i64, username: &str) {
        self.state
            .set(AuthState::Unlocked {
                user_id,
                username: username.to_string(),
                at: std::time::Instant::now(),
            })
            .await;

        if let Err(err) = self.repo.set_persisted_login_user_id(Some(user_id)).await {
            warn!(user_id, "failed to persist auth session: {err}");
        }
    }

    /// Require authenticated state, return (user_id, username).
    pub async fn require_auth(&self) -> Result<(i64, String), AppError> {
        match self.state.get().await {
            AuthState::Unlocked {
                user_id, username, ..
            } => Ok((user_id, username)),
            AuthState::NeedsSetup => Err(AppError::SetupRequired),
            AuthState::Locked => Err(AppError::Unauthorized),
        }
    }

    // ── Passkey CRUD ──

    pub async fn passkey_list(&self) -> Result<Vec<PasskeyInfo>, AppError> {
        let (user_id, _) = self.require_auth().await?;
        let rows = self
            .repo
            .list_passkeys(user_id)
            .await
            .map_err(AppError::Database)?;
        Ok(rows
            .into_iter()
            .map(|r| PasskeyInfo {
                id: r.id,
                credential_id: r.credential_id,
                name: r.name,
            })
            .collect())
    }

    pub async fn passkey_delete(&self, credential_id: &str) -> Result<bool, AppError> {
        let (user_id, _) = self.require_auth().await?;
        self.repo
            .delete_passkey(user_id, credential_id)
            .await
            .map_err(AppError::Database)
    }

    pub async fn passkey_rename(&self, credential_id: &str, name: &str) -> Result<bool, AppError> {
        let (user_id, _) = self.require_auth().await?;
        self.repo
            .rename_passkey(user_id, credential_id, name)
            .await
            .map_err(AppError::Database)
    }

    /// Start passkey registration — returns a challenge JSON for the browser WebAuthn API.
    pub async fn passkey_register_start(&self) -> Result<String, AppError> {
        let (user_id, username) = self.require_auth().await?;
        // 生成随机 challenge
        let challenge = uuid::Uuid::new_v4().to_string();
        let response = serde_json::json!({
            "challenge": challenge,
            "rp": { "name": "Nexus Terminal", "id": "localhost" },
            "user": { "id": user_id.to_string(), "name": username, "displayName": username },
            "pubKeyCredParams": [{ "type": "public-key", "alg": -7 }],
            "timeout": 60000,
            "attestation": "none",
        });
        // 存储 challenge 供 finish 验证
        *self.temp_2fa_secret.lock().unwrap() = Some(challenge);
        Ok(response.to_string())
    }

    /// Finish passkey registration — stores the credential.
    pub async fn passkey_register_finish(
        &self,
        credential_id: &str,
        public_key: &str,
        name: &str,
    ) -> Result<i64, AppError> {
        let (user_id, _) = self.require_auth().await?;
        // 清除 challenge
        *self.temp_2fa_secret.lock().unwrap() = None;
        let id = self
            .repo
            .create_passkey(user_id, credential_id, public_key, 0, None, name)
            .await
            .map_err(AppError::Database)?;
        info!(user_id, credential_id, "passkey registered");
        Ok(id)
    }
}
