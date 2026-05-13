//! Secure storage for sensitive credentials (API keys, tokens).
//!
//! Provides a trait-based abstraction over system keyring and encrypted file storage.
//! The factory function automatically selects the best available backend.

pub mod encrypted_store;
pub mod keyring_adapter;

use secrecy::SecretString;
use std::sync::Arc;

/// Error type for secure storage operations.
#[derive(Debug, thiserror::Error)]
pub enum SecureStorageError {
    #[error("Storage unavailable: {0}")]
    StorageUnavailable(String),
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("I/O error: {0}")]
    IoError(String),
}

impl From<std::io::Error> for SecureStorageError {
    fn from(e: std::io::Error) -> Self {
        SecureStorageError::IoError(e.to_string())
    }
}

/// Trait for secure credential storage.
pub trait SecureStorage: Send + Sync {
    /// Store an API key securely.
    fn store(&self, account_id: &str, key: &str) -> Result<(), SecureStorageError>;

    /// Retrieve an API key. Returns None if not found.
    fn retrieve(&self, account_id: &str) -> Result<Option<SecretString>, SecureStorageError>;

    /// Delete an API key.
    fn delete(&self, account_id: &str) -> Result<(), SecureStorageError>;

    /// Check if this storage backend is available.
    fn is_available(&self) -> bool;
}

/// Create the best available secure storage backend.
///
/// Priority: Keyring > Encrypted File > Insecure (warning logged)
///
/// Returns `Arc<dyn SecureStorage>` so callers can clone the handle
/// for use in `spawn_blocking` (requires `'static`).
pub fn create_secure_storage() -> Arc<dyn SecureStorage> {
    // Check if secure storage is disabled
    if std::env::var("SECURE_STORAGE").as_deref() == Ok("disabled") {
        tracing::warn!("Secure storage disabled — API keys stored in plaintext");
        return Arc::new(InsecureStorage::new());
    }

    // Try keyring first
    let keyring = keyring_adapter::KeyringStorage::new();
    if keyring.is_available() {
        tracing::info!("Using system keyring for secure credential storage");
        return Arc::new(keyring);
    }

    // Fallback to encrypted file
    tracing::warn!("Keyring not available, using encrypted file storage");
    match encrypted_store::EncryptedFileStorage::new() {
        Ok(storage) => Arc::new(storage),
        Err(e) => {
            tracing::error!("Failed to create encrypted storage: {}", e);
            Arc::new(InsecureStorage::new())
        }
    }
}

/// Insecure fallback storage (in-memory). Only used when nothing else works.
/// Stores keys in a HashMap for testing/dev purposes.
///
/// # Why `std::sync::Mutex` and not `tokio::sync::Mutex`?
///
/// Shared in-memory store for `InsecureStorage` (all instances share the same map).
/// Uses `std::sync::Mutex` because `SecureStorage` methods are synchronous and
/// there are no `.await` points inside lock scopes.
pub struct InsecureStorage {
    _private: (),
}

thread_local! {
    // Thread-local store for `InsecureStorage` — each test thread gets its own map,
    // preventing state leakage between tests.
    static INSECURE_STORE: std::cell::RefCell<std::collections::HashMap<String, String>>
        = std::cell::RefCell::new(std::collections::HashMap::new());
}

impl InsecureStorage {
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Clear the thread-local store. Use in test teardown to prevent state leakage.
    #[cfg(test)]
    pub fn clear() {
        INSECURE_STORE.with(|store| store.borrow_mut().clear());
    }
}

impl Default for InsecureStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl SecureStorage for InsecureStorage {
    fn store(&self, account_id: &str, key: &str) -> Result<(), SecureStorageError> {
        tracing::warn!(
            "Storing API key in memory (insecure — use keyring or encrypted file in production)"
        );
        INSECURE_STORE.with(|store| {
            store
                .borrow_mut()
                .insert(account_id.to_string(), key.to_string());
        });
        Ok(())
    }

    fn retrieve(&self, account_id: &str) -> Result<Option<SecretString>, SecureStorageError> {
        INSECURE_STORE.with(|store| {
            let store = store.borrow();
            match store.get(account_id) {
                Some(key) if !key.is_empty() => Ok(Some(SecretString::new(key.clone()))),
                _ => Ok(None),
            }
        })
    }

    fn delete(&self, account_id: &str) -> Result<(), SecureStorageError> {
        INSECURE_STORE.with(|store| {
            store.borrow_mut().remove(account_id);
        });
        Ok(())
    }

    fn is_available(&self) -> bool {
        true
    }
}
