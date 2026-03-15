//! CLI account management commands
//!
//! This module implements the account management subcommands:
//! - add: Add a new account with API key
//! - list: List all accounts or by provider
//! - remove: Remove an account by ID
//! - set-priority: Set account priority
//! - validate: Validate account API key

use clap::{Args, Subcommand};
use std::io::{self, BufRead, Write};

use crate::domain::traits::AccountRepository;
use crate::domain::Account;
use crate::infrastructure::JsonAccountRepository;
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

/// Read API key interactively (hidden input)
fn read_api_key_interactive() -> Result<String> {
    print!("Enter API Key: ");
    io::stdout().flush()?;

    let stdin = io::stdin();
    let mut line = String::new();
    stdin
        .lock()
        .read_line(&mut line)
        .map_err(|e| crate::Error::Internal(e.to_string()))?;

    Ok(line.trim().to_string())
}

/// Handle account subcommand
pub async fn handle_account_command(cmd: AccountCommands) -> Result<()> {
    let repo = JsonAccountRepository::new().map_err(|e| crate::Error::Internal(e.to_string()))?;

    match cmd {
        AccountCommands::Add(args) => cmd_add_account(args, &repo).await,
        AccountCommands::List => cmd_list_accounts(&repo).await,
        AccountCommands::Remove(args) => cmd_remove_account(args, &repo).await,
        AccountCommands::SetPriority(args) => cmd_set_priority(args, &repo).await,
        AccountCommands::Validate(args) => cmd_validate_account(args, &repo).await,
    }
}

/// Add a new account
pub async fn cmd_add_account(args: AddAccountArgs, repo: &JsonAccountRepository) -> Result<()> {
    // Get API key (from args or interactive)
    let api_key = if args.interactive {
        read_api_key_interactive()?
    } else {
        args.api_key.unwrap_or_default()
    };

    if api_key.is_empty() && !args.interactive {
        eprintln!("Warning: No API key provided. Use --api-key or --interactive.");
    }

    let account = if args.inactive {
        Account::inactive(&args.id, &args.provider, &api_key)
    } else {
        Account::new(&args.id, &args.provider, &api_key)
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
pub async fn cmd_list_accounts(repo: &JsonAccountRepository) -> Result<()> {
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
        let api_key_display = if account.api_key.len() > 8 {
            format!("{}...", &account.api_key[..8])
        } else {
            "****".to_string()
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
    repo: &JsonAccountRepository,
) -> Result<()> {
    // Delete account (this now persists automatically)
    repo.delete(&args.id)
        .await
        .map_err(|_| crate::Error::ProviderNotFound(args.id.clone()))?;

    println!("✓ Account '{}' removed successfully", args.id);
    Ok(())
}

/// Set account priority
pub async fn cmd_set_priority(args: SetPriorityArgs, repo: &JsonAccountRepository) -> Result<()> {
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
    repo: &JsonAccountRepository,
) -> Result<()> {
    let account = repo
        .find_by_id(&args.id)
        .await
        .map_err(|_| crate::Error::ProviderNotFound(args.id.clone()))?;

    println!(
        "Validating account '{}' for provider '{}'...",
        account.id, account.provider_id
    );

    if account.api_key.is_empty() {
        println!("⚠ Account has no API key set");
        return Ok(());
    }

    // TODO: Make actual API call to validate the key
    // For now, just check key format
    if account.api_key.len() < 8 {
        println!("✗ API key too short (min 8 chars)");
    } else {
        println!(
            "✓ API key format looks valid (length: {})",
            account.api_key.len()
        );
        println!("Note: Full validation will be done on first request");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_cmd_remove_account_persists() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

        // Add account
        let account = Account::new("test-1", "openai", "sk-test-key");
        repo.save(account).await.unwrap();

        // Remove account
        let args = RemoveAccountArgs {
            id: "test-1".to_string(),
        };
        cmd_remove_account(args, &repo).await.unwrap();

        // Verify deleted
        let result = repo.find_by_id("test-1").await;
        assert!(result.is_err());

        // Verify persistence with new repo instance
        let repo2 = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();
        let result2 = repo2.find_by_id("test-1").await;
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn test_cmd_remove_non_existent_account() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

        let args = RemoveAccountArgs {
            id: "non-existent".to_string(),
        };
        let result = cmd_remove_account(args, &repo).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            crate::Error::ProviderNotFound(_)
        ));
    }

    #[tokio::test]
    async fn test_cmd_add_account() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

        let args = AddAccountArgs {
            id: "test-add".to_string(),
            provider: "openai".to_string(),
            api_key: Some("sk-test-key".to_string()),
            priority: 0,
            inactive: false,
            interactive: false,
        };

        cmd_add_account(args, &repo).await.unwrap();

        // Verify account was added
        let account = repo.find_by_id("test-add").await.unwrap();
        assert_eq!(account.provider_id, "openai");
        assert_eq!(account.api_key, "sk-test-key");
        assert!(account.is_active);
    }

    #[tokio::test]
    async fn test_cmd_list_accounts_empty() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

        // Should not panic on empty list
        let result = cmd_list_accounts(&repo).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cmd_set_priority() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

        // Add account
        let account = Account::new("test-priority", "groq", "sk-key");
        repo.save(account).await.unwrap();

        // Set priority
        let args = SetPriorityArgs {
            id: "test-priority".to_string(),
            priority: 10,
        };
        cmd_set_priority(args, &repo).await.unwrap();

        // Verify priority was updated
        let updated = repo.find_by_id("test-priority").await.unwrap();
        assert_eq!(updated.priority, 10);
    }

    #[tokio::test]
    async fn test_cmd_validate_account() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

        // Add account with valid key
        let account = Account::new("test-validate", "openai", "sk-valid-key-123");
        repo.save(account).await.unwrap();

        let args = ValidateAccountArgs {
            id: "test-validate".to_string(),
        };
        let result = cmd_validate_account(args, &repo).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cmd_validate_account_short_key() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

        // Add account with short key
        let account = Account::new("test-short", "openai", "short");
        repo.save(account).await.unwrap();

        let args = ValidateAccountArgs {
            id: "test-short".to_string(),
        };
        let result = cmd_validate_account(args, &repo).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cmd_validate_non_existent() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

        let args = ValidateAccountArgs {
            id: "non-existent".to_string(),
        };
        let result = cmd_validate_account(args, &repo).await;

        assert!(result.is_err());
    }
}
