//! Domain layer - Core business entities and traits
//!
//! This module contains the domain model entities, traits (ports),
//! and error types for the LLM API Router.

pub mod entities;
pub mod errors;
pub mod traits;

// Re-export entities and errors explicitly to avoid ambiguity
pub use entities::{
    Account, ChatRequest, ChatResponse, Choice, Message, Model, Provider, Usage,
    LlmRequest, LlmResponse,
};

// Re-export errors explicitly
pub use errors::{DomainError, DomainResult};

// Re-export traits but exclude DomainResult to avoid ambiguity
pub use traits::{
    LlmGateway, LlmProvider, ProviderRepository, AccountRepository,
};
