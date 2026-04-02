# Technical Design: CLI and Configuration Options for Routing Strategies (Issue #29)

## Architecture

### New Module: `src/config/routing.rs`

```rust
use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RoutingStrategy {
    #[default]
    Auto,
    CostOptimized,
    Cascading,
    Failover,
    LoadBalanced,
}

impl std::str::FromStr for RoutingStrategy {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(RoutingStrategy::Auto),
            "cost-optimized" | "cost_optimized" => Ok(RoutingStrategy::CostOptimized),
            "cascading" => Ok(RoutingStrategy::Cascading),
            "failover" => Ok(RoutingStrategy::Failover),
            "load-balanced" | "load_balanced" => Ok(RoutingStrategy::LoadBalanced),
            _ => Err(format!(
                "Invalid routing strategy '{}'. Valid values: auto, cost-optimized, cascading, failover, load-balanced",
                s
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RoutingConfig {
    pub strategy: RoutingStrategy,
    pub cascading_enabled: bool,
    pub cascading_min_quality: f64,
    pub cascading_max_tiers: u32,
    pub cascading_per_tier_timeout_ms: u64,
    pub budget_mode: bool,
    pub max_cost_per_million: Option<f64>,
    pub max_retries: u32,
    pub timeout_seconds: u64,
}

impl RoutingConfig {
    /// Load configuration from CLI flags and environment variables.
    /// CLI flags take priority over environment variables.
    pub fn from_cli_and_env(
        routing_strategy: &str,
        cascading: bool,
        quality_threshold: f64,
        budget_mode: bool,
        max_retries: u32,
        timeout: u64,
    ) -> Result<Self, String> {
        // Validate quality threshold
        if !(0.0..=1.0).contains(&quality_threshold) {
            return Err("Quality threshold must be between 0.0 and 1.0".to_string());
        }

        // Parse strategy (CLI takes priority)
        let strategy = routing_strategy.parse::<RoutingStrategy>()?;

        // Load env vars (only used if CLI didn't override)
        let cascading_enabled = cascading
            || env::var("CASCADING_ENABLED").map(|v| v == "true").unwrap_or(false);

        let cascading_min_quality = if quality_threshold != 0.75 {
            quality_threshold // CLI override
        } else {
            env::var("CASCADING_MIN_QUALITY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.75)
        };

        let cascading_max_tiers = env::var("CASCADING_MAX_TIERS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3);

        let cascading_per_tier_timeout_ms = env::var("CASCADING_PER_TIER_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5000);

        let budget_mode = budget_mode
            || env::var("BUDGET_MODE").map(|v| v == "true").unwrap_or(false);

        let max_retries = if max_retries != 3 {
            max_retries // CLI override
        } else {
            env::var("MAX_RETRIES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3)
        };

        let timeout_seconds = if timeout != 60 {
            timeout // CLI override
        } else {
            env::var("REQUEST_TIMEOUT_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(60)
        };

        Ok(Self {
            strategy,
            cascading_enabled,
            cascading_min_quality,
            cascading_max_tiers,
            cascading_per_tier_timeout_ms,
            budget_mode,
            max_cost_per_million: None,
            max_retries,
            timeout_seconds,
        })
    }
}
```

### Updated `Cli` Struct

```rust
// src/presentation/cli/mod.rs
#[derive(Debug, Parser)]
#[command(
    after_help = r#"
ROUTING STRATEGIES:
    auto            Planner decides based on context (default)
    cost-optimized  Always select cheapest capable model
    cascading       Start cheap, escalate if quality is low
    failover        Sequential fallback on failure
    load-balanced   Health-weighted distribution

EXAMPLES:
    llm-router --routing-strategy cascading --quality-threshold 0.85
    llm-router --budget-mode --max-retries 5
    llm-router --routing-strategy failover --timeout 30
"#
)]
pub struct Cli {
    // ... existing fields ...

    /// Routing strategy: auto, cost-optimized, cascading, failover, load-balanced
    #[arg(long, default_value = "auto")]
    pub routing_strategy: String,

    /// Enable cascading (quality-based escalation)
    #[arg(long)]
    pub cascading: bool,

    /// Minimum quality score for cascading (0.0-1.0)
    #[arg(long, default_value = "0.75")]
    pub quality_threshold: f64,

    /// Enable budget mode (select cheapest model)
    #[arg(long)]
    pub budget_mode: bool,

    /// Maximum retries per request
    #[arg(long, default_value = "3")]
    pub max_retries: u32,

    /// Request timeout in seconds
    #[arg(long, default_value = "60")]
    pub timeout: u64,
}
```

### Integration with Planner

```rust
// In main.rs or handle_command()
let routing_config = RoutingConfig::from_cli_and_env(
    &cli.routing_strategy,
    cli.cascading,
    cli.quality_threshold,
    cli.budget_mode,
    cli.max_retries,
    cli.timeout,
)?;

// Pass to planner
let planner_config = ExecutionPlannerConfig {
    cascading_enabled: routing_config.cascading_enabled
        || routing_config.strategy == RoutingStrategy::Cascading,
    cost_optimization_enabled: routing_config.strategy == RoutingStrategy::CostOptimized
        || routing_config.budget_mode,
    load_balancing_enabled: routing_config.strategy == RoutingStrategy::LoadBalanced,
    failover_enabled: routing_config.strategy == RoutingStrategy::Failover,
    max_retries: routing_config.max_retries,
    timeout_seconds: routing_config.timeout_seconds,
    // ... other fields from existing config ...
};
```

### Quality Config Wiring

```rust
// Wire Settings.cascading_min_quality_score to planner
let quality_config = QualityConfig {
    min_quality_score: routing_config.cascading_min_quality,
    max_tiers: routing_config.cascading_max_tiers,
    per_tier_timeout_ms: routing_config.cascading_per_tier_timeout_ms,
    ..Default::default()
};
```

### Logging

```rust
// In LlmRouter::route_request()
tracing::info!(
    request_id = %context.request_id,
    strategy = %plan.plan_type().name(),
    accounts = %plan.account_count(),
    "Routing request"
);
```

## Files Modified

| File | Lines Changed | Description |
|------|---------------|-------------|
| `src/config/routing.rs` | +120 | **NEW** — RoutingConfig struct + CLI/env loading |
| `src/config/mod.rs` | +2 | Re-export RoutingConfig |
| `src/presentation/cli/mod.rs` | +40 | Add routing flags to Cli struct |
| `src/app/services/execution_plan/planner.rs` | +10 | Wire RoutingConfig to planner |
| `src/app/router/llm_router.rs` | +10 | Pass routing config, add logging |
| `src/main.rs` | +10 | Initialize RoutingConfig |
| `docs/cli.md` | +30 | Document new flags |
| `docs/routing.md` | +20 | Document configuration options |

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Config priority confusion | Clear documentation + logging of final config |
| Backward compat with existing env vars | Keep existing env var parsing, add new ones alongside |
| Invalid quality threshold | Validate at CLI parse time |
| Invalid strategy name | FromStr impl with clear error message |
