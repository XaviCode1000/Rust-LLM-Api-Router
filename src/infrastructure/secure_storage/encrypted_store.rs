//! Encrypted file storage fallback when system keyring is unavailable.
//!
//! Uses AES-256-GCM encryption with a key derived from machine ID via Argon2.
//! Data is stored in ~/.local/share/rust-llm-api-router/credentials.enc

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash,
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use rand::rngs::OsRng;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::{SecureStorage, SecureStorageError};

const SALT: &str = "rust-llm-api-router-credential-encryption-salt";

/// Encrypted file storage for credentials.
pub struct EncryptedFileStorage {
    file_path: PathBuf,
    cipher: Aes256Gcm,
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

        // Derive encryption key from machine hostname + salt using Argon2
        let hostname = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "unknown-host".to_string());

        let salt = SaltString::encode_b64(SALT.as_bytes()).map_err(|e: password_hash::Error| {
            SecureStorageError::EncryptionFailed(e.to_string())
        })?;

        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(hostname.as_bytes(), &salt)
            .map_err(|e| SecureStorageError::EncryptionFailed(e.to_string()))?;

        // Use first 32 bytes of hash as AES-256 key
        let hash = password_hash.hash.ok_or_else(|| {
            SecureStorageError::EncryptionFailed("No hash from Argon2".to_string())
        })?;
        let key_bytes = hash.as_bytes();

        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);
        let cipher = Aes256Gcm::new(key);

        Ok(Self {
            file_path: data_dir.join("credentials.enc"),
            cipher,
        })
    }

    fn load_credentials(&self) -> Result<CredentialStore, SecureStorageError> {
        if !self.file_path.exists() {
            return Ok(CredentialStore::default());
        }

        let encrypted_data = std::fs::read(&self.file_path)
            .map_err(|e| SecureStorageError::IoError(e.to_string()))?;

        if encrypted_data.is_empty() {
            return Ok(CredentialStore::default());
        }

        // Format: [12-byte nonce][encrypted data]
        if encrypted_data.len() <= 12 {
            return Err(SecureStorageError::DecryptionFailed(
                "Data too short".to_string(),
            ));
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let decrypted = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| SecureStorageError::DecryptionFailed("Decryption failed".to_string()))?;

        let json = String::from_utf8(decrypted)
            .map_err(|_| SecureStorageError::DecryptionFailed("Invalid UTF-8".to_string()))?;

        serde_json::from_str(&json).map_err(|e| SecureStorageError::DecryptionFailed(e.to_string()))
    }

    fn save_credentials(&self, store: &CredentialStore) -> Result<(), SecureStorageError> {
        let json = serde_json::to_string_pretty(store)
            .map_err(|e| SecureStorageError::EncryptionFailed(e.to_string()))?;

        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, json.as_bytes())
            .map_err(|_| SecureStorageError::EncryptionFailed("Encryption failed".to_string()))?;

        // Format: [12-byte nonce][encrypted data]
        let mut encrypted_data = nonce.to_vec();
        encrypted_data.extend(ciphertext);

        std::fs::write(&self.file_path, encrypted_data)
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
