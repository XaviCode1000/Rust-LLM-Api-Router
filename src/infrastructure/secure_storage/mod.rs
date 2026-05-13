//! Secure storage for sensitive credentials (API keys, tokens).
//!
//! Provides a trait-based abstraction over system keyring and encrypted file storage.
//! The factory function automatically selects the best available backend.

pub mod encrypted_store;
pub mod keyring_adapter;

use secrecy::SecretString;

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
pub fn create_secure_storage() -> Box<dyn SecureStorage> {
    // Check if secure storage is disabled
    if std::env::var("SECURE_STORAGE").as_deref() == Ok("disabled") {
        tracing::warn!("Secure storage disabled — API keys stored in plaintext");
        return Box::new(InsecureStorage::new());
    }

    // Try keyring first
    let keyring = keyring_adapter::KeyringStorage::new();
    if keyring.is_available() {
        tracing::info!("Using system keyring for secure credential storage");
        return Box::new(keyring);
    }

    // Fallback to encrypted file
    tracing::warn!("Keyring not available, using encrypted file storage");
    match encrypted_store::EncryptedFileStorage::new() {
        Ok(storage) => Box::new(storage),
        Err(e) => {
            tracing::error!("Failed to create encrypted storage: {}", e);
            Box::new(InsecureStorage::new())
        }
    }
}

/// Insecure fallback storage (in-memory). Only used when nothing else works.
/// Stores keys in a HashMap for testing/dev purposes.
///
/// # Why `std::sync::Mutex` and not `tokio::sync::Mutex`?
///
/// The `SecureStorage` trait has synchronous methods (not async). There are
/// no `.await` points inside lock scopes, so `std::sync::Mutex` is correct
/// per async-no-lock-await rule. Using `tokio::sync::Mutex` here would
/// require `.block_on()` in sync methods, which is an anti-pattern.
pub struct InsecureStorage {
    store: std::sync::Mutex<std::collections::HashMap<String, String>>,
}

impl InsecureStorage {
    pub fn new() -> Self {
        Self {
            store: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
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
        let mut store = self
            .store
            .lock()
            .map_err(|e| SecureStorageError::StorageUnavailable(format!("Lock poisoned: {e}")))?;
        store.insert(account_id.to_string(), key.to_string());
        Ok(())
    }

    fn retrieve(&self, account_id: &str) -> Result<Option<SecretString>, SecureStorageError> {
        let store = self
            .store
            .lock()
            .map_err(|e| SecureStorageError::StorageUnavailable(format!("Lock poisoned: {e}")))?;
        Ok(store.get(account_id).map(|k| SecretString::new(k.clone())))
    }

    fn delete(&self, account_id: &str) -> Result<(), SecureStorageError> {
        let mut store = self
            .store
            .lock()
            .map_err(|e| SecureStorageError::StorageUnavailable(format!("Lock poisoned: {e}")))?;
        store.remove(account_id);
        Ok(())
    }

    fn is_available(&self) -> bool {
        true
    }
}
