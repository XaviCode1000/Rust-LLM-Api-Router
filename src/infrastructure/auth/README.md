# Authentication Module

This module provides pluggable authentication strategies for LLM providers.

## Overview

The authentication system implements the Strategy pattern to support different authentication flows required by various LLM providers.

## Supported Strategies

| Strategy | Description | Use Case |
|----------|-------------|----------|
| `ApiKeyAuthStrategy` | Simple bearer token | Groq, OpenRouter, Mistral, Cerebras |
| `PkceAuthStrategy` | OAuth 2.1 with PKCE | Modern OAuth providers |
| `DeviceFlowAuthStrategy` | OAuth 2.0 Device Flow | Headless devices, CLI tools |

## Quick Start

### API Key Authentication

```rust
use rust_llm_api_router::infrastructure::auth::ApiKeyAuthStrategy;

let strategy = ApiKeyAuthStrategy::new("groq");
let account = strategy.complete_auth("gsk_xxx".to_string()).await?;
```

### OAuth 2.1 PKCE

```rust
use rust_llm_api_router::infrastructure::auth::PkceAuthStrategy;

let strategy = PkceAuthStrategy::new(
    "client_id",
    None,
    "https://auth.provider.com/authorize",
    "https://auth.provider.com/token",
    "http://localhost/callback",
    vec!["read".to_string()],
)?;
```

### Device Flow

```rust
use rust_llm_api_router::infrastructure::auth::DeviceFlowAuthStrategy;

let strategy = DeviceFlowAuthStrategy::new(
    "device_client_id",
    None,
    "https://auth.provider.com/device",
    "https://auth.provider.com/token",
    vec!["read".to_string()],
    Some(5),
)?;
```

## Security

- **Token Storage**: Uses `keyring` for secure credential storage
- **Memory Safety**: Zeroize sensitive data after use
- **No Credential Logging**: Credentials are never logged

## See Also

- [Domain Authentication Strategy Trait](../../domain/services/auth_strategy.rs)
- [RFC 8628: OAuth 2.0 Device Authorization Grant](https://datatracker.ietf.org/doc/html/rfc8628)
- [OAuth 2.1](https://datatracker.ietf.org/doc/html/draft-ietf-oauth-v2-1)
