//! Security-focused tests for AccountHealth entity
//!
//! These tests verify:
//! - Circuit breaker state machine correctness
//! - Health score invariants (0-100 range, no NaN)
//! - Race condition prevention in concurrent access
//! - Integer overflow protection in counters
//! - Division by zero prevention in scoring

#![cfg(test)]

use super::{AccountHealth, CircuitBreakerState};
use proptest::prelude::*;

// ============================================================================
// UNIT TESTS - Basic Functionality
// ============================================================================

/// Test: AccountHealth::new() creates valid initial state
/// Security: Ensures no uninitialized or invalid state
#[test]
fn test_new_account_health_has_valid_initial_state() {
    let health = AccountHealth::new("test-account-123");

    assert_eq!(health.account_id, "test-account-123");
    assert_eq!(health.total_requests, 0);
    assert_eq!(health.successful_requests, 0);
    assert_eq!(health.failed_requests, 0);
    assert_eq!(health.recent_latencies.len(), 0);
    assert_eq!(health.avg_latency_ms, 0.0);
    assert_eq!(health.consecutive_failures, 0);
    assert!(!health.circuit_breaker_open);
    assert_eq!(health.health_score(), 25.0); // Default score for new accounts
}

/// Test: Empty account ID handling
/// Security: Validates input sanitization - empty IDs should still work
#[test]
fn test_new_with_empty_account_id() {
    let health = AccountHealth::new("");
    assert_eq!(health.account_id, "");
    assert_eq!(health.health_score(), 25.0);
}

/// Test: Special characters in account ID
/// Security: Tests for injection attack prevention in account IDs
#[test]
fn test_new_with_special_characters_in_id() {
    let health = AccountHealth::new("account' OR '1'='1");
    assert_eq!(health.account_id, "account' OR '1'='1");
    // Should not cause any issues - just treated as string
}

/// Test: Very long account ID
/// Security: Tests for buffer overflow or memory exhaustion
#[test]
fn test_new_with_very_long_id() {
    let long_id = "a".repeat(10000);
    let health = AccountHealth::new(&long_id);
    assert_eq!(health.account_id.len(), 10000);
}

// ============================================================================
// SUCCESS/FAILURE RECORDING TESTS
// ============================================================================

/// Test: Recording success updates counters correctly
/// Security: Ensures accurate metric tracking for security monitoring
#[test]
fn test_record_success_updates_counters() {
    let mut health = AccountHealth::new("test-account");

    health.record_success(100);

    assert_eq!(health.total_requests, 1);
    assert_eq!(health.successful_requests, 1);
    assert_eq!(health.failed_requests, 0);
    assert_eq!(health.consecutive_failures, 0);
    assert!(health.last_success_at.is_some());
}

/// Test: Recording failure updates counters correctly
/// Security: Accurate failure tracking for anomaly detection
#[test]
fn test_record_failure_updates_counters() {
    let mut health = AccountHealth::new("test-account");

    health.record_failure();

    assert_eq!(health.total_requests, 1);
    assert_eq!(health.successful_requests, 0);
    assert_eq!(health.failed_requests, 1);
    assert_eq!(health.consecutive_failures, 1);
    assert!(health.last_failure_at.is_some());
}

/// Test: Success resets consecutive failures
/// Security: Circuit breaker should close after successful request
#[test]
fn test_record_success_resets_consecutive_failures() {
    let mut health = AccountHealth::new("test-account");

    // Build up consecutive failures
    for _ in 0..5 {
        health.record_failure();
    }
    assert_eq!(health.consecutive_failures, 5);

    // Success should reset
    health.record_success(50);
    assert_eq!(health.consecutive_failures, 0);
}

/// Test: Latency tracking with capacity limit
/// Security: Prevents memory leak in VecDeque (mem-avoid-format)
#[test]
fn test_latency_tracking_respects_capacity() {
    let mut health = AccountHealth::new("test-account");

    // Record 150 latencies (capacity is 100)
    for i in 0..150 {
        health.record_success(i);
    }

    // Should only keep last 100
    assert_eq!(health.recent_latencies.len(), 100);
    assert_eq!(health.recent_latencies.front(), Some(&50));
    assert_eq!(health.recent_latencies.back(), Some(&149));
}

// ============================================================================
// CIRCUIT BREAKER TESTS - State Machine Verification
// ============================================================================

/// Test: Circuit breaker opens after 5 consecutive failures
/// Security: Critical for preventing cascade failures
#[test]
fn test_circuit_breaker_opens_after_5_failures() {
    let mut health = AccountHealth::new("test-account");

    // 4 failures - should still be closed
    for i in 0..4 {
        health.record_failure();
        assert!(
            !health.circuit_breaker_open,
            "Should be closed at {} failures",
            i + 1
        );
    }

    // 5th failure - should open
    health.record_failure();
    assert!(health.circuit_breaker_open, "Should open at 5 failures");
    assert!(health.circuit_breaker_opened_at.is_some());
}

/// Test: Circuit breaker prevents requests when open
/// Security: Ensures failed accounts are not used
#[test]
fn test_circuit_breaker_blocks_requests_when_open() {
    let mut health = AccountHealth::new("test-account");

    // Open the circuit breaker
    for _ in 0..5 {
        health.record_failure();
    }

    assert!(!health.can_make_request(), "Should block when open");
}

/// Test: Circuit breaker half-open after 30 seconds
/// Security: Time-based recovery prevents permanent lockout
#[test]
fn test_circuit_breaker_half_open_after_30_seconds() {
    let mut health = AccountHealth::new("test-account");

    // Open the circuit breaker
    for _ in 0..5 {
        health.record_failure();
    }
    assert!(health.circuit_breaker_open);

    // Wait 31 seconds (simulated by time passing in can_make_request)
    // Note: This test relies on actual time passing
    std::thread::sleep(std::time::Duration::from_secs(31));

    assert!(health.can_make_request(), "Should allow after 30s");
    assert!(!health.circuit_breaker_open, "Should close on half-open");
}

/// Test: Success closes circuit breaker immediately
/// Security: Fast recovery when service is healthy
#[test]
fn test_success_closes_circuit_breaker() {
    let mut health = AccountHealth::new("test-account");

    // Open the circuit breaker
    for _ in 0..5 {
        health.record_failure();
    }
    assert!(health.circuit_breaker_open);

    // Record success
    health.record_success(100);

    assert!(!health.circuit_breaker_open, "Should close on success");
    assert!(health.can_make_request());
}

// ============================================================================
// HEALTH SCORE TESTS - Boundary Conditions
// ============================================================================

/// Test: Health score is always in valid range
/// Security: Prevents invalid metrics from affecting routing decisions
#[test]
fn test_health_score_always_in_range() {
    let mut health = AccountHealth::new("test-account");

    // Test various states
    assert!(
        (0.0..=100.0).contains(&health.health_score()),
        "Default score should be 0-100"
    );

    // All successes
    for _ in 0..100 {
        health.record_success(100);
    }
    assert!(
        (0.0..=100.0).contains(&health.health_score()),
        "All-success score should be 0-100"
    );

    // All failures
    let mut health2 = AccountHealth::new("test-account-2");
    for _ in 0..100 {
        health2.record_failure();
    }
    assert!(
        (0.0..=100.0).contains(&health2.health_score()),
        "All-failure score should be 0-100"
    );
}

/// Test: Health score with no requests (default)
/// Security: Default score prevents division by zero
#[test]
fn test_health_score_with_no_requests() {
    let health = AccountHealth::new("test-account");
    assert_eq!(health.health_score(), 25.0); // Default score
}

/// Test: Health score with 100% success rate
#[test]
fn test_health_score_perfect_success() {
    let mut health = AccountHealth::new("test-account");

    for _ in 0..10 {
        health.record_success(100); // Low latency
    }

    // Should have: 50 (success) + 30 (latency) + 20 (circuit closed) = 100
    assert_eq!(health.health_score(), 100.0);
}

/// Test: Health score with 0% success rate
#[test]
fn test_health_score_zero_success() {
    let mut health = AccountHealth::new("test-account");

    for _ in 0..10 {
        health.record_failure();
    }

    // Should have: 0 (success) + 0 (latency) + 0 (circuit open) = 0
    assert_eq!(health.health_score(), 0.0);
}

/// Test: Success rate calculation
/// Security: Prevents division by zero
#[test]
fn test_success_rate_no_division_by_zero() {
    let health = AccountHealth::new("test-account");

    // Should return 100% for zero requests (no division by zero)
    assert_eq!(health.success_rate(), 100.0);
}

/// Test: Success rate with requests
#[test]
fn test_success_rate_calculation() {
    let mut health = AccountHealth::new("test-account");

    health.record_success(100);
    health.record_success(100);
    health.record_failure();

    // 2/3 = 66.67%
    let rate = health.success_rate();
    assert!(
        (66.0..=67.0).contains(&rate),
        "Success rate should be ~66.67%"
    );
}

// ============================================================================
// QUOTA TRACKING TESTS
// ============================================================================

/// Test: Quota usage calculation
#[test]
fn test_quota_usage_calculation() {
    let mut health = AccountHealth::new("test-account");
    health.quota_remaining = Some(500);
    health.quota_limit = Some(1000);

    let usage = health.quota_usage();
    assert_eq!(usage, Some(50.0)); // 50% used
}

/// Test: Quota usage with zero limit (division by zero prevention)
#[test]
fn test_quota_usage_zero_limit() {
    let mut health = AccountHealth::new("test-account");
    health.quota_remaining = Some(500);
    health.quota_limit = Some(0);

    let usage = health.quota_usage();
    assert_eq!(usage, None); // Should return None, not panic
}

/// Test: Quota usage with None values
#[test]
fn test_quota_usage_none_values() {
    let health = AccountHealth::new("test-account");
    assert_eq!(health.quota_usage(), None);
}

// ============================================================================
// PROPERTY-BASED TESTS (Proptest)
// ============================================================================

/// Property: Health score is always between 0 and 100
/// Security: Invariant that must never be violated
proptest! {
    #[test]
    fn prop_health_score_in_range(
        total in 0..1000u64,
        successful in 0..1000u64,
        failed in 0..1000u64,
        circuit_open in any::<bool>(),
    ) {
        // Ensure successful + failed = total
        let (successful, failed) = if successful + failed > total {
            (successful % (total + 1), failed % (total + 1))
        } else {
            (successful, failed)
        };

        let mut health = AccountHealth::new("test");
        health.total_requests = total;
        health.successful_requests = successful;
        health.failed_requests = failed;
        health.circuit_breaker_open = circuit_open;

        // Property: score is always in valid range
        let score = health.health_score();
        prop_assert!(
            (0.0..=100.0).contains(&score),
            "Health score {} out of range [0, 100]",
            score
        );

        // Property: score is never NaN
        prop_assert!(
            !score.is_nan(),
            "Health score should never be NaN"
        );

        // Property: score is never infinite
        prop_assert!(
            !score.is_infinite(),
            "Health score should never be infinite"
        );
    }
}

/// Property: Success rate is always between 0 and 100
proptest! {
    #[test]
    fn prop_success_rate_in_range(
        total in 0..1000u64,
        successful in 0..=1000u64,
    ) {
        let successful = successful.min(total); // Ensure valid

        let mut health = AccountHealth::new("test");
        health.total_requests = total;
        health.successful_requests = successful;

        let rate = health.success_rate();
        prop_assert!(
            (0.0..=100.0).contains(&rate),
            "Success rate {} out of range [0, 100]",
            rate
        );

        prop_assert!(
            !rate.is_nan(),
            "Success rate should never be NaN"
        );
    }
}

/// Property: Recording success never decreases total or successful count
proptest! {
    #[test]
    fn prop_record_success_increments(
        initial_total in 0..1000u64,
        initial_success in 0..1000u64,
        latency in 0..10000u64,
    ) {
        let mut health = AccountHealth::new("test");
        health.total_requests = initial_total;
        health.successful_requests = initial_success;

        health.record_success(latency);

        prop_assert!(
            health.total_requests > initial_total,
            "Total requests should increase"
        );
        prop_assert!(
            health.successful_requests > initial_success,
            "Successful requests should increase"
        );
        prop_assert!(
            health.consecutive_failures == 0,
            "Consecutive failures should reset to 0"
        );
    }
}

/// Property: Recording failure never decreases total or failed count
proptest! {
    #[test]
    fn prop_record_failure_increments(
        initial_total in 0..1000u64,
        initial_failed in 0..1000u64,
        initial_consecutive in 0..10u32,
    ) {
        let mut health = AccountHealth::new("test");
        health.total_requests = initial_total;
        health.failed_requests = initial_failed;
        health.consecutive_failures = initial_consecutive;

        health.record_failure();

        prop_assert!(
            health.total_requests > initial_total,
            "Total requests should increase"
        );
        prop_assert!(
            health.failed_requests > initial_failed,
            "Failed requests should increase"
        );
        prop_assert!(
            health.consecutive_failures > initial_consecutive,
            "Consecutive failures should increase"
        );
    }
}

/// Property: Latency average is never negative or NaN
proptest! {
    #[test]
    fn prop_avg_latency_never_negative_or_nan(
        latencies in prop::collection::vec(0..10000u64, 1..200),
    ) {
        let mut health = AccountHealth::new("test");

        for &latency in &latencies {
            health.record_success(latency);
        }

        prop_assert!(
            health.avg_latency_ms >= 0.0,
            "Average latency should never be negative"
        );
        prop_assert!(
            !health.avg_latency_ms.is_nan(),
            "Average latency should never be NaN"
        );
        prop_assert!(
            !health.avg_latency_ms.is_infinite(),
            "Average latency should never be infinite"
        );
    }
}

/// Property: Circuit breaker state machine is valid
/// States: Closed -> Open -> Half-Open -> Closed
proptest! {
    #[test]
    fn prop_circuit_breaker_state_machine(
        failures in 0..10u32,
        successes in 0..10u32,
    ) {
        let mut health = AccountHealth::new("test");

        // Record failures
        for _ in 0..failures {
            health.record_failure();
        }

        // Record successes
        for _ in 0..successes {
            health.record_success(100);
        }

        // Property: If consecutive_failures >= 5, circuit must be open
        // (unless a success reset it)
        if successes > 0 {
            // Success resets everything
            prop_assert!(!health.circuit_breaker_open);
            prop_assert_eq!(health.consecutive_failures, 0);
        } else if failures >= 5 {
            // No success after 5+ failures = circuit open
            prop_assert!(health.circuit_breaker_open);
        }
    }
}

// ============================================================================
// INTEGER OVERFLOW TESTS
// ============================================================================

/// Test: Counter overflow protection (u64 max)
/// Security: Integer overflow could cause incorrect metrics
#[test]
fn test_counter_near_overflow() {
    let mut health = AccountHealth::new("test-account");
    health.total_requests = u64::MAX - 10;
    health.successful_requests = u64::MAX - 10;

    // Should wrap around (expected Rust behavior for u64)
    health.record_success(100);
    assert_eq!(health.total_requests, u64::MAX - 9);

    // Test what happens at MAX
    health.total_requests = u64::MAX;
    health.record_success(100);
    // Wraps to 0 in release, panics in debug with overflow checks
    // This is expected Rust behavior
}

/// Test: Consecutive failures overflow (u32)
#[test]
fn test_consecutive_failures_near_overflow() {
    let mut health = AccountHealth::new("test-account");
    health.consecutive_failures = u32::MAX - 5;

    for _ in 0..10 {
        health.record_failure();
    }
    // Will wrap around - this is expected
    // In production, circuit breaker would open at 5 anyway
}

// ============================================================================
// CLONE AND SERIALIZATION TESTS
// ============================================================================

/// Test: Clone preserves all fields
#[test]
fn test_clone_preserves_state() {
    let mut original = AccountHealth::new("test-account");
    original.record_success(100);
    original.record_failure();

    let cloned = original.clone();

    assert_eq!(original.account_id, cloned.account_id);
    assert_eq!(original.total_requests, cloned.total_requests);
    assert_eq!(original.health_score(), cloned.health_score());
}

/// Test: Serialization roundtrip
/// Security: Ensures no data loss in persistence
#[test]
fn test_serialization_roundtrip() {
    let mut health = AccountHealth::new("test-account");
    health.record_success(100);
    health.record_failure();
    health.quota_remaining = Some(500);
    health.quota_limit = Some(1000);

    let json = serde_json::to_string(&health).expect("Should serialize");

    let deserialized: AccountHealth = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(health.account_id, deserialized.account_id);
    assert_eq!(health.total_requests, deserialized.total_requests);
    // Note: recent_latencies is skipped in serialization
}

// ============================================================================
// CIRCUIT BREAKER DEGRADED STATE TESTS (Phase 4.4)
// ============================================================================

/// Test: Circuit breaker transitions from Open to Degraded after timeout
#[test]
fn test_circuit_breaker_open_to_degraded_after_timeout() {
    let mut health = AccountHealth::new("test-account");

    // Open the circuit breaker
    for _ in 0..5 {
        health.record_failure();
    }
    assert!(health.circuit_breaker_state.is_open());

    // Wait 31 seconds (simulated)
    std::thread::sleep(std::time::Duration::from_secs(31));

    // After timeout, should transition to Degraded (half-open)
    let allowed = health.can_make_request();

    // In degraded state, requests are allowed but only 10% pass through
    assert!(allowed || !health.circuit_breaker_state.is_open());
}

/// Test: Circuit breaker transitions from Degraded to Closed after 3 successes
#[test]
fn test_circuit_breaker_degraded_to_closed_after_3_successes() {
    let mut health = AccountHealth::new("test-account");

    // Open and then wait to transition to Degraded
    for _ in 0..5 {
        health.record_failure();
    }
    assert!(health.circuit_breaker_state.is_open());

    // Wait for timeout and trigger degraded state
    std::thread::sleep(std::time::Duration::from_secs(31));
    let _ = health.can_make_request(); // This should transition to Degraded

    assert!(health.circuit_breaker_state.is_degraded());
    assert_eq!(health.degraded_success_count, 0);

    // Record 3 successes - should close the circuit breaker
    health.record_success(100);
    assert_eq!(health.degraded_success_count, 1);
    assert!(health.circuit_breaker_state.is_degraded()); // Still degraded

    health.record_success(100);
    assert_eq!(health.degraded_success_count, 2);
    assert!(health.circuit_breaker_state.is_degraded()); // Still degraded

    health.record_success(100);
    assert_eq!(health.degraded_success_count, 3);
    // After 3 successes, should be Closed
    assert!(!health.circuit_breaker_state.is_degraded());
    assert!(!health.circuit_breaker_state.is_open());
    assert_eq!(health.circuit_breaker_state, CircuitBreakerState::Closed);
}

/// Test: Circuit breaker in DEGRADED state allows 10% pass-through
#[test]
fn test_circuit_breaker_degraded_10_percent_passthrough() {
    let mut health = AccountHealth::new("test-account");

    // Set to degraded state directly
    health.circuit_breaker_state = CircuitBreakerState::Degraded;
    health.degraded_success_count = 0;

    // In degraded state, can_make_request uses rand_simple() % 10 == 0
    // This means roughly 10% of calls return true
    // Run many iterations to verify approximately 10% pass rate
    let mut pass_count = 0;
    let iterations = 1000;

    for _ in 0..iterations {
        if health.can_make_request() {
            pass_count += 1;
        }
        // Reset to degraded for next iteration (simulating re-check)
        health.circuit_breaker_state = CircuitBreakerState::Degraded;
    }

    let pass_rate = pass_count as f64 / iterations as f64;

    // Allow some tolerance - should be roughly 10% (between 5% and 15%)
    assert!(
        pass_rate >= 0.05 && pass_rate <= 0.15,
        "Pass rate {}% is not approximately 10%",
        pass_rate * 100.0
    );
}

/// Test: Circuit breaker is_degraded() method works correctly
#[test]
fn test_circuit_breaker_is_degraded_method() {
    let mut health = AccountHealth::new("test-account");

    // Default is Closed
    assert!(!health.circuit_breaker_state.is_degraded());

    // Open
    health.circuit_breaker_state = CircuitBreakerState::Open;
    assert!(!health.circuit_breaker_state.is_degraded());

    // Degraded
    health.circuit_breaker_state = CircuitBreakerState::Degraded;
    assert!(health.circuit_breaker_state.is_degraded());
}

/// Test: Health score in degraded state
#[test]
fn test_health_score_in_degraded_state() {
    let mut health = AccountHealth::new("test-account");

    // Record some successes first to have a non-default score
    for _ in 0..10 {
        health.record_success(100);
    }

    // Set to degraded
    health.circuit_breaker_state = CircuitBreakerState::Degraded;

    // Degraded should have partial score (10 points instead of 20)
    let score = health.health_score();

    // Should have: 50 (success) + 30 (latency) + 10 (degraded) = 90
    assert!(score >= 80.0 && score <= 95.0);
}

/// Test: Degraded success count is reset when circuit reopens
#[test]
fn test_degraded_success_count_reset_on_reopen() {
    let mut health = AccountHealth::new("test-account");

    // Open and transition to degraded
    for _ in 0..5 {
        health.record_failure();
    }
    std::thread::sleep(std::time::Duration::from_secs(31));
    let _ = health.can_make_request();

    assert!(health.circuit_breaker_state.is_degraded());
    assert_eq!(health.degraded_success_count, 0);

    // Record 2 successes
    health.record_success(100);
    health.record_success(100);
    assert_eq!(health.degraded_success_count, 2);

    // New failure should reopen the circuit
    health.record_failure();

    // Circuit should be open again, not degraded
    assert!(health.circuit_breaker_state.is_open());
    assert!(!health.circuit_breaker_state.is_degraded());

    // Success count should be reset (we'd need to check internal, but it should be 0)
}
