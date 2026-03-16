use crate::domain::Account;
use crate::domain::traits::DomainResult;
use crate::domain::services::auth_strategy::AuthenticationStrategy;
use async_trait::async_trait;

/// API Key authentication strategy implementation.
///
/// This strategy handles legacy API key authentication where the API key
/// is provided directly by the user and used in the Authorization header.
pub struct ApiKeyAuthStrategy {
    provider_id: String,
}

impl ApiKeyAuthStrategy {
    /// Creates a new API Key authentication strategy.
    ///
    /// # Arguments
    /// * `provider_id` - The ID of the provider this strategy is for
    ///
    /// # Returns
    /// A new `ApiKeyAuthStrategy` instance
    pub fn new(provider_id: impl Into<String>) -> Self {
        Self {
            provider_id: provider_id.into(),
        }
    }
}

#[async_trait]
impl AuthenticationStrategy for ApiKeyAuthStrategy {
    /// Initiates the API key authentication process.
    ///
    /// For API key authentication, this simply returns an Ok() indicating
    /// that the user should provide their API key through the CLI.
    ///
    /// # Returns
    /// A DomainResult containing an empty string (no verifier needed for API key)
    async fn initiate_auth(&self) -> DomainResult<String> {
        // For API key auth, we just indicate that the user needs to provide their key
        Ok("".to_string())
    }

    /// Completes the API key authentication process.
    ///
    /// # Arguments
    /// * `response` - The API key provided by the user
    ///
    /// # Returns
    /// A DomainResult containing the authenticated Account or an error
    async fn complete_auth(&self, api_key: String) -> DomainResult<Account> {
        if api_key.trim().is_empty() {
            return Err(crate::domain::DomainError::InvalidCredentials);
        }

        let account = Account::new_api_key(
            format!("api_key_{}", uuid::Uuid::new_v4()),
            &self.provider_id,
            api_key.trim(),
        );

        Ok(account)
    }

    /// Refreshes the access token for API key authentication.
    ///
    /// API keys don't expire or need refreshing, so this just returns the same account.
    ///
    /// # Arguments
    /// * `account` - The account to refresh (unused for API key)
    ///
    /// # Returns
    /// A DomainResult containing the same account
    async fn refresh_token(&self, account: &Account) -> DomainResult<Account> {
        Ok(account.clone())
    }

    /// Revokes tokens for API key authentication.
    ///
    /// For API keys, there's nothing to revoke on the server side,
    /// but we return the account as-is.
    ///
    /// # Arguments
    /// * `account` - The account to revoke (unused for API key)
    ///
    /// # Returns
    /// A DomainResult containing the same account
    async fn revoke_token(&self, account: &Account) -> DomainResult<Account> {
        Ok(account.clone())
    }

    /// Gets the type of authentication this strategy represents.
    ///
    /// # Returns
    /// "api_key" string slice
    fn auth_type(&self) -> &'static str {
        "api_key"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DomainError;

    #[tokio::test]
    async fn test_api_key_strategy_initiate_auth() {
        let strategy = ApiKeyAuthStrategy::new("test-provider");
        let result = strategy.initiate_auth().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[tokio::test]
    async fn test_api_key_strategy_complete_auth_success() {
        let strategy = ApiKeyAuthStrategy::new("test-provider");
        let result = strategy.complete_auth("sk-test-key-123".to_string()).await;
        assert!(result.is_ok());
        
        let account = result.unwrap();
        assert_eq!(account.provider_id, "test-provider");
        assert_eq!(account.api_key, Some("sk-test-key-123".to_string()));
        assert_eq!(account.auth_strategy_type, "api_key");
        assert!(account.is_active);
    }

    #[tokio::test]
    async fn test_api_key_strategy_complete_auth_empty_key() {
        let strategy = ApiKeyAuthStrategy::new("test-provider");
        let result = strategy.complete_auth("".to_string()).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DomainError::InvalidCredentials
        ));
    }

    #[tokio::test]
    async fn test_api_key_strategy_refresh_token() {
        let strategy = ApiKeyAuthStrategy::new("test-provider");
        let account = Account::new_api_key("test-id", "test-provider", "sk-test-key");
        let result = strategy.refresh_token(&account).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), account);
    }

    #[tokio::test]
    async fn test_api_key_strategy_revoke_token() {
        let strategy = ApiKeyAuthStrategy::new("test-provider");
        let account = Account::new_api_key("test-id", "test-provider", "sk-test-key");
        let result = strategy.revoke_token(&account).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_api_key_strategy_auth_type() {
        let strategy = ApiKeyAuthStrategy::new("test-provider");
        assert_eq!(strategy.auth_type(), "api_key");
    }
}