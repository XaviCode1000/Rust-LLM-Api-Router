//! Integration tests for JSON repositories
//!
//! Tests cover JsonAccountRepository.

use rust_llm_api_router::domain::{Account, AccountRepository, DomainError};
use rust_llm_api_router::infrastructure::JsonAccountRepository;
use std::sync::Arc;
use tempfile::TempDir;

/// Helper to create test account
fn test_account(id: &str, provider: &str, api_key: &str) -> Account {
    Account::new(id, provider, api_key)
}

/// Helper to create inactive test account
fn test_inactive_account(id: &str, provider: &str, api_key: &str) -> Account {
    let mut account = Account::new(id, provider, api_key);
    account.is_active = false;
    account
}

// ============================================================================
// JsonAccountRepository Tests
// ============================================================================

#[tokio::test]
async fn test_account_repository_save_and_find() {
    let temp_dir = TempDir::new().unwrap();
    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap()
    );

    let account = test_account("test-1", "openai", "sk-test-key");
    repo.save(account.clone()).await.unwrap();

    let found = repo.find_by_id("test-1").await.unwrap();
    assert_eq!(found.id, "test-1");
}

#[tokio::test]
async fn test_account_repository_find_non_existent() {
    let temp_dir = TempDir::new().unwrap();
    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap()
    );

    let result = repo.find_by_id("non-existent").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DomainError::AccountNotFound(_)));
}

#[tokio::test]
async fn test_account_repository_find_all_empty() {
    let temp_dir = TempDir::new().unwrap();
    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap()
    );

    let accounts = repo.find_all().await.unwrap();
    assert!(accounts.is_empty());
}

#[tokio::test]
async fn test_account_repository_find_all_with_data() {
    let temp_dir = TempDir::new().unwrap();
    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap()
    );

    let account1 = test_account("acc-1", "openai", "sk-key-1");
    let account2 = test_account("acc-2", "groq", "gq-key-2");

    repo.save(account1).await.unwrap();
    repo.save(account2).await.unwrap();

    let accounts = repo.find_all().await.unwrap();
    assert_eq!(accounts.len(), 2);
}

#[tokio::test]
async fn test_account_repository_find_active() {
    let temp_dir = TempDir::new().unwrap();
    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap()
    );

    let active1 = test_account("active-1", "openai", "sk-key-1");
    let active2 = test_account("active-2", "groq", "gq-key-2");
    let inactive = test_inactive_account("inactive-1", "openai", "sk-key-3");

    repo.save(active1.clone()).await.unwrap();
    repo.save(active2.clone()).await.unwrap();
    repo.save(inactive).await.unwrap();

    let accounts = repo.find_active().await.unwrap();
    assert_eq!(accounts.len(), 2);

    // Should be sorted by priority (lower = higher priority)
    assert!(accounts[0].priority <= accounts[1].priority);
}

#[tokio::test]
async fn test_account_repository_find_active_by_provider() {
    let temp_dir = TempDir::new().unwrap();
    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap()
    );

    let account1 = test_account("acc-1", "openai", "key-1");
    let account2 = test_inactive_account("acc-2", "openai", "key-2");
    let account3 = test_account("acc-3", "groq", "key-3");

    repo.save(account1).await.unwrap();
    repo.save(account2).await.unwrap();
    repo.save(account3).await.unwrap();

    let accounts = repo.find_active_by_provider("openai").await.unwrap();
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].id, "acc-1");

    let groq_accounts = repo.find_active_by_provider("groq").await.unwrap();
    assert_eq!(groq_accounts.len(), 1);
    assert_eq!(groq_accounts[0].id, "acc-3");
}

#[tokio::test]
async fn test_account_repository_find_active_by_provider_empty() {
    let temp_dir = TempDir::new().unwrap();
    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap()
    );

    let accounts = repo.find_active_by_provider("non-existent").await.unwrap();
    assert!(accounts.is_empty());
}

#[tokio::test]
async fn test_account_repository_update() {
    let temp_dir = TempDir::new().unwrap();
    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap()
    );

    let mut account = test_account("test-1", "openai", "sk-test-key");
    account.priority = 10;
    repo.save(account.clone()).await.unwrap();

    // Update the account
    account.is_active = false;
    account.priority = 20;
    repo.save(account.clone()).await.unwrap();

    let found = repo.find_by_id("test-1").await.unwrap();
    assert!(!found.is_active);
    assert_eq!(found.priority, 20);
}

#[tokio::test]
async fn test_account_repository_update_non_existent() {
    let temp_dir = TempDir::new().unwrap();
    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap()
    );

    let account = test_account("new-acc", "openai", "sk-new-key");
    repo.save(account.clone()).await.unwrap();

    let found = repo.find_by_id("new-acc").await.unwrap();
    assert_eq!(found.provider_id, "openai");
}

#[tokio::test]
async fn test_account_repository_priority_sorting() {
    let temp_dir = TempDir::new().unwrap();
    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap()
    );

    let mut acc1 = test_account("acc-1", "openai", "key-1");
    let mut acc2 = test_account("acc-2", "openai", "key-2");
    let mut acc3 = test_account("acc-3", "openai", "key-3");

    acc1.priority = 30;
    acc2.priority = 10;
    acc3.priority = 20;

    repo.save(acc1).await.unwrap();
    repo.save(acc2).await.unwrap();
    repo.save(acc3).await.unwrap();

    let accounts = repo.find_active().await.unwrap();

    // Should be sorted by priority ascending
    assert_eq!(accounts[0].id, "acc-2"); // priority 10
    assert_eq!(accounts[1].id, "acc-3"); // priority 20
    assert_eq!(accounts[2].id, "acc-1"); // priority 30
}

#[tokio::test]
async fn test_account_repository_multiple_providers() {
    let temp_dir = TempDir::new().unwrap();
    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap()
    );

    let openai_acc = test_account("openai-1", "openai", "sk-key");
    let groq_acc = test_account("groq-1", "groq", "gq-key");
    let anthropic_acc = test_account("anthropic-1", "anthropic", "ak-key");

    repo.save(openai_acc).await.unwrap();
    repo.save(groq_acc).await.unwrap();
    repo.save(anthropic_acc).await.unwrap();

    let all = repo.find_all().await.unwrap();
    assert_eq!(all.len(), 3);

    let openai_accounts = repo.find_active_by_provider("openai").await.unwrap();
    assert_eq!(openai_accounts.len(), 1);
    assert_eq!(openai_accounts[0].provider_id, "openai");
}

#[tokio::test]
async fn test_account_repository_persistence_across_instances() {
    let temp_dir = TempDir::new().unwrap();

    // Create repo and save account
    let repo1: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap()
    );
    let account = test_account("persist-test", "openai", "sk-persist-key");
    repo1.save(account).await.unwrap();

    // Create new repo instance pointing to same directory
    let repo2: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap()
    );

    // Should find the saved account
    let found = repo2.find_by_id("persist-test").await.unwrap();
    assert_eq!(found.id, "persist-test");
    assert_eq!(found.api_key, "sk-persist-key");
}

#[tokio::test]
async fn test_account_repository_find_by_id_preserves_data() {
    let temp_dir = TempDir::new().unwrap();
    let repo: Arc<dyn AccountRepository> = Arc::new(
        JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap()
    );

    let account = test_account("data-test", "mistral", "mi-test-key-123");
    repo.save(account.clone()).await.unwrap();

    let found = repo.find_by_id("data-test").await.unwrap();
    assert_eq!(found.id, "data-test");
    assert_eq!(found.provider_id, "mistral");
    assert_eq!(found.api_key, "mi-test-key-123");
    assert!(found.is_active);
    assert_eq!(found.priority, 0); // default priority
}
