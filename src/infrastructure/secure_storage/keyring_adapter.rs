//! System keyring implementation using the `keyring` crate.
//!
//! Supports:
//! - macOS: Keychain
//! - Windows: Credential Manager
//! - Linux: Secret Service (libsecret)

use keyring::Entry;
use secrecy::SecretString;

use super::{SecureStorage, SecureStorageError};

const SERVICE_NAME: &str = "rust-llm-api-router";

/// Secure storage backed by the system keyring.
pub struct KeyringStorage;

impl KeyringStorage {
    /// Create a new keyring storage instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    fn entry_for(&self, account_id: &str) -> Result<Entry, SecureStorageError> {
        Entry::new(SERVICE_NAME, account_id)
            .map_err(|e| SecureStorageError::StorageUnavailable(e.to_string()))
    }
}

impl SecureStorage for KeyringStorage {
    fn store(&self, account_id: &str, key: &str) -> Result<(), SecureStorageError> {
        let entry = self.entry_for(account_id)?;
        entry
            .set_password(key)
            .map_err(|e| SecureStorageError::StorageUnavailable(e.to_string()))
    }

    fn retrieve(&self, account_id: &str) -> Result<Option<SecretString>, SecureStorageError> {
        let entry = self.entry_for(account_id)?;
        match entry.get_password() {
            Ok(password) => Ok(Some(SecretString::new(password))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(SecureStorageError::KeyNotFound(e.to_string())),
        }
    }

    fn delete(&self, account_id: &str) -> Result<(), SecureStorageError> {
        let entry = self.entry_for(account_id)?;
        // Ignore error if entry doesn't exist
        let _ = entry.delete_password();
        Ok(())
    }

    fn is_available(&self) -> bool {
        // Try to create a test entry and clean it up
        match Entry::new(SERVICE_NAME, "__secure_storage_test__") {
            Ok(entry) => {
                let _ = entry.set_password("test");
                let _ = entry.delete_password();
                true
            },
            Err(_) => false,
        }
    }
}

impl Default for KeyringStorage {
    fn default() -> Self {
        Self::new()
    }
}
