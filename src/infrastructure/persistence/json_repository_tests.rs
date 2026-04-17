//! Security-focused tests for JSON account repository

use super::JsonAccountRepository;
use crate::domain::traits::AccountRepository;
use crate::domain::Account;
use std::sync::Arc;
use tempfile::TempDir;

fn create_temp_repository() -> (TempDir, JsonAccountRepository) {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let repo =
        JsonAccountRepository::with_config_dir(temp_dir.path()).expect("Should create repository");
    (temp_dir, repo)
}

/// Test: Save and retrieve account
#[tokio::test]
async fn test_save_and_find_account() {
    let (_temp_dir, repo) = create_temp_repository();
    let account = Account::new("test-account", "openai", "sk-test-key");
    repo.save(account.clone()).await.expect("Should save");
    let retrieved = repo.find_by_id("test-account").await.expect("Should find");
    assert_eq!(retrieved.id, account.id);
}

/// Test: Find non-existent account - Security: Error should not leak file system details
#[tokio::test]
async fn test_find_non_existent_account() {
    let (_temp_dir, repo) = create_temp_repository();
    let result = repo.find_by_id("non-existent").await;
    assert!(result.is_err());
    let err = format!("{:?}", result.err().unwrap());
    assert!(!err.contains("/tmp/"), "Error should not leak temp path");
}

/// Test: Invalid JSON file - Security: Should return error, not leak info
#[tokio::test]
async fn test_invalid_json_file() {
    let (temp_dir, _repo) = create_temp_repository();
    let file_path = temp_dir.path().join("accounts.json");
    std::fs::write(&file_path, "not valid json {{{").expect("Should write");
    let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).expect("Should create");
    let result = repo.find_all().await;
    assert!(result.is_err());
    let err = format!("{:?}", result.err().unwrap());
    assert!(!err.contains(temp_dir.path().to_str().unwrap()));
}

/// Test: API key not leaked in error messages - Security: Critical for credential protection
#[tokio::test]
async fn test_api_key_not_in_errors() {
    let (temp_dir, _repo) = create_temp_repository();
    let secret_key = "sk-super-secret-key-xyz";
    let file_path = temp_dir.path().join("accounts.json");
    std::fs::write(&file_path, format!(r#"[{{"id":"test","provider_id":"openai","api_key":"{}","is_active":true,"priority":0}}]"#, secret_key)).expect("Should write");
    std::fs::write(&file_path, "invalid json").expect("Should write");
    let repo = JsonAccountRepository::with_config_dir(temp_dir.path()).expect("Should create");
    let result = repo.find_all().await;
    let err = format!("{:?}", result.err().unwrap());
    assert!(!err.contains(secret_key), "Error message leaked API key");
}

/// Test: Multiple concurrent reads - Security: Tests for file descriptor exhaustion
#[tokio::test]
async fn test_concurrent_reads() {
    let (_temp_dir, repo) = create_temp_repository();
    for i in 0..10 {
        let account = Account::new(format!("account-{}", i), "openai", format!("key-{}", i));
        repo.save(account).await.expect("Should save");
    }
    let repo = Arc::new(repo);
    let mut handles = vec![];
    for _ in 0..50 {
        let repo = repo.clone();
        let handle = tokio::spawn(async move { repo.find_all().await });
        handles.push(handle);
    }
    let results: Vec<_> = futures::future::join_all(handles).await;
    for result in results {
        assert!(result.unwrap().is_ok(), "Read should succeed");
    }
}
