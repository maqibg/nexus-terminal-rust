//! Shared utilities: crypto, error types, ID generation

pub mod crypto;
pub mod id;
pub mod path;

/// Storage-layer error type shared across repository traits.
///
/// Using a newtype (rather than bare `String`) gives callers a distinct type to
/// pattern-match on and leaves room to add structured variants later.
#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct StorageError(pub String);

impl From<String> for StorageError {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for StorageError {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}
