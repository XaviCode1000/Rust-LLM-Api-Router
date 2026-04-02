//! Encrypted file storage fallback when system keyring is unavailable.
//!
//! Uses JSON serialization for credential storage.
//! Data is stored in ~/.local/share/rust-llm-api-router/credentials.json
//!
//! Note: This is a placeholder implementation. Full AES-256-GCM encryption
//! will be added in a future iteration.

use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::{SecureStorage, SecureStorageError};

/// Encrypted file storage for credentials.
pub struct EncryptedFileStorage {
    file_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct CredentialStore {
    entries: HashMap<String, String>,
}

impl EncryptedFileStorage {
    /// Create a new encrypted file storage instance.
    pub fn new() -> Result<Self, SecureStorageError> {
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| SecureStorageError::IoError("Cannot find data directory".to_string()))?
            .join("rust-llm-api-router");

        std::fs::create_dir_all(&data_dir)
            .map_err(|e| SecureStorageError::IoError(e.to_string()))?;

        Ok(Self {
            file_path: data_dir.join("credentials.json"),
        })
    }

    fn load_credentials(&self) -> Result<CredentialStore, SecureStorageError> {
        if !self.file_path.exists() {
            return Ok(CredentialStore::default());
        }

        let data = std::fs::read_to_string(&self.file_path)
            .map_err(|e| SecureStorageError::IoError(e.to_string()))?;

        serde_json::from_str(&data).map_err(|e| SecureStorageError::DecryptionFailed(e.to_string()))
    }

    fn save_credentials(&self, store: &CredentialStore) -> Result<(), SecureStorageError> {
        let data = serde_json::to_string_pretty(store)
            .map_err(|e| SecureStorageError::EncryptionFailed(e.to_string()))?;

        std::fs::write(&self.file_path, data)
            .map_err(|e| SecureStorageError::IoError(e.to_string()))
    }
}

impl SecureStorage for EncryptedFileStorage {
    fn store(&self, account_id: &str, key: &str) -> Result<(), SecureStorageError> {
        let mut credentials = self.load_credentials()?;
        credentials
            .entries
            .insert(account_id.to_string(), key.to_string());
        self.save_credentials(&credentials)
    }

    fn retrieve(&self, account_id: &str) -> Result<Option<SecretString>, SecureStorageError> {
        let credentials = self.load_credentials()?;
        Ok(credentials
            .entries
            .get(account_id)
            .map(|k| SecretString::new(k.clone())))
    }

    fn delete(&self, account_id: &str) -> Result<(), SecureStorageError> {
        let mut credentials = self.load_credentials()?;
        credentials.entries.remove(account_id);
        self.save_credentials(&credentials)
    }

    fn is_available(&self) -> bool {
        true
    }
}
