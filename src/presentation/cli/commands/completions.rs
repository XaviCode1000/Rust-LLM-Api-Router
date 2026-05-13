//! Shell completions command
//!
//! Generates shell completion scripts for bash, zsh, fish, and powershell.
//! Only available when the `completions` feature is enabled.

use crate::Result;
use clap::{Args, CommandFactory};
use clap_complete::Shell;

/// Completions arguments
#[derive(Debug, Args)]
pub struct CompletionsArgs {
    /// Target shell
    #[arg(value_enum)]
    pub shell: Shell,
}

/// Completions subcommands
#[derive(Debug, clap::Subcommand)]
pub enum CompletionsCommands {
    /// Generate shell completions
    Generate(CompletionsArgs),
}

/// Generate and print shell completions to stdout
pub fn handle_completions_command(cmd: CompletionsCommands) -> Result<()> {
    match cmd {
        CompletionsCommands::Generate(args) => {
            let mut cmd = crate::presentation::cli::Cli::command();
            let bin_name = cmd.get_name().to_string();
            clap_complete::generate(args.shell, &mut cmd, bin_name, &mut std::io::stdout());
            Ok(())
        }
    }
}
