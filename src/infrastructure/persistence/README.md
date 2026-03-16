# Persistence Module

JSON file-based storage implementations for the domain repository traits.

## Overview

This module provides concrete implementations of the domain repository traits using JSON file storage. It follows the Repository pattern to abstract data persistence.

## Storage Location

Configuration is stored in the XDG config directory:

```
~/.config/rust-llm-api-router/
├── providers.json    # Provider configurations
└── accounts.json    # Account (API key) data
```

## Implementations

### JsonAccountRepository

Implements [`AccountRepository`](../../domain/traits/mod.rs) for account storage.

```rust
use rust_llm_api_router::infrastructure::persistence::JsonAccountRepository;

let account_repo = JsonAccountRepository::new(
    dirs::config_dir()
        .unwrap()
        .join("rust-llm-api-router")
        .join("accounts.json")
).await?;
```

### JsonProviderRepository

Implements [`ProviderRepository`](../../domain/traits/mod.rs) for provider storage.

```rust
use rust_llm_api_router::infrastructure::persistence::JsonProviderRepository;

let provider_repo = JsonProviderRepository::new(
    dirs::config_dir()
        .unwrap()
        .join("rust-llm-api-router")
        .join("providers.json")
).await?;
```

## Features

- **Async File I/O**: Non-blocking file operations
- **Automatic Creation**: Creates directories and files if they don't exist
- **Error Handling**: Converts I/O errors to domain errors
- **Thread-Safe**: Uses internal mutex for concurrent access

## Design Decisions

| Decision | Rationale |
|----------|------------|
| JSON format | Human-readable, easy to backup |
| File-based | No external database dependencies |
| XDG standard | Follows Linux conventions |
| Sync to disk | Ensures data durability |

## Migration

For production deployments requiring database persistence, implement new repository types:

```rust
use rust_llm_api_router::domain::traits::AccountRepository;
use rust_llm_api_router::domain::Account;

pub struct SqlAccountRepository {
    pool: SqlitePool,
}

#[async_trait]
impl AccountRepository for SqlAccountRepository {
    // ... implement using sqlx
}
```

## See Also

- [Domain Repository Traits](../../domain/traits/mod.rs)
- [Infrastructure Layer](../mod.rs)
