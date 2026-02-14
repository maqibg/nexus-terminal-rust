//! Argon2id (new) + bcrypt (legacy) dual-track password hashing.

use argon2::{
    password_hash::{
        rand_core::OsRng, PasswordHash, PasswordHasher as _, PasswordVerifier, SaltString,
    },
    Argon2,
};

/// Result of password verification.
pub struct VerifyResult {
    pub valid: bool,
    /// If true, the caller should re-hash with Argon2id and update the DB.
    pub needs_rehash: bool,
}

/// Hash a new password with Argon2id.
pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| format!("argon2 hash failed: {e}"))
}

/// Verify password against a stored hash (Argon2id or bcrypt).
pub fn verify_password(password: &str, stored_hash: &str) -> Result<VerifyResult, String> {
    // bcrypt hashes start with "$2b$" or "$2a$"
    if stored_hash.starts_with("$2b$") || stored_hash.starts_with("$2a$") {
        let valid = bcrypt::verify(password, stored_hash)
            .map_err(|e| format!("bcrypt verify failed: {e}"))?;
        return Ok(VerifyResult {
            valid,
            needs_rehash: valid,
        });
    }

    // Argon2id hash
    let parsed = PasswordHash::new(stored_hash).map_err(|e| format!("invalid hash format: {e}"))?;
    let valid = Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok();
    Ok(VerifyResult {
        valid,
        needs_rehash: false,
    })
}
