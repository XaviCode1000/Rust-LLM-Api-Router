//! Domain traits (ports)
//!
//! These traits define the abstractions (ports) that the domain uses
//! to interact with external systems. Implementations are provided
//! by the infrastructure layer.

use async_trait::async_trait;

use super::{
    Account, ChatRequest, ChatResponse, DomainError, LlmRequest, LlmResponse, Model, Provider,
};
use crate::error::Result;

/// Result type alias for domain operations.
pub type DomainResult<T> = Result<T, DomainError>;

/// Legacy trait for LLM providers (backward compatibility).
///
/// New code should use `LlmGateway` instead.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Sends a chat request to the LLM provider.
    async fn chat(&self, request: LlmRequest) -> Result<LlmResponse>;
    /// Lists available models from the provider.
    ///
    /// # Arguments
    /// * `api_key` - The API key for authentication
    ///
    /// # Returns
    /// A list of available models
    async fn list_models(&self, api_key: &str) -> Result<Vec<Model>>;
    /// Returns the provider name.
    fn name(&self) -> &str;
}

/// Gateway trait for interacting with LLM providers.
///
/// This is the primary port for sending chat requests to LLM providers.
/// Implementations handle the communication with specific providers
/// like OpenAI, Anthropic, etc.
#[async_trait]
pub trait LlmGateway: Send + Sync {
    /// Sends a chat request to the LLM provider.
    ///
    /// # Arguments
    /// * `request` - The chat request to send
    /// * `api_key` - The API key for authentication
    ///
    /// # Returns
    /// The chat response from the provider
    async fn chat(&self, request: ChatRequest, api_key: &str) -> DomainResult<ChatResponse>;

    /// Lists available models from the provider.
    ///
    /// # Arguments
    /// * `api_key` - The API key for authentication
    ///
    /// # Returns
    /// A list of available models
    async fn list_models(&self, api_key: &str) -> DomainResult<Vec<Model>>;
}

/// Repository trait for managing providers.
///
/// Defines the contract for provider persistence and retrieval.
#[async_trait]
pub trait ProviderRepository: Send + Sync {
    /// Saves a provider to the repository.
    ///
    /// # Arguments
    /// * `provider` - The provider to save
    ///
    /// # Returns
    /// The saved provider with any generated fields
    async fn save(&self, provider: Provider) -> DomainResult<Provider>;

    /// Retrieves all providers.
    ///
    /// # Returns
    /// A list of all providers
    async fn find_all(&self) -> DomainResult<Vec<Provider>>;

    /// Finds a provider by its ID.
    ///
    /// # Arguments
    /// * `id` - The provider identifier
    ///
    /// # Returns
    /// The provider if found
    async fn find_by_id(&self, id: &str) -> DomainResult<Provider>;

    /// Finds an enabled provider by its ID.
    ///
    /// # Arguments
    /// * `id` - The provider identifier
    ///
    /// # Returns
    /// The enabled provider if found
    async fn find_enabled_by_id(&self, id: &str) -> DomainResult<Provider>;

    /// Deletes a provider by its ID.
    ///
    /// # Arguments
    /// * `id` - The provider identifier
    ///
    /// # Returns
    /// Ok(()) if deleted, error if not found
    async fn delete(&self, id: &str) -> DomainResult<()>;
}

/// Repository trait for managing accounts.
///
/// Defines the contract for account persistence and retrieval.
#[async_trait]
pub trait AccountRepository: Send + Sync {
    /// Saves an account to the repository.
    ///
    /// # Arguments
    /// * `account` - The account to save
    ///
    /// # Returns
    /// The saved account with any generated fields
    async fn save(&self, account: Account) -> DomainResult<Account>;

    /// Retrieves all accounts.
    ///
    /// # Returns
    /// A list of all accounts
    async fn find_all(&self) -> DomainResult<Vec<Account>>;

    /// Finds an account by its ID.
    ///
    /// # Arguments
    /// * `id` - The account identifier
    ///
    /// # Returns
    /// The account if found
    async fn find_by_id(&self, id: &str) -> DomainResult<Account>;

    /// Finds all active accounts, ordered by priority.
    ///
    /// # Returns
    /// A list of active accounts sorted by priority
    async fn find_active(&self) -> DomainResult<Vec<Account>>;

    /// Finds active accounts for a specific provider.
    ///
    /// # Arguments
    /// * `provider_id` - The provider identifier
    ///
    /// # Returns
    /// A list of active accounts for the provider
    async fn find_active_by_provider(&self, provider_id: &str) -> DomainResult<Vec<Account>>;

    /// Deletes an account by its ID.
    ///
    /// # Arguments
    /// * `id` - The account identifier
    ///
    /// # Returns
    /// Ok(()) if deleted, error if not found
    async fn delete(&self, id: &str) -> DomainResult<()>;
}
