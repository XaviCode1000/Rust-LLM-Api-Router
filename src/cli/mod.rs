//! CLI command definitions
//!
//! This module defines the command-line interface for the LLM API Router.
//!
//! It uses the `clap` crate to define subcommands and flags.

pub mod account_commands;
pub mod auth_commands;
pub mod provider_commands;

use account_commands::AccountCommands;
use auth_commands::AuthCommands;
use clap::Parser;
use provider_commands::ProviderCommands;

/// CLI command enum
#[derive(Debug, Parser)]
#[command(name = "llm-router")]
#[command(about = "LLM API Router - Proxy for LLM providers")]
pub struct Cli {
    /// Host to bind to (server mode)
    #[arg(long, default_value = "0.0.0.0")]
    pub host: String,

    /// Port to bind to (server mode)
    #[arg(short, long, default_value = "8080")]
    pub port: u16,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    pub log_level: String,

    /// CLI subcommands
    #[command(subcommand)]
    pub commands: Option<CliCommands>,
}

#[derive(Debug, clap::Subcommand)]
pub enum CliCommands {
    /// Provider management commands
    #[command(subcommand)]
    Provider(ProviderCommands),

    /// Account management commands
    #[command(subcommand)]
    Account(AccountCommands),

    /// Authentication commands
    #[command(subcommand)]
    Auth(AuthCommands),
}

/// Handle CLI commands
pub async fn handle_command(command: CliCommands) -> crate::error::Result<()> {
    match command {
        CliCommands::Provider(provider_cmd) => {
            provider_commands::handle_provider_command(provider_cmd).await
        }
        CliCommands::Account(account_cmd) => {
            account_commands::handle_account_command(account_cmd).await
        }
        CliCommands::Auth(auth_cmd) => {
            auth_commands::handle_auth_command(auth_cmd).await
        }
    }
}
