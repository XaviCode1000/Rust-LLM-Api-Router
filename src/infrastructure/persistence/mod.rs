//! Persistence layer - Data storage implementations
//!
//! This module provides concrete implementations of the domain repository traits
//! using JSON file storage. It follows the Repository pattern to abstract
//! data persistence from the domain layer.
//!
//! # Storage
//!
//! Data is stored in the XDG config directory:
//! ```text
//! ~/.config/rust-llm-api-router/
//! ├── providers.json    # Provider configurations
//! └── accounts.json    # Account (API key) data
//! ```
//!
//! # Implementations
//!
//! - JsonAccountRepository: Account persistence
//! - JsonProviderRepository: Provider persistence
//!
//! # Example
//!
//! ```no_run
//! use rust_llm_api_router::infrastructure::persistence::{
//!     JsonAccountRepository, JsonProviderRepository,
//! };
//! use rust_llm_api_router::domain::{Account, Provider};
//!
//! let account_repo = JsonAccountRepository::new(
//!     PathBuf::from("/tmp/accounts.json")
//! ).await.unwrap();
//!
//! let provider_repo = JsonProviderRepository::new(
//!     PathBuf::from("/tmp/providers.json")
//! ).await.unwrap();
//! ```
//!
//! # Design Decisions
//!
//! - **JSON Storage**: Simple, portable, no external dependencies
//! - **File-based**: Suitable for small to medium deployments (<1000 accounts)
//! - **Sync/Async**: File operations wrapped in async functions
//! - **Error Handling**: All I/O errors converted to domain errors

pub mod json_account_repository;
pub mod json_provider_repository;

pub use json_account_repository::JsonAccountRepository;
pub use json_provider_repository::JsonProviderRepository;

#[cfg(test)]
mod json_repository_tests;
