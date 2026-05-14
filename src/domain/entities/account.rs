use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Account credentials for an LLM provider supporting multiple authentication strategies.
///
/// This entity can handle both legacy API key authentication and modern OAuth 2.0 flows
/// (including PKCE and Device Flow) with secure storage of tokens.
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop, PartialEq)]
pub struct Account {
    pub id: String,
    pub provider_id: String,
    /// Legacy API key for authentication (kept for backward compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// OAuth 2.0 access token (never serialized to disk for security)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    /// OAuth 2.0 refresh token for obtaining new access tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// OpenID Connect ID token (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
    /// Token expiration timestamp (seconds since UNIX epoch)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_expires_at: Option<u64>,
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

impl Account {
    /// Creates a new `Account` with API key authentication.
    ///
    /// This is an alias for `new_api_key` for backward compatibility.
    pub fn new(
        id: impl Into<String>,
        provider_id: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        Self::new_api_key(id, provider_id, api_key)
    }

    /// Creates a new `Account` with API key authentication.
    pub fn new_api_key(
        id: impl Into<String>,
        provider_id: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            provider_id: provider_id.into(),
            api_key: Some(api_key.into()),
            access_token: None,
            refresh_token: None,
            id_token: None,
            token_expires_at: None,
            is_active: true,
            priority: 0,
            created_at: Some(now()),
            last_used_at: None,
            auth_strategy_type: "api_key".to_string(),
        }
    }

    /// Creates a new `Account` with OAuth 2.0 authentication.
    pub fn new_oauth(
        id: impl Into<String>,
        provider_id: impl Into<String>,
        access_token: impl Into<String>,
        refresh_token: Option<impl Into<String>>,
        id_token: Option<impl Into<String>>,
        expires_in_seconds: Option<u64>,
    ) -> Self {
        let mut account = Self {
            id: id.into(),
            provider_id: provider_id.into(),
            api_key: None,
            access_token: Some(access_token.into()),
            refresh_token: refresh_token.map(|t| t.into()),
            id_token: id_token.map(|t| t.into()),
            token_expires_at: expires_in_seconds.map(|expires| now() + expires),
            is_active: true,
            priority: 0,
            created_at: Some(now()),
            last_used_at: None,
            auth_strategy_type: "oauth".to_string(),
        };

        // Set expiration time if provided
        if let Some(expires_in) = expires_in_seconds {
            account.token_expires_at = Some(now() + expires_in);
        }

        account
    }

    /// Creates an inactive account.
    pub fn inactive(
        id: impl Into<String>,
        provider_id: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            provider_id: provider_id.into(),
            api_key: Some(api_key.into()),
            access_token: None,
            refresh_token: None,
            id_token: None,
            token_expires_at: None,
            is_active: false,
            priority: 0,
            created_at: Some(now()),
            last_used_at: None,
            auth_strategy_type: "api_key".to_string(),
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
    ///
    /// Returns true if no expiration is set (for API keys) or if the token
    /// has expired or will expire within 60 seconds.
    pub fn is_token_expired(&self) -> bool {
        // API keys don't expire
        if self.api_key.is_some() && self.access_token.is_none() {
            return false;
        }

        // Check OAuth token expiration
        self.token_expires_at.is_none_or(|expires| {
            let now = now();
            // Consider token expired if it's already expired or will expire in < 60 seconds
            now >= expires.saturating_sub(60)
        })
    }

    /// Gets the current access token, preferring OAuth over API key.
    pub fn get_access_token(&self) -> Option<&str> {
        // Prefer OAuth access token if available
        if let Some(token) = &self.access_token {
            return Some(token.as_str());
        }

        // Fall back to API key
        self.api_key.as_deref()
    }

    /// Gets the authentication type for this account.
    pub fn auth_type(&self) -> &'static str {
        match self.auth_strategy_type.as_str() {
            "api_key" => "api_key",
            "pkce" => "pkce",
            "device_flow" => "device_flow",
            "oauth" => "oauth",
            _ => "unknown",
        }
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
        assert_eq!(account.api_key, Some("sk-test-key".to_string()));
        assert!(account.access_token.is_none());
        assert!(account.refresh_token.is_none());
        assert!(account.id_token.is_none());
        assert!(account.token_expires_at.is_none());
        assert!(account.is_active);
        assert_eq!(account.priority, 0);
        assert_eq!(account.auth_strategy_type, "api_key");
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
        assert!(account.api_key.is_none());
        assert_eq!(account.access_token, Some("access-token".to_string()));
        assert_eq!(account.refresh_token, Some("refresh-token".to_string()));
        assert_eq!(account.id_token, Some("id-token".to_string()));
        assert!(account.token_expires_at.is_some());
        assert!(account.is_active);
        assert_eq!(account.priority, 0);
        assert_eq!(account.auth_strategy_type, "oauth");
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
        let mut account = Account::new_api_key("acc6", "prov6", "api-key");
        account.api_key = None;
        account.access_token = None;
        assert!(account.get_access_token().is_none());
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
