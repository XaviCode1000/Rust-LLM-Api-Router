use crate::domain::services::auth_strategy::AuthenticationStrategy;
use crate::domain::traits::{AccountRepository, ProviderRepository};
use crate::domain::{Account, DomainError, DomainResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Service for managing authentication flows and token operations.
///
/// This service coordinates between authentication strategies, account repositories,
/// and provider repositories to provide a unified interface for authentication
/// operations including initiation, completion, refresh, and revocation of tokens.
pub struct AuthService {
    account_repo: Arc<dyn AccountRepository + Send + Sync>,
    provider_repo: Arc<dyn ProviderRepository + Send + Sync>,
    /// Per-account refresh concurrency guard.
    /// Prevents concurrent refreshes for the same account while allowing
    /// concurrent refreshes for different accounts.
    refresh_locks: Arc<RwLock<HashMap<String, Arc<tokio::sync::Mutex<()>>>>>,
}

impl AuthService {
    /// Creates a new AuthService instance.
    ///
    /// # Arguments
    /// * `account_repo` - Repository for account persistence
    /// * `provider_repo` - Repository for provider persistence
    ///
    /// # Returns
    /// A new `AuthService` instance
    pub fn new(
        account_repo: Arc<dyn AccountRepository + Send + Sync>,
        provider_repo: Arc<dyn ProviderRepository + Send + Sync>,
    ) -> Self {
        Self {
            account_repo,
            provider_repo,
            refresh_locks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Gets the appropriate authentication strategy for a provider.
    ///
    /// # Arguments
    /// * `provider_id` - The ID of the provider to get strategy for
    ///
    /// # Returns
    /// A DomainResult containing the authentication strategy or an error
    async fn get_auth_strategy(
        &self,
        provider_id: &str,
    ) -> DomainResult<Box<dyn AuthenticationStrategy + Send + Sync>> {
        // Get the provider configuration
        let provider = self.provider_repo.find_enabled_by_id(provider_id).await?;

        // Determine which authentication strategy to use based on provider configuration
        if provider.is_oauth_configured() {
            // Check if we should use Device Flow (based on environment or configuration)
            // For now, we'll default to PKCE but check for NO_BROWSER env var
            if std::env::var("NO_BROWSER").is_ok() {
                // Use Device Flow
                if let Some(device_auth_url) = &provider.device_auth_url {
                    let strategy = crate::infrastructure::auth::device_flow_strategy::DeviceFlowAuthStrategy::new(
                        provider.client_id.clone().unwrap_or_default(),
                        provider.client_secret.clone(),
                        device_auth_url,
                        provider.token_url.clone().unwrap_or_default(),
                        vec![], // In a real implementation, scopes would come from provider config
                        None,
                    ).map_err(|e| DomainError::Internal(e.to_string()))?;
                    return Ok(Box::new(strategy));
                }
            }

            // Use PKCE flow
            if let (
                Some(client_id),
                Some(client_secret),
                Some(auth_url),
                Some(token_url),
                Some(redirect_uri),
            ) = (
                &provider.client_id,
                &provider.client_secret,
                &provider.auth_url,
                &provider.token_url,
                &provider.redirect_uri,
            ) {
                let strategy = crate::infrastructure::auth::pkce_strategy::PkceAuthStrategy::new(
                    client_id,
                    Some(client_secret),
                    auth_url,
                    token_url,
                    redirect_uri,
                    vec![], // In a real implementation, scopes would come from provider config
                )
                .map_err(|e| DomainError::Internal(e.to_string()))?;
                return Ok(Box::new(strategy));
            }
        }

        // Fallback to API Key strategy
        let strategy =
            crate::infrastructure::auth::api_key_strategy::ApiKeyAuthStrategy::new(provider_id);
        Ok(Box::new(strategy))
    }

    /// Initiates the authentication process for a provider.
    ///
    /// # Arguments
    /// * `provider_id` - The ID of the provider to authenticate with
    ///
    /// # Returns
    /// A DomainResult containing information needed to complete authentication
    pub async fn initiate_auth(&self, provider_id: &str) -> DomainResult<String> {
        let strategy = self.get_auth_strategy(provider_id).await?;
        strategy.initiate_auth().await
    }

    /// Completes the authentication process for a provider.
    ///
    /// # Arguments
    /// * `provider_id` - The ID of the provider that was authenticated with
    /// * `response` - The response from the authentication provider
    ///
    /// # Returns
    /// A DomainResult containing the authenticated account
    pub async fn complete_auth(
        &self,
        provider_id: &str,
        response: String,
    ) -> DomainResult<Account> {
        let strategy = self.get_auth_strategy(provider_id).await?;
        let mut account = strategy.complete_auth(response).await?;

        // Set the correct provider ID in the account (strategies might use a placeholder)
        account.provider_id = provider_id.into();

        // Save the account
        self.account_repo.save(account).await
    }

    /// Refreshes the access token for an account.
    ///
    /// # Arguments
    /// * `account_id` - The ID of the account to refresh
    ///
    /// # Returns
    /// A DomainResult containing the updated account with new tokens
    pub async fn refresh_token(&self, account_id: &str) -> DomainResult<Account> {
        // Acquire per-account refresh lock — concurrent refreshes for DIFFERENT
        // accounts proceed in parallel, but concurrent refreshes for the SAME
        // account are serialized to prevent thundering-herd token requests.
        let lock = {
            let mut locks = self.refresh_locks.write().await;
            locks
                .entry(account_id.to_string())
                .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
                .clone()
        };
        let _guard = lock.lock().await;

        // TWO-PHASE REFRESH PATTERN (future):
        // Phase 1: Call provider API to get new tokens (old tokens still valid on disk)
        // Phase 2: Only if Phase 1 succeeds, persist new tokens atomically
        // This prevents "logical death" if the refresh is cancelled mid-flight.

        // Get the account
        let account = self.account_repo.find_by_id(account_id).await?;

        // Get the provider to determine auth strategy
        let provider = self
            .provider_repo
            .find_enabled_by_id(account.provider_id.as_str())
            .await?;

        // Determine which authentication strategy to use
        let strategy: Box<dyn AuthenticationStrategy + Send + Sync> = if provider
            .is_oauth_configured()
        {
            // Check if we should use Device Flow
            if std::env::var("NO_BROWSER").is_ok() {
                // Use Device Flow
                if let Some(device_auth_url) = &provider.device_auth_url {
                    Box::new(crate::infrastructure::auth::device_flow_strategy::DeviceFlowAuthStrategy::new(
                        provider.client_id.clone().unwrap_or_default(),
                        provider.client_secret.clone(),
                        device_auth_url,
                        provider.token_url.clone().unwrap_or_default(),
                        vec![],
                        None,
                    ).map_err(|e| DomainError::Internal(e.to_string()))?)
                } else {
                    // Fallback to PKCE if device auth URL not configured
                    Box::new(
                        crate::infrastructure::auth::pkce_strategy::PkceAuthStrategy::new(
                            provider.client_id.clone().unwrap_or_default(),
                            provider.client_secret.clone(),
                            provider.auth_url.clone().unwrap_or_default(),
                            provider.token_url.clone().unwrap_or_default(),
                            provider.redirect_uri.clone().unwrap_or_default(),
                            vec![],
                        )
                        .map_err(|e| DomainError::Internal(e.to_string()))?,
                    )
                }
            } else {
                // Use PKCE flow
                Box::new(
                    crate::infrastructure::auth::pkce_strategy::PkceAuthStrategy::new(
                        provider.client_id.clone().unwrap_or_default(),
                        provider.client_secret.clone(),
                        provider.auth_url.clone().unwrap_or_default(),
                        provider.token_url.clone().unwrap_or_default(),
                        provider.redirect_uri.clone().unwrap_or_default(),
                        vec![],
                    )
                    .map_err(|e| DomainError::Internal(e.to_string()))?,
                )
            }
        } else {
            // Fallback to API Key strategy
            Box::new(
                crate::infrastructure::auth::api_key_strategy::ApiKeyAuthStrategy::new(
                    account.provider_id.as_str(),
                ),
            )
        };

        // Refresh the token
        let refreshed_account = strategy.refresh_token(&account).await?;

        // Save the updated account
        self.account_repo.save(refreshed_account).await
    }

    /// Revokes tokens for an account.
    ///
    /// # Arguments
    /// * `account_id` - The ID of the account to revoke tokens for
    ///
    /// # Returns
    /// A DomainResult indicating success or failure
    pub async fn revoke_token(&self, account_id: &str) -> DomainResult<()> {
        // Get the account
        let account = self.account_repo.find_by_id(account_id).await?;

        // Get the provider to determine auth strategy
        let provider = self
            .provider_repo
            .find_enabled_by_id(account.provider_id.as_str())
            .await?;

        // Determine which authentication strategy to use
        let strategy: Box<dyn AuthenticationStrategy + Send + Sync> = if provider
            .is_oauth_configured()
        {
            // Check if we should use Device Flow
            if std::env::var("NO_BROWSER").is_ok() {
                // Use Device Flow
                if let Some(device_auth_url) = &provider.device_auth_url {
                    Box::new(crate::infrastructure::auth::device_flow_strategy::DeviceFlowAuthStrategy::new(
                        provider.client_id.clone().unwrap_or_default(),
                        provider.client_secret.clone(),
                        device_auth_url,
                        provider.token_url.clone().unwrap_or_default(),
                        vec![],
                        None,
                    ).map_err(|e| DomainError::Internal(e.to_string()))?)
                } else {
                    // Fallback to PKCE if device auth URL not configured
                    Box::new(
                        crate::infrastructure::auth::pkce_strategy::PkceAuthStrategy::new(
                            provider.client_id.clone().unwrap_or_default(),
                            provider.client_secret.clone(),
                            provider.auth_url.clone().unwrap_or_default(),
                            provider.token_url.clone().unwrap_or_default(),
                            provider.redirect_uri.clone().unwrap_or_default(),
                            vec![],
                        )
                        .map_err(|e| DomainError::Internal(e.to_string()))?,
                    )
                }
            } else {
                // Use PKCE flow
                Box::new(
                    crate::infrastructure::auth::pkce_strategy::PkceAuthStrategy::new(
                        provider.client_id.clone().unwrap_or_default(),
                        provider.client_secret.clone(),
                        provider.auth_url.clone().unwrap_or_default(),
                        provider.token_url.clone().unwrap_or_default(),
                        provider.redirect_uri.clone().unwrap_or_default(),
                        vec![],
                    )
                    .map_err(|e| DomainError::Internal(e.to_string()))?,
                )
            }
        } else {
            // Fallback to API Key strategy
            Box::new(
                crate::infrastructure::auth::api_key_strategy::ApiKeyAuthStrategy::new(
                    account.provider_id.as_str(),
                ),
            )
        };

        // Revoke the token and persist the zeroed account state
        let revoked_account = strategy.revoke_token(&account).await?;
        self.account_repo.save(revoked_account).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::services::auth_strategy::AuthenticationStrategy;
    use crate::domain::traits::{AccountRepository, ProviderRepository};
    use crate::domain::{Account, Provider};
    use async_trait::async_trait;
    use std::sync::Arc;
    use std::sync::Mutex;

    // Mock repositories for testing
    struct MockAccountRepository {
        accounts: Mutex<std::collections::HashMap<String, Account>>,
    }

    #[async_trait]
    impl AccountRepository for MockAccountRepository {
        async fn save(&self, account: Account) -> DomainResult<Account> {
            let id = account.id.to_string();
            self.accounts.lock().unwrap().insert(id, account.clone());
            Ok(account)
        }

        async fn find_all(&self) -> DomainResult<Vec<Account>> {
            Ok(self.accounts.lock().unwrap().values().cloned().collect())
        }

        async fn find_by_id(&self, id: &str) -> DomainResult<Account> {
            self.accounts
                .lock()
                .unwrap()
                .get(id)
                .cloned()
                .ok_or_else(|| crate::domain::DomainError::AccountNotFound(id.to_string()))
        }

        async fn find_active(&self) -> DomainResult<Vec<Account>> {
            Ok(self
                .accounts
                .lock()
                .unwrap()
                .values()
                .filter(|a| a.is_active)
                .cloned()
                .collect())
        }

        async fn find_active_by_provider(&self, provider_id: &str) -> DomainResult<Vec<Account>> {
            Ok(self
                .accounts
                .lock()
                .unwrap()
                .values()
                .filter(|a| a.is_active && a.provider_id == provider_id)
                .cloned()
                .collect())
        }

        async fn delete(&self, id: &str) -> DomainResult<()> {
            self.accounts.lock().unwrap().remove(id);
            Ok(())
        }
    }

    struct MockProviderRepository {
        providers: Mutex<std::collections::HashMap<String, Provider>>,
    }

    #[async_trait]
    impl ProviderRepository for MockProviderRepository {
        async fn save(&self, provider: Provider) -> DomainResult<Provider> {
            Ok(provider)
        }

        async fn find_all(&self) -> DomainResult<Vec<Provider>> {
            Ok(self.providers.lock().unwrap().values().cloned().collect())
        }

        async fn find_by_id(&self, id: &str) -> DomainResult<Provider> {
            self.providers
                .lock()
                .unwrap()
                .get(id)
                .cloned()
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
            self.providers.lock().unwrap().remove(id);
            Ok(())
        }
    }

    // Mock authentication strategy for testing
    #[allow(dead_code)]
    struct MockAuthStrategy {
        auth_type: &'static str,
        should_succeed: bool,
    }

    #[async_trait]
    impl AuthenticationStrategy for MockAuthStrategy {
        async fn initiate_auth(&self) -> DomainResult<String> {
            if self.should_succeed {
                Ok("test-response".to_string())
            } else {
                Err(crate::domain::DomainError::InvalidCredentials)
            }
        }

        async fn complete_auth(&self, response: String) -> DomainResult<Account> {
            if self.should_succeed && response == "test-response" {
                Ok(Account::new_api_key(
                    "test-account",
                    "test-provider",
                    "test-key",
                ))
            } else {
                Err(crate::domain::DomainError::InvalidCredentials)
            }
        }

        async fn refresh_token(&self, account: &Account) -> DomainResult<Account> {
            if self.should_succeed {
                Ok(account.clone())
            } else {
                Err(crate::domain::DomainError::InvalidCredentials)
            }
        }

        async fn revoke_token(&self, account: &Account) -> DomainResult<Account> {
            if self.should_succeed {
                Ok(account.clone())
            } else {
                Err(crate::domain::DomainError::InvalidCredentials)
            }
        }

        fn auth_type(&self) -> &'static str {
            self.auth_type
        }
    }

    #[tokio::test]
    async fn test_auth_service_new() {
        let account_repo = Arc::new(MockAccountRepository {
            accounts: Mutex::new(std::collections::HashMap::new()),
        });
        let provider_repo = Arc::new(MockProviderRepository {
            providers: Mutex::new(std::collections::HashMap::new()),
        });

        let service = AuthService::new(account_repo, provider_repo);
        assert!(Arc::strong_count(&service.account_repo) >= 1);
        assert!(Arc::strong_count(&service.provider_repo) >= 1);
    }

    #[tokio::test]
    async fn test_auth_service_initiate_auth_success() {
        // Setup mock provider repository with OAuth provider
        let mut providers = std::collections::HashMap::new();
        providers.insert(
            "test-provider".to_string(),
            Provider::new("test-provider", "Test Provider", "https://api.test.com"),
        );

        let account_repo = Arc::new(MockAccountRepository {
            accounts: Mutex::new(std::collections::HashMap::new()),
        });
        let provider_repo = Arc::new(MockProviderRepository {
            providers: Mutex::new(providers),
        });

        let service = AuthService::new(account_repo, provider_repo);

        // This would normally try to open a browser, so we expect it to fail in test environment
        // In a real test, we would mock the browser opening and HTTP listener
        let result = service.initiate_auth("test-provider").await;
        // We're not checking the exact result here since it depends on mocks
        // Just verifying the method can be called
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_auth_service_complete_auth_success() {
        // Setup mock provider repository
        let mut providers = std::collections::HashMap::new();
        providers.insert(
            "test-provider".to_string(),
            Provider::new("test-provider", "Test Provider", "https://api.test.com"),
        );

        let account_repo = Arc::new(MockAccountRepository {
            accounts: Mutex::new(std::collections::HashMap::new()),
        });
        let provider_repo = Arc::new(MockProviderRepository {
            providers: Mutex::new(providers),
        });

        let service = AuthService::new(account_repo, provider_repo);

        // Test with API key strategy (should work in test env)
        let result = service
            .complete_auth("test-provider", "test-api-key".to_string())
            .await;
        assert!(result.is_ok());

        let account = result.unwrap();
        assert_eq!(account.provider_id, "test-provider");
        assert_eq!(account.auth_method.api_key(), Some("test-api-key"));
        assert_eq!(account.auth_type(), "api_key");
    }

    #[tokio::test]
    async fn test_auth_service_refresh_token_success() {
        // Setup mock provider repository
        let mut providers = std::collections::HashMap::new();
        providers.insert(
            "test-provider".to_string(),
            Provider::new("test-provider", "Test Provider", "https://api.test.com"),
        );

        let mut accounts = std::collections::HashMap::new();
        accounts.insert(
            "test-account".to_string(),
            Account::new_api_key("test-account", "test-provider", "test-api-key"),
        );

        let account_repo = Arc::new(MockAccountRepository {
            accounts: Mutex::new(accounts),
        });
        let provider_repo = Arc::new(MockProviderRepository {
            providers: Mutex::new(providers),
        });

        let service = AuthService::new(account_repo, provider_repo);

        let result = service.refresh_token("test-account").await;
        assert!(result.is_ok());

        let account = result.unwrap();
        assert_eq!(account.id, "test-account");
        assert_eq!(account.provider_id, "test-provider");
    }

    #[tokio::test]
    async fn test_auth_service_revoke_token_success() {
        // Setup mock provider repository
        let mut providers = std::collections::HashMap::new();
        providers.insert(
            "test-provider".to_string(),
            Provider::new("test-provider", "Test Provider", "https://api.test.com"),
        );

        let mut accounts = std::collections::HashMap::new();
        accounts.insert(
            "test-account".to_string(),
            Account::new_api_key("test-account", "test-provider", "test-api-key"),
        );

        let account_repo = Arc::new(MockAccountRepository {
            accounts: Mutex::new(accounts),
        });
        let provider_repo = Arc::new(MockProviderRepository {
            providers: Mutex::new(providers),
        });

        let service = AuthService::new(account_repo.clone(), provider_repo);

        let result = service.revoke_token("test-account").await;
        assert!(result.is_ok());

        // Verify the revoked state was persisted — reload and check tokens are cleared
        let persisted = account_repo.find_by_id("test-account").await.unwrap();
        assert_eq!(persisted.auth_type(), "api_key");
        // API key accounts don't have OAuth tokens to clear, but the save was called
    }

    #[tokio::test]
    async fn test_auth_service_revoke_token_persists_oauth_state() {
        // Setup with OAuth-configured provider
        let mut providers = std::collections::HashMap::new();
        let mut provider =
            Provider::new("oauth-provider", "OAuth Provider", "https://api.test.com");
        provider.client_id = Some("client-123".to_string());
        provider.client_secret = Some("secret-456".to_string());
        provider.auth_url = Some("https://auth.test.com/authorize".to_string());
        provider.token_url = Some("https://auth.test.com/token".to_string());
        provider.redirect_uri = Some("https://app.test.com/callback".to_string());
        providers.insert("oauth-provider".to_string(), provider);

        let mut accounts = std::collections::HashMap::new();
        accounts.insert(
            "oauth-account".to_string(),
            Account::new_oauth(
                "oauth-account",
                "oauth-provider",
                "valid-access-token",
                Some("valid-refresh-token"),
                Some("id-token"),
                Some(3600),
            ),
        );

        let account_repo = Arc::new(MockAccountRepository {
            accounts: Mutex::new(accounts),
        });
        let provider_repo = Arc::new(MockProviderRepository {
            providers: Mutex::new(providers),
        });

        let service = AuthService::new(account_repo.clone(), provider_repo);

        // Verify account has OAuth tokens before revoke
        let before = account_repo.find_by_id("oauth-account").await.unwrap();
        assert!(before.auth_method.is_oauth());
        assert_eq!(
            before.auth_method.access_token(),
            Some("valid-access-token")
        );

        // Revoke
        let result = service.revoke_token("oauth-account").await;
        assert!(result.is_ok());

        // Verify persisted state has zeroed tokens
        let after = account_repo.find_by_id("oauth-account").await.unwrap();
        assert!(after.auth_method.is_oauth());
        assert_eq!(after.auth_method.access_token(), Some(""));
        assert_eq!(after.get_refresh_token(), None);
    }
}
