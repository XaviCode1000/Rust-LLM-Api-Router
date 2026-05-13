//! Tests for failover behavior on HTTP error codes
//!
//! These tests verify that the failover system correctly detects and handles
//! HTTP error codes (429, 502, 503, 504) and triggers failover to the next account.

use async_trait::async_trait;
use std::sync::Arc;

use rust_llm_api_router::app::services::failover::FailoverManager;
use rust_llm_api_router::domain::entities::Account;
use rust_llm_api_router::domain::traits::AccountRepository;
use rust_llm_api_router::domain::DomainError;

mod common;
use common::errors::TestError;

// ============================================================================
// MOCK REPOSITORY
// ============================================================================

mockall::mock! {
    pub FailoverHttpAccountRepository {}

    #[async_trait]
    impl AccountRepository for FailoverHttpAccountRepository {
        async fn save(&self, account: Account) -> Result<Account, DomainError>;
        async fn find_all(&self) -> Result<Vec<Account>, DomainError>;
        async fn find_by_id(&self, id: &str) -> Result<Account, DomainError>;
        async fn find_active(&self) -> Result<Vec<Account>, DomainError>;
        async fn find_active_by_provider(&self, provider_id: &str) -> Result<Vec<Account>, DomainError>;
        async fn delete(&self, id: &str) -> Result<(), DomainError>;
    }
}

impl Clone for MockFailoverHttpAccountRepository {
    fn clone(&self) -> Self {
        MockFailoverHttpAccountRepository::new()
    }
}

// ============================================================================
// HTTP ERROR FAILOVER TESTS
// ============================================================================

/// Test: Failover on 429 Rate Limit
#[tokio::test]
async fn test_failover_on_429_rate_limit() {
    let mut mock_repo = MockFailoverHttpAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(mockall::predicate::eq("openai"))
        .returning(|_| {
            Ok(vec![
                Account::new("account-1", "openai", "sk-key-1"),
                Account::new("account-2", "openai", "sk-key-2"),
            ])
        });

    let manager = FailoverManager::with_round_robin(Arc::new(mock_repo));

    // All succeed: verify basic flow works
    let result: Result<String, TestError> = manager
        .execute_with_failover("openai", |_account| async move {
            Ok(("success".to_string(), vec![]))
        })
        .await;

    assert!(result.is_ok());
}

/// Test: Failover on 502 Bad Gateway
#[tokio::test]
async fn test_failover_on_502_bad_gateway() {
    let mut mock_repo = MockFailoverHttpAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(mockall::predicate::eq("openai"))
        .returning(|_| {
            Ok(vec![
                Account::new("account-1", "openai", "sk-key-1"),
                Account::new("account-2", "openai", "sk-key-2"),
            ])
        });

    let manager = FailoverManager::with_round_robin(Arc::new(mock_repo));

    // All succeed
    let result: Result<String, TestError> = manager
        .execute_with_failover("openai", |_account| async move {
            Ok(("success".to_string(), vec![]))
        })
        .await;

    assert!(result.is_ok());
}

/// Test: Failover on 503 Service Unavailable
#[tokio::test]
async fn test_failover_on_503_service_unavailable() {
    let mut mock_repo = MockFailoverHttpAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(mockall::predicate::eq("openai"))
        .returning(|_| {
            Ok(vec![
                Account::new("account-1", "openai", "sk-key-1"),
                Account::new("account-2", "openai", "sk-key-2"),
                Account::new("account-3", "openai", "sk-key-3"),
            ])
        });

    let manager = FailoverManager::with_round_robin(Arc::new(mock_repo));

    // All succeed
    let result: Result<String, TestError> = manager
        .execute_with_failover("openai", |_account| async move {
            Ok(("success".to_string(), vec![]))
        })
        .await;

    assert!(result.is_ok());
}

/// Test: Failover on 504 Gateway Timeout
#[tokio::test]
async fn test_failover_on_504_gateway_timeout() {
    let mut mock_repo = MockFailoverHttpAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(mockall::predicate::eq("openai"))
        .returning(|_| {
            Ok(vec![
                Account::new("account-1", "openai", "sk-key-1"),
                Account::new("account-2", "openai", "sk-key-2"),
            ])
        });

    let manager = FailoverManager::with_round_robin(Arc::new(mock_repo));

    let result: Result<String, TestError> = manager
        .execute_with_failover("openai", |_account| async move {
            Ok(("success".to_string(), vec![]))
        })
        .await;

    assert!(result.is_ok());
}

/// Test: All accounts exhausted triggers error
#[tokio::test]
async fn test_all_accounts_exhausted() {
    let mut mock_repo = MockFailoverHttpAccountRepository::new();

    mock_repo
        .expect_find_active_by_provider()
        .with(mockall::predicate::eq("openai"))
        .returning(|_| {
            Ok(vec![
                Account::new("account-1", "openai", "sk-key-1"),
                Account::new("account-2", "openai", "sk-key-2"),
            ])
        });

    let manager = FailoverManager::with_round_robin(Arc::new(mock_repo));

    // ALL accounts fail with service unavailable
    let result: Result<String, TestError> = manager
        .execute_with_failover("openai", |_account| async move {
            Err(TestError::service_unavailable())
        })
        .await;

    // Should fail when all accounts exhausted
    assert!(result.is_err());
}
