//! AuthService — orchestrates login, setup, 2FA, password change, passkey registration.

use std::sync::Arc;

use api_contract::error::AppError;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use session_core::{AuthState, AuthStateStore};
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;
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

// ── WebAuthn helper types ──

struct PendingPasskeyReg {
    challenge: Vec<u8>,
    created_at: std::time::Instant,
}

/// navigator.credentials.create() serialized response.
#[derive(Deserialize)]
struct CredentialJson {
    id: String,
    response: CredentialResponseFields,
}

#[derive(Deserialize)]
struct CredentialResponseFields {
    #[serde(rename = "clientDataJSON")]
    client_data_json: String,
    #[serde(rename = "attestationObject")]
    attestation_object: String,
}

#[derive(Deserialize)]
struct ClientData {
    #[serde(rename = "type")]
    type_: String,
    challenge: String,
    // origin is intentionally not validated for local Tauri app
}

pub struct AuthService {
    repo: Arc<dyn AuthRepository>,
    state: AuthStateStore,
    /// Pending user_id during 2FA login verification.
    pending_2fa_user: Mutex<Option<i64>>,
    /// Temporary 2FA secret during TOTP setup (cleared after activation).
    temp_2fa_secret: Mutex<Option<String>>,
    /// Pending WebAuthn registration state with creation timestamp (TTL=5min, cleared after finish).
    passkey_reg_state: Mutex<Option<PendingPasskeyReg>>,
}

impl AuthService {
    pub fn new(repo: Arc<dyn AuthRepository>, state: AuthStateStore) -> Self {
        Self {
            repo,
            state,
            pending_2fa_user: Mutex::new(None),
            temp_2fa_secret: Mutex::new(None),
            passkey_reg_state: Mutex::new(None),
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
                    .map_err(AppError::from)?;
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
        let count = self.repo.user_count().await.map_err(AppError::from)?;
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
            .map_err(AppError::from)?;
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
            .map_err(AppError::from)?
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

        // Check 2FA: lock → read → drop before any await
        if user.two_factor_enabled && user.two_factor_secret.is_some() {
            let mut pending = self.pending_2fa_user.lock().await;
            // 防并发账户覆盖：单用户应用拒绝第二个并发登录请求
            if pending.is_some() {
                return Err(AppError::Validation("another login is in progress".into()));
            }
            *pending = Some(user.id);
            drop(pending);
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
        // lock → clone → drop before await
        let user_id = self
            .pending_2fa_user
            .lock()
            .await
            .ok_or(AppError::Validation("no pending 2FA login".into()))?;

        let user = self
            .repo
            .find_user_by_id(user_id)
            .await
            .map_err(AppError::from)?
            .ok_or(AppError::Unauthorized)?;

        let secret = user
            .two_factor_secret
            .as_deref()
            .ok_or(AppError::Internal("2FA secret missing".into()))?;

        if !totp::verify_token(secret, token).map_err(AppError::Internal)? {
            // 验证失败时清理 pending 状态，防止暴力破解
            *self.pending_2fa_user.lock().await = None;
            return Err(AppError::Validation("invalid 2FA token".into()));
        }

        *self.pending_2fa_user.lock().await = None;
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
            .map_err(AppError::from)?;
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
            .map_err(AppError::from)?
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
            .map_err(AppError::from)?;
        info!(user_id, "password changed");
        Ok(())
    }

    // ── 2FA Setup ──

    pub async fn setup_2fa(&self) -> Result<TwoFactorSetupResponse, AppError> {
        let (_, username) = self.require_auth().await?;
        let (secret, url) = totp::generate_secret(&username).map_err(AppError::Internal)?;
        *self.temp_2fa_secret.lock().await = Some(secret.clone());
        Ok(TwoFactorSetupResponse { secret, url })
    }

    pub async fn verify_and_activate_2fa(&self, token: &str) -> Result<(), AppError> {
        let (user_id, _) = self.require_auth().await?;
        // lock → clone → drop before await
        let secret = self
            .temp_2fa_secret
            .lock()
            .await
            .clone()
            .ok_or(AppError::Validation("no pending 2FA setup".into()))?;

        if !totp::verify_token(&secret, token).map_err(AppError::Internal)? {
            return Err(AppError::Validation("invalid 2FA token".into()));
        }

        self.repo
            .set_two_factor_secret(user_id, Some(&secret))
            .await
            .map_err(AppError::from)?;
        *self.temp_2fa_secret.lock().await = None;
        info!(user_id, "2FA enabled");
        Ok(())
    }

    pub async fn disable_2fa(&self, pw: &str) -> Result<(), AppError> {
        let (user_id, _) = self.require_auth().await?;
        let user = self
            .repo
            .find_user_by_id(user_id)
            .await
            .map_err(AppError::from)?
            .ok_or(AppError::Unauthorized)?;

        let result =
            password::verify_password(pw, &user.hashed_password).map_err(AppError::Internal)?;
        if !result.valid {
            return Err(AppError::Validation("password is incorrect".into()));
        }

        self.repo
            .set_two_factor_secret(user_id, None)
            .await
            .map_err(AppError::from)?;
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
            .map_err(AppError::from)?;
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
            .map_err(AppError::from)
    }

    pub async fn passkey_rename(&self, credential_id: &str, name: &str) -> Result<bool, AppError> {
        let (user_id, _) = self.require_auth().await?;
        self.repo
            .rename_passkey(user_id, credential_id, name)
            .await
            .map_err(AppError::from)
    }

    /// Start passkey registration — returns `PublicKeyCredentialCreationOptions` JSON for the browser.
    pub async fn passkey_register_start(&self) -> Result<String, AppError> {
        let (user_id, username) = self.require_auth().await?;

        // Generate 32-byte random challenge
        let challenge: Vec<u8> = rand::random::<[u8; 32]>().to_vec();
        let challenge_b64 = URL_SAFE_NO_PAD.encode(&challenge);

        // user.id in WebAuthn must be opaque bytes (base64url-encoded)
        let user_id_b64 = URL_SAFE_NO_PAD.encode(user_id.to_le_bytes());

        let options = serde_json::json!({
            "publicKey": {
                "rp": { "name": "Nexus Terminal", "id": "localhost" },
                "user": {
                    "id": user_id_b64,
                    "name": username,
                    "displayName": username
                },
                "challenge": challenge_b64,
                "pubKeyCredParams": [
                    { "alg": -7, "type": "public-key" },
                    { "alg": -257, "type": "public-key" }
                ],
                "timeout": 60000,
                "authenticatorSelection": {
                    "residentKey": "required",
                    "requireResidentKey": true,
                    "userVerification": "preferred"
                },
                "attestation": "none"
            }
        });

        *self.passkey_reg_state.lock().await = Some(PendingPasskeyReg {
            challenge,
            created_at: std::time::Instant::now(),
        });

        serde_json::to_string(&options)
            .map_err(|e| AppError::Internal(format!("serialize options: {e}")))
    }

    /// Finish passkey registration — validates the browser credential and stores the passkey.
    ///
    /// `credential_json` must be the JSON-serialized `PublicKeyCredential` from the browser's
    /// `navigator.credentials.create()` response (with binary fields base64url-encoded).
    pub async fn passkey_register_finish(
        &self,
        credential_json: &str,
        name: &str,
    ) -> Result<i64, AppError> {
        let (user_id, _) = self.require_auth().await?;

        // Take state atomically — this prevents replay even if the finish call fails later
        let pending = self
            .passkey_reg_state
            .lock()
            .await
            .take()
            .ok_or(AppError::Validation(
                "no pending passkey registration".into(),
            ))?;

        // TTL: 5 minutes
        if pending.created_at.elapsed() > std::time::Duration::from_secs(300) {
            return Err(AppError::Validation("passkey registration expired".into()));
        }

        let cred: CredentialJson = serde_json::from_str(credential_json)
            .map_err(|e| AppError::Validation(format!("invalid credential JSON: {e}")))?;

        // 1. Verify clientDataJSON
        let cd_bytes = URL_SAFE_NO_PAD
            .decode(&cred.response.client_data_json)
            .map_err(|e| AppError::Validation(format!("clientDataJSON decode: {e}")))?;
        let client_data: ClientData = serde_json::from_slice(&cd_bytes)
            .map_err(|e| AppError::Validation(format!("clientDataJSON parse: {e}")))?;

        if client_data.type_ != "webauthn.create" {
            return Err(AppError::Validation("wrong clientDataJSON.type".into()));
        }
        let expected_challenge = URL_SAFE_NO_PAD.encode(&pending.challenge);
        if client_data.challenge != expected_challenge {
            return Err(AppError::Validation("challenge mismatch".into()));
        }

        // 2. Parse attestationObject (CBOR) and extract authData
        let att_bytes = URL_SAFE_NO_PAD
            .decode(&cred.response.attestation_object)
            .map_err(|e| AppError::Validation(format!("attestationObject decode: {e}")))?;
        let auth_data = extract_auth_data(&att_bytes)?;

        // 3. Verify rpIdHash = SHA-256("localhost")
        if auth_data.len() < 37 {
            return Err(AppError::Validation("authData too short".into()));
        }
        let expected_hash = Sha256::digest(b"localhost");
        if auth_data[0..32] != expected_hash[..] {
            return Err(AppError::Validation("rpIdHash mismatch".into()));
        }

        // 4. Extract credential ID and COSE public key from authData
        let (credential_id, cose_key_bytes) = extract_credential_from_auth_data(&auth_data)?;

        // Cross-validate: browser-reported ID must match ID embedded in authData
        if cred.id != credential_id {
            return Err(AppError::Validation(
                "credential ID mismatch between JSON and authData".into(),
            ));
        }

        // Store COSE key as base64url; the repo stores it in the public_key column
        let public_key_b64 = URL_SAFE_NO_PAD.encode(&cose_key_bytes);

        let id = self
            .repo
            .create_passkey(user_id, &credential_id, &public_key_b64, 0, None, name)
            .await
            .map_err(AppError::from)?;

        info!(user_id, %credential_id, "passkey registered");
        Ok(id)
    }
}

// ── WebAuthn attestationObject / authData helpers ──

/// Extract `authData` bytes from a CBOR-encoded `attestationObject`.
/// Handles both text string keys ("authData") and CTAP2 integer keys (3).
fn extract_auth_data(att_bytes: &[u8]) -> Result<Vec<u8>, AppError> {
    use ciborium::value::Value;

    let value: Value = ciborium::de::from_reader(att_bytes)
        .map_err(|e| AppError::Validation(format!("attestationObject CBOR: {e}")))?;

    let map = match value {
        Value::Map(m) => m,
        _ => {
            return Err(AppError::Validation(
                "attestationObject is not a map".into(),
            ))
        }
    };

    for (k, v) in map {
        let is_auth_data_key = match &k {
            Value::Text(s) => s == "authData",
            Value::Integer(i) => i128::from(*i) == 3,
            _ => false,
        };
        if is_auth_data_key {
            if let Value::Bytes(bytes) = v {
                return Ok(bytes);
            }
        }
    }

    Err(AppError::Validation(
        "authData not found in attestationObject".into(),
    ))
}

/// Extract `(credential_id_base64url, cose_key_bytes)` from parsed `authData`.
///
/// authData layout:
/// ```text
/// [0..32]    rpIdHash
/// [32]       flags
/// [33..37]   signCount (u32 BE)
/// [37..53]   aaguid (16 bytes)
/// [53..55]   credentialIdLength (u16 BE)
/// [55..55+L] credentialId
/// [55+L..]   credentialPublicKey (CBOR COSE key)
/// ```
fn extract_credential_from_auth_data(auth_data: &[u8]) -> Result<(String, Vec<u8>), AppError> {
    let flags = auth_data[32];
    // Bit 6 (0x40) = AT flag: attested credential data present
    if flags & 0x40 == 0 {
        return Err(AppError::Validation(
            "AT flag not set: no credential data in authData".into(),
        ));
    }
    if auth_data.len() < 55 {
        return Err(AppError::Validation(
            "authData too short for credential".into(),
        ));
    }

    let cred_id_len = u16::from_be_bytes([auth_data[53], auth_data[54]]) as usize;
    let cred_end = 55 + cred_id_len;

    if auth_data.len() < cred_end {
        return Err(AppError::Validation(
            "authData truncated before credentialId end".into(),
        ));
    }

    let credential_id = URL_SAFE_NO_PAD.encode(&auth_data[55..cred_end]);
    let cose_key_bytes = auth_data[cred_end..].to_vec();

    Ok((credential_id, cose_key_bytes))
}
