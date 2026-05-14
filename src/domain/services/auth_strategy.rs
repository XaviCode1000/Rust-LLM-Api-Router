use crate::domain::traits::DomainResult;
use crate::domain::Account;
use async_trait::async_trait;

/// Authentication strategy trait defining the contract for different authentication methods.
///
/// This trait encapsulates different authentication mechanisms (API Key, OAuth 2.1 PKCE, Device Flow)
/// allowing the system to work with various authentication methods interchangeably.
#[async_trait]
pub trait AuthenticationStrategy: Send + Sync {
    /// Initiates the authentication process.
    ///
    /// For interactive flows (PKCE, Device Flow), this might involve displaying a URL or user code.
    /// For non-interactive flows (API Key), this might validate or request the API key.
    ///
    /// # Returns
    /// A DomainResult containing information needed to complete the authentication (like a verifier for PKCE)
    /// or an error if initiation fails.
    async fn initiate_auth(&self) -> DomainResult<String>;

    /// Completes the authentication process with the response from the provider.
    ///
    /// # Arguments
    /// * `response` - The response from the authentication provider (authorization code for PKCE,
    ///                device code response for Device Flow, or empty for API Key)
    ///
    /// # Returns
    /// A DomainResult containing the authenticated Account or an error if completion fails.
    #[must_use = "auth state changes must be persisted — do not discard the returned Account"]
    async fn complete_auth(&self, response: String) -> DomainResult<Account>;

    /// Refreshes the access token using the refresh token.
    ///
    /// # Arguments
    /// * `account` - The account containing the refresh token
    ///
    /// # Returns
    /// A DomainResult containing the updated account with new tokens or an error if refresh fails.
    #[must_use = "auth state changes must be persisted — do not discard the returned Account"]
    async fn refresh_token(&self, account: &Account) -> DomainResult<Account>;

    /// Revokes tokens and clears credentials.
    ///
    /// # Arguments
    /// * `account` - The account to revoke tokens for
    ///
    /// # Returns
    /// A DomainResult containing the account with tokens cleared (or an error if revocation fails).
    #[must_use = "auth state changes must be persisted — do not discard the returned Account"]
    async fn revoke_token(&self, account: &Account) -> DomainResult<Account>;

    /// Gets the type of authentication this strategy represents.
    ///
    /// # Returns
    /// A string slice representing the authentication type (e.g., "api_key", "pkce", "device_flow")
    fn auth_type(&self) -> &'static str;
}
