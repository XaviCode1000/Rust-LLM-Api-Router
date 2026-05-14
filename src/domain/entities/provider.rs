use serde::{Deserialize, Serialize};

use crate::domain::providers::ProviderId;

/// LLM Provider configuration supporting both legacy and OAuth authentication.
///
/// This entity now includes fields for OAuth 2.0 client credentials and redirect URIs
/// to support modern authentication flows like PKCE and Device Flow.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Provider {
    pub id: ProviderId,
    pub name: String,
    pub base_url: String,
    /// Whether the provider is enabled for use
    pub enabled: bool,
    /// OAuth 2.0 client ID for authentication flows
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    /// OAuth 2.0 client secret for authentication flows (kept confidential)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    /// Authorization endpoint URL for OAuth flows
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_url: Option<String>,
    /// Token endpoint URL for OAuth flows
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_url: Option<String>,
    /// Redirect URI for OAuth 2.1 PKCE flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,
    /// Device authorization endpoint for OAuth 2.0 Device Flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_auth_url: Option<String>,
}

impl Provider {
    /// Creates a new `Provider` with basic configuration.
    pub fn new(
        id: impl Into<ProviderId>,
        name: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            base_url: base_url.into(),
            enabled: true,
            client_id: None,
            client_secret: None,
            auth_url: None,
            token_url: None,
            redirect_uri: None,
            device_auth_url: None,
        }
    }

    /// Creates a disabled provider.
    pub fn disabled(
        id: impl Into<ProviderId>,
        name: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            base_url: base_url.into(),
            enabled: false,
            client_id: None,
            client_secret: None,
            auth_url: None,
            token_url: None,
            redirect_uri: None,
            device_auth_url: None,
        }
    }

    /// Creates a new `Provider` with OAuth 2.0 configuration.
    pub fn with_oauth(
        mut self,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        auth_url: impl Into<String>,
        token_url: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) -> Self {
        self.client_id = Some(client_id.into());
        self.client_secret = Some(client_secret.into());
        self.auth_url = Some(auth_url.into());
        self.token_url = Some(token_url.into());
        self.redirect_uri = Some(redirect_uri.into());
        self
    }

    /// Sets the device authorization URL for Device Flow support.
    pub fn with_device_auth_url(mut self, device_auth_url: impl Into<String>) -> Self {
        self.device_auth_url = Some(device_auth_url.into());
        self
    }

    /// Checks if the provider is configured for OAuth 2.0 authentication.
    pub fn is_oauth_configured(&self) -> bool {
        self.client_id.is_some()
            && self.client_secret.is_some()
            && self.auth_url.is_some()
            && self.token_url.is_some()
            && self.redirect_uri.is_some()
    }

    /// Checks if the provider is configured for Device Flow.
    pub fn is_device_flow_configured(&self) -> bool {
        self.is_oauth_configured() && self.device_auth_url.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_new() {
        let provider = Provider::new("prov1", "Test Provider", "https://api.test.com");
        assert_eq!(provider.id, "prov1");
        assert_eq!(provider.name, "Test Provider");
        assert_eq!(provider.base_url, "https://api.test.com");
        assert!(provider.enabled);
        assert!(provider.client_id.is_none());
        assert!(provider.client_secret.is_none());
        assert!(provider.auth_url.is_none());
        assert!(provider.token_url.is_none());
        assert!(provider.redirect_uri.is_none());
        assert!(provider.device_auth_url.is_none());
    }

    #[test]
    fn test_provider_disabled() {
        let provider = Provider::disabled("prov2", "Disabled Provider", "https://api.test.com");
        assert_eq!(provider.id, "prov2");
        assert_eq!(provider.name, "Disabled Provider");
        assert_eq!(provider.base_url, "https://api.test.com");
        assert!(!provider.enabled);
    }

    #[test]
    fn test_provider_with_oauth() {
        let provider = Provider::new("prov3", "OAuth Provider", "https://api.oauth.com")
            .with_oauth(
                "client-id",
                "client-secret",
                "https://auth.example.com/oauth2/authorize",
                "https://auth.example.com/oauth2/token",
                "http://localhost:3000/callback",
            );

        assert_eq!(provider.client_id, Some("client-id".to_string()));
        assert_eq!(provider.client_secret, Some("client-secret".to_string()));
        assert_eq!(
            provider.auth_url,
            Some("https://auth.example.com/oauth2/authorize".to_string())
        );
        assert_eq!(
            provider.token_url,
            Some("https://auth.example.com/oauth2/token".to_string())
        );
        assert_eq!(
            provider.redirect_uri,
            Some("http://localhost:3000/callback".to_string())
        );
        assert!(provider.is_oauth_configured());
        assert!(!provider.is_device_flow_configured()); // device_auth_url not set
    }

    #[test]
    fn test_provider_with_device_flow() {
        let provider = Provider::new("prov4", "Device Flow Provider", "https://api.device.com")
            .with_oauth(
                "client-id",
                "client-secret",
                "https://auth.example.com/oauth2/authorize",
                "https://auth.example.com/oauth2/token",
                "http://localhost:3000/callback",
            )
            .with_device_auth_url("https://auth.example.com/oauth2/device");

        assert!(provider.is_oauth_configured());
        assert!(provider.is_device_flow_configured());
    }

    #[test]
    fn test_provider_is_oauth_configured_returns_false_when_missing() {
        let provider = Provider::new("prov5", "Incomplete Provider", "https://api.test.com")
            .with_oauth(
                "client-id",
                "client-secret",
                "https://auth.example.com/oauth2/authorize",
                "https://auth.example.com/oauth2/token",
                "http://localhost:3000/callback",
            );

        // Remove one required field
        let provider = Provider {
            auth_url: None,
            ..provider
        };

        assert!(!provider.is_oauth_configured());
    }
}
