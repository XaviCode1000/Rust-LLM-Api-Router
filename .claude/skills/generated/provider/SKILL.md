---
name: provider
description: "Skill for the Provider area of Rust-LLM-Api-Router. 12 symbols across 11 files."
---

# Provider

12 symbols | 11 files | Cohesion: 67%

## When to Use

- Working with code in `src/`
- Understanding how client, logging_middleware, chat work
- Modifying provider-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `src/interfaces/handlers/chat_handler.rs` | make_provider_request, make_streaming_provider_request |
| `tests/mock_http_tests.rs` | test_concurrent_requests_to_mock_servers |
| `src/infrastructure/http_client.rs` | client |
| `src/interfaces/middleware/mod.rs` | logging_middleware |
| `src/infrastructure/provider/openai.rs` | chat |
| `src/infrastructure/provider/groq.rs` | chat |
| `src/infrastructure/provider/anthropic.rs` | chat |
| `src/infrastructure/gateway/llm_gateway.rs` | fetch_provider_models |
| `src/domain/entities/account.rs` | get_access_token |
| `src/app/router/llm_router.rs` | forward_to_provider |

## Entry Points

Start here when exploring this area:

- **`client`** (Function) — `src/infrastructure/http_client.rs:42`
- **`logging_middleware`** (Function) — `src/interfaces/middleware/mod.rs:11`
- **`chat`** (Function) — `src/infrastructure/provider/openai.rs:34`
- **`chat`** (Function) — `src/infrastructure/provider/groq.rs:35`
- **`chat`** (Function) — `src/infrastructure/provider/anthropic.rs:92`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `client` | Function | `src/infrastructure/http_client.rs` | 42 |
| `logging_middleware` | Function | `src/interfaces/middleware/mod.rs` | 11 |
| `chat` | Function | `src/infrastructure/provider/openai.rs` | 34 |
| `chat` | Function | `src/infrastructure/provider/groq.rs` | 35 |
| `chat` | Function | `src/infrastructure/provider/anthropic.rs` | 92 |
| `get_access_token` | Function | `src/domain/entities/account.rs` | 166 |
| `is_success` | Function | `src/app/services/execution_plan/outcome.rs` | 21 |
| `test_concurrent_requests_to_mock_servers` | Function | `tests/mock_http_tests.rs` | 89 |
| `make_provider_request` | Function | `src/interfaces/handlers/chat_handler.rs` | 256 |
| `make_streaming_provider_request` | Function | `src/interfaces/handlers/chat_handler.rs` | 317 |
| `fetch_provider_models` | Function | `src/infrastructure/gateway/llm_gateway.rs` | 234 |
| `forward_to_provider` | Function | `src/app/router/llm_router.rs` | 348 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Handle_command → Is_success` | cross_community | 4 |
| `Make_provider_request → Mock_base_url` | cross_community | 3 |
| `Make_streaming_provider_request → Mock_base_url` | cross_community | 3 |
| `Test_remove_provider_success → Is_success` | cross_community | 3 |
| `Test_remove_provider_from_multiple → Is_success` | cross_community | 3 |
| `Test_enable_provider_success → Is_success` | cross_community | 3 |
| `Test_enable_already_enabled_provider → Is_success` | cross_community | 3 |
| `Test_disable_provider_success → Is_success` | cross_community | 3 |
| `Test_disable_already_disabled_provider → Is_success` | cross_community | 3 |
| `Test_validate_provider_success → Is_success` | cross_community | 3 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Handlers | 4 calls |
| Entities | 1 calls |

## How to Explore

1. `gitnexus_context({name: "client"})` — see callers and callees
2. `gitnexus_query({query: "provider"})` — find related execution flows
3. Read key files listed above for implementation details
