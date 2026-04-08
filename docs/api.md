# API Reference

## Base URL

All API endpoints are accessible at `http://localhost:8080` (configurable via `--host` and `--port` CLI flags).

## Authentication

The LLM API Router uses Bearer token authentication for API endpoints. The token should be a valid API key from one of your configured accounts.

```
Authorization: Bearer <your-api-key>
```

The API key is validated against your configured accounts, not directly against the provider.

## Endpoints

### POST /v1/chat/completions

Create a chat completion (OpenAI-compatible).

#### Request Body

```json
{
  "model": "groq:llama-3.3-70b-versatile",
  "messages": [
    {
      "role": "user",
      "content": "Hello, world!"
    }
  ],
  "temperature": 0.7,
  "max_tokens": 1024,
  "stream": false
}
```

#### Model Format

Models must be specified with the provider prefix: `provider:model-name`

Examples:
- `groq:llama-3.3-70b-versatile`
- `groq:llama-3.1-8b-instant`
- `openai:gpt-3.5-turbo`

#### Intelligent Routing

The router supports multiple intelligent routing strategies for cost optimization:

| Strategy | Description | When to Use |
|----------|-------------|-------------|
| **Auto** (default) | Planner selects based on context | General use |
| **Cost-Aware** (#23) | Selects cheapest capable model upfront | Budget-critical, predictable queries |
| **Cascading** (#24) | Starts cheap, escalates if quality low | Quality-critical, variable queries |
| **Task-Based** (#26) | Routes by task type (Code, Reasoning, etc.) | Task-specific model preferences |
| **Failover** | Sequential fallback on failure | Reliability-critical |
| **Load-Balanced** | Health-weighted distribution | High-throughput scenarios |

Configure via CLI flags:
```bash
llm-router --routing-strategy cascading --quality-threshold 0.85
llm-router --routing-strategy cost-optimized --budget-mode
llm-router --routing-strategy failover --max-retries 5
```

See [docs/routing.md](routing.md) for detailed configuration.

#### Response

```json
{
  "id": "chatcmpl-unique-id",
  "object": "chat.completion",
  "created": 1773367546,
  "model": "groq:llama-3.3-70b-versatile",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Hello! It's nice to meet you. Is there something I can help you with?"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 8,
    "total_tokens": 18
  }
}
```

#### Streaming Response

When `stream: true`, the endpoint returns Server-Sent Events (SSE):

```
data: {"id":"chatcmpl-123","choices":[{"delta":{"content":"Hello"},"index":0}]}

data: {"id":"chatcmpl-123","choices":[{"delta":{"content":" world"},"index":0}]}

data: [DONE]
```

### GET /v1/models

List available models (currently returns placeholder data).

#### Response

```json
{
  "object": "list",
  "data": []
}
```

> **Note**: This endpoint is planned for improvement in future versions to return actual model lists from configured providers.

### GET /health

Basic health check.

#### Response

```json
{
  "status": "healthy",
  "timestamp": 1773367298,
  "version": "0.1.0"
}
```

### GET /health/detail

Detailed system health including provider and account status.

#### Response

```json
{
  "status": "healthy",
  "timestamp": 1773367298,
  "version": "0.1.0",
  "providers": {
    "total": 5,
    "enabled": 4,
    "disabled": 1
  },
  "accounts": {
    "total": 5,
    "active": 5,
    "inactive": 0
  }
}
```

### GET /accounts

List all registered accounts (API keys partially masked).

#### Response

```json
[
  {
    "id": "openai-account-1",
    "provider_id": "openai",
    "priority": 0,
    "is_active": true,
    "api_key_masked": "****"
  },
  {
    "id": "groq-2",
    "provider_id": "groq",
    "priority": 0,
    "is_active": true,
    "api_key_masked": "gsk_DVyb..."
  }
]
```

### GET /metrics

Prometheus metrics endpoint.

#### Response (text/plain)

```
# HELP llm_router_requests_total Total number of HTTP requests
# TYPE llm_router_requests_total counter
llm_router_requests_total{method="POST",endpoint="/v1/chat/completions",status="200"} 1
llm_router_requests_total{method="GET",endpoint="/health",status="200"} 5
# HELP execution_plans_created_total Total execution plans created
# TYPE execution_plans_created_total counter
execution_plans_created_total{plan_type="failover"} 3
```

## Error Responses

All error responses follow this format:

```json
{
  "error": {
    "message": "Error description",
    "type": "error_type",
    "param": null,
    "code": null
  }
}
```

### Common Error Types

| Error Type | Description | HTTP Status |
|------------|-------------|-------------|
| `no_accounts` | No active accounts found for the specified provider | 400 |
| `provider_error` | Error communicating with the provider API | 502 |
| `invalid_request_error` | Invalid request parameters | 400 |
| `internal` | Internal server error | 500 |
| `authentication_error` | Invalid or missing API key | 401 |

## Working Models (Verified)

### Groq Provider

With a valid Groq API key:
- `llama-3.3-70b-versatile` -- verified working
- `llama-3.1-8b-instant` -- verified working
- `groq/compound` -- verified working
- `groq/compound-mini` -- verified working

> **Note**: Models like `llama3-8b-8192` and `mixtral-8x7b-32768` have been decommissioned by Groq and are no longer functional.

### OpenAI Provider

Requires a valid API key from https://platform.openai.com/account/api-keys:
- `gpt-3.5-turbo` -- verified working
- `gpt-4` -- verified working
- `gpt-4-turbo` -- verified working

## Usage Examples

### Basic Request

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <your-api-key>" \
  -d '{
    "model": "groq:llama-3.3-70b-versatile",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

### With Custom Parameters

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <your-api-key>" \
  -d '{
    "model": "groq:llama-3.1-8b-instant",
    "messages": [
      {"role": "system", "content": "You are a helpful assistant."},
      {"role": "user", "content": "Explain quantum computing in simple terms."}
    ],
    "temperature": 0.3,
    "max_tokens": 150
  }'
```

### Streaming Request

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <your-api-key>" \
  -d '{
    "model": "groq:llama-3.3-70b-versatile",
    "messages": [{"role": "user", "content": "Write a poem about Rust."}],
    "stream": true
  }'
```

### With Cascading Routing

```bash
# Start server with cascading routing
llm-router --routing-strategy cascading --quality-threshold 0.8

# Then make requests as normal - cascading happens server-side
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <your-api-key>" \
  -d '{
    "model": "auto",
    "messages": [{"role": "user", "content": "Explain quantum computing"}]
  }'
```

## Execution Plan Types

The router uses different execution plan types based on the routing strategy:

| Plan Type | Description | Use Case |
|-----------|-------------|----------|
| `Standard` | Single account execution | Simple requests |
| `Failover` | Sequential fallback on failure | Reliability-critical |
| `LoadBalanced` | Health-weighted distribution | High throughput |
| `CostOptimized` | Cheapest provider selection | Budget-critical |
| `Cascading` | Quality-based escalation | Quality-critical |

## See Also

- [Architecture](architecture.md) -- System architecture overview
- [Routing Strategies](routing.md) -- Detailed routing configuration
- [CLI Reference](cli.md) -- Command-line interface documentation
- [Testing Guide](TESTING_GUIDE.md) -- How to run tests
