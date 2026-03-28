//! CLI account management commands
//!
//! This module implements the account management subcommands:
//! - add: Add a new account with API key
//! - list: List all accounts or by provider
//! - remove: Remove an account by ID
//! - set-priority: Set account priority
//! - validate: Validate account API key

use clap::{Args, Subcommand};

use crate::domain::traits::AccountRepository;
use crate::domain::Account;
use crate::presentation::cli::input::read_api_key_interactive;
use crate::Result;

/// Add account arguments
#[derive(Debug, Args)]
pub struct AddAccountArgs {
    /// Account unique identifier
    #[arg(long)]
    pub id: String,

    /// Provider ID this account belongs to
    #[arg(long)]
    pub provider: String,

    /// API key for authentication (or use --interactive)
    #[arg(long)]
    pub api_key: Option<String>,

    /// Account priority (lower = higher priority)
    #[arg(long, default_value = "0")]
    pub priority: i32,

    /// Start inactive
    #[arg(long)]
    pub inactive: bool,

    /// Interactive mode (prompt for API key)
    #[arg(long, short)]
    pub interactive: bool,
}

/// Remove account arguments
#[derive(Debug, Args)]
pub struct RemoveAccountArgs {
    /// Account ID to remove
    #[arg(short, long)]
    pub id: String,
}

/// Set priority arguments
#[derive(Debug, Args)]
pub struct SetPriorityArgs {
    /// Account ID
    #[arg(short, long)]
    pub id: String,

    /// New priority value
    #[arg(short, long)]
    pub priority: i32,
}

/// Validate account arguments
#[derive(Debug, Args)]
pub struct ValidateAccountArgs {
    /// Account ID to validate
    #[arg(short, long)]
    pub id: String,
}

/// Account management subcommands
#[derive(Debug, Subcommand)]
pub enum AccountCommands {
    /// Add a new account
    Add(AddAccountArgs),

    /// List all accounts
    List,

    /// Remove an account by ID
    Remove(RemoveAccountArgs),

    /// Set account priority
    SetPriority(SetPriorityArgs),

    /// Validate account API key
    Validate(ValidateAccountArgs),
}

/// Handle account subcommand
pub async fn handle_account_command(
    cmd: AccountCommands,
    repo: &impl AccountRepository,
) -> Result<()> {
    match cmd {
        AccountCommands::Add(args) => cmd_add_account(args, repo).await,
        AccountCommands::List => cmd_list_accounts(repo).await,
        AccountCommands::Remove(args) => cmd_remove_account(args, repo).await,
        AccountCommands::SetPriority(args) => cmd_set_priority(args, repo).await,
        AccountCommands::Validate(args) => cmd_validate_account(args, repo).await,
    }
}

/// Add a new account
pub async fn cmd_add_account(args: AddAccountArgs, repo: &impl AccountRepository) -> Result<()> {
    // Get API key (from args or interactive)
    let api_key: Option<String> = if args.interactive {
        Some(read_api_key_interactive()?)
    } else {
        args.api_key
    };

    if api_key.is_none() && !args.interactive {
        eprintln!("Warning: No API key provided. Use --api-key or --interactive.");
    }

    let account = if args.inactive {
        Account::inactive(&args.id, &args.provider, api_key.unwrap_or_default())
    } else {
        Account::new_api_key(&args.id, &args.provider, api_key.unwrap_or_default())
    }
    .with_priority(args.priority);

    repo.save(account)
        .await
        .map_err(|e| crate::Error::Internal(e.to_string()))?;

    println!(
        "✓ Account '{}' added for provider '{}'",
        args.id, args.provider
    );
    Ok(())
}

/// List all accounts
pub async fn cmd_list_accounts(repo: &impl AccountRepository) -> Result<()> {
    let accounts = repo
        .find_all()
        .await
        .map_err(|e| crate::Error::Internal(e.to_string()))?;

    if accounts.is_empty() {
        println!("No accounts registered.");
        return Ok(());
    }

    println!(
        "{:<20} {:<20} {:<10} {:<8} API Key",
        "ID", "Provider", "Priority", "Status"
    );
    println!("{:-<90}", "");

    for account in accounts {
        let status = if account.is_active {
            "✓ Active"
        } else {
            "✗ Inactive"
        };
        let api_key_display = if let Some(ref key) = account.api_key {
            if key.len() > 8 {
                format!("{}...", &key[..8])
            } else {
                "****".to_string()
            }
        } else {
            "None".to_string()
        };
        println!(
            "{:<20} {:<20} {:<10} {:<8} {}",
            account.id, account.provider_id, account.priority, status, api_key_display
        );
    }

    Ok(())
}

/// Remove an account
pub async fn cmd_remove_account(
    args: RemoveAccountArgs,
    repo: &impl AccountRepository,
) -> Result<()> {
    // Delete account (this now persists automatically)
    repo.delete(&args.id)
        .await
        .map_err(|_| crate::Error::ProviderNotFound(args.id.clone()))?;

    println!("✓ Account '{}' removed successfully", args.id);
    Ok(())
}

/// Set account priority
pub async fn cmd_set_priority(args: SetPriorityArgs, repo: &impl AccountRepository) -> Result<()> {
    let mut account = repo
        .find_by_id(&args.id)
        .await
        .map_err(|_| crate::Error::ProviderNotFound(args.id.clone()))?;

    account.priority = args.priority;

    repo.save(account)
        .await
        .map_err(|e| crate::Error::Internal(e.to_string()))?;

    println!("✓ Account '{}' priority set to {}", args.id, args.priority);
    Ok(())
}

/// Validate account API key
pub async fn cmd_validate_account(
    args: ValidateAccountArgs,
    repo: &impl AccountRepository,
) -> Result<()> {
    let account = repo
        .find_by_id(&args.id)
        .await
        .map_err(|_| crate::Error::ProviderNotFound(args.id.clone()))?;

    println!(
        "Validating account '{}' for provider '{}'...",
        account.id, account.provider_id
    );

    let api_key = match &account.api_key {
        Some(key) => key,
        None => {
            println!("⚠ Account has no API key set");
            return Ok(());
        }
    };

    // TODO: Make actual API call to validate the key
    // For now, just check key format
    if api_key.len() < 8 {
        println!("✗ API key too short (min 8 chars)");
    } else {
        println!("✓ API key format looks valid (length: {})", api_key.len());
        println!("Note: Full validation will be done on first request");
    }

    Ok(())
}
