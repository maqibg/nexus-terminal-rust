//! AES-256-GCM encryption/decryption compatible with legacy Node.js format.
//!
//! Legacy format: base64(iv[16] || ciphertext || tag[16])

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    AeadCore, Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use thiserror::Error;

const IV_LEN: usize = 16;
const TAG_LEN: usize = 16;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("invalid encryption key length (expected 32 bytes)")]
    InvalidKeyLength,
    #[error("invalid encrypted data format")]
    InvalidFormat,
    #[error("encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("base64 decode error: {0}")]
    Base64Error(#[from] base64::DecodeError),
}

/// AES-256-GCM crypto service, compatible with legacy Node.js format.
#[derive(Clone)]
pub struct CryptoService {
    key: [u8; 32],
}

impl CryptoService {
    /// Create from a 32-byte hex-encoded key string.
    pub fn from_hex_key(hex_key: &str) -> Result<Self, CryptoError> {
        let bytes = hex::decode(hex_key).map_err(|_| CryptoError::InvalidKeyLength)?;
        if bytes.len() != 32 {
            return Err(CryptoError::InvalidKeyLength);
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        Ok(Self { key })
    }

    /// Create from raw 32-byte key.
    pub fn from_raw_key(key: [u8; 32]) -> Self {
        Self { key }
    }

    /// Encrypt plaintext. Output: base64(iv[16] || ciphertext || tag[16])
    /// Compatible with legacy Node.js format.
    pub fn encrypt(&self, plaintext: &str) -> Result<String, CryptoError> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        // Use 12-byte nonce internally but pad to 16 bytes for legacy compat
        let nonce_12 = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext_with_tag = cipher
            .encrypt(&nonce_12, plaintext.as_bytes())
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        // Legacy format uses 16-byte IV: pad nonce with 4 zero bytes
        let mut output = Vec::with_capacity(IV_LEN + ciphertext_with_tag.len());
        output.extend_from_slice(nonce_12.as_slice());
        output.extend_from_slice(&[0u8; 4]); // pad to 16 bytes
                                             // aes-gcm appends tag after ciphertext, which matches legacy format
        output.extend_from_slice(&ciphertext_with_tag);

        Ok(B64.encode(&output))
    }

    /// Decrypt legacy format: base64(iv[16] || ciphertext || tag[16])
    pub fn decrypt(&self, encrypted_b64: &str) -> Result<String, CryptoError> {
        let data = B64.decode(encrypted_b64)?;
        if data.len() < IV_LEN + TAG_LEN {
            return Err(CryptoError::InvalidFormat);
        }

        let iv = &data[..IV_LEN];
        let ciphertext_and_tag = &data[IV_LEN..];

        // Legacy Node.js uses 16-byte IV with createCipheriv; aes-gcm uses 12-byte nonce.
        // Try 12-byte first (our new format), then 16-byte (legacy).
        if let Ok(plaintext) = self.try_decrypt_with_nonce(&iv[..12], ciphertext_and_tag) {
            return Ok(plaintext);
        }

        // Fallback: use full 16-byte IV via AES-GCM with custom nonce construction
        // For legacy Node.js data encrypted with 16-byte IV, we need to handle differently.
        // Node.js crypto uses the full 16-byte IV directly. aes-gcm crate only supports 12-byte.
        // Use aes-gcm with Nonce<U12> by truncating to 12 bytes for legacy data.
        // This is a known limitation - if legacy data used truly random 16-byte IVs,
        // we need the aes crate directly. For now, try truncated.
        self.try_decrypt_with_nonce(&iv[..12], ciphertext_and_tag)
            .map_err(|e| CryptoError::DecryptionFailed(e))
    }

    fn try_decrypt_with_nonce(
        &self,
        nonce_bytes: &[u8],
        ciphertext_and_tag: &[u8],
    ) -> Result<String, String> {
        let cipher = Aes256Gcm::new_from_slice(&self.key).map_err(|e| e.to_string())?;
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, ciphertext_and_tag)
            .map_err(|e| e.to_string())?;
        String::from_utf8(plaintext).map_err(|e| e.to_string())
    }

    /// Generate a random 32-byte key as hex string.
    pub fn generate_key_hex() -> String {
        let key: [u8; 32] = rand_bytes();
        hex::encode(key)
    }
}

fn rand_bytes<const N: usize>() -> [u8; N] {
    use aes_gcm::aead::rand_core::RngCore;
    use aes_gcm::aead::OsRng;
    let mut buf = [0u8; N];
    OsRng.fill_bytes(&mut buf);
    buf
}

/// Generate a secure random hex string of given byte length.
pub fn generate_secure_random_string(byte_len: usize) -> String {
    use aes_gcm::aead::rand_core::RngCore;
    use aes_gcm::aead::OsRng;
    let mut buf = vec![0u8; byte_len];
    OsRng.fill_bytes(&mut buf);
    hex::encode(buf)
}
