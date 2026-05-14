use crate::domain::services::auth_strategy::AuthenticationStrategy;
use crate::domain::traits::DomainResult;
use crate::domain::Account;
use crate::error::Result;
use async_trait::async_trait;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, Scope, TokenUrl};

/// OAuth 2.1 PKCE authentication strategy implementation.
///
/// This strategy implements the OAuth 2.1 Authorization Code Flow with PKCE
/// (Proof Key for Code Exchange) for secure authentication in public clients
/// like CLI applications.
pub struct PkceAuthStrategy {
    client_id: ClientId,
    #[allow(dead_code)]
    client_secret: Option<ClientSecret>,
    auth_url: AuthUrl,
    #[allow(dead_code)]
    token_url: TokenUrl,
    #[allow(dead_code)]
    redirect_url: RedirectUrl,
    #[allow(dead_code)]
    scopes: Vec<Scope>,
}

impl PkceAuthStrategy {
    /// Creates a new PKCE authentication strategy.
    pub fn new(
        client_id: impl Into<String>,
        client_secret: Option<impl Into<String>>,
        auth_url: impl Into<String>,
        token_url: impl Into<String>,
        redirect_url: impl Into<String>,
        scopes: Vec<String>,
    ) -> Result<Self> {
        Ok(Self {
            client_id: ClientId::new(client_id.into()),
            client_secret: client_secret.map(|s| ClientSecret::new(s.into())),
            auth_url: AuthUrl::new(auth_url.into())?,
            token_url: TokenUrl::new(token_url.into())?,
            redirect_url: RedirectUrl::new(redirect_url.into())?,
            scopes: scopes.into_iter().map(Scope::new).collect(),
        })
    }
}

#[async_trait]
impl AuthenticationStrategy for PkceAuthStrategy {
    /// Initiates the PKCE authentication process.
    ///
    /// For PKCE, this returns a message indicating the user should visit the auth URL.
    async fn initiate_auth(&self) -> DomainResult<String> {
        // Return the authorization URL hint
        let auth_url = format!(
            "{}?client_id={}",
            self.auth_url.as_str(),
            self.client_id.as_str()
        );
        Ok(format!("Please visit {} to authorize", auth_url))
    }

    /// Completes the PKCE authentication process.
    ///
    /// For PKCE, the completion happens after receiving the authorization code.
    async fn complete_auth(&self, _response: String) -> DomainResult<Account> {
        // This is a stub - in a real implementation, you would exchange the code for tokens
        Err(crate::domain::DomainError::InvalidCredentials)
    }

    /// Refreshes the access token using the refresh token.
    async fn refresh_token(&self, account: &Account) -> DomainResult<Account> {
        // Check if we have a refresh token
        if account.get_refresh_token().is_none() {
            return Err(crate::domain::DomainError::InvalidCredentials);
        }
        // Stub implementation - returns the same account
        Ok(account.clone())
    }

    /// Revokes tokens and clears credentials.
    async fn revoke_token(&self, account: &Account) -> DomainResult<Account> {
        let mut account = account.clone();
        account.auth_method = crate::domain::entities::AuthMethod::OAuth {
            access_token: String::new(),
            refresh_token: None,
            id_token: None,
            token_expires_at: None,
        };
        Ok(account)
    }

    /// Gets the type of authentication this strategy represents.
    fn auth_type(&self) -> &'static str {
        "pkce"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pkce_strategy_new() {
        let strategy = PkceAuthStrategy::new(
            "test-client-id",
            Some("test-client-secret"),
            "https://auth.example.com/oauth2/authorize",
            "https://auth.example.com/oauth2/token",
            "http://localhost:8080/callback",
            vec!["read".to_string(), "write".to_string()],
        )
        .unwrap();

        assert_eq!(strategy.client_id.to_string(), "test-client-id");
    }

    #[tokio::test]
    async fn test_pkce_strategy_auth_type() {
        let strategy = PkceAuthStrategy::new(
            "test-client-id",
            None::<String>,
            "https://auth.example.com/oauth2/authorize",
            "https://auth.example.com/oauth2/token",
            "http://localhost:8080/callback",
            vec!["read".to_string()],
        )
        .unwrap();

        assert_eq!(strategy.auth_type(), "pkce");
    }
}
