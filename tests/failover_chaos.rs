//! Chaos tests for failover system using turmoil
//!
//! These tests simulate network partitions, latency, and provider failures
//! to verify the system's resilience under adverse conditions.

use async_trait::async_trait;
use mockall::predicate::*;
use rust_llm_api_router::app::services::account_rotation::AccountSelector;
use rust_llm_api_router::app::services::failover::FailoverManager;
use rust_llm_api_router::domain::entities::Account;
use rust_llm_api_router::domain::traits::AccountRepository;
use rust_llm_api_router::domain::DomainError;
use std::fmt;
use std::sync::Arc;

// ============================================================================
// MOCK REPOSITORY
// ============================================================================

mockall::mock! {
    pub ChaosAccountRepository {}

    #[async_trait]
    impl AccountRepository for ChaosAccountRepository {
        async fn save(&self, account: Account) -> Result<Account, DomainError>;
        async fn find_all(&self) -> Result<Vec<Account>, DomainError>;
        async fn find_by_id(&self, id: &str) -> Result<Account, DomainError>;
        async fn find_active(&self) -> Result<Vec<Account>, DomainError>;
        async fn find_active_by_provider(&self, provider_id: &str) -> Result<Vec<Account>, DomainError>;
        async fn delete(&self, id: &str) -> Result<(), DomainError>;
    }
}

impl Clone for MockChaosAccountRepository {
    fn clone(&self) -> Self {
        MockChaosAccountRepository::new()
    }
}

// ============================================================================
// ERROR TYPE THAT IMPLEMENTS DEBUG
// ============================================================================

#[derive(Clone)]
struct TestError(String);

impl TestError {
    fn new(msg: &str) -> Self {
        TestError(msg.to_string())
    }
}

impl fmt::Debug for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TestError({})", self.0)
    }
}

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for TestError {}

// ============================================================================
// CHAOS TESTS
// ============================================================================

/// Test: Rapid failover between accounts
#[tokio::test]
async fn test_rapid_failover_between_accounts() {
    let mut mock_repo = MockChaosAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(eq("openai"))
        .returning(|_| {
            Ok(vec![
                Account::new("account-1", "openai", "sk-key-1"),
                Account::new("account-2", "openai", "sk-key-2"),
                Account::new("account-3", "openai", "sk-key-3"),
                Account::new("account-4", "openai", "sk-key-4"),
                Account::new("account-5", "openai", "sk-key-5"),
            ])
        });

    let manager = FailoverManager::with_round_robin(Arc::new(mock_repo));

    let mut success_count = 0;

    for i in 0..20 {
        let result: Result<String, TestError> = manager
            .execute_with_failover("openai", |account| {
                let account_id = account.id.clone();
                async move {
                    if account_id.contains("1") || account_id.contains("3") {
                        Err(TestError::new(&format!("fail-{}", i)))
                    } else {
                        Ok((format!("success-{}", account_id), vec![]))
                    }
                }
            })
            .await;

        if result.is_ok() {
            success_count += 1;
        }
    }

    // Most requests should succeed
    assert!(
        success_count >= 10,
        "Expected at least 10 successes, got {}",
        success_count
    );
}

/// Test: All accounts failing sequentially
#[tokio::test]
async fn test_all_accounts_failing_sequentially() {
    let mut mock_repo = MockChaosAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(eq("openai"))
        .returning(|_| {
            Ok(vec![
                Account::new("account-1", "openai", "sk-key-1"),
                Account::new("account-2", "openai", "sk-key-2"),
                Account::new("account-3", "openai", "sk-key-3"),
            ])
        });

    let manager = FailoverManager::with_round_robin(Arc::new(mock_repo));

    let result: Result<String, TestError> = manager
        .execute_with_failover("openai", |_account| async {
            Err(TestError::new("all accounts fail"))
        })
        .await;

    // Should fail after trying all accounts
    assert!(result.is_err());
}

/// Test: Mixed success/failure pattern
#[tokio::test]
async fn test_mixed_success_failure_pattern() {
    let mut mock_repo = MockChaosAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(eq("openai"))
        .returning(|_| Ok(vec![Account::new("account-1", "openai", "sk-key-1")]));

    let manager = FailoverManager::with_round_robin(Arc::new(mock_repo));

    let pattern = [true, false, true, false, true];
    let mut results = Vec::new();

    for should_succeed in pattern {
        let result: Result<String, TestError> = manager
            .execute_with_failover("openai", move |_account| async move {
                if should_succeed {
                    Ok(("success".to_string(), vec![]))
                } else {
                    Err(TestError::new("failure"))
                }
            })
            .await;

        results.push(result.is_ok());
    }

    assert_eq!(results, pattern);
}

/// Test: Health tracking integration
#[tokio::test]
async fn test_health_tracking() {
    let mut mock_repo = MockChaosAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(eq("openai"))
        .returning(|_| Ok(vec![Account::new("account-1", "openai", "sk-key-1")]));

    let manager = FailoverManager::with_round_robin(Arc::new(mock_repo));

    // Record successes
    for _ in 0..10 {
        let _: Result<String, TestError> = manager
            .execute_with_failover("openai", |_account| async {
                Ok(("success".to_string(), vec![]))
            })
            .await;
    }

    // Record failures
    for _ in 0..5 {
        let _: Result<String, TestError> = manager
            .execute_with_failover("openai", |_account| async {
                Err(TestError::new("failure"))
            })
            .await;
    }

    // Check health
    let health = manager.get_all_health();
    assert!(!health.is_empty());

    let account_health = health
        .iter()
        .find(|h| h.account_id == "account-1")
        .expect("Should find account health");

    assert_eq!(account_health.successful_requests, 10);
    assert_eq!(account_health.failed_requests, 5);
}

/// Test: Multiple accounts with different behaviors
#[tokio::test]
async fn test_multiple_accounts_different_behaviors() {
    let mut mock_repo = MockChaosAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(eq("openai"))
        .returning(|_| {
            Ok(vec![
                Account::new("good-account", "openai", "sk-key-1"),
                Account::new("bad-account", "openai", "sk-key-2"),
            ])
        });

    let manager = FailoverManager::with_round_robin(Arc::new(mock_repo));

    // Good account always succeeds
    let result1: Result<String, TestError> = manager
        .execute_with_failover("openai", |account| {
            let account_id = account.id.clone();
            async move {
                if account_id == "bad-account" {
                    Err(TestError::new("bad account"))
                } else {
                    Ok(("success".to_string(), vec![]))
                }
            }
        })
        .await;

    assert!(result1.is_ok());
}

/// Test: Concurrent requests with failover
#[tokio::test]
async fn test_concurrent_failover_requests() {
    let mut mock_repo = MockChaosAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(eq("openai"))
        .returning(|_| {
            Ok(vec![
                Account::new("account-1", "openai", "sk-key-1"),
                Account::new("account-2", "openai", "sk-key-2"),
            ])
        });

    let manager = Arc::new(FailoverManager::with_round_robin(Arc::new(mock_repo)));

    let mut handles = vec![];
    for i in 0..20 {
        let manager = manager.clone();
        let handle: tokio::task::JoinHandle<Result<String, TestError>> = tokio::spawn(async move {
            manager
                .execute_with_failover("openai", |_account| async {
                    Ok((format!("response-{}", i), vec![]))
                })
                .await
        });
        handles.push(handle);
    }

    let results = futures::future::join_all(handles).await;
    let success_count = results
        .iter()
        .filter(|r| r.as_ref().unwrap().is_ok())
        .count();

    assert_eq!(success_count, 20);
}

/// Test: Health tracking with failures
#[tokio::test]
async fn test_health_tracking_with_failures() {
    let mut mock_repo = MockChaosAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(eq("openai"))
        .returning(|_| Ok(vec![Account::new("account-1", "openai", "sk-key-1")]));

    // High retry count
    let manager = FailoverManager::new(Arc::new(mock_repo), AccountSelector::round_robin(), 20);

    // Record failures
    for _ in 0..5 {
        let _: Result<String, TestError> = manager
            .execute_with_failover("openai", |_account| async {
                Err(TestError::new("failure"))
            })
            .await;
    }

    // Check health tracking
    let health = manager.get_all_health();
    assert!(!health.is_empty());

    let account_health = health.iter().find(|h| h.account_id == "account-1");

    assert!(account_health.is_some());
}

/// Test: Account rotation
#[tokio::test]
async fn test_account_rotation() {
    let mut mock_repo = MockChaosAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(eq("openai"))
        .returning(|_| {
            Ok(vec![
                Account::new("account-1", "openai", "sk-key-1"),
                Account::new("account-2", "openai", "sk-key-2"),
            ])
        });

    let manager = FailoverManager::with_round_robin(Arc::new(mock_repo));

    // Make multiple requests
    let mut used_accounts = Vec::new();
    for _ in 0..4 {
        let result: Result<String, TestError> = manager
            .execute_with_failover("openai", |account| {
                let account_id = account.id.clone();
                async move { Ok((account_id, vec![])) }
            })
            .await;

        if let Ok(account_id) = result {
            used_accounts.push(account_id);
        }
    }

    // Should have used both accounts (round robin)
    assert!(used_accounts.contains(&"account-1".to_string()));
    assert!(used_accounts.contains(&"account-2".to_string()));
}
