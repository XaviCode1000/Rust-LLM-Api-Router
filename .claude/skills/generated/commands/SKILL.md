---
name: commands
description: "Skill for the Commands area of Rust-LLM-Api-Router. 4 symbols across 4 files."
---

# Commands

4 symbols | 4 files | Cohesion: 30%

## When to Use

- Working with code in `src/`
- Understanding how handle_command, handle_provider_command, handle_completions_command work
- Modifying commands-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `src/presentation/cli/mod.rs` | handle_command |
| `src/presentation/cli/commands/provider.rs` | handle_provider_command |
| `src/presentation/cli/commands/completions.rs` | handle_completions_command |
| `src/presentation/cli/commands/account.rs` | handle_account_command |

## Entry Points

Start here when exploring this area:

- **`handle_command`** (Function) — `src/presentation/cli/mod.rs:96`
- **`handle_provider_command`** (Function) — `src/presentation/cli/commands/provider.rs:113`
- **`handle_completions_command`** (Function) — `src/presentation/cli/commands/completions.rs:25`
- **`handle_account_command`** (Function) — `src/presentation/cli/commands/account.rs:93`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `handle_command` | Function | `src/presentation/cli/mod.rs` | 96 |
| `handle_provider_command` | Function | `src/presentation/cli/commands/provider.rs` | 113 |
| `handle_completions_command` | Function | `src/presentation/cli/commands/completions.rs` | 25 |
| `handle_account_command` | Function | `src/presentation/cli/commands/account.rs` | 93 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Handle_command → Is_tty` | cross_community | 6 |
| `Handle_command → Now` | cross_community | 5 |
| `Handle_command → New` | cross_community | 5 |
| `Handle_command → Find_enabled_by_id` | cross_community | 5 |
| `Handle_command → Is_oauth_configured` | cross_community | 5 |
| `Handle_provider_command → Is_tty` | cross_community | 5 |
| `Handle_account_command → Is_tty` | cross_community | 5 |
| `Handle_command → Is_success` | cross_community | 4 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Tests | 12 calls |
| Cli | 2 calls |

## How to Explore

1. `gitnexus_context({name: "handle_command"})` — see callers and callees
2. `gitnexus_query({query: "commands"})` — find related execution flows
3. Read key files listed above for implementation details
