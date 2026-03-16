//! Authentication module - Multiple authentication strategies
//!
//! This module provides pluggable authentication strategies for LLM providers.
//! It implements the Strategy pattern to support different authentication flows.
//!
//! # Overview
//!
//! The authentication system allows the router to support various authentication
//! methods required by different LLM providers:
//!
//! - **API Key**: Simple bearer token authentication
//! - **OAuth 2.1 PKCE**: Modern OAuth flow for public clients
//! - **OAuth 2.0 Device Flow**: For headless/input-constrained devices
//!
//! # Architecture
//!
//! Each strategy implements the [`AuthenticationStrategy`] trait from the domain layer:
//!
//! ```ignore
//! #[async_trait]
//! pub trait AuthenticationStrategy: Send + Sync {
//!     async fn initiate_auth(&self) -> DomainResult<String>;
//!     async fn complete_auth(&self, response: String) -> DomainResult<Account>;
//!     async fn refresh_token(&self, account: &Account) -> DomainResult<Account>;
//!     async fn revoke_token(&self, account: &Account) -> DomainResult<Account>;
//!     fn auth_type(&self) -> &'static str;
//! }
//! ```
//!
//! # Strategies
//!
//! ## API Key Strategy ([`ApiKeyAuthStrategy`])
//!
//! Simple API key authentication - the most common method for LLM providers.
//! The user provides their API key directly, which is used in the Authorization header.
//!
//! **Flow:**
//! 1. User provides API key via CLI
//! 2. Strategy validates the key is not empty
//! 3. Creates an Account with the API key embedded
//!
//! **Use case:** Groq, OpenRouter, Mistral, Cerebras, OpenAI, etc.
//!
//! ```rust
//! use rust_llm_api_router::infrastructure::auth::ApiKeyAuthStrategy;
//!
//! let strategy = ApiKeyAuthStrategy::new("groq");
//! let account = strategy.complete_auth("gsk_xxx".to_string()).await?;
//! ```
//!
//! ## OAuth 2.1 PKCE Strategy ([`PkceAuthStrategy`])
//!
//! Implements OAuth 2.1 Authorization Code Flow with PKCE (Proof Key for Code Exchange).
//! This is the recommended OAuth flow for public clients like CLI applications.
//!
//! **Flow:**
//! 1. Initiate auth - get authorization URL
//! 2. User visits URL and authorizes
//! 3. Complete auth with authorization code
//! 4. Exchange code for tokens (access + refresh)
//! 5. Use refresh token to get new access tokens
//!
//! **Security features:**
//! - PKCE prevents authorization code interception attacks
//! - Refresh tokens enable long-lived sessions
//! - Secure token storage with keyring
//!
//! **Use case:** Providers requiring OAuth (future providers)
//!
//! ```rust
//! use rust_llm_api_router::infrastructure::auth::PkceAuthStrategy;
//!
//! let strategy = PkceAuthStrategy::new(
//!     "client_id",
//!     None,  // No client secret for public clients
//!     "https://auth.provider.com/authorize",
//!     "https://auth.provider.com/token",
//!     "http://localhost/callback",
//!     vec!["read".to_string(), "write".to_string()],
//! )?;
//!
//! let auth_url = strategy.initiate_auth().await?;
//! ```
//!
//! ## Device Flow Strategy ([`DeviceFlowAuthStrategy`])
//!
//! Implements OAuth 2.0 Device Authorization Grant (RFC 8628) for devices
//! without a browser or with limited input capabilities.
//!
//! **Flow:**
//! 1. Device requests authorization from provider
//! 2. Provider returns device code and user code
//! 3. User visits verification URL on another device
//! 4. Device polls for authorization completion
//! 5. On success, device receives tokens
//!
//! **Use case:** CLI tools, IoT devices, embedded systems
//!
//! ```rust
//! use rust_llm_api_router::infrastructure::auth::DeviceFlowAuthStrategy;
//!
//! let strategy = DeviceFlowAuthStrategy::new(
//!     "device_client_id",
//!     None,
//!     "https://auth.provider.com/device授权",
//!     "https://auth.provider.com/token",
//!     vec!["read".to_string()],
//!     Some(5),  // 5 second polling interval
//! )?;
//! ```
//!
//! # Security Considerations
//!
//! 1. **Token Storage**: API keys and tokens are stored securely using the `keyring` crate
//! 2. **Zeroize**: Sensitive data is zeroed in memory after use
//! 3. **No Logging**: Credentials are never logged
//! 4. **Refresh Tokens**: Enable long-lived sessions without re-authentication
//! 5. **Token Revocation**: Support for revoking tokens when needed
//!
//! # Example: Adding a New Provider with OAuth
//!
//! ```rust
//! use rust_llm_api_router::domain::Account;
//! use rust_llm_api_router::infrastructure::auth::{ApiKeyAuthStrategy, PkceAuthStrategy};
//!
//! // For most providers, use API key
//! let api_key_strategy = ApiKeyAuthStrategy::new("openai");
//! let account = api_key_strategy.complete_auth("sk-xxx".to_string()).await?;
//!
//! // For OAuth-enabled providers
//! let oauth_strategy = PkceAuthStrategy::new(
//!     "my-client-id",
//!     None,
//!     "https://provider.com/oauth/authorize",
//!     "https://provider.com/oauth/token",
//!     "http://localhost:8080/callback",
//!     vec!["chat:write".to_string()],
//! )?;
//! ```
//!
//! # Error Handling
//!
//! Authentication errors are mapped to domain errors:
//!
//! - [`DomainError::InvalidCredentials`]: Invalid API key or OAuth credentials
//! - [`DomainError::AuthenticationFailed`]: Authentication flow failed
//! - [`DomainError::TokenExpired`]: Access token expired, needs refresh

pub mod api_key_strategy;
pub mod device_flow_strategy;
pub mod pkce_strategy;

pub use api_key_strategy::ApiKeyAuthStrategy;
pub use device_flow_strategy::DeviceFlowAuthStrategy;
pub use pkce_strategy::PkceAuthStrategy;
