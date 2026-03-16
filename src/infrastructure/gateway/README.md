# LLM Gateway Module

Provides the primary interface for communicating with LLM providers.

## Overview

The gateway module implements the [`LlmGateway`](../../domain/traits/mod.rs) trait and handles:
- HTTP communication with LLM providers
- Request/response transformation
- Provider-specific adapter routing

## Components

### LlmGatewayImpl

The main gateway implementation that routes requests to different providers.

```rust
use rust_llm_api_router::infrastructure::gateway::LlmGatewayImpl;
use rust_llm_api_router::domain::ChatRequest;

let gateway = LlmGatewayImpl::new(
    http_client,
    ProviderConfig::default()
);

let response = gateway.chat(request, "sk-xxx").await?;
```

### ProviderConfig

Configuration for supported providers.

```rust
use rust_llm_api_router::infrastructure::gateway::ProviderConfig;

let config = ProviderConfig::default()
    .with_provider("groq", "https://api.groq.com/openai/v1")
    .with_provider("openrouter", "https://openrouter.ai/api/v1");
```

## Supported Providers

| Provider | Base URL |
|----------|----------|
| Groq | https://api.groq.com/openai/v1 |
| OpenRouter | https://openrouter.ai/api/v1 |
| Mistral | https://api.mistral.ai/v1 |
| Cerebras | https://api.cerebras.ai/v1 |
| Cloudflare | https://api.cloudflare.com/client/v4/accounts |
| Anthropic | https://api.anthropic.com/v1 |
| OpenAI | https://api.openai.com/v1 |

## Features

- **OpenAI-compatible**: Works with any OpenAI-compatible API
- **Streaming**: Supports SSE streaming responses
- **Error Mapping**: Converts provider errors to domain errors

## Request Flow

```
ChatRequest → LlmGatewayImpl
              ↓
         Parse model (e.g., "groq:llama-3.3-70b-versatile")
              ↓
         Extract provider (groq) + model (llama-3.3-70b-versatile)
              ↓
         Build provider-specific request
              ↓
         HTTP POST to provider
              ↓
         Transform response to OpenAI format
              ↓
         ChatResponse
```

## See Also

- [Domain LlmGateway Trait](../../domain/traits/mod.rs)
- [HTTP Client](../http_client.rs)
- [Provider Adapters](../provider/mod.rs)
