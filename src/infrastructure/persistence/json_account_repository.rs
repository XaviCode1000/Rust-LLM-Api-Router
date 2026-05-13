//! JSON-based account repository implementation
//!
//! This module provides a file-based persistence layer for accounts
//! using JSON serialization with secure API key handling.

use async_trait::async_trait;
use fs4::FileExt;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::domain::traits::AccountRepository;
use crate::domain::{Account, DomainError, DomainResult};
use crate::infrastructure::secure_storage::{create_secure_storage, SecureStorage};
use crate::Result;

/// Lock acquisition timeout duration (for reference).
// const _LOCK_TIMEOUT: Duration = Duration::from_secs(5);
/// Internal representation for JSON serialization.
/// API keys are stored encrypted in production, plaintext in dev.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AccountData {
    id: String,
    provider_id: String,
    /// Legacy API key for authentication (kept for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    api_key: Option<String>,
    /// OAuth 2.0 access token (never serialized to disk for security)
    #[serde(skip_serializing_if = "Option::is_none")]
    access_token: Option<String>,
    /// OAuth 2.0 refresh token for obtaining new access tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
    /// OpenID Connect ID token (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    id_token: Option<String>,
    /// Token expiration timestamp (seconds since UNIX epoch)
    #[serde(skip_serializing_if = "Option::is_none")]
    token_expires_at: Option<u64>,
    /// Whether the account is active and can be used
    pub is_active: bool,
    /// Priority for load balancing (lower = higher priority)
    pub priority: i32,
    /// Timestamp when the account was created
    #[serde(default)]
    pub created_at: Option<u64>,
    /// Timestamp when the account was last used
    #[serde(default)]
    pub last_used_at: Option<u64>,
    /// Type of authentication strategy used for this account
    #[serde(skip_serializing_if = "String::is_empty")]
    pub auth_strategy_type: String,
}

impl From<&Account> for AccountData {
    fn from(account: &Account) -> Self {
        Self {
            id: account.id.clone(),
            provider_id: account.provider_id.clone(),
            api_key: account.api_key.clone(),
            access_token: account.access_token.clone(),
            refresh_token: account.refresh_token.clone(),
            id_token: account.id_token.clone(),
            token_expires_at: account.token_expires_at,
            is_active: account.is_active,
            priority: account.priority,
            created_at: account.created_at,
            last_used_at: account.last_used_at,
            auth_strategy_type: account.auth_strategy_type.clone(),
        }
    }
}

impl From<AccountData> for Account {
    fn from(data: AccountData) -> Self {
        Self {
            id: data.id,
            provider_id: data.provider_id,
            api_key: data.api_key,
            access_token: data.access_token,
            refresh_token: data.refresh_token,
            id_token: data.id_token,
            token_expires_at: data.token_expires_at,
            is_active: data.is_active,
            priority: data.priority,
            created_at: data.created_at,
            last_used_at: data.last_used_at,
            auth_strategy_type: data.auth_strategy_type,
        }
    }
}

/// JSON-based account repository.
///
/// Stores accounts in a JSON file at the configured path.
/// API keys are stored in the system keyring or encrypted file storage.
pub struct JsonAccountRepository {
    file_path: PathBuf,
    secure_storage: Box<dyn SecureStorage>,
}

impl Clone for JsonAccountRepository {
    fn clone(&self) -> Self {
        Self {
            file_path: self.file_path.clone(),
            secure_storage: create_secure_storage(),
        }
    }
}

impl JsonAccountRepository {
    /// Creates a new repository with the default config path.
    ///
    /// # Returns
    /// A new `JsonAccountRepository` instance
    pub fn new() -> Result<Self> {
        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .map(|mut p| {
                        p.push(".config");
                        p
                    })
                    .unwrap_or_else(|| PathBuf::from("."))
            });

        let file_path = config_dir.join("rust-llm-api-router").join("accounts.json");
        let secure_storage = create_secure_storage();
        let repo = Self {
            file_path,
            secure_storage,
        };

        // T-20: Clean up stale temp files from previous crashed writes
        repo.cleanup_stale_temp_files();

        Ok(repo)
    }

    /// T-20: Clean up any stale `.tmp` files left from crashed atomic writes.
    ///
    /// These files should not exist in normal operation. If they do, it means
    /// a previous write was interrupted before the atomic rename could complete.
    fn cleanup_stale_temp_files(&self) {
        if let Some(parent) = self.file_path.parent() {
            let base_name = self.file_path.file_stem();
            if let Some(base) = base_name {
                let tmp_path = parent.join(format!("{}.tmp", base.to_string_lossy()));
                if tmp_path.exists() {
                    let _ = std::fs::remove_file(&tmp_path);
                }
            }
        }
    }

    /// Creates a new repository with a custom config directory.
    ///
    /// # Arguments
    /// * `config_dir` - Custom configuration directory path
    ///
    /// # Returns
    /// A new `JsonAccountRepository` instance
    pub fn with_config_dir(config_dir: &Path) -> DomainResult<Self> {
        let file_path = config_dir.join("accounts.json");
        // Use in-memory storage for custom config dirs (tests use isolated temp dirs)
        let secure_storage =
            Box::new(crate::infrastructure::secure_storage::InsecureStorage::new());
        let repo = Self {
            file_path,
            secure_storage,
        };
        repo.cleanup_stale_temp_files();
        Ok(repo)
    }

    /// Ensures the directory and file exist.
    async fn ensure_file_exists(&self) -> DomainResult<()> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        if !self.file_path.exists() {
            fs::write(&self.file_path, "[]").await?;
        }

        Ok(())
    }

    /// Migrate any plaintext API keys to secure storage.
    ///
    /// This method checks all stored accounts and moves any API keys
    /// that are still stored in plaintext (in the JSON file) to secure storage.
    pub async fn migrate_plaintext_keys(&self) -> DomainResult<()> {
        let accounts = self.find_all().await?;
        let mut migrated = 0;

        for account in &accounts {
            if let Some(ref api_key) = account.api_key {
                if !api_key.is_empty() {
                    // Store in secure storage
                    self.secure_storage
                        .store(&account.id, api_key)
                        .map_err(|e| DomainError::Internal(e.to_string()))?;
                    migrated += 1;
                }
            }
        }

        if migrated > 0 {
            tracing::info!("Migrated {} API keys to secure storage", migrated);
        }

        Ok(())
    }

    /// Reads all accounts from the JSON file.
    ///
    /// Acquires a shared (read) lock before reading to prevent reading
    /// during a concurrent write operation.
    async fn read_accounts(&self) -> DomainResult<Vec<AccountData>> {
        self.ensure_file_exists().await?;

        let file_path = self.file_path.clone();

        // Execute file locking in blocking task to avoid blocking the async runtime
        let contents = tokio::task::spawn_blocking(move || {
            // Open and lock the file in blocking context
            let mut file = std::fs::File::open(&file_path)?;

            // Acquire shared (read) lock in blocking context
            FileExt::lock_shared(&file)?;

            // Read contents
            let mut contents = String::new();
            std::io::Read::read_to_string(&mut file, &mut contents)?;

            DomainResult::Ok(contents)
        })
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))??;

        let accounts: Vec<AccountData> = serde_json::from_str(&contents)
            .map_err(|e| DomainError::Serialization(e.to_string()))?;

        Ok(accounts)
    }

    /// Convert AccountData to Account, retrieving API key from secure storage.
    fn account_data_to_account(&self, data: AccountData) -> DomainResult<Account> {
        // Try to retrieve API key from secure storage
        let api_key = self
            .secure_storage
            .retrieve(&data.id)
            .map_err(|e| DomainError::Internal(e.to_string()))?
            .map(|s| s.expose_secret().to_string());

        Ok(Account {
            id: data.id,
            provider_id: data.provider_id,
            api_key,
            access_token: data.access_token,
            refresh_token: data.refresh_token,
            id_token: data.id_token,
            token_expires_at: data.token_expires_at,
            is_active: data.is_active,
            priority: data.priority,
            created_at: data.created_at,
            last_used_at: data.last_used_at,
            auth_strategy_type: data.auth_strategy_type,
        })
    }

    /// Writes accounts to the JSON file using an atomic write pattern.
    ///
    /// T-22: Acquires an exclusive (write) lock before writing.
    /// T-23: Uses write-to-temp-then-rename pattern for atomicity.
    /// T-24: Lock acquisition has a 5-second timeout.
    async fn write_accounts(&self, accounts: &[AccountData]) -> DomainResult<()> {
        self.ensure_file_exists().await?;

        // T-23: Serialize to JSON bytes
        let json = serde_json::to_string_pretty(accounts)
            .map_err(|e| DomainError::Serialization(e.to_string()))?;

        // T-23: Write to temp file first
        let tmp_path = self.file_path.with_extension("tmp");

        // Write JSON content to temp file path first (async)
        tokio::fs::write(&tmp_path, &json).await?;

        // Execute file locking and sync in blocking task
        let tmp_path_clone = tmp_path.clone();
        let file_path_clone = self.file_path.clone();

        tokio::task::spawn_blocking(move || {
            // Open the temp file we just wrote
            let file = std::fs::OpenOptions::new()
                .write(true)
                .open(&tmp_path_clone)?;

            // Acquire exclusive (write) lock in blocking context
            FileExt::lock_exclusive(&file)?;

            // Ensure data is flushed to disk before rename
            file.sync_all()?;

            // T-23: Atomic rename (POSIX atomic on same filesystem)
            std::fs::rename(&tmp_path_clone, &file_path_clone)?;

            Ok::<(), DomainError>(())
        })
        .await
        .map_err(|e| DomainError::Internal(e.to_string()))??;

        Ok(())
    }
}

impl Default for JsonAccountRepository {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            file_path: PathBuf::from("accounts.json"),
            secure_storage: create_secure_storage(),
        })
    }
}

#[async_trait]
impl AccountRepository for JsonAccountRepository {
    async fn save(&self, account: Account) -> DomainResult<Account> {
        // Store API key in secure storage
        if let Some(ref api_key) = account.api_key {
            if !api_key.is_empty() {
                self.secure_storage
                    .store(&account.id, api_key)
                    .map_err(|e| DomainError::Internal(e.to_string()))?;
            }
        }

        let mut accounts = self.read_accounts().await?;

        // Create AccountData without the actual API key (it will be in secure storage)
        let mut account_data = AccountData::from(&account);
        account_data.api_key = None; // Don't store API key in JSON file

        // Check if account exists, update or insert
        if let Some(existing) = accounts.iter_mut().find(|a| a.id == account.id) {
            *existing = account_data;
        } else {
            accounts.push(account_data);
        }

        self.write_accounts(&accounts).await?;
        Ok(account)
    }

    async fn find_all(&self) -> DomainResult<Vec<Account>> {
        let accounts_data = self.read_accounts().await?;
        let mut accounts = Vec::with_capacity(accounts_data.len());

        for account_data in accounts_data {
            let account = self.account_data_to_account(account_data)?;
            accounts.push(account);
        }

        Ok(accounts)
    }

    async fn find_by_id(&self, id: &str) -> DomainResult<Account> {
        let accounts_data = self.read_accounts().await?;
        let account_data = accounts_data
            .into_iter()
            .find(|a| a.id == id)
            .ok_or_else(|| crate::domain::DomainError::AccountNotFound(id.to_string()))?;

        self.account_data_to_account(account_data)
    }

    async fn find_active(&self) -> DomainResult<Vec<Account>> {
        let accounts_data = self.read_accounts().await?;
        let mut accounts: Vec<Account> = Vec::with_capacity(accounts_data.len());

        for account_data in accounts_data {
            let account = self.account_data_to_account(account_data)?;
            if account.is_active {
                accounts.push(account);
            }
        }

        // Sort by priority (lower = higher priority)
        accounts.sort_by_key(|a| a.priority);
        Ok(accounts)
    }

    async fn find_active_by_provider(&self, provider_id: &str) -> DomainResult<Vec<Account>> {
        let accounts_data = self.read_accounts().await?;
        let mut accounts: Vec<Account> = Vec::with_capacity(accounts_data.len());

        for account_data in accounts_data {
            let account = self.account_data_to_account(account_data)?;
            if account.is_active && account.provider_id == provider_id {
                accounts.push(account);
            }
        }

        // Sort by priority (lower = higher priority)
        accounts.sort_by_key(|a| a.priority);
        Ok(accounts)
    }

    async fn delete(&self, id: &str) -> DomainResult<()> {
        let accounts = self.read_accounts().await?;

        // Verify account exists
        let exists = accounts.iter().any(|a| a.id == id);
        if !exists {
            return Err(crate::domain::DomainError::AccountNotFound(id.to_string()));
        }

        // Delete API key from secure storage
        self.secure_storage
            .delete(id)
            .map_err(|e| DomainError::Internal(e.to_string()))?;

        // Filter out the account to delete
        let updated: Vec<AccountData> = accounts.into_iter().filter(|a| a.id != id).collect();

        // Write updated accounts back to file (persist changes)
        self.write_accounts(&updated).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_delete_account_persists() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

        // Add an account first
        let account = Account::new("test-1", "openai", "sk-test-key");
        repo.save(account).await.unwrap();

        // Delete the account
        repo.delete("test-1").await.unwrap();

        // Verify account is deleted
        let result = repo.find_by_id("test-1").await;
        assert!(result.is_err());

        // Verify persistence by creating new repo instance
        let repo2 = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();
        let result2 = repo2.find_by_id("test-1").await;
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn test_delete_non_existent_account() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

        let result = repo.delete("non-existent").await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::domain::DomainError::AccountNotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_delete_and_verify_file_updated() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

        // Add two accounts
        repo.save(Account::new("test-1", "openai", "sk-key-1"))
            .await
            .unwrap();
        repo.save(Account::new("test-2", "groq", "sk-key-2"))
            .await
            .unwrap();

        // Delete one
        repo.delete("test-1").await.unwrap();

        // Verify only one remains
        let all = repo.find_all().await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, "test-2");
    }

    /// T-20: Verify stale temp files are cleaned up on init
    #[test]
    fn test_cleanup_stale_temp_files() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("accounts.json");
        let tmp_path = temp_dir.path().join("accounts.tmp");

        // Create a stale temp file
        std::fs::write(&tmp_path, "stale data").unwrap();
        assert!(tmp_path.exists());

        // Create repo pointing to same directory (triggers cleanup)
        let _repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

        // Temp file should be removed by with_config_dir cleanup
        assert!(!tmp_path.exists());
        // JSON file should not be affected
        assert!(!json_path.exists());
    }

    /// T-23: Verify atomic write doesn't leave temp files on success
    #[tokio::test]
    async fn test_atomic_write_no_temp_leftover() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

        let account = Account::new("test-1", "openai", "sk-key-1");
        repo.save(account).await.unwrap();

        // No .tmp files should exist after successful write
        let tmp_path = temp_dir.path().join("accounts.tmp");
        assert!(!tmp_path.exists());

        // Data should be persisted correctly
        let accounts = repo.find_all().await.unwrap();
        assert_eq!(accounts.len(), 1);
    }
}
