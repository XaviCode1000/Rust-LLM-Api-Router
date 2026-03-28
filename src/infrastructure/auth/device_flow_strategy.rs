use crate::domain::traits::DomainResult;
use crate::domain::Account;
use crate::error::Result;
use async_trait::async_trait;
use oauth2::{AuthUrl, ClientId, ClientSecret, TokenUrl};

use crate::domain::services::auth_strategy::AuthenticationStrategy;

/// OAuth 2.0 Device Authorization Grant authentication strategy implementation.
///
/// This strategy implements the OAuth 2.0 Device Authorization Grant flow
/// (RFC 8628) for authentication in input-constrained devices or headless
/// environments where a browser is not readily available.
pub struct DeviceFlowAuthStrategy {
    client_id: ClientId,
    #[allow(dead_code)]
    client_secret: Option<ClientSecret>,
    #[allow(dead_code)]
    device_auth_url: AuthUrl,
    #[allow(dead_code)]
    token_url: TokenUrl,
    #[allow(dead_code)]
    scopes: Vec<oauth2::Scope>,
    /// Polling interval in seconds (default: 5 seconds)
    #[allow(dead_code)]
    polling_interval: u64,
}

impl DeviceFlowAuthStrategy {
    /// Creates a new Device Flow authentication strategy.
    pub fn new(
        client_id: impl Into<String>,
        client_secret: Option<impl Into<String>>,
        device_auth_url: impl Into<String>,
        token_url: impl Into<String>,
        scopes: Vec<String>,
        polling_interval: Option<u64>,
    ) -> Result<Self> {
        Ok(Self {
            client_id: ClientId::new(client_id.into()),
            client_secret: client_secret.map(|s| ClientSecret::new(s.into())),
            device_auth_url: AuthUrl::new(device_auth_url.into())?,
            token_url: TokenUrl::new(token_url.into())?,
            scopes: scopes.into_iter().map(oauth2::Scope::new).collect(),
            polling_interval: polling_interval.unwrap_or(5),
        })
    }
}

#[async_trait]
impl AuthenticationStrategy for DeviceFlowAuthStrategy {
    /// Initiates the Device Flow authentication process.
    ///
    /// This is a stub implementation that returns instructions.
    async fn initiate_auth(&self) -> DomainResult<String> {
        // Return instructions for the user
        Ok(format!(
            "Please visit the device authorization URL with client_id: {}",
            self.client_id.as_str()
        ))
    }

    /// Completes the Device Flow authentication process.
    async fn complete_auth(&self, _response: String) -> DomainResult<Account> {
        // Stub - in real implementation would poll for token
        Err(crate::domain::DomainError::InvalidCredentials)
    }

    /// Refreshes the access token using the refresh token.
    async fn refresh_token(&self, account: &Account) -> DomainResult<Account> {
        // Check if we have a refresh token
        if account.refresh_token.is_none() {
            return Err(crate::domain::DomainError::InvalidCredentials);
        }
        // Stub - returns the same account
        Ok(account.clone())
    }

    /// Revokes tokens and clears credentials.
    async fn revoke_token(&self, account: &Account) -> DomainResult<Account> {
        let mut account = account.clone();
        account.access_token = None;
        account.refresh_token = None;
        account.id_token = None;
        account.token_expires_at = None;
        Ok(account)
    }

    /// Gets the type of authentication this strategy represents.
    fn auth_type(&self) -> &'static str {
        "device_flow"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_device_flow_strategy_new() {
        let strategy = DeviceFlowAuthStrategy::new(
            "test-client-id",
            Some("test-client-secret"),
            "https://auth.example.com/oauth2/device",
            "https://auth.example.com/oauth2/token",
            vec!["read".to_string(), "write".to_string()],
            Some(5),
        )
        .unwrap();

        assert_eq!(strategy.client_id.to_string(), "test-client-id");
        assert_eq!(strategy.polling_interval, 5);
    }

    #[tokio::test]
    async fn test_device_flow_strategy_auth_type() {
        let strategy = DeviceFlowAuthStrategy::new(
            "test-client-id",
            None::<String>,
            "https://auth.example.com/oauth2/device",
            "https://auth.example.com/oauth2/token",
            vec!["read".to_string()],
            None,
        )
        .unwrap();

        assert_eq!(strategy.auth_type(), "device_flow");
    }
}
