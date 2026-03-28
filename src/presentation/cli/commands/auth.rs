//! CLI authentication commands

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
        AuthCommands::Login(args) => super::login::handle_login_command(args.provider).await,
        AuthCommands::Logout => super::logout::handle_logout_command().await,
    }
}
