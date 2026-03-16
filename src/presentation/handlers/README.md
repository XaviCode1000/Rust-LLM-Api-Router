# HTTP Handlers

Axum request handlers for the LLM API Router.

## Overview

This module contains the HTTP request handlers that process incoming API requests. Handlers are designed to be thin - delegating business logic to application services.

## Handlers

### Chat Handler

Handles `/v1/chat/completions` - OpenAI-compatible chat API.

```rust
use axum::{
    extract::State,
    Json,
};
use rust_llm_api_router::domain::OpenAIChatRequest;
use rust_llm_api_router::presentation::state::AppState;

async fn chat_completions(
    State(state): State<Arc<AppState>>,
    Json(request): Json<OpenAIChatRequest>,
) -> impl IntoResponse {
    // Delegates to application service
}
```

### Metrics Handler

Handles `/metrics` - Prometheus metrics endpoint.

```rust
use axum::extract::State;
use rust_llm_api_router::presentation::state::AppState;

async fn metrics(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Returns Prometheus-formatted metrics
    let encoder = prometheus::TextEncoder::new();
    let metric_families = state.metrics.collect();
    encoder.encode(&metric_families).unwrap()
}
```

## Error Handling

Errors from application services are mapped to appropriate HTTP status codes:

| Domain Error | HTTP Status |
|--------------|-------------|
| `NotFound` | 404 |
| `InvalidCredentials` | 401 |
| `RateLimited` | 429 |
| `ProviderError` | 502 |
| Other | 500 |

## See Also

- [Presentation Layer](../mod.rs)
- [Routes](../routes.rs)
- [AppState](../state.rs)
