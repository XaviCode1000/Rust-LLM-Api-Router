---
name: router
description: "Skill for the Router area of Rust-LLM-Api-Router. 10 symbols across 4 files."
---

# Router

10 symbols | 4 files | Cohesion: 76%

## When to Use

- Working with code in `src/`
- Understanding how new, new, with_config work
- Modifying router-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `src/app/router/llm_router.rs` | default, new, with_config, with_routing_config, test_llm_router_config_defaults (+2) |
| `tests/gateway_tests.rs` | test_gateway_cache_ttl_configuration |
| `src/presentation/state.rs` | new |
| `src/app/services/execution_plan/planner.rs` | from_routing_config |

## Entry Points

Start here when exploring this area:

- **`new`** (Function) — `src/presentation/state.rs:26`
- **`new`** (Function) — `src/app/router/llm_router.rs:78`
- **`with_config`** (Function) — `src/app/router/llm_router.rs:97`
- **`with_routing_config`** (Function) — `src/app/router/llm_router.rs:118`
- **`from_routing_config`** (Function) — `src/app/services/execution_plan/planner.rs:93`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `new` | Function | `src/presentation/state.rs` | 26 |
| `new` | Function | `src/app/router/llm_router.rs` | 78 |
| `with_config` | Function | `src/app/router/llm_router.rs` | 97 |
| `with_routing_config` | Function | `src/app/router/llm_router.rs` | 118 |
| `from_routing_config` | Function | `src/app/services/execution_plan/planner.rs` | 93 |
| `test_gateway_cache_ttl_configuration` | Function | `tests/gateway_tests.rs` | 163 |
| `default` | Function | `src/app/router/llm_router.rs` | 42 |
| `test_llm_router_config_defaults` | Function | `src/app/router/llm_router.rs` | 508 |
| `infer_provider_from_model_static` | Function | `src/app/router/llm_router.rs` | 536 |
| `infer_provider_static` | Function | `src/app/router/llm_router.rs` | 560 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Test_list_models_no_accounts → With_config` | cross_community | 4 |
| `Test_list_models_no_accounts → Default` | cross_community | 4 |
| `Test_chat_handler_no_active_accounts → With_config` | cross_community | 4 |
| `Test_chat_handler_no_active_accounts → Default` | cross_community | 4 |
| `Test_chat_handler_round_robin_selection → With_config` | cross_community | 4 |
| `Test_chat_handler_round_robin_selection → Default` | cross_community | 4 |
| `Setup_health_app_with_accounts → With_config` | cross_community | 4 |
| `Setup_health_app_with_accounts → Default` | cross_community | 4 |
| `Route_request → Default` | cross_community | 3 |
| `New → New` | cross_community | 3 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Tests | 2 calls |

## How to Explore

1. `gitnexus_context({name: "new"})` — see callers and callees
2. `gitnexus_query({query: "router"})` — find related execution flows
3. Read key files listed above for implementation details
