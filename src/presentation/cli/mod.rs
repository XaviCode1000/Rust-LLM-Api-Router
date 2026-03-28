pub mod commands;
pub mod input;

use clap::Parser;

pub use commands::account::AccountCommands;
pub use commands::auth::AuthCommands;
pub use commands::provider::ProviderCommands;

use crate::infrastructure::{JsonAccountRepository, JsonProviderRepository};

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

    /// Generate shell completions
    #[cfg(feature = "completions")]
    #[command(subcommand)]
    Completions(commands::completions::CompletionsCommands),
}

/// Handle CLI commands
pub async fn handle_command(command: CliCommands) -> crate::error::Result<()> {
    match command {
        CliCommands::Provider(provider_cmd) => {
            let repo =
                JsonProviderRepository::new().map_err(|e| crate::Error::Internal(e.to_string()))?;
            let account_repo =
                JsonAccountRepository::new().map_err(|e| crate::Error::Internal(e.to_string()))?;
            commands::provider::handle_provider_command(provider_cmd, &repo, &account_repo).await
        }
        CliCommands::Account(account_cmd) => {
            let repo =
                JsonAccountRepository::new().map_err(|e| crate::Error::Internal(e.to_string()))?;
            commands::account::handle_account_command(account_cmd, &repo).await
        }
        CliCommands::Auth(auth_cmd) => commands::auth::handle_auth_command(auth_cmd).await,
        #[cfg(feature = "completions")]
        CliCommands::Completions(cmd) => commands::completions::handle_completions_command(cmd),
    }
}
