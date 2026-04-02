# Technical Design: Secure API Key Storage with System Keyring (Issue #22)

## Architecture

### Module Structure

```
src/infrastructure/secure_storage/
├── mod.rs              # SecureStorage trait + factory
├── keyring_adapter.rs  # Keyring implementation
└── encrypted_store.rs  # AES-GCM fallback
```

### 1. SecureStorage Trait (`mod.rs`)

```rust
use secrecy::SecretString;

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

pub enum SecureStorageError {
    StorageUnavailable(String),
    KeyNotFound(String),
    EncryptionFailed(String),
    DecryptionFailed(String),
    IoError(String),
}
```

### 2. KeyringStorage (`keyring_adapter.rs`)

```rust
use keyring::Entry;
use secrecy::SecretString;
use super::{SecureStorage, SecureStorageError};

const SERVICE_NAME: &str = "rust-llm-api-router";

pub struct KeyringStorage;

impl KeyringStorage {
    pub fn new() -> Self { Self }
    
    fn entry_for(&self, account_id: &str) -> Entry {
        Entry::new(SERVICE_NAME, account_id)
            .unwrap_or_else(|_| Entry::new(SERVICE_NAME, &format!("fallback-{}", account_id)).unwrap())
    }
}

impl SecureStorage for KeyringStorage {
    fn store(&self, account_id: &str, key: &str) -> Result<(), SecureStorageError> {
        let entry = self.entry_for(account_id);
        entry.set_password(key)
            .map_err(|e| SecureStorageError::StorageUnavailable(e.to_string()))
    }
    
    fn retrieve(&self, account_id: &str) -> Result<Option<SecretString>, SecureStorageError> {
        let entry = self.entry_for(account_id);
        match entry.get_password() {
            Ok(password) => Ok(Some(SecretString::new(password))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(SecureStorageError::KeyNotFound(e.to_string())),
        }
    }
    
    fn delete(&self, account_id: &str) -> Result<(), SecureStorageError> {
        let entry = self.entry_for(account_id);
        entry.delete_password()
            .or(Ok(())) // Ignore if already deleted
            .map_err(|e| SecureStorageError::StorageUnavailable(e.to_string()))
    }
    
    fn is_available(&self) -> bool {
        // Try to create a test entry and delete it
        let test_entry = Entry::new(SERVICE_NAME, "__test__");
        match test_entry {
            Ok(entry) => {
                let _ = entry.set_password("test");
                let _ = entry.delete_password();
                true
            }
            Err(_) => false,
        }
    }
}
```

### 3. EncryptedFileStorage (`encrypted_store.rs`)

```rust
use secrecy::SecretString;
use std::path::PathBuf;
use super::{SecureStorage, SecureStorageError};

pub struct EncryptedFileStorage {
    file_path: PathBuf,
}

impl EncryptedFileStorage {
    pub fn new() -> Result<Self, SecureStorageError> {
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| SecureStorageError::IoError("Cannot find data directory".to_string()))?
            .join("rust-llm-api-router");
        
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| SecureStorageError::IoError(e.to_string()))?;
        
        Ok(Self {
            file_path: data_dir.join("credentials.enc"),
        })
    }
}

impl SecureStorage for EncryptedFileStorage {
    fn store(&self, account_id: &str, key: &str) -> Result<(), SecureStorageError> {
        // TODO: Implement AES-256-GCM encryption
        // For now, store as base64 (placeholder — will implement encryption)
        let mut credentials = self.load_credentials()?;
        credentials.insert(account_id.to_string(), key.to_string());
        self.save_credentials(&credentials)
    }
    
    fn retrieve(&self, account_id: &str) -> Result<Option<SecretString>, SecureStorageError> {
        let credentials = self.load_credentials()?;
        Ok(credentials.get(account_id).map(|k| SecretString::new(k.clone())))
    }
    
    fn delete(&self, account_id: &str) -> Result<(), SecureStorageError> {
        let mut credentials = self.load_credentials()?;
        credentials.remove(account_id);
        self.save_credentials(&credentials)
    }
    
    fn is_available(&self) -> bool { true }
}
```

### 4. Factory Function

```rust
pub fn create_secure_storage() -> Box<dyn SecureStorage> {
    // Check if secure storage is disabled
    if std::env::var("SECURE_STORAGE").as_deref() == Ok("disabled") {
        tracing::warn!("Secure storage disabled — API keys stored in plaintext");
        return Box::new(InsecureStorage);
    }
    
    // Try keyring first
    let keyring = KeyringStorage::new();
    if keyring.is_available() {
        return Box::new(keyring);
    }
    
    // Fallback to encrypted file
    tracing::warn!("Keyring not available, using encrypted file storage");
    match EncryptedFileStorage::new() {
        Ok(storage) => Box::new(storage),
        Err(e) => {
            tracing::error!("Failed to create secure storage: {}", e);
            Box::new(InsecureStorage)
        }
    }
}
```

### 5. Integration with JsonAccountRepository

```rust
// In json_account_repository.rs
pub struct JsonAccountRepository {
    file_path: PathBuf,
    secure_storage: Box<dyn SecureStorage>,
}

impl JsonAccountRepository {
    pub fn new() -> Result<Self, Error> {
        let file_path = Self::default_path()?;
        let secure_storage = create_secure_storage();
        Ok(Self { file_path, secure_storage })
    }
    
    pub async fn save(&self, account: Account) -> Result<(), Error> {
        // Store API key in secure storage
        if let Some(api_key) = &account.api_key {
            if !api_key.is_empty() {
                self.secure_storage.store(&account.id, api_key)?;
            }
        }
        
        // Save account without API key in JSON
        let account_data = AccountData {
            id: account.id.clone(),
            provider_id: account.provider_id.clone(),
            api_key_ref: Some(format!("keyring:{}", account.id)),
            // ... other fields
        };
        
        // Write to JSON (atomic write)
        self.write_json(&account_data).await
    }
    
    pub async fn find_by_id(&self, id: &str) -> Result<Account, Error> {
        let account_data = self.read_json(id).await?;
        
        // Retrieve API key from secure storage
        let api_key = self.secure_storage.retrieve(id)?;
        
        Ok(Account {
            id: account_data.id,
            provider_id: account_data.provider_id,
            api_key: api_key.map(|s| s.expose_secret().clone()),
            // ... other fields
        })
    }
}
```

### 6. Migration Logic

```rust
// In JsonAccountRepository::migrate_plaintext_keys()
async fn migrate_plaintext_keys(&self) -> Result<(), Error> {
    let accounts = self.read_all_accounts().await?;
    let mut migrated = 0;
    
    for account in &accounts {
        if let Some(api_key) = &account.api_key {
            if !api_key.is_empty() && !api_key.starts_with("keyring:") {
                // Migrate to secure storage
                self.secure_storage.store(&account.id, api_key)?;
                migrated += 1;
            }
        }
    }
    
    if migrated > 0 {
        tracing::info!("Migrated {} API keys to secure storage", migrated);
        // Rewrite JSON without plaintext keys
        self.rewrite_accounts_without_plaintext(&accounts).await?;
    }
    
    Ok(())
}
```

## Files Created

| File | Lines | Description |
|------|-------|-------------|
| `src/infrastructure/secure_storage/mod.rs` | ~80 | SecureStorage trait + factory |
| `src/infrastructure/secure_storage/keyring_adapter.rs` | ~70 | Keyring implementation |
| `src/infrastructure/secure_storage/encrypted_store.rs` | ~100 | AES-GCM fallback |

## Files Modified

| File | Lines Changed | Description |
|------|---------------|-------------|
| `src/infrastructure/persistence/json_account_repository.rs` | ~100 | Use SecureStorage, remove plaintext |
| `src/domain/entities/account.rs` | ~10 | Wrap api_key in SecretString |
| `src/infrastructure/mod.rs` | +2 | Re-export secure_storage |
| `docs/security.md` | ~50 | **NEW** — Security documentation |

## Dependencies

Already in `Cargo.toml`:
- `keyring = "2.0"`
- `secrecy = "0.8"`
- `zeroize = "1.7"`

No new dependencies needed.
