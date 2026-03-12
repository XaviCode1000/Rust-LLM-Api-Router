//! JSON-based provider repository implementation
//!
//! This module provides a file-based persistence layer for providers
//! using JSON serialization.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncReadExt;

use crate::domain::traits::ProviderRepository;
use crate::domain::{DomainResult, Provider};

/// JSON-based provider repository.
///
/// Stores providers in a JSON file at the configured path.
pub struct JsonProviderRepository {
    file_path: PathBuf,
}

/// Internal representation for JSON serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProviderData {
    id: String,
    name: String,
    base_url: String,
    enabled: bool,
}

impl From<&Provider> for ProviderData {
    fn from(provider: &Provider) -> Self {
        Self {
            id: provider.id.clone(),
            name: provider.name.clone(),
            base_url: provider.base_url.clone(),
            enabled: provider.enabled,
        }
    }
}

impl From<ProviderData> for Provider {
    fn from(data: ProviderData) -> Self {
        Self {
            id: data.id,
            name: data.name,
            base_url: data.base_url,
            enabled: data.enabled,
        }
    }
}

impl JsonProviderRepository {
    /// Creates a new repository with the default config path.
    ///
    /// # Returns
    /// A new `JsonProviderRepository` instance
    pub fn new() -> DomainResult<Self> {
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

        let file_path = config_dir
            .join("rust-llm-api-router")
            .join("providers.json");
        Ok(Self { file_path })
    }

    /// Creates a new repository with a custom config directory.
    ///
    /// # Arguments
    /// * `config_dir` - Custom configuration directory path
    ///
    /// # Returns
    /// A new `JsonProviderRepository` instance
    pub fn with_config_dir(config_dir: &Path) -> DomainResult<Self> {
        let file_path = config_dir.join("providers.json");
        Ok(Self { file_path })
    }

    /// Ensures the directory and file exist.
    async fn ensure_file_exists(&self) -> DomainResult<()> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| crate::domain::DomainError::Internal(e.to_string()))?;
        }

        if !self.file_path.exists() {
            fs::write(&self.file_path, "[]")
                .await
                .map_err(|e| crate::domain::DomainError::Internal(e.to_string()))?;
        }

        Ok(())
    }

    /// Reads all providers from the JSON file.
    async fn read_providers(&self) -> DomainResult<Vec<ProviderData>> {
        self.ensure_file_exists().await?;

        let mut file = OpenOptions::new()
            .read(true)
            .open(&self.file_path)
            .await
            .map_err(|e| crate::domain::DomainError::Internal(e.to_string()))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .await
            .map_err(|e| crate::domain::DomainError::Internal(e.to_string()))?;

        let providers: Vec<ProviderData> = serde_json::from_str(&contents)
            .map_err(|e| crate::domain::DomainError::Internal(e.to_string()))?;

        Ok(providers)
    }

    /// Writes providers to the JSON file.
    async fn write_providers(&self, providers: &[ProviderData]) -> DomainResult<()> {
        self.ensure_file_exists().await?;

        let json = serde_json::to_string_pretty(providers)
            .map_err(|e| crate::domain::DomainError::Internal(e.to_string()))?;

        fs::write(&self.file_path, json)
            .await
            .map_err(|e| crate::domain::DomainError::Internal(e.to_string()))?;

        Ok(())
    }
}

impl Default for JsonProviderRepository {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            file_path: PathBuf::from("providers.json"),
        })
    }
}

#[async_trait]
impl ProviderRepository for JsonProviderRepository {
    async fn save(&self, provider: Provider) -> DomainResult<Provider> {
        let mut providers = self.read_providers().await?;

        // Check if provider exists, update or insert
        if let Some(existing) = providers.iter_mut().find(|p| p.id == provider.id) {
            *existing = ProviderData::from(&provider);
        } else {
            providers.push(ProviderData::from(&provider));
        }

        self.write_providers(&providers).await?;
        Ok(provider)
    }

    async fn find_all(&self) -> DomainResult<Vec<Provider>> {
        let providers = self.read_providers().await?;
        Ok(providers.into_iter().map(Provider::from).collect())
    }

    async fn find_by_id(&self, id: &str) -> DomainResult<Provider> {
        let providers = self.read_providers().await?;
        providers
            .into_iter()
            .map(Provider::from)
            .find(|p| p.id == id)
            .ok_or_else(|| crate::domain::DomainError::ProviderNotFound(id.to_string()))
    }

    async fn find_enabled_by_id(&self, id: &str) -> DomainResult<Provider> {
        let provider = self.find_by_id(id).await?;
        if provider.enabled {
            Ok(provider)
        } else {
            Err(crate::domain::DomainError::ProviderDisabled(id.to_string()))
        }
    }
}
