---
name: entities
description: "Skill for the Entities area of Rust-LLM-Api-Router. 97 symbols across 21 files."
---

# Entities

97 symbols | 21 files | Cohesion: 64%

## When to Use

- Working with code in `src/`
- Understanding how metrics_middleware, health, health_detail work
- Modifying entities-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `src/domain/entities/account_health_tests.rs` | test_record_failure_updates_counters, test_record_success_resets_consecutive_failures, test_circuit_breaker_opens_after_5_failures, test_circuit_breaker_blocks_requests_when_open, test_circuit_breaker_half_open_after_30_seconds (+17) |
| `src/domain/entities/account_health.rs` | record_failure, success_rate, record_success, update_avg_latency, health_score (+17) |
| `src/domain/entities/account.rs` | new_oauth, is_token_expired, now, test_account_new_oauth, test_account_get_access_token_prefers_oauth (+8) |
| `src/domain/entities/provider.rs` | new, with_oauth, with_device_auth_url, test_provider_new, test_provider_with_oauth (+2) |
| `src/app/services/account_rotation_tests.rs` | create_account_with_latency, create_account_with_circuit_breaker, test_latency_strategy_selects_lowest_latency, test_latency_strategy_excludes_open_circuit_breaker, test_round_robin_high_concurrency |
| `benches/execution_plan_benchmarks.rs` | benchmark_config_builder, benchmark_config_presets, benchmark_context_creation |
| `src/interfaces/handlers/health_handler.rs` | health, health_detail, current_timestamp |
| `src/app/services/execution_plan/planner.rs` | apply_rotation_strategy, filter_accounts, is_model_compatible |
| `src/domain/entities/mod.rs` | with_temperature, with_max_tokens, with_stream |
| `src/domain/entities/openai_types.rs` | new, current_timestamp |

## Entry Points

Start here when exploring this area:

- **`metrics_middleware`** (Function) — `src/interfaces/middleware/mod.rs:74`
- **`health`** (Function) — `src/interfaces/handlers/health_handler.rs:46`
- **`health_detail`** (Function) — `src/interfaces/handlers/health_handler.rs:56`
- **`new`** (Function) — `src/domain/entities/openai_types.rs:53`
- **`current_timestamp`** (Function) — `src/domain/entities/openai_types.rs:253`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `metrics_middleware` | Function | `src/interfaces/middleware/mod.rs` | 74 |
| `health` | Function | `src/interfaces/handlers/health_handler.rs` | 46 |
| `health_detail` | Function | `src/interfaces/handlers/health_handler.rs` | 56 |
| `new` | Function | `src/domain/entities/openai_types.rs` | 53 |
| `current_timestamp` | Function | `src/domain/entities/openai_types.rs` | 253 |
| `new_oauth` | Function | `src/domain/entities/account.rs` | 77 |
| `is_token_expired` | Function | `src/domain/entities/account.rs` | 151 |
| `new` | Function | `src/app/services/execution_plan/tracing.rs` | 35 |
| `log` | Function | `src/app/services/execution_plan/tracing.rs` | 72 |
| `total_weight` | Function | `src/app/services/execution_plan/implementations.rs` | 290 |
| `select_by_weight` | Function | `src/app/services/execution_plan/implementations.rs` | 295 |
| `record_failure` | Function | `src/domain/entities/account_health.rs` | 133 |
| `success_rate` | Function | `src/domain/entities/account_health.rs` | 227 |
| `success_rate` | Function | `src/app/services/execution_plan/types.rs` | 164 |
| `record_success` | Function | `src/domain/entities/account_health.rs` | 102 |
| `health_score` | Function | `src/domain/entities/account_health.rs` | 195 |
| `new` | Function | `src/domain/entities/account_health.rs` | 77 |
| `is_degraded` | Function | `src/domain/entities/account_health.rs` | 21 |
| `can_make_request` | Function | `src/domain/entities/account_health.rs` | 155 |
| `new` | Function | `src/domain/entities/account.rs` | 46 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Benchmark_health_tracking → Is_open` | cross_community | 6 |
| `Benchmark_health_tracking → Now` | cross_community | 6 |
| `Test_execute_with_failover_success_first_attempt → Is_open` | cross_community | 6 |
| `Test_execute_with_failover_success_first_attempt → Now` | cross_community | 6 |
| `Test_execute_with_failover_all_fail → Is_open` | cross_community | 6 |
| `Test_execute_with_failover_all_fail → Now` | cross_community | 6 |
| `Test_circuit_breaker_blocks_failed_account → Is_open` | cross_community | 6 |
| `Test_circuit_breaker_blocks_failed_account → Now` | cross_community | 6 |
| `Test_concurrent_health_map_access → Is_open` | cross_community | 6 |
| `Test_concurrent_health_map_access → Now` | cross_community | 6 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Services | 4 calls |
| Tests | 3 calls |
| Execution_plan | 1 calls |

## How to Explore

1. `gitnexus_context({name: "metrics_middleware"})` — see callers and callees
2. `gitnexus_query({query: "entities"})` — find related execution flows
3. Read key files listed above for implementation details
