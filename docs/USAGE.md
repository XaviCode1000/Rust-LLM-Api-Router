# Usage Guide

Practical examples and workflows for using the LLM API Router in real scenarios.

---

## Table of Contents

- [Quick Setup](#quick-setup)
- [Basic Usage](#basic-usage)
- [Streaming Responses](#streaming-responses)
- [Using with Popular Clients](#using-with-popular-clients)
- [Multi-Provider Workflows](#multi-provider-workflows)
- [Production Setup](#production-setup)
- [Troubleshooting](#troubleshooting)

---

## Quick Setup

### First Time in 5 Minutes

```bash
# 1. Start the server
./target/release/llm-router --port 8080

# 2. Add a provider
./target/release/llm-router provider add \
  --id groq \
  --name "Groq" \
  --base-url "https://api.groq.com/openai/v1"

# 3. Add your API key
./target/release/llm-router account add \
  --id my-groq \
  --provider groq \
  --api-key "your-api-key-here"

# 4. Test it
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "groq:llama-3.3-70b-versatile",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

---

## Basic Usage

### Model Naming Convention

All models use the `provider:model-name` format:

```bash
groq:llama-3.3-70b-versatile
openai:gpt-4o
anthropic:claude-3-5-sonnet-20241022
mistral:mistral-large-latest
```

### Non-Streaming Request

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "groq:llama-3.3-70b-versatile",
    "messages": [
      {"role": "system", "content": "You are a helpful assistant."},
      {"role": "user", "content": "Explain quantum computing in simple terms."}
    ],
    "temperature": 0.7,
    "max_tokens": 500
  }'
```

**Response:**
```json
{
  "id": "chatcmpl-xxx",
  "object": "chat.completion",
  "created": 1773367546,
  "model": "groq:llama-3.3-70b-versatile",
  "choices": [{
    "index": 0,
    "message": {
      "role": "assistant",
      "content": "Quantum computing uses qubits..."
    },
    "finish_reason": "stop"
  }],
  "usage": {
    "prompt_tokens": 25,
    "completion_tokens": 120,
    "total_tokens": 145
  }
}
```

### Streaming Responses

```bash
curl -N -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Accept: text/event-stream" \
  -d '{
    "model": "groq:llama-3.3-70b-versatile",
    "messages": [{"role": "user", "content": "Count from 1 to 5"}],
    "stream": true,
    "max_tokens": 50
  }'
```

The server sends Server-Sent Events (SSE):
```
data: {"id":"chatcmpl-xxx","object":"chat.completion.chunk","choices":[{"delta":{"content":"1"},"index":0}]}

data: {"id":"chatcmpl-xxx","object":"chat.completion.chunk","choices":[{"delta":{"content":", "},"index":0}]}

data: {"id":"chatcmpl-xxx","object":"chat.completion.chunk","choices":[{"delta":{"content":"2"},"index":0}]}

...

data: [DONE]
```

---

## Using with Popular Clients

### Python OpenAI SDK

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://localhost:8080/v1",
    api_key="any-key-works-here"
)

response = client.chat.completions.create(
    model="groq:llama-3.3-70b-versatile",
    messages=[{"role": "user", "content": "Hello!"}]
)

print(response.choices[0].message.content)
```

### Node.js (OpenAI SDK)

```javascript
import OpenAI from "openai";

const client = new OpenAI({
  baseURL: "http://localhost:8080/v1",
  apiKey: "any-key-works-here",
});

const response = await client.chat.completions.create({
  model: "groq:llama-3.3-70b-versatile",
  messages: [{ role: "user", content: "Hello!" }],
});

console.log(response.choices[0].message.content);
```

### OpenCode

Edit `.opencode/opencode.json`:
```json
{
  "model": "groq:llama-3.3-70b-versatile",
  "provider": {
    "openai": {
      "options": {
        "baseURL": "http://localhost:8080/v1",
        "apiKey": "demo-key"
      }
    }
  }
}
```

### Claude Desktop / Any OpenAI-Compatible Client

Just point the `base_url` to `http://localhost:8080/v1` and use any API key. The router accepts the OpenAI format, so any client that speaks OpenAI will work.

---

## Multi-Provider Workflows

### Setup Multiple Providers

```bash
# Add Groq
./target/release/llm-router provider add \
  --id groq --name "Groq" \
  --base-url "https://api.groq.com/openai/v1"

# Add OpenAI
./target/release/llm-router provider add \
  --id openai --name "OpenAI" \
  --base-url "https://api.openai.com/v1"

# Add Anthropic
./target/release/llm-router provider add \
  --id anthropic --name "Anthropic" \
  --base-url "https://api.anthropic.com/v1"

# Add accounts for each
./target/release/llm-router account add --id groq-1 --provider groq --api-key "$GROQ_API_KEY"
./target/release/llm-router account add --id openai-1 --provider openai --api-key "$OPENAI_API_KEY"
./target/release/llm-router account add --id anthropic-1 --provider anthropic --api-key "$ANTHROPIC_API_KEY"

# Verify
./target/release/llm-router provider list
./target/release/llm-router account list
```

### Switch Between Providers

Just change the model prefix in your request:

```bash
# Use Groq
curl ... -d '{"model": "groq:llama-3.3-70b-versatile", ...}'

# Use OpenAI
curl ... -d '{"model": "openai:gpt-4o", ...}'

# Use Anthropic
curl ... -d '{"model": "anthropic:claude-3-5-sonnet-20241022", ...}'
```

### Enable Intelligent Routing

```bash
# Start with cascading routing (cheapest first, escalate if quality is low)
./target/release/llm-router --routing-strategy cascading --quality-threshold 0.85

# Or use cost-optimized (always pick cheapest capable model)
./target/release/llm-router --routing-strategy cost-optimized --budget-mode

# Or let the planner decide automatically
./target/release/llm-router --routing-strategy auto
```

> 📖 **Full routing documentation:** [routing.md](routing.md)

---

## Production Setup

### Docker (Recommended)

```bash
docker run -d \
  --name llm-router \
  -p 8080:8080 \
  -v llm-router-config:/root/.config/rust-llm-api-router \
  ghcr.io/xavicode1000/rust-llm-api-router:latest \
  --port 8080 --routing-strategy auto
```

### With Environment Variables

```bash
docker run -d \
  --name llm-router \
  -p 8080:8080 \
  -e LOG_LEVEL=warn \
  -e ROUTING_STRATEGY=cascading \
  -e CASCADING_MIN_QUALITY=0.8 \
  -e MAX_RETRIES=5 \
  -e REQUEST_TIMEOUT_SECONDS=120 \
  ghcr.io/xavicode1000/rust-llm-api-router:latest
```

### Health Checks

```bash
# Basic health
curl http://localhost:8080/health
# {"status":"healthy","timestamp":1773367546,"version":"0.1.0"}

# Detailed health
curl http://localhost:8080/health/detail
# Includes provider and account status
```

### Monitoring

```bash
# Prometheus metrics
curl http://localhost:8080/metrics

# Example metrics:
# llm_router_requests_total{method="POST",endpoint="/v1/chat/completions",status="200"} 142
# llm_router_requests_total{method="GET",endpoint="/health",status="200"} 1024
```

---

## Troubleshooting

### "No active accounts found for provider"

**Cause:** No API keys registered for the provider you're trying to use.

**Fix:**
```bash
# Check registered accounts
./target/release/llm-router account list

# Add account if missing
./target/release/llm-router account add --id my-account --provider groq --api-key "your-key"

# Validate the account
./target/release/llm-router account validate --id my-account
```

### "Provider returned 401 Unauthorized"

**Cause:** Invalid API key.

**Fix:**
```bash
# Verify your API key is correct
./target/release/llm-router account validate --id my-account

# If invalid, remove and re-add
./target/release/llm-router account remove --id my-account
./target/release/llm-router account add --id my-account --provider groq --api-key "correct-key"
```

### "Model not found" or "Invalid model"

**Cause:** Model name format is wrong or model doesn't exist on the provider.

**Fix:**
- Use `provider:model-name` format (e.g., `groq:llama-3.3-70b-versatile`)
- Check available models: `./target/release/llm-router provider models --provider groq`

### Port Already in Use

**Cause:** Another process is using port 8080.

**Fix:**
```bash
# Use a different port
./target/release/llm-router --port 8081

# Or kill the existing process
lsof -ti:8080 | xargs kill -9
```

### Request Timeout

**Cause:** Provider is slow or unreachable.

**Fix:**
```bash
# Increase timeout
./target/release/llm-router --timeout 120

# Check provider health
curl http://localhost:8080/health/detail

# Verify provider is enabled
./target/release/llm-router provider list
```

### CORS Issues (Browser Client)

The router doesn't set CORS headers by default. If you're calling it from a browser:

**Fix:** Put a reverse proxy in front (nginx, Caddy) that adds CORS headers:

```nginx
# nginx example
location /v1/ {
    proxy_pass http://localhost:8080;
    add_header Access-Control-Allow-Origin *;
    add_header Access-Control-Allow-Methods "GET, POST, OPTIONS";
    add_header Access-Control-Allow-Headers "Content-Type, Authorization";
}
```

---

## See Also

- [API Reference](api.md) — All endpoints and response formats
- [CLI Reference](cli.md) — All CLI commands and options
- [Configuration](CONFIG.md) — Environment variables and config files
- [Routing Strategies](routing.md) — Cost-Aware, Cascading, Task-Based routing
- [Deployment Guide](deployment.md) — Docker, systemd, Kubernetes
- [Security](security.md) — Auth, OAuth, secure storage
