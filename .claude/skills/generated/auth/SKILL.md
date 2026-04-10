---
name: auth
description: "Skill for the Auth area of Rust-LLM-Api-Router. 31 symbols across 5 files."
---

# Auth

31 symbols | 5 files | Cohesion: 81%

## When to Use

- Working with code in `src/`
- Understanding how new, is_oauth_configured, is_device_flow_configured work
- Modifying auth-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `src/app/services/auth/service.rs` | get_auth_strategy, refresh_token, revoke_token, find_enabled_by_id, test_auth_service_refresh_token_success (+7) |
| `src/infrastructure/auth/api_key_strategy.rs` | new, initiate_auth, complete_auth, refresh_token, revoke_token (+6) |
| `src/infrastructure/auth/pkce_strategy.rs` | new, test_pkce_strategy_new, test_pkce_strategy_auth_type |
| `src/infrastructure/auth/device_flow_strategy.rs` | new, test_device_flow_strategy_new, test_device_flow_strategy_auth_type |
| `src/domain/entities/provider.rs` | is_oauth_configured, is_device_flow_configured |

## Entry Points

Start here when exploring this area:

- **`new`** (Function) — `src/infrastructure/auth/api_key_strategy.rs:21`
- **`is_oauth_configured`** (Function) — `src/domain/entities/provider.rs:98`
- **`is_device_flow_configured`** (Function) — `src/domain/entities/provider.rs:107`
- **`refresh_token`** (Function) — `src/app/services/auth/service.rs:141`
- **`revoke_token`** (Function) — `src/app/services/auth/service.rs:210`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `new` | Function | `src/infrastructure/auth/api_key_strategy.rs` | 21 |
| `is_oauth_configured` | Function | `src/domain/entities/provider.rs` | 98 |
| `is_device_flow_configured` | Function | `src/domain/entities/provider.rs` | 107 |
| `refresh_token` | Function | `src/app/services/auth/service.rs` | 141 |
| `revoke_token` | Function | `src/app/services/auth/service.rs` | 210 |
| `new` | Function | `src/app/services/auth/service.rs` | 24 |
| `initiate_auth` | Function | `src/app/services/auth/service.rs` | 106 |
| `complete_auth` | Function | `src/app/services/auth/service.rs` | 119 |
| `new` | Function | `src/infrastructure/auth/pkce_strategy.rs` | 27 |
| `new` | Function | `src/infrastructure/auth/device_flow_strategy.rs` | 30 |
| `initiate_auth` | Function | `src/infrastructure/auth/api_key_strategy.rs` | 37 |
| `complete_auth` | Function | `src/infrastructure/auth/api_key_strategy.rs` | 49 |
| `refresh_token` | Function | `src/infrastructure/auth/api_key_strategy.rs` | 72 |
| `revoke_token` | Function | `src/infrastructure/auth/api_key_strategy.rs` | 86 |
| `test_api_key_strategy_initiate_auth` | Function | `src/infrastructure/auth/api_key_strategy.rs` | 105 |
| `test_api_key_strategy_complete_auth_success` | Function | `src/infrastructure/auth/api_key_strategy.rs` | 113 |
| `test_api_key_strategy_complete_auth_empty_key` | Function | `src/infrastructure/auth/api_key_strategy.rs` | 126 |
| `test_api_key_strategy_refresh_token` | Function | `src/infrastructure/auth/api_key_strategy.rs` | 134 |
| `test_api_key_strategy_revoke_token` | Function | `src/infrastructure/auth/api_key_strategy.rs` | 143 |
| `test_api_key_strategy_auth_type` | Function | `src/infrastructure/auth/api_key_strategy.rs` | 151 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Handle_command → Find_enabled_by_id` | cross_community | 5 |
| `Handle_command → Is_oauth_configured` | cross_community | 5 |
| `Test_auth_service_initiate_auth_success → Find_enabled_by_id` | cross_community | 4 |
| `Test_auth_service_initiate_auth_success → Is_oauth_configured` | cross_community | 4 |
| `Test_auth_service_complete_auth_success → Find_enabled_by_id` | cross_community | 4 |
| `Test_auth_service_complete_auth_success → Is_oauth_configured` | cross_community | 4 |
| `Test_auth_service_complete_auth_success → Now` | cross_community | 4 |
| `Test_auth_service_refresh_token_success → Now` | cross_community | 3 |
| `Test_auth_service_revoke_token_success → Now` | cross_community | 3 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Entities | 6 calls |

## How to Explore

1. `gitnexus_context({name: "new"})` — see callers and callees
2. `gitnexus_query({query: "auth"})` — find related execution flows
3. Read key files listed above for implementation details
