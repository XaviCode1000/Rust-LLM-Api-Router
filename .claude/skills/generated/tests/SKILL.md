---
name: tests
description: "Skill for the Tests area of Rust-LLM-Api-Router. 522 symbols across 60 files."
---

# Tests

522 symbols | 60 files | Cohesion: 76%

## When to Use

- Working with code in `tests/`
- Understanding how create_repo_with_account, create_repo_with_accounts, create_manager_with_provider work
- Modifying tests-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `tests/chat_handler_wiremock_tests.rs` | setup_test_app_with_mock_provider, create_success_mock_response, create_error_mock_response, test_chat_handler_success_with_mock_provider, test_chat_handler_success_with_temperature_parameter (+24) |
| `tests/provider_commands_tests.rs` | setup_test_environment, create_add_provider_args, handle_provider_command_with_dir, test_add_provider_success, test_add_provider_without_api_key (+21) |
| `tests/provider_commands_integration_tests.rs` | create_test_repo, create_add_args, test_cli_add_provider_with_real_validation, test_cli_add_provider_url_validation, test_cli_add_provider_without_api_key (+19) |
| `tests/account_commands_tests.rs` | setup_test_environment, create_add_account_args, handle_account_command_with_dir, test_add_account_success, test_add_account_without_api_key (+18) |
| `tests/cli_account_commands_tests.rs` | create_test_repo, test_cli_validate_account_success, test_cli_validate_account_empty_key, test_cli_validate_account_short_key, test_cli_validate_account_not_found (+18) |
| `tests/cli_provider_commands_tests.rs` | test_cli_list_providers_empty, test_cli_list_providers_with_data, test_cli_list_providers_displays_enabled_disabled, test_cli_enable_provider_success, test_cli_enable_provider_already_enabled (+17) |
| `tests/chat_handler_coverage_boost.rs` | create_empty_test_app_state, create_test_app_state, create_test_app_state_with_mock, setup_list_models_app, test_list_models_success (+16) |
| `tests/cli_account_commands_extended_tests.rs` | test_cmd_validate_account_valid_key, test_cmd_validate_account_short_key, test_cmd_validate_account_empty_key, test_cmd_validate_account_not_found, test_cmd_add_account_basic (+16) |
| `tests/chat_handler_full_integration_tests.rs` | create_test_app_state, setup_full_test_env, test_chat_handler_provider_503_failover, test_chat_handler_no_active_accounts, test_chat_handler_round_robin_selection (+14) |
| `tests/cascading_routing_e2e_tests.rs` | create_test_provider_pricing, create_test_accounts, create_test_context, test_streaming_prevents_cascading_concept, test_cost_budget_zero_means_unlimited (+12) |

## Entry Points

Start here when exploring this area:

- **`create_repo_with_account`** (Function) — `tests/common/mod.rs:59`
- **`create_repo_with_accounts`** (Function) — `tests/common/mod.rs:70`
- **`create_manager_with_provider`** (Function) — `tests/common/mod.rs:120`
- **`create_manager_with_multi_account`** (Function) — `tests/common/mod.rs:132`
- **`create_manager_with_retries`** (Function) — `tests/common/mod.rs:138`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `create_repo_with_account` | Function | `tests/common/mod.rs` | 59 |
| `create_repo_with_accounts` | Function | `tests/common/mod.rs` | 70 |
| `create_manager_with_provider` | Function | `tests/common/mod.rs` | 120 |
| `create_manager_with_multi_account` | Function | `tests/common/mod.rs` | 132 |
| `create_manager_with_retries` | Function | `tests/common/mod.rs` | 138 |
| `with_round_robin` | Function | `src/app/services/failover.rs` | 103 |
| `execute_with_failover` | Function | `src/app/services/failover.rs` | 173 |
| `get_all_health` | Function | `src/app/services/failover.rs` | 315 |
| `round_robin` | Function | `src/app/services/account_rotation.rs` | 498 |
| `create_context_with_provider` | Function | `tests/common/mod.rs` | 190 |
| `create_reliability_context` | Function | `tests/common/mod.rs` | 196 |
| `create_cost_optimized_context` | Function | `tests/common/mod.rs` | 202 |
| `create_low_latency_context` | Function | `tests/common/mod.rs` | 208 |
| `record_filter` | Function | `src/app/services/execution_plan/tracing.rs` | 210 |
| `error` | Function | `src/app/services/execution_plan/tracing.rs` | 235 |
| `log_account_filtering` | Function | `src/app/services/execution_plan/tracing.rs` | 333 |
| `create_plan` | Function | `src/app/services/execution_plan/planner.rs` | 347 |
| `record_planning_started` | Function | `src/app/services/execution_plan/metrics.rs` | 203 |
| `record_planning_completed` | Function | `src/app/services/execution_plan/metrics.rs` | 208 |
| `with_preferred_providers` | Function | `src/app/services/execution_plan/context.rs` | 50 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Handle_command → Is_tty` | cross_community | 6 |
| `Test_get_provider_base_url_uses_mock_url → Default_providers` | cross_community | 6 |
| `Benchmark_health_tracking → Is_open` | cross_community | 6 |
| `Benchmark_health_tracking → Now` | cross_community | 6 |
| `Test_streaming_with_empty_chunk → Default_providers` | cross_community | 6 |
| `Test_streaming_with_valid_utf8_handling → Default_providers` | cross_community | 6 |
| `Test_chat_handler_no_active_accounts → Default_providers` | cross_community | 6 |
| `Test_chat_handler_round_robin_selection → Default_providers` | cross_community | 6 |
| `Test_execute_with_failover_success_first_attempt → Is_open` | cross_community | 6 |
| `Test_execute_with_failover_success_first_attempt → Now` | cross_community | 6 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Cli | 24 calls |
| Entities | 22 calls |
| Execution_plan | 21 calls |
| Provider | 10 calls |
| Handlers | 5 calls |
| Services | 4 calls |
| Router | 3 calls |
| Config | 1 calls |

## How to Explore

1. `gitnexus_context({name: "create_repo_with_account"})` — see callers and callees
2. `gitnexus_query({query: "tests"})` — find related execution flows
3. Read key files listed above for implementation details
