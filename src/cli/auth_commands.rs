use crate::error::Result;

/// Arguments for the login command
#[derive(Debug, clap::Args)]
pub struct LoginArgs {
    /// The provider to authenticate with (e.g., openai, groq, openrouter)
    #[arg(short, long)]
    pub provider: String,
}

/// Authentication commands
#[derive(Debug, clap::Subcommand)]
pub enum AuthCommands {
    /// Initiate authentication flow
    Login(LoginArgs),

    /// Revoke tokens and clear credentials
    Logout,
}

/// Handle auth subcommand
pub async fn handle_auth_command(cmd: AuthCommands) -> Result<()> {
    match cmd {
        AuthCommands::Login(args) => handle_login_command(args.provider).await,
        AuthCommands::Logout => handle_logout_command().await,
    }
}

// Import the handler functions from the presentation layer
mod auth_handlers {
    use super::*;

    pub async fn handle_login_command(provider: String) -> Result<()> {
        crate::presentation::cli::commands::login::handle_login_command(provider).await
    }

    pub async fn handle_logout_command() -> Result<()> {
        crate::presentation::cli::commands::logout::handle_logout_command().await
    }
}

// Re-export the handler functions
pub use auth_handlers::{handle_login_command, handle_logout_command};
