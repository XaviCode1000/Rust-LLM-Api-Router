---
name: persistence
description: "Skill for the Persistence area of Rust-LLM-Api-Router. 40 symbols across 4 files."
---

# Persistence

40 symbols | 4 files | Cohesion: 86%

## When to Use

- Working with code in `src/`
- Understanding how new, with_config_dir, new work
- Modifying persistence-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `src/infrastructure/persistence/json_account_repository.rs` | new, cleanup_stale_temp_files, with_config_dir, write_accounts, save (+14) |
| `src/infrastructure/persistence/json_provider_repository.rs` | new, with_config_dir, ensure_file_exists, read_providers, write_providers (+8) |
| `src/infrastructure/persistence/json_repository_tests.rs` | create_temp_repository, test_save_and_find_account, test_find_non_existent_account, test_invalid_json_file, test_api_key_not_in_errors (+1) |
| `src/infrastructure/secure_storage/mod.rs` | store, retrieve |

## Entry Points

Start here when exploring this area:

- **`new`** (Function) — `src/infrastructure/persistence/json_provider_repository.rs:87`
- **`with_config_dir`** (Function) — `src/infrastructure/persistence/json_provider_repository.rs:112`
- **`new`** (Function) — `src/infrastructure/persistence/json_account_repository.rs:119`
- **`with_config_dir`** (Function) — `src/infrastructure/persistence/json_account_repository.rs:167`
- **`migrate_plaintext_keys`** (Function) — `src/infrastructure/persistence/json_account_repository.rs:201`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `new` | Function | `src/infrastructure/persistence/json_provider_repository.rs` | 87 |
| `with_config_dir` | Function | `src/infrastructure/persistence/json_provider_repository.rs` | 112 |
| `new` | Function | `src/infrastructure/persistence/json_account_repository.rs` | 119 |
| `with_config_dir` | Function | `src/infrastructure/persistence/json_account_repository.rs` | 167 |
| `migrate_plaintext_keys` | Function | `src/infrastructure/persistence/json_account_repository.rs` | 201 |
| `ensure_file_exists` | Function | `src/infrastructure/persistence/json_provider_repository.rs` | 118 |
| `read_providers` | Function | `src/infrastructure/persistence/json_provider_repository.rs` | 135 |
| `write_providers` | Function | `src/infrastructure/persistence/json_provider_repository.rs` | 156 |
| `default` | Function | `src/infrastructure/persistence/json_provider_repository.rs` | 171 |
| `save` | Function | `src/infrastructure/persistence/json_provider_repository.rs` | 180 |
| `find_all` | Function | `src/infrastructure/persistence/json_provider_repository.rs` | 194 |
| `find_by_id` | Function | `src/infrastructure/persistence/json_provider_repository.rs` | 199 |
| `find_enabled_by_id` | Function | `src/infrastructure/persistence/json_provider_repository.rs` | 208 |
| `delete` | Function | `src/infrastructure/persistence/json_provider_repository.rs` | 217 |
| `test_delete_provider_persists` | Function | `src/infrastructure/persistence/json_provider_repository.rs` | 242 |
| `test_delete_non_existent_provider` | Function | `src/infrastructure/persistence/json_provider_repository.rs` | 264 |
| `cleanup_stale_temp_files` | Function | `src/infrastructure/persistence/json_account_repository.rs` | 148 |
| `write_accounts` | Function | `src/infrastructure/persistence/json_account_repository.rs` | 284 |
| `save` | Function | `src/infrastructure/persistence/json_account_repository.rs` | 339 |
| `delete` | Function | `src/infrastructure/persistence/json_account_repository.rs` | 420 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Migrate_plaintext_keys → New` | cross_community | 6 |
| `Migrate_plaintext_keys → Is_available` | cross_community | 6 |
| `Test_gateway_with_custom_config → New` | cross_community | 5 |
| `Test_gateway_with_custom_config → Is_available` | cross_community | 5 |
| `Migrate_plaintext_keys → Cleanup_stale_temp_files` | cross_community | 5 |
| `Test_gateway_with_custom_config → Cleanup_stale_temp_files` | cross_community | 4 |
| `Test_delete_provider_persists → Ensure_file_exists` | intra_community | 4 |
| `Test_delete_provider_persists → New` | intra_community | 4 |
| `Test_delete_provider_persists → From_str` | cross_community | 4 |
| `Test_delete_account_persists → New` | cross_community | 4 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Services | 2 calls |
| Secure_storage | 1 calls |

## How to Explore

1. `gitnexus_context({name: "new"})` — see callers and callees
2. `gitnexus_query({query: "persistence"})` — find related execution flows
3. Read key files listed above for implementation details
