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

/// Internal representation for JSON serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProviderData {
    id: String,
    name: String,
    base_url: String,
    enabled: bool,
    /// OAuth 2.0 client ID for authentication flows
    #[serde(skip_serializing_if = "Option::is_none")]
    client_id: Option<String>,
    /// OAuth 2.0 client secret for authentication flows (kept confidential)
    #[serde(skip_serializing_if = "Option::is_none")]
    client_secret: Option<String>,
    /// Authorization endpoint URL for OAuth flows
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_url: Option<String>,
    /// Token endpoint URL for OAuth flows
    #[serde(skip_serializing_if = "Option::is_none")]
    token_url: Option<String>,
    /// Redirect URI for OAuth 2.1 PKCE flow
    #[serde(skip_serializing_if = "Option::is_none")]
    redirect_uri: Option<String>,
    /// Device authorization endpoint for OAuth 2.0 Device Flow
    #[serde(skip_serializing_if = "Option::is_none")]
    device_auth_url: Option<String>,
}

impl From<&Provider> for ProviderData {
    fn from(provider: &Provider) -> Self {
        Self {
            id: provider.id.clone(),
            name: provider.name.clone(),
            base_url: provider.base_url.clone(),
            enabled: provider.enabled,
            client_id: provider.client_id.clone(),
            client_secret: provider.client_secret.clone(),
            auth_url: provider.auth_url.clone(),
            token_url: provider.token_url.clone(),
            redirect_uri: provider.redirect_uri.clone(),
            device_auth_url: provider.device_auth_url.clone(),
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
            client_id: data.client_id,
            client_secret: data.client_secret,
            auth_url: data.auth_url,
            token_url: data.token_url,
            redirect_uri: data.redirect_uri,
            device_auth_url: data.device_auth_url,
        }
    }
}

/// JSON-based provider repository.
///
/// Stores providers in a JSON file at the configured path.
pub struct JsonProviderRepository {
    file_path: PathBuf,
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

    async fn delete(&self, id: &str) -> DomainResult<()> {
        let providers = self.read_providers().await?;

        // Verify provider exists
        let exists = providers.iter().any(|p| p.id == id);
        if !exists {
            return Err(crate::domain::DomainError::ProviderNotFound(id.to_string()));
        }

        // Filter out the provider to delete
        let updated: Vec<ProviderData> = providers.into_iter().filter(|p| p.id != id).collect();

        // Write updated providers back to file (persist changes)
        self.write_providers(&updated).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_delete_provider_persists() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonProviderRepository::with_config_dir(temp_dir.path()).unwrap();

        // Add a provider first
        let provider = Provider::new("test-provider", "Test", "https://test.api.com");
        repo.save(provider).await.unwrap();

        // Delete the provider
        repo.delete("test-provider").await.unwrap();

        // Verify provider is deleted
        let result = repo.find_by_id("test-provider").await;
        assert!(result.is_err());

        // Verify persistence by creating new repo instance
        let repo2 = JsonProviderRepository::with_config_dir(temp_dir.path()).unwrap();
        let result2 = repo2.find_by_id("test-provider").await;
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn test_delete_non_existent_provider() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonProviderRepository::with_config_dir(temp_dir.path()).unwrap();

        let result = repo.delete("non-existent").await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::domain::DomainError::ProviderNotFound(_)));
    }
}
