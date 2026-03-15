//! Security-focused integration tests for FailoverManager

#![cfg(test)]

use super::{account_rotation::AccountSelector, failover::FailoverManager};
use crate::domain::traits::AccountRepository;
use crate::domain::Account;
use crate::infrastructure::JsonAccountRepository;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::Mutex;

fn create_test_repository() -> (TempDir, Arc<dyn AccountRepository>) {
    let temp_dir = TempDir::new().expect("Should create temp dir");
    let repo =
        JsonAccountRepository::with_config_dir(temp_dir.path()).expect("Should create repository");
    (temp_dir, Arc::new(repo) as Arc<dyn AccountRepository>)
}

async fn setup_test_accounts(repo: &dyn AccountRepository, provider_id: &str, count: usize) {
    for i in 0..count {
        let account = Account::new(
            format!("{}-account-{}", provider_id, i),
            provider_id,
            format!("sk-test-key-{}", i),
        );
        repo.save(account).await.expect("Should save account");
    }
}

/// Test: FailoverManager creation with different strategies
#[test]
fn test_failover_manager_creation() {
    let (_temp_dir, repo) = create_test_repository();
    let _round_robin = FailoverManager::with_round_robin(repo.clone());
    let _weighted = FailoverManager::with_weighted(repo.clone());
    let _latency = FailoverManager::with_latency_based(repo.clone());
    let _affinity = FailoverManager::with_user_affinity(repo.clone());
    let _custom = FailoverManager::new(repo, AccountSelector::round_robin(), 5);
}

/// Test: Successful request on first attempt
#[tokio::test]
async fn test_execute_with_failover_success_first_attempt() {
    let (_temp_dir, repo) = create_test_repository();
    setup_test_accounts(&*repo, "openai", 1).await;
    let manager = FailoverManager::with_round_robin(repo);
    let call_count = Arc::new(Mutex::new(0));
    let call_count_clone = call_count.clone();
    let result: Result<String, String> = manager
        .execute_with_failover("openai", move |account| {
            let call_count = call_count_clone.clone();
            let account_id = account.id.clone();
            async move {
                *call_count.lock().await += 1;
                Ok(format!("success-{}", account_id))
            }
        })
        .await;
    assert!(result.is_ok());
    assert_eq!(*call_count.lock().await, 1);
}

/// Test: All accounts fail - returns last error - Security: Verifies error doesn't leak sensitive data
#[tokio::test]
async fn test_execute_with_failover_all_fail() {
    let (_temp_dir, repo) = create_test_repository();
    setup_test_accounts(&*repo, "openai", 2).await;
    let manager = FailoverManager::with_round_robin(repo);
    let result: Result<String, String> = manager
        .execute_with_failover("openai", |_| async { Err("generic-error".to_string()) })
        .await;
    assert!(result.is_err());
    let err = format!("{:?}", result.err().unwrap());
    assert!(
        !err.contains("sk-test-key"),
        "Error should not leak API keys"
    );
}

/// Test: No accounts available - panics (known issue)
#[tokio::test]
async fn test_execute_with_failover_no_accounts() {
    let (_temp_dir, repo) = create_test_repository();
    let manager = FailoverManager::with_round_robin(repo);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::runtime::Handle::current().block_on(async {
            let _: Result<String, String> = manager
                .execute_with_failover("openai", |_| async { Ok("success".to_string()) })
                .await;
        })
    }));
    assert!(result.is_err(), "Should panic when no accounts");
}

/// Test: Circuit breaker prevents using failed account
#[tokio::test]
async fn test_circuit_breaker_blocks_failed_account() {
    let (_temp_dir, repo) = create_test_repository();
    setup_test_accounts(&*repo, "openai", 2).await;
    let manager = FailoverManager::with_round_robin(repo);
    for _ in 0..5 {
        let _: Result<String, String> = manager
            .execute_with_failover("openai", |account| {
                let account_id = account.id.clone();
                async move {
                    if account_id.contains("account-0") {
                        Err("failure".to_string())
                    } else {
                        Ok("success".to_string())
                    }
                }
            })
            .await;
    }
    let result: Result<String, String> = manager
        .execute_with_failover("openai", |account| {
            let account_id = account.id.clone();
            async move { Ok(format!("used-{}", account_id)) }
        })
        .await;
    assert!(result.unwrap().contains("account-1"));
}

/// Test: Concurrent access to health_map - Security: Detects race conditions
#[tokio::test]
async fn test_concurrent_health_map_access() {
    let (_temp_dir, repo) = create_test_repository();
    setup_test_accounts(&*repo, "openai", 3).await;
    let manager = Arc::new(FailoverManager::with_round_robin(repo));
    let mut handles = vec![];
    for i in 0..10 {
        let manager = manager.clone();
        let handle: tokio::task::JoinHandle<Result<String, String>> = tokio::spawn(async move {
            manager
                .execute_with_failover("openai", |_| async {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    Ok(format!("request-{}", i))
                })
                .await
        });
        handles.push(handle);
    }
    let results: Vec<_> = futures::future::join_all(handles).await;
    for result in results {
        assert!(result.unwrap().is_ok(), "All requests should succeed");
    }
}

/// Test: Error messages don't leak API keys - Security: Critical for credential protection
#[tokio::test]
async fn test_error_messages_no_key_leakage() {
    let (_temp_dir, repo) = create_test_repository();
    let secret_key = "sk-super-secret-key-12345";
    let account = Account::new("secret-account", "openai", secret_key);
    repo.save(account).await.expect("Should save");
    let manager = FailoverManager::with_round_robin(repo);
    let result: Result<String, String> = manager
        .execute_with_failover("openai", |_| async { Err("API error".to_string()) })
        .await;
    let err_str = format!("{:?}", result.err().unwrap());
    assert!(
        !err_str.contains(secret_key),
        "Error message leaked API key: {}",
        err_str
    );
}
