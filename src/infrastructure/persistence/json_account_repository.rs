//! JSON-based account repository implementation
//!
//! This module provides a file-based persistence layer for accounts
//! using JSON serialization with secure API key handling.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncReadExt;

use crate::domain::traits::AccountRepository;
use crate::domain::{Account, DomainResult};
use crate::Result;

/// Internal representation for JSON serialization.
/// API keys are stored encrypted in production, plaintext in dev.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AccountData {
    id: String,
    provider_id: String,
    api_key: String,
    is_active: bool,
    priority: i32,
    #[serde(default)]
    created_at: Option<u64>,
    #[serde(default)]
    last_used_at: Option<u64>,
}

impl From<&Account> for AccountData {
    fn from(account: &Account) -> Self {
        Self {
            id: account.id.clone(),
            provider_id: account.provider_id.clone(),
            api_key: account.api_key.clone(),
            is_active: account.is_active,
            priority: account.priority,
            created_at: None,
            last_used_at: None,
        }
    }
}

impl From<AccountData> for Account {
    fn from(data: AccountData) -> Self {
        Self {
            id: data.id,
            provider_id: data.provider_id,
            api_key: data.api_key,
            is_active: data.is_active,
            priority: data.priority,
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
        Ok(Self { file_path })
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
        Ok(Self { file_path })
    }

    /// Ensures the directory and file exist.
    async fn ensure_file_exists(&self) -> DomainResult<()> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| crate::domain::DomainError::io(e.to_string()))?;
        }

        if !self.file_path.exists() {
            fs::write(&self.file_path, "[]")
                .await
                .map_err(|e| crate::domain::DomainError::io(e.to_string()))?;
        }

        Ok(())
    }

    /// Reads all accounts from the JSON file.
    async fn read_accounts(&self) -> DomainResult<Vec<AccountData>> {
        self.ensure_file_exists().await?;

        let mut file = OpenOptions::new()
            .read(true)
            .open(&self.file_path)
            .await
            .map_err(|e| crate::domain::DomainError::io(e.to_string()))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .await
            .map_err(|e| crate::domain::DomainError::io(e.to_string()))?;

        let accounts: Vec<AccountData> = serde_json::from_str(&contents)
            .map_err(|e| crate::domain::DomainError::serialization(e.to_string()))?;

        Ok(accounts)
    }

    /// Writes accounts to the JSON file.
    async fn write_accounts(&self, accounts: &[AccountData]) -> DomainResult<()> {
        self.ensure_file_exists().await?;

        let json = serde_json::to_string_pretty(accounts)
            .map_err(|e| crate::domain::DomainError::serialization(e.to_string()))?;

        fs::write(&self.file_path, json)
            .await
            .map_err(|e| crate::domain::DomainError::io(e.to_string()))?;

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
}
