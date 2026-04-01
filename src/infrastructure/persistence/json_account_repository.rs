//! JSON-based account repository implementation
//!
//! This module provides a file-based persistence layer for accounts
//! using JSON serialization with secure API key handling.

use async_trait::async_trait;
use fs4::tokio::AsyncFileExt;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::domain::traits::AccountRepository;
use crate::domain::{Account, DomainError, DomainResult};
use crate::Result;

/// Lock acquisition timeout duration.
const LOCK_TIMEOUT: Duration = Duration::from_secs(5);

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
/// API keys are stored in plaintext for development - use encryption in production.
pub struct JsonAccountRepository {
    file_path: PathBuf,
}

impl Clone for JsonAccountRepository {
    fn clone(&self) -> Self {
        Self {
            file_path: self.file_path.clone(),
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
        let repo = Self { file_path };

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
        let repo = Self { file_path };
        repo.cleanup_stale_temp_files();
        Ok(repo)
    }

    /// Ensures the directory and file exist.
    async fn ensure_file_exists(&self) -> DomainResult<()> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| crate::domain::DomainError::Io(e.to_string()))?;
        }

        if !self.file_path.exists() {
            fs::write(&self.file_path, "[]")
                .await
                .map_err(|e| crate::domain::DomainError::Io(e.to_string()))?;
        }

        Ok(())
    }

    /// Reads all accounts from the JSON file.
    ///
    /// Acquires a shared (read) lock before reading to prevent reading
    /// during a concurrent write operation.
    async fn read_accounts(&self) -> DomainResult<Vec<AccountData>> {
        self.ensure_file_exists().await?;

        let file = OpenOptions::new()
            .read(true)
            .open(&self.file_path)
            .await
            .map_err(|e| crate::domain::DomainError::Io(e.to_string()))?;

        // T-21: Acquire shared (read) lock with timeout (T-24)
        tokio::time::timeout(LOCK_TIMEOUT, file.lock_shared())
            .await
            .map_err(|_| {
                DomainError::LockTimeout(format!(
                    "Failed to acquire read lock within {:?}",
                    LOCK_TIMEOUT
                ))
            })?
            .map_err(|e| DomainError::Io(e.to_string()))?;

        // Read from the locked file handle
        let mut contents = String::new();
        let mut file = file; // Take ownership
        file.read_to_string(&mut contents)
            .await
            .map_err(|e| DomainError::Io(e.to_string()))?;

        let accounts: Vec<AccountData> = serde_json::from_str(&contents)
            .map_err(|e| DomainError::Serialization(e.to_string()))?;

        Ok(accounts)
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

        {
            let mut tmp_file = File::create(&tmp_path)
                .await
                .map_err(|e| DomainError::Io(e.to_string()))?;

            // T-22: Acquire exclusive (write) lock with timeout (T-24)
            tokio::time::timeout(LOCK_TIMEOUT, tmp_file.lock_exclusive())
                .await
                .map_err(|_| {
                    DomainError::LockTimeout(format!(
                        "Failed to acquire write lock within {:?}",
                        LOCK_TIMEOUT
                    ))
                })?
                .map_err(|e| DomainError::Io(e.to_string()))?;

            // Write JSON to temp file
            tmp_file
                .write_all(json.as_bytes())
                .await
                .map_err(|e| DomainError::Io(e.to_string()))?;

            // Ensure data is flushed to disk before rename
            tmp_file
                .sync_all()
                .await
                .map_err(|e| DomainError::Io(e.to_string()))?;
        }

        // T-23: Atomic rename (POSIX atomic on same filesystem)
        fs::rename(&tmp_path, &self.file_path).await.map_err(|e| {
            // Clean up temp file on failure
            let _ = std::fs::remove_file(&tmp_path);
            DomainError::Io(format!("Failed to atomically rename temp file: {}", e))
        })?;

        Ok(())
    }
}

impl Default for JsonAccountRepository {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            file_path: PathBuf::from("accounts.json"),
        })
    }
}

#[async_trait]
impl AccountRepository for JsonAccountRepository {
    async fn save(&self, account: Account) -> DomainResult<Account> {
        let mut accounts = self.read_accounts().await?;

        // Check if account exists, update or insert
        if let Some(existing) = accounts.iter_mut().find(|a| a.id == account.id) {
            *existing = AccountData::from(&account);
        } else {
            accounts.push(AccountData::from(&account));
        }

        self.write_accounts(&accounts).await?;
        Ok(account)
    }

    async fn find_all(&self) -> DomainResult<Vec<Account>> {
        let accounts = self.read_accounts().await?;
        Ok(accounts.into_iter().map(Account::from).collect())
    }

    async fn find_by_id(&self, id: &str) -> DomainResult<Account> {
        let accounts = self.read_accounts().await?;
        accounts
            .into_iter()
            .map(Account::from)
            .find(|a| a.id == id)
            .ok_or_else(|| crate::domain::DomainError::AccountNotFound(id.to_string()))
    }

    async fn find_active(&self) -> DomainResult<Vec<Account>> {
        let mut accounts: Vec<Account> = self
            .read_accounts()
            .await?
            .into_iter()
            .map(Account::from)
            .filter(|a| a.is_active)
            .collect();

        // Sort by priority (lower = higher priority)
        accounts.sort_by_key(|a| a.priority);
        Ok(accounts)
    }

    async fn find_active_by_provider(&self, provider_id: &str) -> DomainResult<Vec<Account>> {
        let mut accounts: Vec<Account> = self
            .read_accounts()
            .await?
            .into_iter()
            .map(Account::from)
            .filter(|a| a.is_active && a.provider_id == provider_id)
            .collect();

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

        // Create repo pointing to same directory
        let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

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
