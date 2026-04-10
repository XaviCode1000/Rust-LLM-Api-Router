---
name: config
description: "Skill for the Config area of Rust-LLM-Api-Router. 4 symbols across 1 files."
---

# Config

4 symbols | 1 files | Cohesion: 86%

## When to Use

- Working with code in `src/`
- Understanding how from_cli_and_env work
- Modifying config-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `src/config/routing.rs` | from_cli_and_env, test_routing_config_default_values, test_routing_config_cascading_enabled, test_routing_config_invalid_quality_threshold |

## Entry Points

Start here when exploring this area:

- **`from_cli_and_env`** (Function) — `src/config/routing.rs:85`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `from_cli_and_env` | Function | `src/config/routing.rs` | 85 |
| `test_routing_config_default_values` | Function | `src/config/routing.rs` | 208 |
| `test_routing_config_cascading_enabled` | Function | `src/config/routing.rs` | 223 |
| `test_routing_config_invalid_quality_threshold` | Function | `src/config/routing.rs` | 235 |

## How to Explore

1. `gitnexus_context({name: "from_cli_and_env"})` — see callers and callees
2. `gitnexus_query({query: "config"})` — find related execution flows
3. Read key files listed above for implementation details
