//! Property-based tests for routing logic
//!
//! Uses proptest's TestRunner directly to verify invariants across the
//! routing configuration, execution plan types, and quality evaluation systems.

use proptest::prelude::*;
use proptest::test_runner::{Config, TestRunner};

use rust_llm_api_router::app::services::execution_plan::types::ExecutionPlanType;
use rust_llm_api_router::app::services::execution_plan::ExecutionPlannerConfig;
use rust_llm_api_router::app::services::quality::evaluator::{QualityConfig, QualityScore};
use rust_llm_api_router::config::{RoutingConfig, RoutingStrategy};

fn runner() -> TestRunner {
    TestRunner::new(Config::with_cases(100))
}

// =============================================================================
// ExecutionPlanType Properties (exhaustive since only 5 variants)
// =============================================================================

#[test]
fn test_execution_plan_type_name_is_non_empty() {
    for pt in [
        ExecutionPlanType::Standard,
        ExecutionPlanType::Failover,
        ExecutionPlanType::LoadBalanced,
        ExecutionPlanType::CostOptimized,
        ExecutionPlanType::Cascading,
    ] {
        assert!(!pt.name().is_empty());
    }
}

#[test]
fn test_execution_plan_type_supports_failover() {
    for (pt, expected) in [
        (ExecutionPlanType::Standard, false),
        (ExecutionPlanType::Failover, true),
        (ExecutionPlanType::LoadBalanced, true),
        (ExecutionPlanType::CostOptimized, false),
        (ExecutionPlanType::Cascading, false),
    ] {
        assert_eq!(pt.supports_failover(), expected);
    }
}

#[test]
fn test_execution_plan_type_supports_load_balancing() {
    for (pt, expected) in [
        (ExecutionPlanType::Standard, false),
        (ExecutionPlanType::Failover, false),
        (ExecutionPlanType::LoadBalanced, true),
        (ExecutionPlanType::CostOptimized, false),
        (ExecutionPlanType::Cascading, false),
    ] {
        assert_eq!(pt.supports_load_balancing(), expected);
    }
}

#[test]
fn test_execution_plan_type_cost_optimized_includes_cascading() {
    for (pt, expected) in [
        (ExecutionPlanType::Standard, false),
        (ExecutionPlanType::Failover, false),
        (ExecutionPlanType::LoadBalanced, false),
        (ExecutionPlanType::CostOptimized, true),
        (ExecutionPlanType::Cascading, true),
    ] {
        assert_eq!(pt.is_cost_optimized(), expected);
    }
}

#[test]
fn test_execution_plan_type_cascading_only_for_cascading() {
    for (pt, expected) in [
        (ExecutionPlanType::Standard, false),
        (ExecutionPlanType::Failover, false),
        (ExecutionPlanType::LoadBalanced, false),
        (ExecutionPlanType::CostOptimized, false),
        (ExecutionPlanType::Cascading, true),
    ] {
        assert_eq!(pt.supports_cascading(), expected);
    }
}

#[test]
fn test_execution_plan_type_display() {
    for pt in [
        ExecutionPlanType::Standard,
        ExecutionPlanType::Failover,
        ExecutionPlanType::LoadBalanced,
        ExecutionPlanType::CostOptimized,
        ExecutionPlanType::Cascading,
    ] {
        assert_eq!(format!("{}", pt), pt.name());
    }
}

// =============================================================================
// RoutingStrategy Properties
// =============================================================================

#[test]
fn test_routing_strategy_roundtrip() {
    for s in [
        "auto",
        "cost-optimized",
        "cost_optimized",
        "cascading",
        "failover",
        "load-balanced",
        "load_balanced",
    ] {
        let strategy: RoutingStrategy = s.parse().unwrap();
        let display = format!("{}", strategy);
        let normalized = s.replace('_', "-");
        if normalized != "cost_optimized" && normalized != "load_balanced" {
            assert!(display.parse::<RoutingStrategy>().is_ok(), "Should parse: {}", display);
        }
    }
}

#[test]
fn test_routing_strategy_invalid_never_parses() {
    let valid = [
        "auto",
        "cost-optimized",
        "cost_optimized",
        "cascading",
        "failover",
        "load-balanced",
        "load_balanced",
    ];
    let mut runner = runner();
    runner
        .run(&"[a-z]{3,15}", |invalid_str| {
            if !valid.contains(&invalid_str.as_str()) {
                prop_assert!(invalid_str.parse::<RoutingStrategy>().is_err());
            }
            Ok(())
        })
        .unwrap();
}

// =============================================================================
// ExecutionPlannerConfig Properties
// =============================================================================

#[test]
fn test_from_routing_config_cascading_maps_correctly() {
    let mut runner = runner();
    runner
        .run(
            &((0u8..2).prop_map(|x| x == 1), 1u32..100, 1u64..3600),
            |(cascading, max_retries, timeout)| {
                let config = RoutingConfig {
                    strategy: if cascading {
                        RoutingStrategy::Cascading
                    } else {
                        RoutingStrategy::Auto
                    },
                    cascading_enabled: cascading,
                    cascading_min_quality: 0.75,
                    cascading_max_tiers: 3,
                    cascading_per_tier_timeout_ms: 5000,
                    budget_mode: false,
                    max_cost_per_million: None,
                    max_retries,
                    timeout_seconds: timeout,
                };
                let pc = ExecutionPlannerConfig::from_routing_config(&config);
                // Cascading should be enabled when strategy is Cascading OR flag is true
                prop_assert_eq!(
                    pc.cascading_enabled,
                    cascading,
                    "cascading should match input flag"
                );
                prop_assert_eq!(pc.default_max_retries, max_retries);
                prop_assert_eq!(pc.default_timeout_seconds, timeout.min(u32::MAX as u64) as u32);
                Ok(())
            },
        )
        .unwrap();
}

#[test]
fn test_from_routing_config_cost_optimized_strategy_maps_correctly() {
    let mut runner = runner();
    runner
        .run(&(0u8..2).prop_map(|x| x == 1), |cascading| {
            let config = RoutingConfig {
                strategy: RoutingStrategy::CostOptimized,
                cascading_enabled: cascading,
                cascading_min_quality: 0.75,
                cascading_max_tiers: 3,
                cascading_per_tier_timeout_ms: 5000,
                budget_mode: false,
                max_cost_per_million: None,
                max_retries: 3,
                timeout_seconds: 60,
            };
            let pc = ExecutionPlannerConfig::from_routing_config(&config);
            prop_assert!(pc.cost_optimization_enabled);
            prop_assert!(!pc.budget_mode_enabled);
            Ok(())
        })
        .unwrap();
}

#[test]
fn test_from_routing_config_budget_mode_maps_correctly() {
    let mut runner = runner();
    runner
        .run(&(0u8..2).prop_map(|x| x == 1), |budget| {
            let config = RoutingConfig {
                strategy: RoutingStrategy::Auto,
                cascading_enabled: false,
                cascading_min_quality: 0.75,
                cascading_max_tiers: 3,
                cascading_per_tier_timeout_ms: 5000,
                budget_mode: budget,
                max_cost_per_million: None,
                max_retries: 3,
                timeout_seconds: 60,
            };
            let pc = ExecutionPlannerConfig::from_routing_config(&config);
            prop_assert_eq!(pc.budget_mode_enabled, budget);
            // Note: budget_mode does NOT auto-enable cascading in current impl
            Ok(())
        })
        .unwrap();
}

#[test]
fn test_from_routing_config_load_balanced_strategy_maps_correctly() {
    let config = RoutingConfig {
        strategy: RoutingStrategy::LoadBalanced,
        cascading_enabled: false,
        cascading_min_quality: 0.75,
        cascading_max_tiers: 3,
        cascading_per_tier_timeout_ms: 5000,
        budget_mode: false,
        max_cost_per_million: None,
        max_retries: 3,
        timeout_seconds: 60,
    };
    let pc = ExecutionPlannerConfig::from_routing_config(&config);
    assert!(pc.load_balancing_enabled);
    assert!(!pc.cost_optimization_enabled);
}

#[test]
fn test_from_routing_config_failover_strategy_maps_correctly() {
    let config = RoutingConfig {
        strategy: RoutingStrategy::Failover,
        cascading_enabled: false,
        cascading_min_quality: 0.75,
        cascading_max_tiers: 3,
        cascading_per_tier_timeout_ms: 5000,
        budget_mode: false,
        max_cost_per_million: None,
        max_retries: 3,
        timeout_seconds: 60,
    };
    let pc = ExecutionPlannerConfig::from_routing_config(&config);
    assert!(pc.failover_enabled);
}

#[test]
fn test_config_defaults_are_sensible() {
    let mut runner = runner();
    runner
        .run(&(1u32..100u32, 1u64..3600u64), |(max_retries, timeout)| {
            let config = RoutingConfig {
                strategy: RoutingStrategy::Auto,
                cascading_enabled: false,
                cascading_min_quality: 0.75,
                cascading_max_tiers: 3,
                cascading_per_tier_timeout_ms: 5000,
                budget_mode: false,
                max_cost_per_million: None,
                max_retries,
                timeout_seconds: timeout,
            };
            let pc = ExecutionPlannerConfig::from_routing_config(&config);
            prop_assert_eq!(pc.default_plan_type, ExecutionPlanType::Standard);
            prop_assert!(pc.enable_auto_selection);
            prop_assert!(pc.max_accounts_per_plan > 0);
            prop_assert!(pc.max_accounts_per_plan <= 10);
            prop_assert!(pc.circuit_breaker_threshold > 0);
            prop_assert!(pc.circuit_breaker_timeout_seconds > 0);
            Ok(())
        })
        .unwrap();
}

// =============================================================================
// QualityConfig & QualityScore Properties
// =============================================================================

#[test]
fn test_quality_config_defaults_are_sensible() {
    let config = QualityConfig::default();
    assert!((0.5..=0.9).contains(&config.min_quality_score));
    assert!(config.min_response_length > 0);
    assert!((1..=10).contains(&config.max_tiers));
    assert!(config.per_tier_timeout_ms > 0);
}

#[test]
fn test_quality_score_calculation() {
    let mut runner = runner();
    runner
        .run(&(0u32..5, 1u32..5, 0.0f64..1.0), |(passed, total, threshold)| {
            let passed = passed.min(total);
            let failed: Vec<String> = (0..(total - passed))
                .map(|i| format!("check_{}", i))
                .collect();
            let score = QualityScore::new(passed, total, failed, threshold);
            prop_assert!((0.0..=1.0).contains(&score.score));
            let expected = passed as f64 / total as f64;
            prop_assert!((score.score - expected).abs() < 1e-10);
            prop_assert_eq!(score.is_acceptable, score.score >= threshold);
            prop_assert_eq!(score.checks_failed.len() as u32, total - passed);
            Ok(())
        })
        .unwrap();
}

#[test]
fn test_quality_score_zero_passed() {
    let mut runner = runner();
    runner
        .run(&(1u32..5, 0.1f64..1.0), |(total, threshold)| {
            let failed: Vec<String> = (0..total).map(|i| format!("check_{}", i)).collect();
            let score = QualityScore::new(0, total, failed, threshold);
            prop_assert!((score.score - 0.0).abs() < 1e-10);
            prop_assert!(!score.is_acceptable);
            prop_assert_eq!(score.checks_failed.len() as u32, total);
            Ok(())
        })
        .unwrap();
}

#[test]
fn test_routing_config_valid_quality_thresholds() {
    let mut runner = runner();
    runner
        .run(
            &(0.0f64..1.0, "(auto|cost-optimized|cascading|failover|load-balanced)"),
            |(quality, strategy)| {
                let result =
                    RoutingConfig::from_cli_and_env(&strategy, false, quality, false, 3, 60);
                prop_assert!(result.is_ok(), "Valid quality {} should produce config", quality);
                let config = result.unwrap();
                prop_assert!((0.0..=1.0).contains(&config.cascading_min_quality));
                Ok(())
            },
        )
        .unwrap();
}

#[test]
fn test_routing_config_invalid_quality_thresholds() {
    let mut runner = runner();
    runner
        .run(&prop_oneof![(-10.0f64..-0.001f64), (1.001f64..100.0f64)], |quality| {
            let result = RoutingConfig::from_cli_and_env("auto", false, quality, false, 3, 60);
            prop_assert!(result.is_err(), "Quality {} should be rejected", quality);
            Ok(())
        })
        .unwrap();
}
