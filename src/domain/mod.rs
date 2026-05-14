//! Domain layer - Core business entities, traits, and errors
//!
//! This module contains the domain model, which is the core of the application.
//! Following Clean Architecture principles, this layer has no external dependencies
//! and represents pure business logic.
//!
//! # Architecture
//!
//! The domain layer is organized into:
//! - **Entities**: Core business objects (Account, Provider, ChatRequest, etc.)
//! - **Traits**: Port interfaces that define contracts for infrastructure
//! - **Errors**: Domain-specific error types
//! - **Services**: Domain services like authentication strategies
//!
//! # Key Entities
//!
//! - [`Account`]: Represents an API key registered for a provider
//! - [`Provider`]: Configuration for an LLM provider
//! - [`ChatRequest`]: Request to the LLM gateway
//! - [`ChatResponse`]: Response from the LLM gateway
//! - [`AccountHealth`]: Health metrics for account monitoring
//!
//! # Key Traits (Ports)
//!
//! - [`AccountRepository`]: Persistence contract for accounts
//! - [`ProviderRepository`]: Persistence contract for providers
//! - [`LlmGateway`]: Contract for LLM provider communication
//!
//! # Example
//!
//! ```no_run
//! use rust_llm_api_router::domain::{Account, Provider, AccountRepository};
//!
//! // Domain entities are pure - no infrastructure dependencies
//! let account = Account::new(
//!     "groq-1".to_string(),
//!     "groq".to_string(),
//!     "sk-xxx".to_string(),
//! );
//! ```
//!
//! # Design Principles
//!
//! 1. **No external dependencies**: Domain entities use only std/Rust types
//! 2. **Immutable where possible**: Entities use builder patterns or constructor methods
//! 3. **Explicit error types**: Domain-specific errors via [`DomainError`]
//! 4. **Trait-based interfaces**: Infrastructure depends on domain traits, not vice versa

pub mod entities;
pub mod errors;
pub mod providers;
pub mod services;
pub mod traits;

// Re-export entities and errors explicitly to avoid ambiguity
pub use entities::{
    Account, AccountHealth, AccountId, AuthMethod, ChatRequest, ChatResponse, Choice, LlmRequest,
    LlmResponse, Message, Model, ModelPricing, OpenAIChatRequest, OpenAIChatResponse, OpenAIChoice,
    OpenAIError, OpenAIErrorResponse, OpenAIMessage, OpenAIUsage, Provider, Usage,
};

// Re-export providers module
pub use providers::{known_providers, ProviderId, ProviderSelection, SelectionState};

// Re-export errors explicitly
pub use errors::DomainError;
pub use traits::DomainResult;

// Re-export services explicitly
pub use services::auth_strategy::AuthenticationStrategy;
pub use services::model_selector::{
    CostAwareSelector, ModelSelector, SelectionError, SelectionResult,
};
pub use services::query_complexity::{ClassifierConfig, QueryClassifier, QueryComplexity};

// Re-export traits but exclude DomainResult to avoid ambiguity
pub use traits::{AccountRepository, LlmGateway, LlmProvider, ProviderRepository};
