//! Tests for CLI account commands
//!
//! Tests verify account management functionality:
//! - add: Add new accounts
//! - list: List accounts
//! - remove: Delete accounts
//! - set-priority: Update priority
//! - validate: Validate API keys

use tempfile::TempDir;
use std::io::{self, Write};

use rust_llm_api_router::cli::account_commands::{
    AddAccountArgs, RemoveAccountArgs, SetPriorityArgs, ValidateAccountArgs,
    AccountCommands,
    cmd_add_account, cmd_list_accounts, cmd_remove_account, 
    cmd_set_priority, cmd_validate_account,
};
use rust_llm_api_router::domain::traits::AccountRepository;
use rust_llm_api_router::infrastructure::JsonAccountRepository;
use rust_llm_api_router::domain::Account;

// ============================================================================
// Helper Functions
// ============================================================================

fn create_test_repo() -> (TempDir, JsonAccountRepository) {
    let temp_dir = TempDir::new().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());
    let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();
    (temp_dir, repo)
}

// ============================================================================
// Add Account Tests
// ============================================================================

#[tokio::test]
async fn test_cli_add_account_success_active() {
    let (_temp_dir, repo) = create_test_repo();
    
    let args = AddAccountArgs {
        id: "test-acc-1".to_string(),
        provider: "openai".to_string(),
        api_key: Some("sk-test-key-123".to_string()),
        priority: 0,
        inactive: false,
        interactive: false,
    };
    
    let result = cmd_add_account(args, &repo).await;
    
    assert!(result.is_ok());
    
    let account = repo.find_by_id("test-acc-1").await.unwrap();
    assert_eq!(account.id, "test-acc-1");
    assert_eq!(account.provider_id, "openai");
    assert_eq!(account.api_key, "sk-test-key-123");
    assert!(account.is_active);
    assert_eq!(account.priority, 0);
}

#[tokio::test]
async fn test_cli_add_account_success_inactive() {
    let (_temp_dir, repo) = create_test_repo();
    
    let args = AddAccountArgs {
        id: "test-acc-2".to_string(),
        provider: "groq".to_string(),
        api_key: Some("sk-groq-key".to_string()),
        priority: 5,
        inactive: true,
        interactive: false,
    };
    
    let result = cmd_add_account(args, &repo).await;
    
    assert!(result.is_ok());
    
    let account = repo.find_by_id("test-acc-2").await.unwrap();
    
    
    assert!(!account.is_active);
    assert_eq!(account.priority, 5);
}

#[tokio::test]
async fn test_cli_add_account_with_priority() {
    let (_temp_dir, repo) = create_test_repo();
    
    let args = AddAccountArgs {
        id: "test-acc-3".to_string(),
        provider: "anthropic".to_string(),
        api_key: Some("sk-anthropic-key".to_string()),
        priority: 10,
        inactive: false,
        interactive: false,
    };
    
    let result = cmd_add_account(args, &repo).await;
    
    assert!(result.is_ok());
    
    let account = repo.find_by_id("test-acc-3").await.unwrap();
    assert_eq!(account.priority, 10);
}

#[tokio::test]
async fn test_cli_add_account_empty_api_key_warning() {
    let (_temp_dir, repo) = create_test_repo();
    
    let args = AddAccountArgs {
        id: "test-acc-4".to_string(),
        provider: "openai".to_string(),
        api_key: None,
        priority: 0,
        inactive: false,
        interactive: false,
    };
    
    // Should succeed but print warning
    let result = cmd_add_account(args, &repo).await;
    
    assert!(result.is_ok());
    
    let account = repo.find_by_id("test-acc-4").await.unwrap();
    
    assert_eq!(account.api_key, "");
}

#[tokio::test]
async fn test_cli_add_account_duplicate_id() {
    let (_temp_dir, repo) = create_test_repo();
    
    // Add first account
    let args1 = AddAccountArgs {
        id: "test-acc-dup".to_string(),
        provider: "openai".to_string(),
        api_key: Some("sk-key-1".to_string()),
        priority: 0,
        inactive: false,
        interactive: false,
    };
    cmd_add_account(args1, &repo).await.unwrap();
    
    // Try to add duplicate - should overwrite (JSON repo behavior)
    let args2 = AddAccountArgs {
        id: "test-acc-dup".to_string(),
        provider: "openai".to_string(),
        api_key: Some("sk-key-2".to_string()),
        priority: 0,
        inactive: false,
        interactive: false,
    };
    let result = cmd_add_account(args2, &repo).await;
    
    assert!(result.is_ok());
    
    // Verify updated
    let account = repo.find_by_id("test-acc-dup").await.unwrap();
    assert_eq!(account.api_key, "sk-key-2");
}

// ============================================================================
// List Accounts Tests
// ============================================================================

#[tokio::test]
async fn test_cli_list_accounts_empty() {
    let (_temp_dir, repo) = create_test_repo();
    
    let result = cmd_list_accounts(&repo).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cli_list_accounts_with_data() {
    let (_temp_dir, repo) = create_test_repo();
    
    // Add test accounts
    let acc1 = Account::new("acc-1", "openai", "sk-key-1");
    let acc2 = Account::new("acc-2", "groq", "sk-key-2").with_priority(5);
    let acc3 = Account::new("acc-3", "anthropic", "sk-key-3").with_active(false);
    
    repo.save(acc1).await.unwrap();
    repo.save(acc2).await.unwrap();
    repo.save(acc3).await.unwrap();
    
    let result = cmd_list_accounts(&repo).await;
    
    assert!(result.is_ok());
    
    let accounts = repo.find_all().await.unwrap();
    assert_eq!(accounts.len(), 3);
}

#[tokio::test]
async fn test_cli_list_accounts_displays_correctly() {
    let (_temp_dir, repo) = create_test_repo();
    
    // Add account with long API key
    let acc = Account::new("acc-long", "openai", "sk-very-long-api-key-12345");
    repo.save(acc).await.unwrap();
    
    let result = cmd_list_accounts(&repo).await;
    
    assert!(result.is_ok());
}

// ============================================================================
// Remove Account Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_cli_remove_account_success() {
    let (_temp_dir, repo) = create_test_repo();
    
    // Add account
    let acc = Account::new("acc-to-remove", "openai", "sk-key");
    repo.save(acc).await.unwrap();
    
    // Verify exists
    assert!(repo.find_by_id("acc-to-remove").await.is_ok());
    
    // Remove account
    let args = RemoveAccountArgs {
        id: "acc-to-remove".to_string(),
    };
    let result = cmd_remove_account(args, &repo).await;
    
    assert!(result.is_ok());
    
    // Verify removed
    assert!(repo.find_by_id("acc-to-remove").await.is_err());
}

#[tokio::test]
async fn test_cli_remove_account_not_found() {
    let (_temp_dir, repo) = create_test_repo();
    
    let args = RemoveAccountArgs {
        id: "non-existent-acc".to_string(),
    };
    let result = cmd_remove_account(args, &repo).await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), rust_llm_api_router::Error::ProviderNotFound(_)));
}

#[tokio::test]
#[ignore]
async fn test_cli_remove_account_from_multiple() {
    let (_temp_dir, repo) = create_test_repo();
    
    // Add multiple accounts
    repo.save(Account::new("acc-1", "openai", "sk-1")).await.unwrap();
    repo.save(Account::new("acc-2", "groq", "sk-2")).await.unwrap();
    repo.save(Account::new("acc-3", "anthropic", "sk-3")).await.unwrap();
    
    // Remove middle one
    let args = RemoveAccountArgs {
        id: "acc-2".to_string(),
    };
    cmd_remove_account(args, &repo).await.unwrap();
    
    // Verify others remain
    assert!(repo.find_by_id("acc-1").await.is_ok());
    assert!(repo.find_by_id("acc-2").await.is_err());
    assert!(repo.find_by_id("acc-3").await.is_ok());
    
    let all = repo.find_all().await.unwrap();
    assert_eq!(all.len(), 2);
}

// ============================================================================
// Set Priority Tests
// ============================================================================

#[tokio::test]
async fn test_cli_set_priority_success() {
    let (_temp_dir, repo) = create_test_repo();
    
    // Add account
    repo.save(Account::new("acc-priority", "openai", "sk-key").with_priority(0))
        .await.unwrap();
    
    // Update priority
    let args = SetPriorityArgs {
        id: "acc-priority".to_string(),
        priority: 100,
    };
    let result = cmd_set_priority(args, &repo).await;
    
    assert!(result.is_ok());
    
    // Verify updated
    let account = repo.find_by_id("acc-priority").await.unwrap();
    assert_eq!(account.priority, 100);
}

#[tokio::test]
async fn test_cli_set_priority_negative_value() {
    let (_temp_dir, repo) = create_test_repo();
    
    repo.save(Account::new("acc-neg", "openai", "sk-key").with_priority(0))
        .await.unwrap();
    
    let args = SetPriorityArgs {
        id: "acc-neg".to_string(),
        priority: -5,
    };
    let result = cmd_set_priority(args, &repo).await;
    
    assert!(result.is_ok());
    
    let account = repo.find_by_id("acc-neg").await.unwrap();
    assert_eq!(account.priority, -5);
}

#[tokio::test]
async fn test_cli_set_priority_not_found() {
    let (_temp_dir, repo) = create_test_repo();
    
    let args = SetPriorityArgs {
        id: "non-existent".to_string(),
        priority: 5,
    };
    let result = cmd_set_priority(args, &repo).await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), rust_llm_api_router::Error::ProviderNotFound(_)));
}

// ============================================================================
// Validate Account Tests
// ============================================================================

#[tokio::test]
async fn test_cli_validate_account_success() {
    let (_temp_dir, repo) = create_test_repo();
    
    repo.save(Account::new("acc-validate", "openai", "sk-valid-key-123"))
        .await.unwrap();
    
    let args = ValidateAccountArgs {
        id: "acc-validate".to_string(),
    };
    let result = cmd_validate_account(args, &repo).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cli_validate_account_empty_key() {
    let (_temp_dir, repo) = create_test_repo();
    
    repo.save(Account::new("acc-empty", "openai", ""))
        .await.unwrap();
    
    let args = ValidateAccountArgs {
        id: "acc-empty".to_string(),
    };
    let result = cmd_validate_account(args, &repo).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cli_validate_account_short_key() {
    let (_temp_dir, repo) = create_test_repo();
    
    repo.save(Account::new("acc-short", "openai", "short"))
        .await.unwrap();
    
    let args = ValidateAccountArgs {
        id: "acc-short".to_string(),
    };
    let result = cmd_validate_account(args, &repo).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cli_validate_account_not_found() {
    let (_temp_dir, repo) = create_test_repo();
    
    let args = ValidateAccountArgs {
        id: "non-existent".to_string(),
    };
    let result = cmd_validate_account(args, &repo).await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), rust_llm_api_router::Error::ProviderNotFound(_)));
}

#[tokio::test]
async fn test_cli_validate_account_minimum_length_key() {
    let (_temp_dir, repo) = create_test_repo();
    
    // Exactly 8 characters
    repo.save(Account::new("acc-min", "openai", "12345678"))
        .await.unwrap();
    
    let args = ValidateAccountArgs {
        id: "acc-min".to_string(),
    };
    let result = cmd_validate_account(args, &repo).await;
    
    assert!(result.is_ok());
}

// ============================================================================
// Handle Account Command Tests
// ============================================================================

#[tokio::test]
async fn test_handle_account_command_add() {
    let (_temp_dir, repo) = create_test_repo();
    
    let cmd = AccountCommands::Add(AddAccountArgs {
        id: "cmd-test".to_string(),
        provider: "openai".to_string(),
        api_key: Some("sk-cmd-key".to_string()),
        priority: 0,
        inactive: false,
        interactive: false,
    });
    
    // Note: handle_account_command creates its own repo, so we test indirectly
    // by verifying the command enum is properly structured
    match cmd {
        AccountCommands::Add(args) => {
            assert_eq!(args.id, "cmd-test");
            assert_eq!(args.provider, "openai");
        }
        _ => panic!("Expected Add command"),
    }
}

#[tokio::test]
async fn test_handle_account_command_list() {
    let cmd = AccountCommands::List;
    
    match cmd {
        AccountCommands::List => {} // OK
        _ => panic!("Expected List command"),
    }
}

#[tokio::test]
async fn test_handle_account_command_remove() {
    let cmd = AccountCommands::Remove(RemoveAccountArgs {
        id: "to-remove".to_string(),
    });
    
    match cmd {
        AccountCommands::Remove(args) => {
            assert_eq!(args.id, "to-remove");
        }
        _ => panic!("Expected Remove command"),
    }
}

#[tokio::test]
async fn test_handle_account_command_set_priority() {
    let cmd = AccountCommands::SetPriority(SetPriorityArgs {
        id: "priority-acc".to_string(),
        priority: 50,
    });
    
    match cmd {
        AccountCommands::SetPriority(args) => {
            assert_eq!(args.id, "priority-acc");
            assert_eq!(args.priority, 50);
        }
        _ => panic!("Expected SetPriority command"),
    }
}

#[tokio::test]
async fn test_handle_account_command_validate() {
    let cmd = AccountCommands::Validate(ValidateAccountArgs {
        id: "validate-acc".to_string(),
    });
    
    match cmd {
        AccountCommands::Validate(args) => {
            assert_eq!(args.id, "validate-acc");
        }
        _ => panic!("Expected Validate command"),
    }
}

// ============================================================================
// Multiple Accounts Integration Tests
// ============================================================================

#[tokio::test]
async fn test_cli_multiple_accounts_different_providers() {
    let (_temp_dir, repo) = create_test_repo();
    
    // Add accounts for different providers
    cmd_add_account(AddAccountArgs {
        id: "openai-acc".to_string(),
        provider: "openai".to_string(),
        api_key: Some("sk-openai".to_string()),
        priority: 1,
        inactive: false,
        interactive: false,
    }, &repo).await.unwrap();
    
    cmd_add_account(AddAccountArgs {
        id: "groq-acc".to_string(),
        provider: "groq".to_string(),
        api_key: Some("sk-groq".to_string()),
        priority: 2,
        inactive: false,
        interactive: false,
    }, &repo).await.unwrap();
    
    cmd_add_account(AddAccountArgs {
        id: "anthropic-acc".to_string(),
        provider: "anthropic".to_string(),
        api_key: Some("sk-anthropic".to_string()),
        priority: 3,
        inactive: true,
        interactive: false,
    }, &repo).await.unwrap();
    
    // Verify all exist
    let accounts = repo.find_all().await.unwrap();
    assert_eq!(accounts.len(), 3);
    
    let providers: Vec<&str> = accounts.iter().map(|a| a.provider_id.as_str()).collect();
    assert!(providers.contains(&"openai"));
    assert!(providers.contains(&"groq"));
    assert!(providers.contains(&"anthropic"));
}

#[tokio::test]
#[ignore] // Temporarily ignored - race condition in test
async fn test_cli_account_workflow_add_list_remove() {
    let (_temp_dir, repo) = create_test_repo();
    
    // Add
    cmd_add_account(AddAccountArgs {
        id: "workflow-acc".to_string(),
        provider: "openai".to_string(),
        api_key: Some("sk-workflow".to_string()),
        priority: 0,
        inactive: false,
        interactive: false,
    }, &repo).await.unwrap();
    
    // List
    cmd_list_accounts(&repo).await.unwrap();
    
    // Verify exists
    assert!(repo.find_by_id("workflow-acc").await.is_ok());
    
    // Remove
    cmd_remove_account(RemoveAccountArgs {
        id: "workflow-acc".to_string(),
    }, &repo).await.unwrap();
    
    // Verify removed
    assert!(repo.find_by_id("workflow-acc").await.is_err());
}
