//! Extended CLI tests for account and provider commands
//!
//! Comprehensive tests for CLI command handlers.

use rust_llm_api_router::cli::account_commands::{
    cmd_add_account, cmd_list_accounts, cmd_remove_account, cmd_set_priority, cmd_validate_account,
    AddAccountArgs, RemoveAccountArgs, SetPriorityArgs, ValidateAccountArgs,
};
use rust_llm_api_router::domain::{Account, AccountRepository};
use rust_llm_api_router::infrastructure::JsonAccountRepository;
use tempfile::TempDir;

// ============================================================================
// Add Account Command Tests
// ============================================================================

#[tokio::test]
async fn test_cmd_add_account_basic() {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

    let args = AddAccountArgs {
        id: "test-basic".to_string(),
        provider: "openai".to_string(),
        api_key: Some("sk-test-key-123".to_string()),
        priority: 0,
        inactive: false,
        interactive: false,
    };

    let result = cmd_add_account(args, &repo).await;
    assert!(result.is_ok());

    // Verify account was added
    let account = repo.find_by_id("test-basic").await.unwrap();
    assert_eq!(account.id, "test-basic");
    assert_eq!(account.provider_id, "openai");
    assert_eq!(account.auth_method.api_key(), Some("sk-test-key-123"));
    assert!(account.is_active);
    assert_eq!(account.priority, 0);
}

#[tokio::test]
async fn test_cmd_add_account_with_priority() {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

    let args = AddAccountArgs {
        id: "test-priority".to_string(),
        provider: "groq".to_string(),
        api_key: Some("sk-groq-key".to_string()),
        priority: 10,
        inactive: false,
        interactive: false,
    };

    cmd_add_account(args, &repo).await.unwrap();

    let account = repo.find_by_id("test-priority").await.unwrap();
    assert_eq!(account.priority, 10);
}

#[tokio::test]
async fn test_cmd_add_account_inactive() {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

    let args = AddAccountArgs {
        id: "test-inactive".to_string(),
        provider: "anthropic".to_string(),
        api_key: Some("sk-anthropic-key".to_string()),
        priority: 0,
        inactive: true,
        interactive: false,
    };

    cmd_add_account(args, &repo).await.unwrap();

    let account = repo.find_by_id("test-inactive").await.unwrap();
    assert!(!account.is_active);
}

#[tokio::test]
async fn test_cmd_add_account_empty_api_key() {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

    let args = AddAccountArgs {
        id: "test-empty-key".to_string(),
        provider: "openai".to_string(),
        api_key: None,
        priority: 0,
        inactive: false,
        interactive: false,
    };

    // Should succeed but print warning
    let result = cmd_add_account(args, &repo).await;
    assert!(result.is_ok());

    // Account should be created - empty key stored as AuthMethod::ApiKey with empty string
    let account = repo.find_by_id("test-empty-key").await.unwrap();
    assert_eq!(account.auth_method.api_key(), Some("")); // Empty key stored as empty string in enum
}

#[tokio::test]
async fn test_cmd_add_account_duplicate_id() {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

    // Add first account
    let args1 = AddAccountArgs {
        id: "test-dup".to_string(),
        provider: "openai".to_string(),
        api_key: Some("sk-key-1".to_string()),
        priority: 0,
        inactive: false,
        interactive: false,
    };
    cmd_add_account(args1, &repo).await.unwrap();

    // Add duplicate
    let args2 = AddAccountArgs {
        id: "test-dup".to_string(),
        provider: "groq".to_string(),
        api_key: Some("sk-key-2".to_string()),
        priority: 5,
        inactive: false,
        interactive: false,
    };
    cmd_add_account(args2, &repo).await.unwrap();

    // Should update existing account
    let account = repo.find_by_id("test-dup").await.unwrap();
    assert_eq!(account.provider_id, "groq");
    assert_eq!(account.auth_method.api_key(), Some("sk-key-2"));
    assert_eq!(account.priority, 5);
}

// ============================================================================
// Remove Account Command Tests
// ============================================================================

#[tokio::test]
async fn test_cmd_remove_account_success() {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

    // Add account
    repo.save(Account::new("test-remove", "openai", "sk-key"))
        .await
        .unwrap();

    // Remove account
    let args = RemoveAccountArgs {
        id: "test-remove".to_string(),
        force: true, // Skip confirmation in tests
    };
    let result = cmd_remove_account(args, &repo).await;
    assert!(result.is_ok());

    // Verify removed
    let find_result = repo.find_by_id("test-remove").await;
    assert!(find_result.is_err());
}

#[tokio::test]
async fn test_cmd_remove_account_persists_to_file() {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

    // Add account
    repo.save(Account::new("test-persist", "groq", "sk-groq-key"))
        .await
        .unwrap();

    // Remove account
    let args = RemoveAccountArgs {
        id: "test-persist".to_string(),
        force: true, // Skip confirmation in tests
    };
    cmd_remove_account(args, &repo).await.unwrap();

    // Create new repo instance (simulates restart)
    let repo2 = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();
    let find_result = repo2.find_by_id("test-persist").await;

    // Should still be deleted in new instance
    assert!(find_result.is_err());
}

#[tokio::test]
async fn test_cmd_remove_account_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

    let args = RemoveAccountArgs {
        id: "non-existent".to_string(),
        force: true, // Skip confirmation in tests
    };
    let result = cmd_remove_account(args, &repo).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        rust_llm_api_router::Error::ProviderNotFound(_)
    ));
}

#[tokio::test]
async fn test_cmd_remove_account_multiple() {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

    // Add multiple accounts
    repo.save(Account::new("acc-1", "openai", "sk-1"))
        .await
        .unwrap();
    repo.save(Account::new("acc-2", "groq", "sk-2"))
        .await
        .unwrap();
    repo.save(Account::new("acc-3", "anthropic", "sk-3"))
        .await
        .unwrap();

    // Remove middle one
    let args = RemoveAccountArgs {
        id: "acc-2".to_string(),
        force: true, // Skip confirmation in tests
    };
    cmd_remove_account(args, &repo).await.unwrap();

    // Verify others remain
    assert!(repo.find_by_id("acc-1").await.is_ok());
    assert!(repo.find_by_id("acc-3").await.is_ok());

    // Verify removed
    assert!(repo.find_by_id("acc-2").await.is_err());

    // Verify count
    let all = repo.find_all().await.unwrap();
    assert_eq!(all.len(), 2);
}

// ============================================================================
// Set Priority Command Tests
// ============================================================================

#[tokio::test]
async fn test_cmd_set_priority_success() {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

    // Add account
    repo.save(Account::new("test-priority", "openai", "sk-key"))
        .await
        .unwrap();

    // Set priority
    let args = SetPriorityArgs {
        id: "test-priority".to_string(),
        priority: 100,
    };
    cmd_set_priority(args, &repo).await.unwrap();

    // Verify updated
    let account = repo.find_by_id("test-priority").await.unwrap();
    assert_eq!(account.priority, 100);
}

#[tokio::test]
async fn test_cmd_set_priority_negative() {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

    repo.save(Account::new("test-neg", "groq", "sk-key"))
        .await
        .unwrap();

    let args = SetPriorityArgs {
        id: "test-neg".to_string(),
        priority: -50,
    };
    cmd_set_priority(args, &repo).await.unwrap();

    let account = repo.find_by_id("test-neg").await.unwrap();
    assert_eq!(account.priority, -50);
}

#[tokio::test]
async fn test_cmd_set_priority_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

    let args = SetPriorityArgs {
        id: "non-existent".to_string(),
        priority: 10,
    };
    let result = cmd_set_priority(args, &repo).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_cmd_set_priority_persists() {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

    repo.save(Account::new("test-persist-prio", "openai", "sk-key"))
        .await
        .unwrap();

    // Set priority
    let args = SetPriorityArgs {
        id: "test-persist-prio".to_string(),
        priority: 999,
    };
    cmd_set_priority(args, &repo).await.unwrap();

    // New repo instance
    let repo2 = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();
    let account = repo2.find_by_id("test-persist-prio").await.unwrap();
    assert_eq!(account.priority, 999);
}

// ============================================================================
// Validate Account Command Tests
// ============================================================================

#[tokio::test]
async fn test_cmd_validate_account_valid_key() {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

    repo.save(Account::new("test-valid", "openai", "sk-valid-key-123456"))
        .await
        .unwrap();

    let args = ValidateAccountArgs {
        id: "test-valid".to_string(),
    };
    let result = cmd_validate_account(args, &repo).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cmd_validate_account_short_key() {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

    repo.save(Account::new("test-short", "groq", "short"))
        .await
        .unwrap();

    let args = ValidateAccountArgs {
        id: "test-short".to_string(),
    };
    let result = cmd_validate_account(args, &repo).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cmd_validate_account_empty_key() {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

    repo.save(Account::new("test-empty", "anthropic", ""))
        .await
        .unwrap();

    let args = ValidateAccountArgs {
        id: "test-empty".to_string(),
    };
    let result = cmd_validate_account(args, &repo).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cmd_validate_account_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

    let args = ValidateAccountArgs {
        id: "non-existent".to_string(),
    };
    let result = cmd_validate_account(args, &repo).await;

    assert!(result.is_err());
}

// ============================================================================
// List Accounts Command Tests
// ============================================================================

#[tokio::test]
async fn test_cmd_list_accounts_empty() {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

    let result = cmd_list_accounts(&repo).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cmd_list_accounts_with_data() {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

    // Add accounts
    repo.save(Account::new("acc-1", "openai", "sk-key-1"))
        .await
        .unwrap();
    repo.save(Account::new("acc-2", "groq", "sk-key-2"))
        .await
        .unwrap();

    let result = cmd_list_accounts(&repo).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cmd_list_accounts_mixed_status() {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

    // Add active account
    repo.save(Account::new("active-acc", "openai", "sk-key"))
        .await
        .unwrap();

    // Add inactive account
    let mut inactive = Account::new("inactive-acc", "groq", "sk-key");
    inactive.is_active = false;
    repo.save(inactive).await.unwrap();

    let result = cmd_list_accounts(&repo).await;
    assert!(result.is_ok());
}

// ============================================================================
// Integration Tests - Multiple Commands
// ============================================================================

#[tokio::test]
async fn test_cli_full_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();

    // 1. Add account
    let add_args = AddAccountArgs {
        id: "workflow-acc".to_string(),
        provider: "openai".to_string(),
        api_key: Some("sk-workflow-key".to_string()),
        priority: 5,
        inactive: false,
        interactive: false,
    };
    cmd_add_account(add_args, &repo).await.unwrap();

    // 2. Verify added
    let account = repo.find_by_id("workflow-acc").await.unwrap();
    assert_eq!(account.priority, 5);

    // 3. Update priority
    let priority_args = SetPriorityArgs {
        id: "workflow-acc".to_string(),
        priority: 20,
    };
    cmd_set_priority(priority_args, &repo).await.unwrap();

    // 4. Verify priority updated
    let account = repo.find_by_id("workflow-acc").await.unwrap();
    assert_eq!(account.priority, 20);

    // 5. Validate account
    let validate_args = ValidateAccountArgs {
        id: "workflow-acc".to_string(),
    };
    cmd_validate_account(validate_args, &repo).await.unwrap();

    // 6. List accounts (should show 1)
    cmd_list_accounts(&repo).await.unwrap();

    // 7. Remove account
    let remove_args = RemoveAccountArgs {
        id: "workflow-acc".to_string(),
        force: true, // Skip confirmation in tests
    };
    cmd_remove_account(remove_args, &repo).await.unwrap();

    // 8. Verify removed
    let find_result = repo.find_by_id("workflow-acc").await;
    assert!(find_result.is_err());

    // 9. List accounts (should be empty)
    cmd_list_accounts(&repo).await.unwrap();
}
