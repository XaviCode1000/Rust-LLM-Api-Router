---
name: execution-plan
description: "Skill for the Execution_plan area of Rust-LLM-Api-Router. 180 symbols across 14 files."
---

# Execution_plan

180 symbols | 14 files | Cohesion: 81%

## When to Use

- Working with code in `src/`
- Understanding how new, record_plan_created, record_plan_started work
- Modifying execution_plan-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `src/app/services/execution_plan/plan.rs` | context, update_status, set_outcome, create_test_context, create_test_plan (+26) |
| `src/app/services/execution_plan/cascading.rs` | new, test_cascading_tier_new, current_tier, escalate_to_next_tier, total_cost_microdollars (+20) |
| `src/app/services/execution_plan/implementations.rs` | failover_chain, cheapest_provider, create_test_context, create_test_accounts, test_standard_execution_plan (+20) |
| `src/app/services/execution_plan/metrics.rs` | new, record_plan_created, record_plan_started, record_plan_completed, record_planning_duration (+19) |
| `src/app/services/execution_plan/tracing.rs` | reason, option, selected, metadata, build (+17) |
| `src/app/services/execution_plan/planner.rs` | new, cost_optimized, build, with_default_config, test_execution_planner_select_plan_type_standard (+11) |
| `src/app/services/execution_plan/types.rs` | new, as_fallback, with_priority, with_execution_order, with_model_id (+7) |
| `src/app/services/execution_plan/context.rs` | is_account_preferred, new, default, test_execution_context_new, test_execution_context_preferred_providers (+5) |
| `src/app/services/execution_plan/execution.rs` | failure, success, test_execution_result_failure, test_execution_result_success, non_streaming (+3) |
| `src/domain/services/model_selector.rs` | test_model_pricing_estimate_cost, test_model_pricing_estimate_cost_fractional |

## Entry Points

Start here when exploring this area:

- **`new`** (Function) — `src/app/services/execution_plan/metrics.rs:52`
- **`record_plan_created`** (Function) — `src/app/services/execution_plan/metrics.rs:152`
- **`record_plan_started`** (Function) — `src/app/services/execution_plan/metrics.rs:157`
- **`record_plan_completed`** (Function) — `src/app/services/execution_plan/metrics.rs:162`
- **`record_planning_duration`** (Function) — `src/app/services/execution_plan/metrics.rs:167`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `new` | Function | `src/app/services/execution_plan/metrics.rs` | 52 |
| `record_plan_created` | Function | `src/app/services/execution_plan/metrics.rs` | 152 |
| `record_plan_started` | Function | `src/app/services/execution_plan/metrics.rs` | 157 |
| `record_plan_completed` | Function | `src/app/services/execution_plan/metrics.rs` | 162 |
| `record_planning_duration` | Function | `src/app/services/execution_plan/metrics.rs` | 167 |
| `record_plan_type` | Function | `src/app/services/execution_plan/metrics.rs` | 172 |
| `record_fallback_used` | Function | `src/app/services/execution_plan/metrics.rs` | 183 |
| `record_outcome` | Function | `src/app/services/execution_plan/metrics.rs` | 188 |
| `record_planning_error` | Function | `src/app/services/execution_plan/metrics.rs` | 198 |
| `estimate_cost` | Function | `src/domain/entities/mod.rs` | 196 |
| `new` | Function | `src/app/services/execution_plan/types.rs` | 106 |
| `as_fallback` | Function | `src/app/services/execution_plan/types.rs` | 129 |
| `with_priority` | Function | `src/app/services/execution_plan/types.rs` | 136 |
| `with_execution_order` | Function | `src/app/services/execution_plan/types.rs` | 142 |
| `with_model_id` | Function | `src/app/services/execution_plan/types.rs` | 148 |
| `new` | Function | `src/app/services/execution_plan/cascading.rs` | 34 |
| `failover_chain` | Function | `src/app/services/execution_plan/implementations.rs` | 179 |
| `cheapest_provider` | Function | `src/app/services/execution_plan/implementations.rs` | 449 |
| `failure` | Function | `src/app/services/execution_plan/execution.rs` | 76 |
| `success` | Function | `src/app/services/execution_plan/execution.rs` | 90 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Test_cascading_current_tier → Health_score` | cross_community | 4 |
| `Test_cascading_cost_tracking → Health_score` | cross_community | 4 |
| `Execute → Health_score` | cross_community | 4 |
| `Is_primary_healthy → Is_open` | cross_community | 4 |
| `Is_primary_healthy → Health_score` | cross_community | 4 |
| `Build_cost_optimized → With_execution_order` | cross_community | 4 |
| `Build_cost_optimized → As_primary` | cross_community | 4 |
| `Build_cost_optimized → Estimate_cost` | intra_community | 4 |
| `Build_cascading → With_execution_order` | cross_community | 4 |
| `Build_cascading → As_primary` | cross_community | 4 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Entities | 4 calls |
| Tests | 4 calls |
| Provider | 1 calls |

## How to Explore

1. `gitnexus_context({name: "new"})` — see callers and callees
2. `gitnexus_query({query: "execution_plan"})` — find related execution flows
3. Read key files listed above for implementation details
