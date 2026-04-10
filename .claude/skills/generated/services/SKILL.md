---
name: services
description: "Skill for the Services area of Rust-LLM-Api-Router. 136 symbols across 14 files."
---

# Services

136 symbols | 14 files | Cohesion: 86%

## When to Use

- Working with code in `src/`
- Understanding how new, new, with_max_cost work
- Modifying services-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `src/domain/services/query_complexity.rs` | default, new, test_classify_short_greeting_is_low, test_classify_simple_question_is_low, test_classify_empty_user_messages_is_low (+29) |
| `src/app/services/account_rotation_tests.rs` | test_rate_limit_info_parses_remaining, test_rate_limit_info_parses_limit, test_rate_limit_info_parses_reset, test_rate_limit_info_parses_all_headers, test_rate_limit_info_case_insensitive (+26) |
| `src/domain/services/model_selector.rs` | new, with_max_cost, tier_price, capability_tier, default (+22) |
| `src/app/services/account_rotation.rs` | from_headers, create_test_health, test_latency_strategy_selects_lowest_latency, test_latency_strategy_excludes_circuit_breaker_open, test_latency_strategy_excludes_no_quota_accounts (+11) |
| `src/domain/services/token_validator.rs` | count_tokens, validate, extract_model_name, test_count_tokens_simple, test_count_tokens_conversation (+4) |
| `src/app/services/failover.rs` | update_rate_limits, test_backoff_exponential_increase, test_backoff_max_delay_capped, test_backoff_jitter_variation, new (+3) |
| `src/domain/entities/account_health.rs` | test_rate_limit_parsing_remaining, test_rate_limit_parsing_empty_headers, test_rate_limit_parsing_case_insensitive, test_rate_limit_parsing_invalid_values |
| `tests/security_tests.rs` | test_mutex_poisoning_recovery |
| `src/domain/services/model_context_limits.rs` | get_context_limit |
| `src/config/routing.rs` | from_str |

## Entry Points

Start here when exploring this area:

- **`new`** (Function) — `src/domain/services/query_complexity.rs:233`
- **`new`** (Function) — `src/domain/services/model_selector.rs:102`
- **`with_max_cost`** (Function) — `src/domain/services/model_selector.rs:123`
- **`from_headers`** (Function) — `src/app/services/account_rotation.rs:213`
- **`select_with_health`** (Function) — `src/app/services/account_rotation.rs:351`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `new` | Function | `src/domain/services/query_complexity.rs` | 233 |
| `new` | Function | `src/domain/services/model_selector.rs` | 102 |
| `with_max_cost` | Function | `src/domain/services/model_selector.rs` | 123 |
| `from_headers` | Function | `src/app/services/account_rotation.rs` | 213 |
| `select_with_health` | Function | `src/app/services/account_rotation.rs` | 351 |
| `select_for_user` | Function | `src/app/services/account_rotation.rs` | 428 |
| `count_tokens` | Function | `src/domain/services/token_validator.rs` | 18 |
| `validate` | Function | `src/domain/services/token_validator.rs` | 45 |
| `get_context_limit` | Function | `src/domain/services/model_context_limits.rs` | 64 |
| `with_pricing` | Function | `src/domain/entities/mod.rs` | 232 |
| `calculate_delay` | Function | `src/app/services/account_rotation.rs` | 72 |
| `classify` | Function | `src/domain/services/query_complexity.rs` | 247 |
| `classify_task` | Function | `src/domain/services/query_complexity.rs` | 303 |
| `classify_full` | Function | `src/domain/services/query_complexity.rs` | 335 |
| `weighted` | Function | `src/app/services/account_rotation.rs` | 506 |
| `latency_based` | Function | `src/app/services/account_rotation.rs` | 514 |
| `user_affinity` | Function | `src/app/services/account_rotation.rs` | 522 |
| `new` | Function | `src/app/services/failover.rs` | 55 |
| `with_weighted` | Function | `src/app/services/failover.rs` | 111 |
| `with_latency_based` | Function | `src/app/services/failover.rs` | 119 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Benchmark_health_tracking → Is_open` | cross_community | 6 |
| `Test_execute_with_failover_success_first_attempt → Is_open` | cross_community | 6 |
| `Test_execute_with_failover_all_fail → Is_open` | cross_community | 6 |
| `Test_circuit_breaker_blocks_failed_account → Is_open` | cross_community | 6 |
| `Test_concurrent_health_map_access → Is_open` | cross_community | 6 |
| `Test_memory_bounded_health_tracking → Is_open` | cross_community | 6 |
| `Test_circuit_breaker_prevents_dos → Is_open` | cross_community | 6 |
| `Test_circuit_breaker_timeout → Is_open` | cross_community | 6 |
| `Test_health_tracking_concurrent → Is_open` | cross_community | 6 |
| `Test_multi_provider_isolation → Is_open` | cross_community | 6 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Entities | 13 calls |
| Tests | 6 calls |

## How to Explore

1. `gitnexus_context({name: "new"})` — see callers and callees
2. `gitnexus_query({query: "services"})` — find related execution flows
3. Read key files listed above for implementation details
