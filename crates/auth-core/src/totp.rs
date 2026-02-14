//! TOTP 2FA operations wrapping totp-rs.

use totp_rs::{Algorithm, Secret, TOTP};

const ISSUER: &str = "NexusTerminal";
const DIGITS: usize = 6;
const STEP: u64 = 30;
const SKEW: u8 = 1;

/// Generate a new TOTP secret. Returns (base32_secret, otpauth_url).
pub fn generate_secret(username: &str) -> Result<(String, String), String> {
    let secret = Secret::generate_secret();
    let totp = TOTP::new(
        Algorithm::SHA1,
        DIGITS,
        SKEW,
        STEP,
        secret.to_bytes().map_err(|e| e.to_string())?,
        Some(ISSUER.to_string()),
        username.to_string(),
    )
    .map_err(|e| e.to_string())?;
    let base32 = secret.to_encoded().to_string();
    let url = totp.get_url();
    Ok((base32, url))
}

/// Verify a TOTP token against a base32 secret.
pub fn verify_token(base32_secret: &str, token: &str) -> Result<bool, String> {
    let secret = Secret::Encoded(base32_secret.to_string());
    let totp = TOTP::new(
        Algorithm::SHA1,
        DIGITS,
        SKEW,
        STEP,
        secret.to_bytes().map_err(|e| e.to_string())?,
        Some(ISSUER.to_string()),
        String::new(),
    )
    .map_err(|e| e.to_string())?;
    totp.check_current(token).map_err(|e| e.to_string())
}
