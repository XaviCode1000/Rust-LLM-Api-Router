---
name: gateway
description: "Skill for the Gateway area of Rust-LLM-Api-Router. 11 symbols across 2 files."
---

# Gateway

11 symbols | 2 files | Cohesion: 68%

## When to Use

- Working with code in `src/`
- Understanding how with_provider, build, with_config work
- Modifying gateway-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `src/infrastructure/gateway/llm_gateway.rs` | with_provider, build, with_config, providers, test_provider_config_builder (+1) |
| `tests/gateway_tests.rs` | test_provider_config_builder, test_provider_config_builder_overwrite, test_gateway_with_default_config, test_gateway_with_custom_config, test_gateway_with_multiple_custom_providers |

## Entry Points

Start here when exploring this area:

- **`with_provider`** (Function) — `src/infrastructure/gateway/llm_gateway.rs:80`
- **`build`** (Function) — `src/infrastructure/gateway/llm_gateway.rs:100`
- **`with_config`** (Function) — `src/infrastructure/gateway/llm_gateway.rs:213`
- **`providers`** (Function) — `src/infrastructure/gateway/llm_gateway.rs:229`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `with_provider` | Function | `src/infrastructure/gateway/llm_gateway.rs` | 80 |
| `build` | Function | `src/infrastructure/gateway/llm_gateway.rs` | 100 |
| `with_config` | Function | `src/infrastructure/gateway/llm_gateway.rs` | 213 |
| `providers` | Function | `src/infrastructure/gateway/llm_gateway.rs` | 229 |
| `test_provider_config_builder` | Function | `tests/gateway_tests.rs` | 29 |
| `test_provider_config_builder_overwrite` | Function | `tests/gateway_tests.rs` | 81 |
| `test_gateway_with_default_config` | Function | `tests/gateway_tests.rs` | 99 |
| `test_gateway_with_custom_config` | Function | `tests/gateway_tests.rs` | 115 |
| `test_gateway_with_multiple_custom_providers` | Function | `tests/gateway_tests.rs` | 137 |
| `test_provider_config_builder` | Function | `src/infrastructure/gateway/llm_gateway.rs` | 404 |
| `test_gateway_with_custom_config` | Function | `src/infrastructure/gateway/llm_gateway.rs` | 439 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Test_gateway_with_custom_config → New` | cross_community | 5 |
| `Test_gateway_with_custom_config → Is_available` | cross_community | 5 |
| `Test_gateway_with_custom_config → Cleanup_stale_temp_files` | cross_community | 4 |
| `Test_gateway_with_custom_config → Default_providers` | cross_community | 4 |
| `Test_gateway_with_custom_config → Default_providers` | cross_community | 4 |
| `Test_gateway_with_multiple_custom_providers → Default_providers` | cross_community | 4 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Tests | 6 calls |
| Handlers | 3 calls |
| Router | 1 calls |
| Persistence | 1 calls |

## How to Explore

1. `gitnexus_context({name: "with_provider"})` — see callers and callees
2. `gitnexus_query({query: "gateway"})` — find related execution flows
3. Read key files listed above for implementation details
