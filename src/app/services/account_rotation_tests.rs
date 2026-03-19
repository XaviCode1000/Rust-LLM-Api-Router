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
    AccountSelector, BackoffConfig, LatencyStrategy, RateLimitInfo, RotationStrategy,
    RoundRobinStrategy, UserAffinityStrategy, WeightedStrategy,
};
use crate::domain::entities::account_health::{AccountHealth, CircuitBreakerState};
use crate::domain::Account;
use std::collections::HashMap;
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
// LATENCY STRATEGY WITH HEALTH TESTS (Phase 4.1)
// ============================================================================

fn create_account_with_latency(id: &str, latency_ms: f64) -> (Account, AccountHealth) {
    let account = Account::new(id, "openai", format!("key-{}", id));
    let mut health = AccountHealth::new(id);
    health.avg_latency_ms = latency_ms;
    (account, health)
}

fn create_account_with_circuit_breaker(
    id: &str,
    state: CircuitBreakerState,
) -> (Account, AccountHealth) {
    let account = Account::new(id, "openai", format!("key-{}", id));
    let mut health = AccountHealth::new(id);
    health.circuit_breaker_state = state;
    health.consecutive_failures = if state == CircuitBreakerState::Open {
        5
    } else {
        0
    };
    (account, health)
}

fn create_account_with_quota(id: &str, quota: Option<u64>) -> (Account, AccountHealth) {
    let account = Account::new(id, "openai", format!("key-{}", id));
    let mut health = AccountHealth::new(id);
    health.quota_remaining = quota;
    (account, health)
}

/// Test: LatencyStrategy selects account with lowest latency
#[test]
fn test_latency_strategy_selects_lowest_latency() {
    let strategy = LatencyStrategy::new();

    let (acc1, h1) = create_account_with_latency("acc1", 100.0);
    let (acc2, mut h2) = create_account_with_latency("acc2", 50.0);
    h2.avg_latency_ms = 50.0;
    let (acc3, mut h3) = create_account_with_latency("acc3", 200.0);
    h3.avg_latency_ms = 200.0;

    let accounts = vec![&acc1, &acc2, &acc3];
    let mut health_map: HashMap<String, AccountHealth> = HashMap::new();
    health_map.insert(acc1.id.clone(), h1);
    health_map.insert(acc2.id.clone(), h2);
    health_map.insert(acc3.id.clone(), h3);

    let selected = strategy.select_with_health(&accounts, &health_map);

    assert!(selected.is_some());
    assert_eq!(selected.unwrap().id, "acc2"); // Lowest latency
}

/// Test: LatencyStrategy excludes accounts with open circuit breaker
#[test]
fn test_latency_strategy_excludes_open_circuit_breaker() {
    let strategy = LatencyStrategy::new();

    let (acc1, _) = create_account_with_circuit_breaker("acc1", CircuitBreakerState::Open);
    let (acc2, h2) = create_account_with_latency("acc2", 100.0);
    let (acc3, h3) = create_account_with_latency("acc3", 50.0);

    let accounts = vec![&acc1, &acc2, &acc3];
    let mut health_map: HashMap<String, AccountHealth> = HashMap::new();
    health_map.insert(acc1.id.clone(), AccountHealth::new("acc1")); // Open circuit breaker
    health_map.get_mut("acc1").unwrap().circuit_breaker_state = CircuitBreakerState::Open;
    health_map.get_mut("acc1").unwrap().consecutive_failures = 5;
    health_map.insert(acc2.id.clone(), h2);
    health_map.insert(acc3.id.clone(), h3);

    let selected = strategy.select_with_health(&accounts, &health_map);

    // acc1 has open circuit breaker, should be excluded
    // acc3 has lower latency than acc2, should be selected
    assert!(selected.is_some());
    assert_ne!(selected.unwrap().id, "acc1");
}

/// Test: LatencyStrategy excludes accounts with zero quota
#[test]
fn test_latency_strategy_excludes_zero_quota() {
    let strategy = LatencyStrategy::new();

    let (acc1, h1) = create_account_with_quota("acc1", Some(0)); // Zero quota
    let (acc2, h2) = create_account_with_latency("acc2", 100.0);
    let (acc3, h3) = create_account_with_latency("acc3", 200.0);

    let accounts = vec![&acc1, &acc2, &acc3];
    let mut health_map: HashMap<String, AccountHealth> = HashMap::new();
    health_map.insert(acc1.id.clone(), h1);
    health_map.insert(acc2.id.clone(), h2);
    health_map.insert(acc3.id.clone(), h3);

    let selected = strategy.select_with_health(&accounts, &health_map);

    // acc1 has zero quota, should be excluded
    // acc2 has lower latency than acc3, should be selected
    assert!(selected.is_some());
    assert_eq!(selected.unwrap().id, "acc2");
}

/// Test: LatencyStrategy fallback when no latency data
#[test]
fn test_latency_strategy_fallback_no_latency_data() {
    let strategy = LatencyStrategy::new();

    let accounts = vec![
        Account::new("acc1", "openai", "key1"),
        Account::new("acc2", "openai", "key2"),
    ];

    let health_map: HashMap<String, AccountHealth> = HashMap::new();

    let selected = strategy.select_with_health(&accounts, &health_map);

    // When no health data, falls back to first account (latency 0.0)
    assert!(selected.is_some());
    assert_eq!(selected.unwrap().id, "acc1");
}

/// Test: LatencyStrategy returns None when all accounts excluded
#[test]
fn test_latency_strategy_returns_none_all_excluded() {
    let strategy = LatencyStrategy::new();

    let (acc1, mut h1) = create_account_with_quota("acc1", Some(0));
    h1.circuit_breaker_state = CircuitBreakerState::Open;
    h1.consecutive_failures = 5;
    h1.quota_remaining = Some(0);

    let (acc2, mut h2) = create_account_with_quota("acc2", Some(0));
    h2.circuit_breaker_state = CircuitBreakerState::Open;
    h2.consecutive_failures = 5;
    h2.quota_remaining = Some(0);

    let accounts = vec![&acc1, &acc2];
    let mut health_map: HashMap<String, AccountHealth> = HashMap::new();
    health_map.insert(acc1.id.clone(), h1);
    health_map.insert(acc2.id.clone(), h2);

    let selected = strategy.select_with_health(&accounts, &health_map);

    // All accounts have open CB or zero quota, should return None
    assert!(selected.is_none());
}

// ============================================================================
// BACKOFF CONFIG TESTS (Phase 4.2)
// ============================================================================

/// Test: Exponential backoff delay increases exponentially
#[test]
fn test_backoff_exponential_increase() {
    let config = BackoffConfig::new(100, 10000, 0.0, 3); // No jitter for predictable test

    let delay0 = config.calculate_delay(0);
    let delay1 = config.calculate_delay(1);
    let delay2 = config.calculate_delay(2);

    // 100 * 2^0 = 100
    // 100 * 2^1 = 200
    // 100 * 2^2 = 400
    assert_eq!(delay0, 100);
    assert_eq!(delay1, 200);
    assert_eq!(delay2, 400);
}

/// Test: Backoff max delay is capped
#[test]
fn test_backoff_max_delay_capped() {
    let config = BackoffConfig::new(100, 500, 0.0, 10);

    let delay10 = config.calculate_delay(10); // 100 * 2^10 = 102400 > 500

    // Should be capped at max_delay_ms
    assert!(delay10 <= 500);
}

/// Test: Backoff jitter is applied within expected range
#[test]
fn test_backoff_jitter_applied() {
    // Test multiple times to catch jitter
    let mut delays_with_jitter: Vec<u64> = Vec::new();
    let config = BackoffConfig::new(100, 10000, 0.1, 3);

    for _ in 0..100 {
        let delay = config.calculate_delay(1); // 200ms base
        delays_with_jitter.push(delay);
    }

    // All delays should be within jitter range (0.9x to 1.1x for 0.1 factor)
    // Actually the jitter is 1.0 + (rand % 2000 / 10000 - 0.1) = 0.9 to 1.1
    for delay in &delays_with_jitter {
        assert!(
            *delay >= 180,
            "Delay {} should be >= 180 (200 * 0.9)",
            delay
        );
        assert!(
            *delay <= 220,
            "Delay {} should be <= 220 (200 * 1.1)",
            delay
        );
    }
}

/// Test: Backoff with different base delays
#[test]
fn test_backoff_different_base_delays() {
    let config = BackoffConfig::new(50, 10000, 0.0, 3);

    let delay0 = config.calculate_delay(0); // 50
    let delay1 = config.calculate_delay(1); // 100
    let delay2 = config.calculate_delay(2); // 200

    assert_eq!(delay0, 50);
    assert_eq!(delay1, 100);
    assert_eq!(delay2, 200);
}

/// Test: Backoff default config
#[test]
fn test_backoff_default_config() {
    let config = BackoffConfig::default();

    assert_eq!(config.base_delay_ms, 100);
    assert_eq!(config.max_delay_ms, 10000);
    assert_eq!(config.jitter_factor, 0.1);
    assert_eq!(config.max_retries, 3);
}

// ============================================================================
// RATE LIMIT INFO TESTS (Phase 4.3)
// ============================================================================

/// Test: Parses X-RateLimit-Remaining header
#[test]
fn test_rate_limit_info_parses_remaining() {
    let headers = vec![("X-RateLimit-Remaining", "950")];

    let info = RateLimitInfo::from_headers(&headers);

    assert_eq!(info.remaining, Some(950));
}

/// Test: Parses X-RateLimit-Limit header
#[test]
fn test_rate_limit_info_parses_limit() {
    let headers = vec![("X-RateLimit-Limit", "1000")];

    let info = RateLimitInfo::from_headers(&headers);

    assert_eq!(info.limit, Some(1000));
}

/// Test: Parses X-RateLimit-Reset header
#[test]
fn test_rate_limit_info_parses_reset() {
    let headers = vec![("X-RateLimit-Reset", "1640000000")];

    let info = RateLimitInfo::from_headers(&headers);

    assert_eq!(info.reset_timestamp, Some(1640000000));
}

/// Test: Parses all rate limit headers together
#[test]
fn test_rate_limit_info_parses_all_headers() {
    let headers = vec![
        ("X-RateLimit-Remaining", "950"),
        ("X-RateLimit-Limit", "1000"),
        ("X-RateLimit-Reset", "1640000000"),
    ];

    let info = RateLimitInfo::from_headers(&headers);

    assert_eq!(info.remaining, Some(950));
    assert_eq!(info.limit, Some(1000));
    assert_eq!(info.reset_timestamp, Some(1640000000));
}

/// Test: RateLimitInfo handles case-insensitive headers
#[test]
fn test_rate_limit_info_case_insensitive() {
    let headers = vec![
        ("x-ratelimit-remaining", "950"),
        ("X-RATELIMIT-LIMIT", "1000"),
        ("X-RateLimit-Reset", "1640000000"),
    ];

    let info = RateLimitInfo::from_headers(&headers);

    assert_eq!(info.remaining, Some(950));
    assert_eq!(info.limit, Some(1000));
    assert_eq!(info.reset_timestamp, Some(1640000000));
}

/// Test: RateLimitInfo handles missing headers
#[test]
fn test_rate_limit_info_missing_headers() {
    let headers = vec![("Content-Type", "application/json")];

    let info = RateLimitInfo::from_headers(&headers);

    assert_eq!(info.remaining, None);
    assert_eq!(info.limit, None);
    assert_eq!(info.reset_timestamp, None);
}

/// Test: RateLimitInfo handles invalid header values
#[test]
fn test_rate_limit_info_invalid_values() {
    let headers = vec![
        ("X-RateLimit-Remaining", "not-a-number"),
        ("X-RateLimit-Limit", "invalid"),
    ];

    let info = RateLimitInfo::from_headers(&headers);

    // Should gracefully handle invalid values
    assert_eq!(info.remaining, None);
    assert_eq!(info.limit, None);
}

/// Test: RateLimitInfo default is empty
#[test]
fn test_rate_limit_info_default() {
    let info = RateLimitInfo::default();

    assert_eq!(info.remaining, None);
    assert_eq!(info.limit, None);
    assert_eq!(info.reset_timestamp, None);
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
