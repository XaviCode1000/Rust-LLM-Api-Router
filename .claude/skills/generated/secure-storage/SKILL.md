---
name: secure-storage
description: "Skill for the Secure_storage area of Rust-LLM-Api-Router. 17 symbols across 4 files."
---

# Secure_storage

17 symbols | 4 files | Cohesion: 91%

## When to Use

- Working with code in `src/`
- Understanding how create_secure_storage, new, new work
- Modifying secure_storage-related functionality

## Key Files

| File | Symbols |
|------|---------|
| `src/infrastructure/secure_storage/keyring_adapter.rs` | is_available, entry_for, store, retrieve, delete (+2) |
| `src/infrastructure/secure_storage/encrypted_store.rs` | load_credentials, save_credentials, store, retrieve, delete |
| `src/infrastructure/secure_storage/mod.rs` | create_secure_storage, new, default |
| `src/infrastructure/persistence/json_account_repository.rs` | clone, default |

## Entry Points

Start here when exploring this area:

- **`create_secure_storage`** (Function) — `src/infrastructure/secure_storage/mod.rs:49`
- **`new`** (Function) — `src/infrastructure/secure_storage/mod.rs:81`
- **`new`** (Function) — `src/infrastructure/secure_storage/keyring_adapter.rs:20`

## Key Symbols

| Symbol | Type | File | Line |
|--------|------|------|------|
| `create_secure_storage` | Function | `src/infrastructure/secure_storage/mod.rs` | 49 |
| `new` | Function | `src/infrastructure/secure_storage/mod.rs` | 81 |
| `new` | Function | `src/infrastructure/secure_storage/keyring_adapter.rs` | 20 |
| `default` | Function | `src/infrastructure/secure_storage/mod.rs` | 89 |
| `is_available` | Function | `src/infrastructure/secure_storage/keyring_adapter.rs` | 54 |
| `clone` | Function | `src/infrastructure/persistence/json_account_repository.rs` | 106 |
| `default` | Function | `src/infrastructure/persistence/json_account_repository.rs` | 329 |
| `load_credentials` | Function | `src/infrastructure/secure_storage/encrypted_store.rs` | 75 |
| `save_credentials` | Function | `src/infrastructure/secure_storage/encrypted_store.rs` | 106 |
| `store` | Function | `src/infrastructure/secure_storage/encrypted_store.rs` | 126 |
| `retrieve` | Function | `src/infrastructure/secure_storage/encrypted_store.rs` | 134 |
| `delete` | Function | `src/infrastructure/secure_storage/encrypted_store.rs` | 142 |
| `entry_for` | Function | `src/infrastructure/secure_storage/keyring_adapter.rs` | 24 |
| `store` | Function | `src/infrastructure/secure_storage/keyring_adapter.rs` | 31 |
| `retrieve` | Function | `src/infrastructure/secure_storage/keyring_adapter.rs` | 38 |
| `delete` | Function | `src/infrastructure/secure_storage/keyring_adapter.rs` | 47 |
| `default` | Function | `src/infrastructure/secure_storage/keyring_adapter.rs` | 74 |

## Execution Flows

| Flow | Type | Steps |
|------|------|-------|
| `Migrate_plaintext_keys → New` | cross_community | 6 |
| `Migrate_plaintext_keys → Is_available` | cross_community | 6 |
| `Test_gateway_with_custom_config → New` | cross_community | 5 |
| `Test_gateway_with_custom_config → Is_available` | cross_community | 5 |
| `Test_delete_account_persists → New` | cross_community | 4 |
| `Test_delete_account_persists → Is_available` | cross_community | 4 |
| `Test_delete_and_verify_file_updated → New` | cross_community | 4 |
| `Test_delete_and_verify_file_updated → Is_available` | cross_community | 4 |
| `Test_atomic_write_no_temp_leftover → New` | cross_community | 4 |
| `Test_atomic_write_no_temp_leftover → Is_available` | cross_community | 4 |

## Connected Areas

| Area | Connections |
|------|-------------|
| Services | 1 calls |
| Persistence | 1 calls |

## How to Explore

1. `gitnexus_context({name: "create_secure_storage"})` — see callers and callees
2. `gitnexus_query({query: "secure_storage"})` — find related execution flows
3. Read key files listed above for implementation details
