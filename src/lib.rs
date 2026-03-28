//! LLM API Router - Clean Architecture implementation
//!
//! A proxy/router for LLM API requests with support for multiple providers.
//!
//! # Architecture
//!
//! This crate follows Clean Architecture principles:
//!
//! - **Domain** - Core business entities and traits
//! - **Application** - Use cases and business logic
//! - **Infrastructure** - External integrations and persistence
//! - **Presentation** - HTTP API and CLI interfaces

pub mod app;
pub mod config;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod interfaces;
pub mod presentation;

pub use error::{Error, Result};

/// Backward-compatible re-exports from the old `cli` module.
///
/// All CLI types now live under `presentation::cli`. These re-exports
/// preserve the old import paths (`crate::cli::*`) so existing tests
/// and external consumers continue to compile without changes.
pub mod cli {
    pub use crate::presentation::cli::Cli;
    pub use crate::presentation::cli::CliCommands;

    pub mod provider_commands {
        pub use crate::presentation::cli::commands::provider::*;
    }

    pub mod account_commands {
        pub use crate::presentation::cli::commands::account::*;
    }

    pub mod auth_commands {
        pub use crate::presentation::cli::commands::auth::*;
    }

    pub async fn handle_command(cmd: CliCommands) -> crate::error::Result<()> {
        crate::presentation::cli::handle_command(cmd).await
    }
}
