---
name: cli
description: "Skill for the Cli area of Rust-LLM-Api-Router. 21 symbols across 10 files."
---

# Cli

21 symbols | 10 files | Cohesion: 68%

## When to Use

- Working with code in `src/`
- Understanding how should_use_color, success, error work
- Modifying cli-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `src/presentation/cli/output.rs` | success, error, info, dim, bold |
| `src/presentation/cli/table.rs` | account_table, mask_api_key, test_account_table_empty, provider_table, test_provider_table_empty |
| `src/presentation/cli/prompt.rs` | confirm, prompt_secret, prompt_text |
| `src/presentation/cli/tty.rs` | should_use_color, is_tty |
| `src/presentation/cli/commands/provider.rs` | cmd_list_models |
| `src/presentation/cli/commands/logout.rs` | handle_logout_command |
| `src/presentation/cli/commands/login.rs` | handle_login_command |
| `src/presentation/cli/commands/auth.rs` | handle_auth_command |
| `src/presentation/cli/spinner.rs` | new |
| `src/presentation/cli/input.rs` | read_api_key_interactive |

## Entry Points

Start here when exploring this area:

- **`should_use_color`** (Function) — `src/presentation/cli/tty.rs:28`
- **`success`** (Function) — `src/presentation/cli/output.rs:13`
- **`error`** (Function) — `src/presentation/cli/output.rs:22`
- **`info`** (Function) — `src/presentation/cli/output.rs:40`
- **`dim`** (Function) — `src/presentation/cli/output.rs:49`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `should_use_color` | Function | `src/presentation/cli/tty.rs` | 28 |
| `success` | Function | `src/presentation/cli/output.rs` | 13 |
| `error` | Function | `src/presentation/cli/output.rs` | 22 |
| `info` | Function | `src/presentation/cli/output.rs` | 40 |
| `dim` | Function | `src/presentation/cli/output.rs` | 49 |
| `bold` | Function | `src/presentation/cli/output.rs` | 59 |
| `cmd_list_models` | Function | `src/presentation/cli/commands/provider.rs` | 197 |
| `handle_logout_command` | Function | `src/presentation/cli/commands/logout.rs` | 8 |
| `handle_login_command` | Function | `src/presentation/cli/commands/login.rs` | 9 |
| `handle_auth_command` | Function | `src/presentation/cli/commands/auth.rs` | 23 |
| `is_tty` | Function | `src/presentation/cli/tty.rs` | 18 |
| `new` | Function | `src/presentation/cli/spinner.rs` | 20 |
| `confirm` | Function | `src/presentation/cli/prompt.rs` | 12 |
| `prompt_secret` | Function | `src/presentation/cli/prompt.rs` | 27 |
| `prompt_text` | Function | `src/presentation/cli/prompt.rs` | 41 |
| `read_api_key_interactive` | Function | `src/presentation/cli/input.rs` | 8 |
| `account_table` | Function | `src/presentation/cli/table.rs` | 43 |
| `provider_table` | Function | `src/presentation/cli/table.rs` | 11 |
| `mask_api_key` | Function | `src/presentation/cli/table.rs` | 85 |
| `test_account_table_empty` | Function | `src/presentation/cli/table.rs` | 132 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Handle_command → Is_tty` | cross_community | 6 |
| `Handle_command → New` | cross_community | 5 |
| `Handle_command → Find_enabled_by_id` | cross_community | 5 |
| `Handle_command → Is_oauth_configured` | cross_community | 5 |
| `Handle_provider_command → Is_tty` | cross_community | 5 |
| `Test_cli_provider_crud_complete_workflow → Is_tty` | cross_community | 5 |
| `Test_cli_multiple_providers_workflow → Is_tty` | cross_community | 5 |
| `Test_cli_full_workflow → Is_tty` | cross_community | 5 |
| `Handle_account_command → Is_tty` | cross_community | 5 |
| `Test_cli_enable_disable_provider_workflow → Is_tty` | cross_community | 5 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Auth | 3 calls |
| Provider | 1 calls |
| Tests | 1 calls |
| Handlers | 1 calls |

## How to Explore

1. `gitnexus_context({name: "should_use_color"})` — see callers and callees
2. `gitnexus_query({query: "cli"})` — find related execution flows
3. Read key files listed above for implementation details
