# API Reference

## Base URL

All API endpoints are accessible at `http://localhost:8080`

## Authentication

The LLM API Router uses Bearer token authentication for API endpoints. 
The token should be a valid API key from one of your configured accounts.

```
Authorization: Bearer <your-api-key>
```

Note: The API key is validated against your configured accounts, not directly against the provider.

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

- `no_accounts`: No active accounts found for the specified provider
- `provider_error`: Error communicating with the provider API
- `invalid_request_error`: Invalid request parameters
- `internal`: Internal server error

## Working Models (Verified)

### Groq Provider

With account `<your-groq-api-key>`:
- `llama-3.3-70b-versatile` ✅
- `llama-3.1-8b-instant` ✅
- `groq/compound` ✅
- `groq/compound-mini` ✅

> **Note**: Models like `llama3-8b-8192` and `mixtral-8x7b-32768` have been decommissioned by Groq and are no longer functional.

## Usage Examples

### Basic Request

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <your-groq-api-key>" \
  -d '{
    "model": "groq:llama-3.3-70b-versatile",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

### With Custom Parameters

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-api-key-here" \
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