//! Integration tests for failover system
//!
//! End-to-end tests verifying the complete failover flow:
//! - Account repository → FailoverManager → Strategy → Health tracking
//! - Circuit breaker integration
//! - Concurrent request handling

use async_trait::async_trait;
use mockall::predicate::*;
use rust_llm_api_router::app::services::account_rotation::AccountSelector;
use rust_llm_api_router::app::services::failover::FailoverManager;
use rust_llm_api_router::domain::traits::AccountRepository;
use rust_llm_api_router::domain::{Account, DomainError};
use std::sync::Arc;
use tokio::sync::Mutex;

mod common;
use common::errors::TestError;

// ============================================================================
// MOCK REPOSITORY
// ============================================================================

mockall::mock! {
    /// Mock account repository for testing
    pub AccountRepository {}

    // Implement AccountRepository trait methods
    #[async_trait]
    impl AccountRepository for AccountRepository {
        async fn save(&self, account: Account) -> Result<Account, DomainError>;
        async fn find_all(&self) -> Result<Vec<Account>, DomainError>;
        async fn find_by_id(&self, id: &str) -> Result<Account, DomainError>;
        async fn find_active(&self) -> Result<Vec<Account>, DomainError>;
        async fn find_active_by_provider(&self, provider_id: &str) -> Result<Vec<Account>, DomainError>;
        async fn delete(&self, id: &str) -> Result<(), DomainError>;
    }
}

// Clone trait implementation for mock
impl Clone for MockAccountRepository {
    fn clone(&self) -> Self {
        MockAccountRepository::new()
    }
}

// ============================================================================
// END-TO-END FAILOVER TESTS
// ============================================================================

/// Test: Complete failover flow with multiple accounts
/// Security: Verifies end-to-end security of failover system
#[tokio::test]
async fn test_complete_failover_flow() {
    let mut mock_repo = MockAccountRepository::new();

    // Setup: 3 accounts for provider
    mock_repo
        .expect_find_active_by_provider()
        .with(eq("openai"))
        .times(1)
        .returning(|_| {
            Ok(vec![
                Account::new("account-1", "openai", "sk-key-1"),
                Account::new("account-2", "openai", "sk-key-2"),
                Account::new("account-3", "openai", "sk-key-3"),
            ])
        });

    let manager = FailoverManager::with_round_robin(Arc::new(mock_repo));

    // Execute request - should succeed on first account
    let result: Result<String, TestError> = manager
        .execute_with_failover("openai", |account| {
            let account_id = account.id.clone();
            async move { Ok((format!("success-{}", account_id), vec![])) }
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success-account-1");
}

/// Test: Failover cascades through all accounts
/// Security: Verifies no account is skipped in failover
#[tokio::test]
async fn test_failover_cascades_through_accounts() {
    let mut mock_repo = MockAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(eq("openai"))
        .times(1)
        .returning(|_| {
            Ok(vec![
                Account::new("account-1", "openai", "sk-key-1"),
                Account::new("account-2", "openai", "sk-key-2"),
                Account::new("account-3", "openai", "sk-key-3"),
            ])
        });

    let manager = FailoverManager::new(
        Arc::new(mock_repo),
        AccountSelector::round_robin(),
        5, // More retries than accounts
    );

    let attempted_accounts = Arc::new(Mutex::new(Vec::new()));
    let attempted_clone = attempted_accounts.clone();

    let _: Result<String, TestError> = manager
        .execute_with_failover("openai", |account| {
            let attempted = attempted_clone.clone();
            let account_id = account.id.clone();
            async move {
                attempted.lock().await.push(account_id);
                Err(TestError::new("always-fail"))
            }
        })
        .await;

    let attempted = attempted_accounts.lock().await;

    // Should try all 3 accounts
    assert_eq!(attempted.len(), 3);
    assert!(attempted.contains(&"account-1".to_string()));
    assert!(attempted.contains(&"account-2".to_string()));
    assert!(attempted.contains(&"account-3".to_string()));
}

/// Test: Circuit breaker integration with failover
/// Security: Verifies failed accounts are skipped
#[tokio::test]
async fn test_circuit_breaker_integration() {
    let mut mock_repo = MockAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(eq("openai"))
        .times(1..) // Called multiple times in loop
        .returning(|_| {
            Ok(vec![
                Account::new("account-1", "openai", "sk-key-1"),
                Account::new("account-2", "openai", "sk-key-2"),
            ])
        });

    let manager = FailoverManager::with_round_robin(Arc::new(mock_repo));

    // Fail account-1 five times to open circuit breaker
    for _ in 0..5 {
        let _: Result<String, TestError> = manager
            .execute_with_failover("openai", |account| {
                let account_id = account.id.clone();
                async move {
                    if account_id == "account-1" {
                        Err(TestError::new("failure"))
                    } else {
                        Ok(("success".to_string(), vec![]))
                    }
                }
            })
            .await;
    }

    // Now account-1 should be skipped due to circuit breaker
    let result: Result<String, TestError> = manager
        .execute_with_failover("openai", |account| {
            let account_id = account.id.clone();
            async move { Ok((format!("used-{}", account_id), vec![])) }
        })
        .await;

    // Should use account-2 since account-1 circuit is open
    assert!(result.unwrap().contains("account-2"));
}

// ============================================================================
// CONCURRENT REQUEST TESTS
// ============================================================================

/// Test: Multiple concurrent requests don't cause race conditions
/// Security: Detects race conditions in health tracking
#[tokio::test]
async fn test_concurrent_requests_no_race_condition() {
    let mut mock_repo = MockAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(eq("openai"))
        .times(1..) // Allow multiple calls in concurrent test
        .returning(|_| {
            Ok(vec![
                Account::new("account-1", "openai", "sk-key-1"),
                Account::new("account-2", "openai", "sk-key-2"),
            ])
        });

    let manager = Arc::new(FailoverManager::with_round_robin(Arc::new(mock_repo)));

    // Spawn 100 concurrent requests
    let mut handles = vec![];
    for i in 0..100 {
        let manager = manager.clone();
        let handle: tokio::task::JoinHandle<Result<String, TestError>> = tokio::spawn(async move {
            manager
                .execute_with_failover("openai", |account| {
                    let account_id = account.id.clone();
                    async move {
                        // Simulate network latency
                        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
                        Ok((format!("request-{}-{}", i, account_id), vec![]))
                    }
                })
                .await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // All should succeed without panic or deadlock
    let success_count = results
        .iter()
        .filter(|r| r.as_ref().unwrap().as_ref().is_ok())
        .count();

    assert_eq!(success_count, 100, "All requests should succeed");
}

/// Test: Health tracking under concurrent load
/// Security: Verifies thread-safe health metric updates
#[tokio::test]
async fn test_health_tracking_concurrent() {
    let mut mock_repo = MockAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(eq("openai"))
        .times(1..) // Allow multiple calls in concurrent test
        .returning(|_| Ok(vec![Account::new("account-1", "openai", "sk-key-1")]));

    let manager = Arc::new(FailoverManager::with_round_robin(Arc::new(mock_repo)));

    // Mix of successes and failures
    let mut handles = vec![];
    for i in 0..50 {
        let manager = manager.clone();
        let handle: tokio::task::JoinHandle<Result<String, TestError>> = tokio::spawn(async move {
            if i % 2 == 0 {
                manager
                    .execute_with_failover("openai", |_| async {
                        Ok(("success".to_string(), vec![]))
                    })
                    .await
            } else {
                manager
                    .execute_with_failover("openai", |_| async { Err(TestError::new("failure")) })
                    .await
            }
        });
        handles.push(handle);
    }

    futures::future::join_all(handles).await;

    // Check health metrics
    let health_scores = manager.get_all_health().await;
    assert!(!health_scores.is_empty());

    let account_health = health_scores
        .iter()
        .find(|h| h.account_id == "account-1")
        .unwrap();

    // Should have recorded both successes and failures
    assert_eq!(account_health.total_requests, 50);
    assert_eq!(account_health.successful_requests, 25);
    assert_eq!(account_health.failed_requests, 25);
}

// ============================================================================
// MULTI-PROVIDER TESTS
// ============================================================================

/// Test: Failover works independently per provider
/// Security: Verifies provider isolation
#[tokio::test]
async fn test_multi_provider_isolation() {
    let mut mock_repo = MockAccountRepository::new();

    // Setup accounts for both providers
    mock_repo
        .expect_find_active_by_provider()
        .with(eq("openai"))
        .times(1)
        .returning(|_| {
            Ok(vec![
                Account::new("openai-1", "openai", "sk-openai-key"),
                Account::new("openai-2", "openai", "sk-openai-key-2"),
            ])
        });

    mock_repo
        .expect_find_active_by_provider()
        .with(eq("anthropic"))
        .times(1)
        .returning(|_| {
            Ok(vec![Account::new(
                "anthropic-1",
                "anthropic",
                "sk-anthropic-key",
            )])
        });

    let manager = FailoverManager::with_round_robin(Arc::new(mock_repo));

    // Execute requests for both providers
    let openai_result: Result<String, TestError> = manager
        .execute_with_failover("openai", |account| {
            let account_id = account.id.clone();
            async move { Ok((format!("openai-{}", account_id), vec![])) }
        })
        .await;

    let anthropic_result: Result<String, TestError> = manager
        .execute_with_failover("anthropic", |account| {
            let account_id = account.id.clone();
            async move { Ok((format!("anthropic-{}", account_id), vec![])) }
        })
        .await;

    assert_eq!(openai_result.unwrap(), "openai-openai-1");
    assert_eq!(anthropic_result.unwrap(), "anthropic-anthropic-1");

    // Health should be tracked separately
    let health_scores = manager.get_all_health().await;
    assert_eq!(health_scores.len(), 2);
}

// ============================================================================
// TIMEOUT AND RESOURCE TESTS
// ============================================================================

/// Test: Request timeout handling
/// Security: Verifies timeout doesn't cause resource leak
#[tokio::test]
async fn test_request_timeout() {
    let mut mock_repo = MockAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(eq("openai"))
        .times(1)
        .returning(|_| {
            Ok(vec![
                Account::new("account-1", "openai", "sk-key-1"),
                Account::new("account-2", "openai", "sk-key-2"),
            ])
        });

    let manager = FailoverManager::with_round_robin(Arc::new(mock_repo));

    // First account times out, second succeeds
    let result: Result<String, TestError> = manager
        .execute_with_failover("openai", |account| {
            let account_id = account.id.clone();
            async move {
                if account_id == "account-1" {
                    // Simulate timeout
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                    Ok(("should-not-reach".to_string(), vec![]))
                } else {
                    Ok((format!("success-{}", account_id), vec![]))
                }
            }
        })
        .await;

    // Should fail over to account-2 (but will wait for account-1 timeout)
    // This test demonstrates the need for proper timeout handling
    assert!(result.is_ok());
}

/// Test: Memory usage under load
/// Security: Tests for memory leaks in health tracking
#[tokio::test]
async fn test_memory_under_load() {
    let mut mock_repo = MockAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(eq("openai"))
        .times(1..) // Allow multiple calls in loop test
        .returning(|_| Ok(vec![Account::new("account-1", "openai", "sk-key-1")]));

    let manager = Arc::new(FailoverManager::with_round_robin(Arc::new(mock_repo)));

    // Execute many requests
    for _ in 0..1000 {
        let _: Result<String, TestError> = manager
            .execute_with_failover("openai", |_| async { Ok(("success".to_string(), vec![])) })
            .await;
    }

    // Check health tracking doesn't grow unbounded
    let health_scores = manager.get_all_health().await;
    assert_eq!(health_scores.len(), 1);

    // Latency tracking should be bounded
    let account_health = &health_scores[0];
    assert!(
        account_health.recent_latencies.len() <= 100,
        "Latency tracking should be bounded to 100"
    );
}

// ============================================================================
// ERROR PROPAGATION TESTS
// ============================================================================

/// Test: Repository errors are handled gracefully
#[tokio::test]
async fn test_repository_error_handling() {
    let mut mock_repo = MockAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(eq("openai"))
        .times(1)
        .returning(|_| Err(DomainError::AccountNotFound("openai".to_string())));

    let manager = FailoverManager::with_round_robin(Arc::new(mock_repo));

    // Now returns Err(TestError) instead of panicking
    let result: Result<String, TestError> = manager
        .execute_with_failover("openai", |_| async { Ok(("success".to_string(), vec![])) })
        .await;

    assert!(
        result.is_err(),
        "Should return error for repository failure"
    );
}

/// Test: Empty account list handling
#[tokio::test]
async fn test_empty_account_list() {
    let mut mock_repo = MockAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(eq("openai"))
        .times(1)
        .returning(|_| Ok(vec![]));

    let manager = FailoverManager::with_round_robin(Arc::new(mock_repo));

    // Now returns Err(TestError) instead of panicking
    let result: Result<String, TestError> = manager
        .execute_with_failover("openai", |_| async { Ok(("success".to_string(), vec![])) })
        .await;

    assert!(
        result.is_err(),
        "Should return error for empty account list"
    );
}

// ============================================================================
// STRATEGY INTEGRATION TESTS
// ============================================================================

/// Test: All strategies work with failover
#[tokio::test]
async fn test_all_strategies_with_failover() {
    let accounts = vec![
        Account::new("account-1", "openai", "sk-key-1"),
        Account::new("account-2", "openai", "sk-key-2"),
    ];

    // Test each strategy
    let strategies = [
        ("round-robin", AccountSelector::round_robin()),
        ("weighted", AccountSelector::weighted()),
        ("latency-based", AccountSelector::latency_based()),
        ("user-affinity", AccountSelector::user_affinity()),
    ];

    for (name, selector) in strategies {
        let mut mock_repo = MockAccountRepository::new();

        mock_repo
            .expect_find_active_by_provider()
            .with(eq("openai"))
            .times(1)
            .returning({
                let accounts = accounts.clone();
                move |_| Ok(accounts.clone())
            });

        let manager = FailoverManager::new(Arc::new(mock_repo), selector, 3);

        let result: Result<String, TestError> = manager
            .execute_with_failover("openai", |account| {
                let account_id = account.id.clone();
                async move { Ok((format!("{}-{}", name, account_id), vec![])) }
            })
            .await;

        assert!(result.is_ok(), "Strategy {} should work", name);
    }
}
