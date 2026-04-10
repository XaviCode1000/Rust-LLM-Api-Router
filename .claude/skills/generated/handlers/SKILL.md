---
name: handlers
description: "Skill for the Handlers area of Rust-LLM-Api-Router. 18 symbols across 4 files."
---

# Handlers

18 symbols | 4 files | Cohesion: 64%

## When to Use

- Working with code in `src/`
- Understanding how mock_base_url, new, default_with_url work
- Modifying handlers-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `src/interfaces/handlers/chat_handler.rs` | get_provider_base_url, test_get_provider_base_url_uses_mock_url, test_get_provider_base_url_uses_provider_config, test_get_provider_base_url_fallback_hardcoded, test_get_provider_base_url_unknown_provider (+6) |
| `src/infrastructure/gateway/llm_gateway.rs` | new, default_with_url, parse_models_response, test_provider_config_default_with_url, test_gateway_default_config |
| `tests/gateway_tests.rs` | test_provider_config_default_with_url |
| `src/infrastructure/http_client.rs` | mock_base_url |

## Entry Points

Start here when exploring this area:

- **`mock_base_url`** (Function) — `src/infrastructure/http_client.rs:47`
- **`new`** (Function) — `src/infrastructure/gateway/llm_gateway.rs:43`
- **`default_with_url`** (Function) — `src/infrastructure/gateway/llm_gateway.rs:58`
- **`chat_completions`** (Function) — `src/interfaces/handlers/chat_handler.rs:33`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `mock_base_url` | Function | `src/infrastructure/http_client.rs` | 47 |
| `new` | Function | `src/infrastructure/gateway/llm_gateway.rs` | 43 |
| `default_with_url` | Function | `src/infrastructure/gateway/llm_gateway.rs` | 58 |
| `chat_completions` | Function | `src/interfaces/handlers/chat_handler.rs` | 33 |
| `test_provider_config_default_with_url` | Function | `tests/gateway_tests.rs` | 63 |
| `get_provider_base_url` | Function | `src/interfaces/handlers/chat_handler.rs` | 224 |
| `test_get_provider_base_url_uses_mock_url` | Function | `src/interfaces/handlers/chat_handler.rs` | 527 |
| `test_get_provider_base_url_uses_provider_config` | Function | `src/interfaces/handlers/chat_handler.rs` | 538 |
| `test_get_provider_base_url_fallback_hardcoded` | Function | `src/interfaces/handlers/chat_handler.rs` | 552 |
| `test_get_provider_base_url_unknown_provider` | Function | `src/interfaces/handlers/chat_handler.rs` | 563 |
| `parse_models_response` | Function | `src/infrastructure/gateway/llm_gateway.rs` | 284 |
| `test_provider_config_default_with_url` | Function | `src/infrastructure/gateway/llm_gateway.rs` | 429 |
| `test_gateway_default_config` | Function | `src/infrastructure/gateway/llm_gateway.rs` | 456 |
| `stream_chat_request` | Function | `src/interfaces/handlers/chat_handler.rs` | 52 |
| `error_response` | Function | `src/interfaces/handlers/chat_handler.rs` | 129 |
| `process_chat_request` | Function | `src/interfaces/handlers/chat_handler.rs` | 134 |
| `select_account` | Function | `src/interfaces/handlers/chat_handler.rs` | 192 |
| `stream_to_sse_events` | Function | `src/interfaces/handlers/chat_handler.rs` | 379 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Test_get_provider_base_url_uses_mock_url → Default_providers` | cross_community | 6 |
| `Test_streaming_with_empty_chunk → Default_providers` | cross_community | 6 |
| `Test_streaming_with_valid_utf8_handling → Default_providers` | cross_community | 6 |
| `Test_chat_handler_no_active_accounts → Default_providers` | cross_community | 6 |
| `Test_chat_handler_round_robin_selection → Default_providers` | cross_community | 6 |
| `Handle_command → New` | cross_community | 5 |
| `Test_openai_contract → Default_providers` | cross_community | 5 |
| `Test_anthropic_contract → Default_providers` | cross_community | 5 |
| `Test_groq_contract → Default_providers` | cross_community | 5 |
| `Chat_completions → Message` | cross_community | 4 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Tests | 5 calls |
| Entities | 2 calls |
| Provider | 1 calls |
| Persistence | 1 calls |

## How to Explore

1. `gitnexus_context({name: "mock_base_url"})` — see callers and callees
2. `gitnexus_query({query: "handlers"})` — find related execution flows
3. Read key files listed above for implementation details
