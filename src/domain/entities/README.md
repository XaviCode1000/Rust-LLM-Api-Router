# Domain Entities

Core business entities representing the domain model.

## Overview

This module contains the pure domain entities with no external dependencies. These types are used throughout the application and are serialized for API responses.

## Entities

### Account

Represents an API key registered for a provider.

```rust
use rust_llm_api_router::domain::entities::Account;

let account = Account::new_api_key(
    "groq-1".to_string(),
    "groq",
    "gsk_xxx"
);
```

### Provider

Configuration for an LLM provider.

```rust
use rust_llm_api_router::domain::entities::Provider;

let provider = Provider::new(
    "groq".to_string(),
    "Groq".to_string(),
    "https://api.groq.com/openai/v1"
);
```

### AccountHealth

Health metrics for account monitoring and failover decisions.

### ChatRequest / ChatResponse

Request and response types for LLM chat completions.

### Message

Conversation message with role and content.

```rust
use rust_llm_api_router::domain::entities::Message;

let msg = Message::user("Hello, world!");
let system_msg = Message::system("You are a helpful assistant.");
```

### Model

Model information from an LLM provider.

## OpenAI Types

For OpenAI-compatible API responses, see `openai_types.rs`:
- `OpenAIChatRequest`
- `OpenAIChatResponse`
- `OpenAIChoice`
- `OpenAIError`
- `OpenAIUsage`

## Design Principles

1. **No external dependencies**: Pure Rust types with `serde` for serialization
2. **Immutable**: Builder patterns for construction
3. **Explicit**: Clear field names and documentation

## See Also

- [Domain Layer](../mod.rs)
- [Domain Traits](../traits/mod.rs)
