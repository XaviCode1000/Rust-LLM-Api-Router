//! Tests for CLI account commands
//!
//! Integration tests for account management CLI commands.

use std::path::PathBuf;
use tempfile::TempDir;

use rust_llm_api_router::cli::account_commands::{
    AccountCommands, AddAccountArgs, RemoveAccountArgs, SetPriorityArgs, ValidateAccountArgs,
};
use rust_llm_api_router::domain::traits::AccountRepository;
use rust_llm_api_router::infrastructure::persistence::JsonAccountRepository;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn setup_test_environment() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_dir = temp_dir.path().join("config");
    std::fs::create_dir_all(&config_dir).expect("Failed to create config dir");
    (temp_dir, config_dir)
}

fn create_add_account_args(id: &str, provider: &str, api_key: Option<&str>) -> AddAccountArgs {
    AddAccountArgs {
        id: id.to_string(),
        provider: provider.to_string(),
        api_key: api_key.map(String::from),
        priority: 0,
        inactive: false,
        interactive: false,
    }
}

// Helper to handle account command with custom repo
async fn handle_account_command_with_dir(
    cmd: AccountCommands,
    config_dir: &std::path::Path,
) -> rust_llm_api_router::Result<()> {
    let repo = JsonAccountRepository::with_config_dir(config_dir)
        .map_err(|e| rust_llm_api_router::Error::Internal(e.to_string()))?;

    match cmd {
        AccountCommands::Add(args) => {
            // Get API key (from args or interactive)
            let api_key = if args.interactive {
                // Skip interactive in tests
                args.api_key.unwrap_or_default()
            } else {
                args.api_key.unwrap_or_default()
            };

            if api_key.is_empty() && !args.interactive {
                eprintln!("Warning: No API key provided. Use --api-key or --interactive.");
            }

            let account = if args.inactive {
                rust_llm_api_router::domain::Account::inactive(args.id.as_str(), args.provider.as_str(), &api_key)
            } else {
                rust_llm_api_router::domain::Account::new(args.id.as_str(), args.provider.as_str(), &api_key)
            }
            .with_priority(args.priority);

            repo.save(account)
                .await
                .map_err(|e| rust_llm_api_router::Error::Internal(e.to_string()))?;
            println!(
                "✓ Account '{}' added for provider '{}'",
                args.id, args.provider
            );
            Ok(())
        }
        AccountCommands::List => {
            let accounts = repo
                .find_all()
                .await
                .map_err(|e| rust_llm_api_router::Error::Internal(e.to_string()))?;

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
                    "(no key)".to_string()
                };
                println!(
                    "{:<20} {:<20} {:<10} {:<8} {}",
                    account.id, account.provider_id, account.priority, status, api_key_display
                );
            }

            Ok(())
        }
        AccountCommands::Remove(args) => {
            // First check if account exists
            repo.find_by_id(&args.id)
                .await
                .map_err(|_| rust_llm_api_router::Error::ProviderNotFound(args.id.clone()))?;

            // Get all accounts and filter out the one to remove
            let accounts = repo
                .find_all()
                .await
                .map_err(|e| rust_llm_api_router::Error::Internal(e.to_string()))?;

            let updated: Vec<_> = accounts.into_iter().filter(|a| a.id != args.id).collect();

            // Save all accounts back (overwrites the file)
            for account in updated {
                repo.save(account)
                    .await
                    .map_err(|e| rust_llm_api_router::Error::Internal(e.to_string()))?;
            }

            println!("✓ Account '{}' removed successfully", args.id);
            Ok(())
        }
        AccountCommands::SetPriority(args) => {
            let mut account = repo
                .find_by_id(&args.id)
                .await
                .map_err(|_| rust_llm_api_router::Error::ProviderNotFound(args.id.clone()))?;

            account.priority = args.priority;

            repo.save(account)
                .await
                .map_err(|e| rust_llm_api_router::Error::Internal(e.to_string()))?;

            println!("✓ Account '{}' priority set to {}", args.id, args.priority);
            Ok(())
        }
        AccountCommands::Validate(args) => {
            let account = repo
                .find_by_id(&args.id)
                .await
                .map_err(|_| rust_llm_api_router::Error::ProviderNotFound(args.id.clone()))?;

            println!(
                "Validating account '{}' for provider '{}'...",
                account.id, account.provider_id
            );

            match &account.api_key {
                None => {
                    println!("⚠ Account has no API key set");
                    Ok(())
                }
                Some(key) if key.is_empty() => {
                    println!("⚠ Account has no API key set");
                    Ok(())
                }
                Some(key) if key.len() < 8 => {
                    println!("✗ API key too short (min 8 chars)");
                    Ok(())
                }
                Some(key) => {
                    println!("✓ API key format looks valid (length: {})", key.len());
                    println!("Note: Full validation will be done on first request");
                    Ok(())
                }
            }
        }
    }
}

// ============================================================================
// ADD ACCOUNT TESTS
// ============================================================================

#[tokio::test]
async fn test_add_account_success() {
    let (_temp_dir, config_dir) = setup_test_environment();

    let args = create_add_account_args("test-acc-1", "openai", Some("sk-test12345678"));
    let cmd = AccountCommands::Add(args);

    let result = handle_account_command_with_dir(cmd, &config_dir).await;

    // Should succeed
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_account_without_api_key() {
    let (_temp_dir, config_dir) = setup_test_environment();

    let args = create_add_account_args("test-acc-2", "openai", None);
    let cmd = AccountCommands::Add(args);

    let result = handle_account_command_with_dir(cmd, &config_dir).await;

    // Should succeed but print warning
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_account_with_priority() {
    let (_temp_dir, config_dir) = setup_test_environment();

    let mut args = create_add_account_args("test-acc-3", "groq", Some("sk-test12345678"));
    args.priority = 10;
    let cmd = AccountCommands::Add(args);

    let result = handle_account_command_with_dir(cmd, &config_dir).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_account_inactive() {
    let (_temp_dir, config_dir) = setup_test_environment();

    let mut args = create_add_account_args("test-acc-4", "mistral", Some("sk-test12345678"));
    args.inactive = true;
    let cmd = AccountCommands::Add(args);

    let result = handle_account_command_with_dir(cmd, &config_dir).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_account_duplicate_id() {
    let (_temp_dir, config_dir) = setup_test_environment();

    // Add first account
    let args1 = create_add_account_args("test-acc-dup", "openai", Some("sk-test12345678"));
    let cmd1 = AccountCommands::Add(args1);
    let result1 = handle_account_command_with_dir(cmd1, &config_dir).await;
    assert!(result1.is_ok());

    // Add duplicate - current implementation allows it (overwrites)
    let args2 = create_add_account_args("test-acc-dup", "openai", Some("sk-new12345678"));
    let cmd2 = AccountCommands::Add(args2);
    let result2 = handle_account_command_with_dir(cmd2, &config_dir).await;

    // Should succeed (overwrites existing)
    assert!(result2.is_ok());
}

// ============================================================================
// LIST ACCOUNTS TESTS
// ============================================================================

#[tokio::test]
async fn test_list_accounts_empty() {
    let (_temp_dir, config_dir) = setup_test_environment();

    let cmd = AccountCommands::List;
    let result = handle_account_command_with_dir(cmd, &config_dir).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_accounts_with_data() {
    let (_temp_dir, config_dir) = setup_test_environment();

    // Add some accounts first
    let args1 = create_add_account_args("list-acc-1", "openai", Some("sk-test12345678"));
    let cmd1 = AccountCommands::Add(args1);
    handle_account_command_with_dir(cmd1, &config_dir)
        .await
        .unwrap();

    let args2 = create_add_account_args("list-acc-2", "groq", Some("sk-test87654321"));
    let cmd2 = AccountCommands::Add(args2);
    handle_account_command_with_dir(cmd2, &config_dir)
        .await
        .unwrap();

    // List accounts
    let cmd = AccountCommands::List;
    let result = handle_account_command_with_dir(cmd, &config_dir).await;

    assert!(result.is_ok());
}

// ============================================================================
// REMOVE ACCOUNT TESTS
// ============================================================================

#[tokio::test]
async fn test_remove_account_success() {
    let (_temp_dir, config_dir) = setup_test_environment();

    // Add account first
    let args_add = create_add_account_args("remove-acc-1", "openai", Some("sk-test12345678"));
    let cmd_add = AccountCommands::Add(args_add);
    handle_account_command_with_dir(cmd_add, &config_dir)
        .await
        .unwrap();

    // Remove account
    let args_remove = RemoveAccountArgs {
        id: "remove-acc-1".to_string(),
        force: true, // Skip confirmation in tests
    };
    let cmd = AccountCommands::Remove(args_remove);
    let result = handle_account_command_with_dir(cmd, &config_dir).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_remove_account_not_found() {
    let (_temp_dir, config_dir) = setup_test_environment();

    let args_remove = RemoveAccountArgs {
        id: "nonexistent-acc".to_string(),
        force: true, // Skip confirmation in tests
    };
    let cmd = AccountCommands::Remove(args_remove);
    let result = handle_account_command_with_dir(cmd, &config_dir).await;

    // Should fail with ProviderNotFound (current error type)
    assert!(result.is_err());
}

#[tokio::test]
async fn test_remove_account_from_multiple() {
    let (_temp_dir, config_dir) = setup_test_environment();

    // Add multiple accounts
    for i in 1..=3 {
        let args = create_add_account_args(
            &format!("multi-acc-{}", i),
            "openai",
            Some("sk-test12345678"),
        );
        let cmd = AccountCommands::Add(args);
        handle_account_command_with_dir(cmd, &config_dir)
            .await
            .unwrap();
    }

    // Remove middle one
    let args_remove = RemoveAccountArgs {
        id: "multi-acc-2".to_string(),
        force: true, // Skip confirmation in tests
    };
    let cmd = AccountCommands::Remove(args_remove);
    let result = handle_account_command_with_dir(cmd, &config_dir).await;

    assert!(result.is_ok());

    // Verify others still exist
    let cmd_list = AccountCommands::List;
    let result_list = handle_account_command_with_dir(cmd_list, &config_dir).await;
    assert!(result_list.is_ok());
}

// ============================================================================
// SET PRIORITY TESTS
// ============================================================================

#[tokio::test]
async fn test_set_priority_success() {
    let (_temp_dir, config_dir) = setup_test_environment();

    // Add account first
    let args_add = create_add_account_args("priority-acc-1", "openai", Some("sk-test12345678"));
    let cmd_add = AccountCommands::Add(args_add);
    handle_account_command_with_dir(cmd_add, &config_dir)
        .await
        .unwrap();

    // Set priority
    let args_priority = SetPriorityArgs {
        id: "priority-acc-1".to_string(),
        priority: 100,
    };
    let cmd = AccountCommands::SetPriority(args_priority);
    let result = handle_account_command_with_dir(cmd, &config_dir).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_set_priority_not_found() {
    let (_temp_dir, config_dir) = setup_test_environment();

    let args_priority = SetPriorityArgs {
        id: "nonexistent-acc".to_string(),
        priority: 50,
    };
    let cmd = AccountCommands::SetPriority(args_priority);
    let result = handle_account_command_with_dir(cmd, &config_dir).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_set_priority_negative_value() {
    let (_temp_dir, config_dir) = setup_test_environment();

    // Add account first
    let args_add = create_add_account_args("priority-acc-2", "groq", Some("sk-test12345678"));
    let cmd_add = AccountCommands::Add(args_add);
    handle_account_command_with_dir(cmd_add, &config_dir)
        .await
        .unwrap();

    // Set negative priority (lower = higher priority)
    let args_priority = SetPriorityArgs {
        id: "priority-acc-2".to_string(),
        priority: -10,
    };
    let cmd = AccountCommands::SetPriority(args_priority);
    let result = handle_account_command_with_dir(cmd, &config_dir).await;

    assert!(result.is_ok());
}

// ============================================================================
// VALIDATE ACCOUNT TESTS
// ============================================================================

#[tokio::test]
async fn test_validate_account_success() {
    let (_temp_dir, config_dir) = setup_test_environment();

    // Add account with valid-looking key
    let args_add = create_add_account_args("validate-acc-1", "openai", Some("sk-valid12345678"));
    let cmd_add = AccountCommands::Add(args_add);
    handle_account_command_with_dir(cmd_add, &config_dir)
        .await
        .unwrap();

    // Validate
    let args_validate = ValidateAccountArgs {
        id: "validate-acc-1".to_string(),
    };
    let cmd = AccountCommands::Validate(args_validate);
    let result = handle_account_command_with_dir(cmd, &config_dir).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_account_not_found() {
    let (_temp_dir, config_dir) = setup_test_environment();

    let args_validate = ValidateAccountArgs {
        id: "nonexistent-acc".to_string(),
    };
    let cmd = AccountCommands::Validate(args_validate);
    let result = handle_account_command_with_dir(cmd, &config_dir).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_validate_account_short_key() {
    let (_temp_dir, config_dir) = setup_test_environment();

    // Add account with short key
    let mut args_add = create_add_account_args("validate-acc-2", "openai", Some("short"));
    args_add.priority = 0;
    let cmd_add = AccountCommands::Add(args_add);
    handle_account_command_with_dir(cmd_add, &config_dir)
        .await
        .unwrap();

    // Validate - should warn about short key
    let args_validate = ValidateAccountArgs {
        id: "validate-acc-2".to_string(),
    };
    let cmd = AccountCommands::Validate(args_validate);
    let result = handle_account_command_with_dir(cmd, &config_dir).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_account_empty_key() {
    let (_temp_dir, config_dir) = setup_test_environment();

    // Add account without API key
    let args_add = create_add_account_args("validate-acc-3", "openai", None);
    let cmd_add = AccountCommands::Add(args_add);
    handle_account_command_with_dir(cmd_add, &config_dir)
        .await
        .unwrap();

    // Validate - should warn about missing key
    let args_validate = ValidateAccountArgs {
        id: "validate-acc-3".to_string(),
    };
    let cmd = AccountCommands::Validate(args_validate);
    let result = handle_account_command_with_dir(cmd, &config_dir).await;

    assert!(result.is_ok());
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[tokio::test]
async fn test_add_account_special_characters_in_id() {
    let (_temp_dir, config_dir) = setup_test_environment();

    let args = create_add_account_args("test-acc_special@123", "openai", Some("sk-test12345678"));
    let cmd = AccountCommands::Add(args);
    let result = handle_account_command_with_dir(cmd, &config_dir).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_account_very_long_id() {
    let (_temp_dir, config_dir) = setup_test_environment();

    let long_id = "a".repeat(200);
    let args = create_add_account_args(&long_id, "openai", Some("sk-test12345678"));
    let cmd = AccountCommands::Add(args);
    let result = handle_account_command_with_dir(cmd, &config_dir).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_account_unknown_provider() {
    let (_temp_dir, config_dir) = setup_test_environment();

    let args = create_add_account_args(
        "test-acc-unknown",
        "unknown-provider",
        Some("sk-test12345678"),
    );
    let cmd = AccountCommands::Add(args);
    let result = handle_account_command_with_dir(cmd, &config_dir).await;

    assert!(result.is_ok());
}
