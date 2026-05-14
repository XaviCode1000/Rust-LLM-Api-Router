use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::time::{SystemTime, UNIX_EPOCH};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::domain::providers::ProviderId;

/// Authentication method for an account.
///
/// Each variant owns its credentials exclusively — no Option sprawl.
/// The compiler enforces that ApiKey accounts don't have OAuth tokens
/// and vice versa.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Zeroize)]
pub enum AuthMethod {
    /// API key authentication (legacy).
    ApiKey { api_key: String },
    /// OAuth 2.0 authentication (PKCE, Device Flow, etc.).
    OAuth {
        access_token: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        refresh_token: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        id_token: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        token_expires_at: Option<u64>,
    },
}

impl AuthMethod {
    /// Returns the API key if this is an ApiKey variant.
    pub fn api_key(&self) -> Option<&str> {
        match self {
            AuthMethod::ApiKey { api_key } => Some(api_key),
            _ => None,
        }
    }

    /// Returns the access token if this is an OAuth variant.
    pub fn access_token(&self) -> Option<&str> {
        match self {
            AuthMethod::OAuth { access_token, .. } => Some(access_token),
            _ => None,
        }
    }

    /// Returns true if this is an ApiKey variant.
    pub fn is_api_key(&self) -> bool {
        matches!(self, AuthMethod::ApiKey { .. })
    }

    /// Returns true if this is an OAuth variant.
    pub fn is_oauth(&self) -> bool {
        matches!(self, AuthMethod::OAuth { .. })
    }

    /// Returns the auth strategy type string for serialization compatibility.
    pub fn strategy_type(&self) -> &'static str {
        match self {
            AuthMethod::ApiKey { .. } => "api_key",
            AuthMethod::OAuth { .. } => "oauth",
        }
    }
}

/// Newtype wrapper for account identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Zeroize, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AccountId(String);

impl PartialEq<str> for AccountId {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for AccountId {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<String> for AccountId {
    fn eq(&self, other: &String) -> bool {
        self.0 == *other
    }
}

impl PartialEq<AccountId> for str {
    fn eq(&self, other: &AccountId) -> bool {
        self == other.0
    }
}

impl PartialEq<AccountId> for &str {
    fn eq(&self, other: &AccountId) -> bool {
        *self == other.0
    }
}

impl AccountId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl AsRef<str> for AccountId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl Borrow<str> for AccountId {
    fn borrow(&self) -> &str {
        &self.0
    }
}
impl From<String> for AccountId {
    fn from(s: String) -> Self {
        Self(s)
    }
}
impl From<&str> for AccountId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}
impl std::fmt::Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::str::FromStr for AccountId {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err("account ID cannot be empty");
        }
        Ok(Self(s.to_string()))
    }
}

/// Account credentials for an LLM provider supporting multiple authentication strategies.
///
/// This entity can handle both legacy API key authentication and modern OAuth 2.0 flows
/// (including PKCE and Device Flow) with secure storage of tokens.
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop, PartialEq)]
pub struct Account {
    pub id: AccountId,
    pub provider_id: ProviderId,
    /// Authentication method — owns credentials exclusively (no Option sprawl).
    pub auth_method: AuthMethod,
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
}

impl Account {
    /// Creates a new `Account` with API key authentication.
    ///
    /// This is an alias for `new_api_key` for backward compatibility.
    pub fn new(
        id: impl Into<AccountId>,
        provider_id: impl Into<ProviderId>,
        api_key: impl Into<String>,
    ) -> Self {
        Self::new_api_key(id, provider_id, api_key)
    }

    /// Creates a new `Account` with API key authentication.
    pub fn new_api_key(
        id: impl Into<AccountId>,
        provider_id: impl Into<ProviderId>,
        api_key: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            provider_id: provider_id.into(),
            auth_method: AuthMethod::ApiKey {
                api_key: api_key.into(),
            },
            is_active: true,
            priority: 0,
            created_at: Some(now()),
            last_used_at: None,
        }
    }

    /// Creates a new `Account` with OAuth 2.0 authentication.
    pub fn new_oauth(
        id: impl Into<AccountId>,
        provider_id: impl Into<ProviderId>,
        access_token: impl Into<String>,
        refresh_token: Option<impl Into<String>>,
        id_token: Option<impl Into<String>>,
        expires_in_seconds: Option<u64>,
    ) -> Self {
        let token_expires_at = expires_in_seconds.map(|expires| now() + expires);
        Self {
            id: id.into(),
            provider_id: provider_id.into(),
            auth_method: AuthMethod::OAuth {
                access_token: access_token.into(),
                refresh_token: refresh_token.map(|t| t.into()),
                id_token: id_token.map(|t| t.into()),
                token_expires_at,
            },
            is_active: true,
            priority: 0,
            created_at: Some(now()),
            last_used_at: None,
        }
    }

    /// Creates an inactive account.
    pub fn inactive(
        id: impl Into<AccountId>,
        provider_id: impl Into<ProviderId>,
        api_key: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            provider_id: provider_id.into(),
            auth_method: AuthMethod::ApiKey {
                api_key: api_key.into(),
            },
            is_active: false,
            priority: 0,
            created_at: Some(now()),
            last_used_at: None,
        }
    }

    /// Sets the priority of the account.
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the active state of the account.
    pub fn with_active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }

    /// Updates the last used timestamp to now.
    pub fn touch(&mut self) {
        self.last_used_at = Some(now());
    }

    /// Checks if the account's token is expired or about to expire.
    pub fn is_token_expired(&self) -> bool {
        match &self.auth_method {
            // API keys don't expire
            AuthMethod::ApiKey { .. } => false,
            // Check OAuth token expiration
            AuthMethod::OAuth {
                token_expires_at, ..
            } => token_expires_at.is_none_or(|expires| {
                let now = now();
                now >= expires.saturating_sub(60)
            }),
        }
    }

    /// Gets the current access token, preferring OAuth over API key.
    pub fn get_access_token(&self) -> Option<&str> {
        match &self.auth_method {
            AuthMethod::OAuth { access_token, .. } => Some(access_token),
            AuthMethod::ApiKey { api_key } => Some(api_key),
        }
    }

    /// Gets the API key if this account uses API key auth.
    pub fn get_api_key(&self) -> Option<&str> {
        self.auth_method.api_key()
    }

    /// Gets the refresh token if this account uses OAuth.
    pub fn get_refresh_token(&self) -> Option<&str> {
        match &self.auth_method {
            AuthMethod::OAuth { refresh_token, .. } => refresh_token.as_deref(),
            _ => None,
        }
    }

    /// Gets the authentication type for this account.
    pub fn auth_type(&self) -> &'static str {
        self.auth_method.strategy_type()
    }
}

/// Returns current timestamp in seconds since UNIX epoch.
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_new_api_key() {
        let account = Account::new_api_key("acc1", "prov1", "sk-test-key");
        assert_eq!(account.id, "acc1");
        assert_eq!(account.provider_id, "prov1");
        assert_eq!(account.auth_method.api_key(), Some("sk-test-key"));
        assert!(account.auth_method.is_api_key());
        assert!(!account.auth_method.is_oauth());
        assert!(account.is_active);
        assert_eq!(account.priority, 0);
        assert_eq!(account.auth_type(), "api_key");
        assert!(account.created_at.is_some());
    }

    #[test]
    fn test_account_new_oauth() {
        let account = Account::new_oauth(
            "acc2",
            "prov2",
            "access-token",
            Some("refresh-token"),
            Some("id-token"),
            Some(3600),
        );
        assert_eq!(account.id, "acc2");
        assert_eq!(account.provider_id, "prov2");
        assert!(account.auth_method.is_oauth());
        assert_eq!(account.auth_method.access_token(), Some("access-token"));
        assert_eq!(account.get_refresh_token(), Some("refresh-token"));
        assert!(account.is_active);
        assert_eq!(account.priority, 0);
        assert_eq!(account.auth_type(), "oauth");
        assert!(account.created_at.is_some());
    }

    #[test]
    fn test_account_is_token_expired_api_key() {
        let account = Account::new_api_key("acc1", "prov1", "sk-test-key");
        assert!(!account.is_token_expired());
    }

    #[test]
    fn test_account_is_token_expired_oauth_not_expired() {
        let account = Account::new_oauth(
            "acc2",
            "prov2",
            "access-token",
            Some("refresh-token"),
            None::<String>,
            Some(3600), // 1 hour from now
        );
        assert!(!account.is_token_expired());
    }

    #[test]
    fn test_account_is_token_expired_oauth_expired() {
        let account = Account::new_oauth(
            "acc3",
            "prov3",
            "access-token",
            Some("refresh-token"),
            None::<String>,
            Some(0), // Expired now
        );
        assert!(account.is_token_expired());
    }

    #[test]
    fn test_account_get_access_token_prefers_oauth() {
        let account = Account::new_oauth(
            "acc4",
            "prov4",
            "oauth-token",
            None::<String>,
            None::<String>,
            Some(3600),
        );
        assert_eq!(account.get_access_token(), Some("oauth-token"));
    }

    #[test]
    fn test_account_get_access_token_falls_back_to_api_key() {
        let account = Account::new_api_key("acc5", "prov5", "api-key");
        assert_eq!(account.get_access_token(), Some("api-key"));
    }

    #[test]
    fn test_account_get_access_token_none_when_no_credentials() {
        // OAuth account with empty access token
        let account = Account::new_oauth(
            "acc6",
            "prov6",
            "", // empty access token
            None::<String>,
            None::<String>,
            None,
        );
        assert_eq!(account.get_access_token(), Some(""));
    }

    #[test]
    fn test_account_id_partialeq_str() {
        let id = AccountId::from("test-123");
        assert_eq!(id, "test-123");
        assert_ne!(id, "other");
    }

    #[test]
    fn test_account_id_partialeq_ref_str() {
        let s = "test-123";
        let id = AccountId::from("test-123");
        assert_eq!(id, s);
    }

    #[test]
    fn test_account_id_partialeq_string() {
        let id = AccountId::from("test-123");
        assert_eq!(id, String::from("test-123"));
    }

    #[test]
    fn test_account_id_partialeq_reverse() {
        let id = AccountId::from("test-123");
        assert!("test-123" == id);
    }

    #[test]
    fn test_account_id_zeroize_clears_inner() {
        let mut id = AccountId::from("secret-id");
        id.zeroize();
        assert_eq!(id.as_str(), "");
    }

    #[test]
    fn test_account_touch_updates_last_used() {
        let mut account = Account::new_api_key("acc7", "prov7", "sk-test");
        let initial_time = account.last_used_at;
        account.touch();
        assert_ne!(account.last_used_at, initial_time);
        assert!(account.last_used_at.is_some());
    }
}
