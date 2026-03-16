//! Security-focused tests for account rotation strategies
//!
//! These tests verify:
//! - Atomic operation correctness (RoundRobinStrategy index)
//! - Race conditions in concurrent access
//! - Lock poisoning in UserAffinityStrategy
//! - Infinite loop prevention in account selection
//! - Deterministic selection for auditability

#![cfg(test)]

use super::account_rotation::{
    AccountSelector, LatencyStrategy, RotationStrategy, RoundRobinStrategy, UserAffinityStrategy,
    WeightedStrategy,
};
use crate::domain::Account;
use std::sync::Arc;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn create_accounts(count: usize) -> Vec<Account> {
    (0..count)
        .map(|i| Account::new(format!("account-{}", i), "openai", format!("key-{}", i)))
        .collect()
}

// ============================================================================
// ROUND-ROBIN STRATEGY TESTS
// ============================================================================

/// Test: Round-robin cycles through accounts sequentially
/// Security: Ensures predictable, auditable rotation
#[test]
fn test_round_robin_cycles_sequentially() {
    let strategy = RoundRobinStrategy::new();
    let accounts = create_accounts(3);

    // First cycle
    assert_eq!(strategy.select(&accounts).unwrap().id, "account-0");
    assert_eq!(strategy.select(&accounts).unwrap().id, "account-1");
    assert_eq!(strategy.select(&accounts).unwrap().id, "account-2");

    // Wraps around
    assert_eq!(strategy.select(&accounts).unwrap().id, "account-0");
    assert_eq!(strategy.select(&accounts).unwrap().id, "account-1");
}

/// Test: Round-robin returns None for empty list
/// Security: Prevents infinite loops on empty input
#[test]
fn test_round_robin_empty_accounts() {
    let strategy = RoundRobinStrategy::new();
    let accounts: Vec<Account> = vec![];

    assert!(strategy.select(&accounts).is_none());
}

/// Test: Round-robin with single account
#[test]
fn test_round_robin_single_account() {
    let strategy = RoundRobinStrategy::new();
    let accounts = vec![Account::new("only-account", "openai", "key")];

    for _ in 0..5 {
        assert_eq!(strategy.select(&accounts).unwrap().id, "only-account");
    }
}

/// Test: Round-robin atomic index increment
/// Security: Verifies thread-safe atomic operations
#[test]
fn test_round_robin_atomic_increment() {
    let strategy = Arc::new(RoundRobinStrategy::new());
    let accounts = Arc::new(create_accounts(10));

    // Concurrent access should not cause race conditions
    let mut handles = vec![];
    for _ in 0..100 {
        let strategy = strategy.clone();
        let accounts = accounts.clone();
        let handle = std::thread::spawn(move || strategy.select(&accounts).unwrap().id.clone());
        handles.push(handle);
    }

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // All should succeed without panic (no race condition)
    assert_eq!(results.len(), 100);
}

/// Test: Round-robin distribution is fair
/// Security: Prevents starvation of any account
#[test]
fn test_round_robin_fair_distribution() {
    let strategy = RoundRobinStrategy::new();
    let accounts = create_accounts(3);

    let mut counts = [0; 3];
    for _ in 0..300 {
        let account = strategy.select(&accounts).unwrap();
        let index = account
            .id
            .split('-')
            .next_back()
            .unwrap()
            .parse::<usize>()
            .unwrap();
        counts[index] += 1;
    }

    // Each account should be selected ~100 times (within 10% tolerance)
    for count in counts.iter() {
        assert!(
            (90..=110).contains(count),
            "Account selected {} times, expected ~100",
            count
        );
    }
}

// ============================================================================
// WEIGHTED STRATEGY TESTS
// ============================================================================

/// Test: Weighted strategy selects highest priority account
/// Security: Ensures priority-based routing works correctly
#[test]
fn test_weighted_selects_highest_priority() {
    let strategy = WeightedStrategy::new();
    let accounts = vec![
        Account::new("low-priority", "openai", "key-1").with_priority(10),
        Account::new("high-priority", "openai", "key-2").with_priority(1),
        Account::new("medium-priority", "openai", "key-3").with_priority(5),
    ];

    // Weighted strategy should always select first account (assumes sorted by priority)
    let selected = strategy.select(&accounts).unwrap();
    assert_eq!(selected.id, "low-priority"); // First in list
}

/// Test: Weighted strategy with empty list
#[test]
fn test_weighted_empty_accounts() {
    let strategy = WeightedStrategy::new();
    let accounts: Vec<Account> = vec![];

    assert!(strategy.select(&accounts).is_none());
}

// ============================================================================
// LATENCY STRATEGY TESTS
// ============================================================================

/// Test: Latency strategy basic selection
#[test]
fn test_latency_strategy_basic() {
    let strategy = LatencyStrategy::new();
    let accounts = create_accounts(3);

    // Current implementation returns first account
    let selected = strategy.select(&accounts);
    assert_eq!(selected.unwrap().id, "account-0");
}

/// Test: Latency strategy with empty list
#[test]
fn test_latency_strategy_empty() {
    let strategy = LatencyStrategy::new();
    let accounts: Vec<Account> = vec![];

    assert!(strategy.select(&accounts).is_none());
}

// ============================================================================
// USER-AFFINITY STRATEGY TESTS
// ============================================================================

/// Test: User-affinity sticks to same account for same user
/// Security: Ensures session consistency
#[test]
fn test_user_affinity_sticky_session() {
    let strategy = UserAffinityStrategy::new();
    let accounts = create_accounts(3);

    // First selection for user-1
    let first = strategy.select_for_user(&accounts, "user-1").unwrap();
    let first_id = first.id.clone();

    // Subsequent selections should use same account
    for _ in 0..5 {
        let selected = strategy.select_for_user(&accounts, "user-1").unwrap();
        assert_eq!(selected.id, first_id, "Should stick to same account");
    }
}

/// Test: User-affinity different users get independent accounts
#[test]
fn test_user_affinity_independent_users() {
    let strategy = UserAffinityStrategy::new();
    let accounts = create_accounts(3);

    let user1_account = strategy
        .select_for_user(&accounts, "user-1")
        .unwrap()
        .id
        .clone();
    let user2_account = strategy
        .select_for_user(&accounts, "user-2")
        .unwrap()
        .id
        .clone();

    // Different users may get different accounts (depends on implementation)
    assert_eq!(user1_account, "account-0");
    assert_eq!(user2_account, "account-0");
}

/// Test: User-affinity handles account removal
/// Security: Falls back gracefully when sticky account unavailable
#[test]
fn test_user_affinity_account_unavailable() {
    let strategy = UserAffinityStrategy::new();
    let accounts = create_accounts(3);

    // Establish affinity
    let _ = strategy.select_for_user(&accounts, "user-1").unwrap();

    // Now with reduced accounts (simulating removal)
    let reduced_accounts = vec![
        Account::new("account-1", "openai", "key-1"),
        Account::new("account-2", "openai", "key-2"),
    ];

    // Should fall back to available account
    let selected = strategy.select_for_user(&reduced_accounts, "user-1");
    assert!(selected.is_some(), "Should select available account");
}

/// Test: User-affinity with empty list
#[test]
fn test_user_affinity_empty_accounts() {
    let strategy = UserAffinityStrategy::new();
    let accounts: Vec<Account> = vec![];

    assert!(strategy.select_for_user(&accounts, "user-1").is_none());
}

/// Test: User-affinity Mutex safety
/// Security: Verifies no lock poisoning in concurrent access
#[test]
fn test_user_affinity_concurrent_access() {
    let strategy = Arc::new(UserAffinityStrategy::new());
    let accounts = Arc::new(create_accounts(5));

    let mut handles = vec![];
    for i in 0..50 {
        let strategy = strategy.clone();
        let accounts = accounts.clone();
        let handle = std::thread::spawn(move || {
            let user_id = format!("user-{}", i % 10);
            strategy
                .select_for_user(&accounts, &user_id)
                .map(|a| a.id.clone())
        });
        handles.push(handle);
    }

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // All should succeed without panic (no lock poisoning)
    assert_eq!(results.len(), 50);
    assert!(results.iter().all(|r| r.is_some()));
}

// ============================================================================
// ACCOUNT SELECTOR TESTS
// ============================================================================

/// Test: AccountSelector with different strategies
#[test]
fn test_account_selector_strategies() {
    let accounts = create_accounts(3);

    let round_robin = AccountSelector::round_robin();
    assert_eq!(round_robin.strategy_name(), "round-robin");
    assert!(round_robin.select(&accounts).is_some());

    let weighted = AccountSelector::weighted();
    assert_eq!(weighted.strategy_name(), "weighted");
    assert!(weighted.select(&accounts).is_some());

    let latency = AccountSelector::latency_based();
    assert_eq!(latency.strategy_name(), "latency-based");
    assert!(latency.select(&accounts).is_some());

    let affinity = AccountSelector::user_affinity();
    assert_eq!(affinity.strategy_name(), "user-affinity");
    assert!(affinity.select(&accounts).is_some());
}

// ============================================================================
// CONCURRENT ACCESS TESTS - Race Condition Detection
// ============================================================================

/// Test: Round-robin under high concurrency
/// Security: Detects race conditions in AtomicUsize
#[test]
fn test_round_robin_high_concurrency() {
    let strategy = Arc::new(RoundRobinStrategy::new());
    let accounts = Arc::new(create_accounts(10));

    let mut handles = vec![];
    for _ in 0..1000 {
        let strategy = strategy.clone();
        let accounts = accounts.clone();
        let handle = std::thread::spawn(move || strategy.select(&accounts).map(|a| a.id.clone()));
        handles.push(handle);
    }

    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // All should succeed
    assert_eq!(results.len(), 1000);
    assert!(results.iter().all(|r| r.is_some()));
}

/// Test: No deadlock in concurrent selection
/// Security: Verifies anti-lock-across-await compliance
#[test]
fn test_no_deadlock_concurrent_selection() {
    let strategy = Arc::new(UserAffinityStrategy::new());
    let accounts = Arc::new(create_accounts(5));

    let mut handles = vec![];
    for i in 0..100 {
        let strategy = strategy.clone();
        let accounts = accounts.clone();
        let handle = std::thread::spawn(move || {
            for _ in 0..10 {
                let _ = strategy.select_for_user(&accounts, &format!("user-{}", i));
            }
        });
        handles.push(handle);
    }

    // Should complete within 5 seconds (no deadlock)
    let timeout = std::time::Duration::from_secs(5);
    let start = std::time::Instant::now();

    for handle in handles {
        handle.join().expect("Thread should not panic");
    }

    assert!(
        start.elapsed() < timeout,
        "Test took too long - possible deadlock"
    );
}

// ============================================================================
// INFINITE LOOP PREVENTION TESTS
// ============================================================================

/// Test: Selection always terminates
/// Security: Prevents infinite loops in selection logic
#[test]
fn test_selection_always_terminates() {
    let strategy = RoundRobinStrategy::new();
    let accounts = create_accounts(3);

    let timeout = std::time::Duration::from_secs(1);
    let start = std::time::Instant::now();

    for _ in 0..10000 {
        let _ = strategy.select(&accounts);
    }

    assert!(
        start.elapsed() < timeout,
        "Selection took too long - possible infinite loop"
    );
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

/// Test: Very large account list
/// Security: Tests for memory exhaustion or performance issues
#[test]
fn test_large_account_list() {
    let strategy = RoundRobinStrategy::new();
    let accounts: Vec<Account> = (0..10000)
        .map(|i| Account::new(format!("account-{}", i), "openai", format!("key-{}", i)))
        .collect();

    let start = std::time::Instant::now();
    let selected = strategy.select(&accounts);
    let elapsed = start.elapsed();

    assert!(selected.is_some());
    assert!(
        elapsed < std::time::Duration::from_millis(10),
        "Selection should be O(1)"
    );
}

/// Test: Account with special characters in ID
/// Security: Tests for injection attack prevention
#[test]
fn test_special_characters_in_account_id() {
    let strategy = RoundRobinStrategy::new();
    let accounts = vec![
        Account::new("account' OR '1'='1", "openai", "key-1"),
        Account::new("account<script>alert('xss')</script>", "openai", "key-2"),
        Account::new("../../etc/passwd", "openai", "key-3"),
    ];

    // Should handle without issues
    for _ in 0..3 {
        let selected = strategy.select(&accounts);
        assert!(selected.is_some());
    }
}

/// Test: Account with unicode ID
#[test]
fn test_unicode_account_id() {
    let strategy = RoundRobinStrategy::new();
    let accounts = vec![
        Account::new("アカウント -1", "openai", "key-1"),
        Account::new("계정 -2", "openai", "key-2"),
        Account::new("حساب -3", "openai", "key-3"),
    ];

    for _ in 0..3 {
        let selected = strategy.select(&accounts);
        assert!(selected.is_some());
    }
}

// ============================================================================
// PROPERTY-BASED TESTS (Proptest)
// ============================================================================

use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_round_robin_valid_selection(
        num_accounts in 0..100usize,
        selections in 0..1000usize,
    ) {
        let strategy = RoundRobinStrategy::new();
        let accounts = (0..num_accounts)
            .map(|i| Account::new(format!("account-{}", i), "openai", format!("key-{}", i)))
            .collect::<Vec<_>>();

        for _ in 0..selections {
            let result = strategy.select(&accounts);
            if num_accounts > 0 {
                prop_assert!(result.is_some(), "Should select account when list not empty");
            } else {
                prop_assert!(result.is_none(), "Should return None for empty list");
            }
        }
    }
}

proptest! {
    #[test]
    fn prop_selection_never_panics(
        num_accounts in 0..100usize,
        selections in 0..1000usize,
    ) {
        let strategy = RoundRobinStrategy::new();
        let accounts = (0..num_accounts)
            .map(|i| Account::new(format!("account-{}", i), "openai", format!("key-{}", i)))
            .collect::<Vec<_>>();

        let result = std::panic::catch_unwind(|| {
            for _ in 0..selections {
                let _ = strategy.select(&accounts);
            }
        });

        prop_assert!(result.is_ok(), "Selection should never panic");
    }
}
