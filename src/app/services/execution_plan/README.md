# Execution Plan Module

The Execution Plan module provides proactive planning for LLM request execution, transforming the system from reactive failover to intelligent pre-execution planning.

## Overview

This module enables the system to:
- **Plan optimal execution paths** before making the first request
- **Select the best account** based on health, priority, and compatibility
- **Support multiple execution strategies** (Standard, Failover, Load Balanced, Cost Optimized)
- **Provide observability** through metrics, tracing, and audit logging

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      ExecutionPlanner                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│  │ Plan Type   │  │ Rotation    │  │ Model Compatibility    │ │
│  │ Selection   │  │ Strategies  │  │ Checking              │ │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    ExecutionPlanImpl                            │
│  ┌──────────────────┐  ┌─────────────────────────────────────┐ │
│  │ ExecutionContext│  │ Vec<PlannedAccount>                 │ │
│  └──────────────────┘  └─────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Plan Execution                                │
│  ┌──────────┐ ┌──────────────┐ ┌─────────────────┐ ┌──────────┐│
│  │ Standard │ │ Failover     │ │ Load Balanced  │ │ Cost     ││
│  │          │ │              │ │                │ │ Optimized││
│  └──────────┘ └──────────────┘ └─────────────────┘ └──────────┘│
└─────────────────────────────────────────────────────────────────┘
```

## Key Components

### ExecutionPlanner

The core orchestrator that creates execution plans:

```rust
use crate::app::services::execution_plan::{
    ExecutionPlanner, ExecutionPlannerBuilder, ExecutionPlanType,
    RotationStrategyType, ExecutionContext, PlanningOptions,
};

let planner = ExecutionPlannerBuilder::new()
    .with_account_repository(account_repo)
    .with_provider_repository(provider_repo)
    .with_health_service(health_service)
    .with_rotation_strategy(RotationStrategyType::RoundRobin)
    .with_default_plan_type(ExecutionPlanType::Failover)
    .build()
    .await?;

let context = ExecutionContext::new(
    "request-123".to_string(),
    "groq:llama-3.3-70b-versatile".to_string(),
    PlanningOptions::default(),
);

let plan = planner.create_plan(ExecutionPlanType::Failover, context).await?;
```

### Execution Plans

Four execution plan types are available:

| Plan Type | Description | Use Case |
|-----------|-------------|----------|
| `Standard` | Single account execution | Simple, low-cost requests |
| `Failover` | Sequential fallback accounts | Reliability-critical requests |
| `LoadBalanced` | Round-robin across accounts | High-volume requests |
| `CostOptimized` | Lowest cost account selection | Budget-constrained workloads |
| `Cascading` | Quality-based escalation across tiers | Cost optimization with quality guarantee |

### Rotation Strategies

| Strategy | Description |
|----------|-------------|
| `RoundRobin` | Sequential rotation across accounts |
| `HealthWeighted` | Favor accounts with better health scores |
| `Priority` | Higher priority accounts first |
| `LRU` | Least recently used accounts |

## Configuration

### Environment Variables

- `PLANNING_TIMEOUT_MS` - Maximum time for planning (default: 5000ms)
- `MAX_ACCOUNTS_PER_PLAN` - Maximum accounts to include in a plan (default: 3)
- `HEALTH_WEIGHT_THRESHOLD` - Minimum health score (default: 50)

### Programmatic Configuration

```rust
let config = ExecutionPlannerConfig::builder()
    .with_planning_timeout_ms(3000)
    .with_max_accounts_per_plan(5)
    .with_health_weight_threshold(30)
    .with_enable_caching(true)
    .with_cache_ttl_secs(300)
    .build();
```

## Metrics

The module exports Prometheus metrics:

| Metric | Type | Description |
|--------|------|-------------|
| `execution_plans_created_total` | Counter | Total plans created |
| `execution_plans_in_flight` | Gauge | Active plans |
| `execution_planning_duration_seconds` | Histogram | Planning time |
| `execution_plan_type_*_total` | Counter | Plans by type |
| `execution_plan_fallback_usage_total` | Counter | Fallback usage |
| `execution_plan_outcome_*_total` | Counter | Outcomes |
| `execution_planning_errors_total` | Counter | Planning errors |

## Tracing

Distributed tracing support via OpenTelemetry:

- **PlanningSpan**: Tracks the planning process
- **ExecutionSpan**: Tracks plan execution
- **DecisionLog**: Records planning decisions

```rust
// Enable tracing
let tracer = ExecutionTracing::new_tracer("execution-planner");

// Create planning span
let _span = tracer
    .start_planning_span("create_failover_plan")
    .with_model("groq:llama-3.3-70b-versatile")
    .with_accounts_count(3)
    .enter();
```

## Error Handling

Errors are handled through the `ExecutionPlanError` type:

```rust
pub enum ExecutionPlanError {
    NoHealthyAccounts,
    NoAccountsForProvider,
    PlanCreationFailed(String),
    AccountNotFound(String),
    ProviderNotFound(String),
    PlanningTimeout,
    MetricError(String),
}
```

## Testing

The module includes comprehensive tests:

```bash
# Run execution plan tests
cargo test execution_plan

# Run with coverage
cargo llvm-cov --package execution_plan
```

## Examples

### Basic Usage

```rust
use crate::app::services::execution_plan::*;

// Create a failover plan
let context = ExecutionContext::new(
    "req-001".to_string(),
    "groq:llama-3.3-70b-versatile".to_string(),
    PlanningOptions::default(),
);

let plan = planner
    .create_plan(ExecutionPlanType::Failover, context)
    .await?;

// Execute the plan
match plan.execute().await {
    Ok(response) => println!("Success: {:?}", response),
    Err(e) => println!("Failed: {:?}", e),
}
```

### Custom Strategy

```rust
use crate::app::services::execution_plan::{RotationStrategyType, ExecutionPlanType};

let planner = ExecutionPlannerBuilder::new()
    .with_rotation_strategy(RotationStrategyType::HealthWeighted)
    .with_default_plan_type(ExecutionPlanType::CostOptimized)
    .with_health_weight_threshold(70)
    .build()
    .await?;
```

### Cascading Execution Plan (Issue #24)

The Cascading plan tries cheaper models first and escalates based on response quality:

```rust
use crate::app::services::quality::evaluator::QualityConfig;

let quality_config = QualityConfig {
    min_quality_score: 0.75,  // Escalate if quality < 75%
    max_tiers: 3,              // Try up to 3 models
    per_tier_timeout_ms: 5000,
    ..Default::default()
};

// Create cascading plan with quality gate
let plan = CascadingExecutionPlan::new(
    context,
    accounts,      // Sorted by cost (cheapest first)
    pricing,
    model_ids,
    quality_config,
    quality_gate,
);

// Execute with cascading logic
let result = plan.execute(config, response_text, tokens_used);

// Check results
match result {
    ExecutionResult::Success { tier_used, quality_score, total_cost, .. } => {
        println!("Success with tier {} (quality: {:.2}, cost: {}µ$)", 
            tier_used, quality_score.unwrap_or(0.0), total_cost);
    }
    ExecutionResult::Failure { .. } => {
        println!("All tiers exhausted");
    }
}
```

**Key Features:**
- **Quality-based escalation**: Uses `HeuristicQualityEvaluator` with 4 checks
- **Cost tracking**: Accumulates costs across tier attempts in microdollars
- **Streaming guard**: Automatically disabled for streaming requests
- **Budget enforcement**: Optional max cost limit per execution

**Quality Evaluation Checks:**
1. **Completeness**: Response not truncated
2. **Length**: Meets minimum character threshold
3. **Structure**: Valid JSON when expected
4. **Coherence**: No error patterns or excessive repetition

See [docs/routing.md](../../docs/routing.md) for detailed comparison with Cost-Aware Routing.

## Migration Guide

### From Reactive to Proactive

**Before (Reactive Failover):**
```rust
// Try first account, fail over on error
let response = account.execute(request).await
    .or_else(|_| fallback_account.execute(request).await);
```

**After (Proactive Planning):**
```rust
// Plan first, then execute
let plan = planner.create_plan(ExecutionPlanType::Failover, context).await?;
let response = plan.execute().await?;
```

## Future Enhancements

- [x] **Cascading routing** with quality-based escalation (Issue #24)
- [x] **Cost-aware routing** with query complexity classification (Issue #23)
- [ ] Model-specific routing (e.g., code models to specialized providers)
- [ ] Cost-based optimization with real-time pricing
- [ ] Predictive health scoring using ML
- [ ] Multi-provider parallel execution
- [ ] Request deduplication across clients
