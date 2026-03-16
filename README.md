# LLM API Router

<p align="center">
  <a href="https://github.com/XaviCode1000/Rust-LLM-Api-Router/actions/workflows/ci.yml">
    <img src="https://img.shields.io/github/actions/workflow/status/XaviCode1000/Rust-LLM-Api-Router/ci.yml?branch=main&label=CI" alt="CI Status">
  </a>
  <a href="https://codecov.io/gh/XaviCode1000/Rust-LLM-Api-Router">
    <img src="https://img.shields.io/codecov/c/github/XaviCode1000/Rust-LLM-Api-Router?label=Coverage" alt="Coverage">
  </a>
  <a href="https://crates.io/crates/rust-llm-api-router">
    <img src="https://img.shields.io/crates/v/rust-llm-api-router" alt="Crate">
  </a>
  <a href="https://docs.rs/rust-llm-api-router">
    <img src="https://img.shields.io/docsrs/rust-llm-api-router" alt="Documentation">
  </a>
  <a href="https://opensource.org/licenses/MIT">
    <img src="https://img.shields.io/badge/License-MIT-green.svg" alt="License: MIT">
  </a>
</p>

A high-performance LLM proxy/router built with Clean Architecture in Rust. Route requests across multiple providers and accounts with automatic failover, health monitoring, and intelligent account selection.

## Features

- **Multi-Provider Support**: Connect to 12+ LLM providers (Groq, OpenRouter, Mistral, Cerebras, Cloudflare, Anthropic, OpenAI, and more)
- **Multi-Account Routing**: Register multiple API keys per provider with automatic rotation
- **Automatic Failover**: Circuit breaker pattern with intelligent retry logic
- **Streaming (SSE)**: Real-time token-by-token streaming responses
- **OpenAI-Compatible API**: Drop-in replacement for OpenAI clients
- **Health Monitoring**: Real-time health checks and metrics
- **Integrated CLI**: Manage providers and accounts from the terminal
- **Execution Planning**: Proactive planning with multiple strategies (Standard, Failover, Load Balanced, Cost Optimized)
- **OAuth 2.1 / PKCE**: Secure authentication flow support

## Quick Start

### 1. Build

```bash
cargo build --release
```

### 2. Register Providers

```bash
# Register a provider
./target/release/llm-router provider add \
  --id groq \
  --name "Groq" \
  --base-url "https://api.groq.com/openai/v1"

# List providers
./target/release/llm-router provider list
```

### 3. Register Accounts with API Keys

```bash
# Register account with API key
./target/release/llm-router account add \
  --id groq-1 \
  --provider groq \
  --api-key "your-api-key-here" \
  --priority 0

# List accounts
./target/release/llm-router account list
```

### 4. Start Server

```bash
./target/release/llm-router --port 8080
```

### 5. Test the API

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <your-test-api-key>" \
  -d '{
    "model": "groq:llama-3.3-70b-versatile",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
```

> **Note**: Models must be specified with the provider prefix (e.g., `groq:model-name`, `openai:gpt-3.5-turbo`)

### Streaming (SSE)

```bash
curl -N -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <your-test-api-key>" \
  -d '{
    "model": "groq:llama-3.3-70b-versatile",
    "messages": [{"role": "user", "content": "Hello!"}],
    "stream": true,
    "max_tokens": 50
  }'
```

## Architecture

This project follows **Clean Architecture** and **Domain-Driven Design (DDD)** principles:

```
┌─────────────────────────────────────────────────────────────┐
│                 Presentation Layer                           │
│  (HTTP Handlers, Routes, CLI, Middleware)                  │
├─────────────────────────────────────────────────────────────┤
│                 Application Layer                            │
│  (Use Cases, Services, Rotation Strategies, Execution Plans)│
├─────────────────────────────────────────────────────────────┤
│                   Domain Layer                               │
│  (Entities, Traits/Ports, Domain Errors, Auth Strategies)  │
├─────────────────────────────────────────────────────────────┤
│              Infrastructure Layer                            │
│  (HTTP Client, JSON Persistence, Provider Adapters, Auth)   │
└─────────────────────────────────────────────────────────────┘
```

For detailed architecture documentation, see [docs/architecture.md](docs/architecture.md).

## API Reference

### POST /v1/chat/completions

OpenAI-compatible chat completions endpoint.

**Request:**

```json
{
  "model": "groq:llama-3.3-70b-versatile",
  "messages": [
    {"role": "user", "content": "Hello!"}
  ],
  "temperature": 0.7,
  "max_tokens": 1024
}
```

**Response:**

```json
{
  "id": "chatcmpl-xxx",
  "object": "chat.completion",
  "created": 1773335238,
  "model": "groq:llama-3.3-70b-versatile",
  "choices": [{
    "index": 0,
    "message": {
      "role": "assistant",
      "content": "Hello! How can I help you?"
    },
    "finish_reason": "stop"
  }],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 8,
    "total_tokens": 18
  }
}
```

### GET /health

Basic health check endpoint.

```bash
curl http://localhost:8080/health
# {"status":"healthy","timestamp":1773335508,"version":"0.1.0"}
```

### GET /health/detail

Detailed system health status.

```bash
curl http://localhost:8080/health/detail
```

### GET /v1/models

List available models from all providers.

```bash
curl http://localhost:8080/v1/models
```

### GET /accounts

List registered accounts (masked API keys).

```bash
curl http://localhost:8080/accounts
```

### GET /metrics

Prometheus metrics endpoint.

```bash
curl http://localhost:8080/metrics
```

## CLI Reference

### Providers

```bash
# Add provider
llm-router provider add --id <id> --name <name> --base-url <url> [--disabled]

# List providers
llm-router provider list

# Enable provider
llm-router provider enable --id <id>

# Disable provider
llm-router provider disable --id <id>

# Remove provider
llm-router provider remove --id <id>

# Validate provider
llm-router provider validate --id <id>
```

### Accounts

```bash
# Add account
llm-router account add \
  --id <id> \
  --provider <provider_id> \
  --api-key <key> \
  [--priority <n>] \
  [--interactive]

# List accounts
llm-router account list

# Set priority
llm-router account set-priority --id <id> --priority <n>

# Remove account
llm-router account remove --id <id>

# Validate account
llm-router account validate --id <id>
```

## Supported Providers

| Provider | Base URL | Status |
|----------|----------|--------|
| Groq | https://api.groq.com/openai/v1 | ✅ Tested |
| OpenRouter | https://openrouter.ai/api/v1 | ✅ Tested |
| Mistral AI | https://api.mistral.ai/v1 | ✅ Tested |
| Cerebras | https://api.cerebras.ai/v1 | ✅ Tested |
| Cloudflare Workers AI | https://api.cloudflare.com/client/v4/accounts | ✅ Tested |
| Anthropic | https://api.anthropic.com/v1 | ✅ Tested |
| OpenAI | https://api.openai.com/v1 | ✅ Tested |
| NVIDIA NIM | https://integrate.api.nvidia.com/v1 | 🔄 Supported |
| Hugging Face | https://api-inference.huggingface.co/models | 🔄 Supported |
| DeepSeek | https://api.deepseek.com/v1 | 🔄 Supported |
| xAI (Grok) | https://api.x.ai/v1 | 🔄 Supported |
| Cohere | https://api.cohere.ai/v1 | 🔄 Supported |
| AI21 | https://api.ai21.com/studio/v1 | 🔄 Supported |
| Google AI Studio | https://generativelanguage.googleapis.com/v1beta | 🔄 Supported |

✅ = Tested | 🔄 = Supported (pending test)

## Verified Working Models

### Groq

- `llama-3.3-70b-versatile`
- `llama-3.1-8b-instant`
- `groq/compound`
- `groq/compound-mini`

## Rotation Strategies

The system supports 4 account rotation strategies:

1. **Round-Robin**: Sequential rotation across accounts
2. **Weighted**: Based on priority (lower number = higher priority)
3. **Latency**: Selects account with lowest latency (WIP)
4. **User-Affinity**: Same account per user (WIP)

## Execution Planning

The **Execution Plan** module transforms the system from reactive failover to proactive planning:

- **Proactive Planning**: Select optimal account before first request
- **Multiple Strategies**: Standard, Failover, Load Balanced, Cost Optimized
- **Intelligent Rotation**: RoundRobin, HealthWeighted, Priority, LRU
- **Integrated Metrics**: Prometheus metrics for monitoring
- **Distributed Tracing**: OpenTelemetry for debugging

See [src/app/services/execution_plan/README.md](src/app/services/execution_plan/README.md) for detailed documentation.

### Execution Strategy Types

| Type | Description |
|------|-------------|
| `Standard` | Single account execution |
| `Failover` | Secondary accounts on failure |
| `LoadBalanced` | Round-robin distribution |
| `CostOptimized` | Lowest cost selection |

## Failover

- **Circuit Breaker**: Opens after 5 consecutive failures
- **Auto-Close**: Retries after 30 seconds
- **Health Scoring**: 0-100 score based on success/latency

## Configuration

### Environment Variables

```bash
# Server port (default: 8080)
PORT=8080

# Host (default: 0.0.0.0)
HOST=0.0.0.0

# Log level (trace, debug, info, warn, error)
LOG_LEVEL=info

# Planning timeout (default: 5000ms)
PLANNING_TIMEOUT_MS=5000

# Max accounts per plan (default: 3)
MAX_ACCOUNTS_PER_PLAN=3
```

### Configuration Files

Configuration is stored in the XDG config directory:

```
~/.config/rust-llm-api-router/
├── providers.json    # Registered providers
└── accounts.json    # Accounts with API keys
```

## Development

### Build

```bash
cargo build
```

### Tests & Coverage

**Achievement: 80.35% Code Coverage with 492 tests passing.**

```bash
# Run all tests (4x faster with nextest)
cargo nextest run --test-threads 2

# Generate coverage report (10x faster than tarpaulin)
cargo llvm-cov --html --output-dir coverage-llvm

# View coverage summary
cargo llvm-cov --summary-only
```

#### Testing Journey

- **Initial**: 32.02% coverage (104 tests)
- **Current**: 80.35% coverage (492 tests)
- **Progress**: +48.33% coverage, +388 tests added

See [docs/TESTING_JOURNEY.md](docs/TESTING_JOURNEY.md) for more details.

#### Optimal Development Stack 2025-26

The project uses the optimal Rust development stack:

- **cargo-nextest**: Test runner (4x faster)
- **cargo-llvm-cov**: Native LLVM coverage (10x faster)
- **sccache**: Build cache (6x faster)
- **cargo-watch**: Auto-rebuild on changes
- **wiremock**: HTTP mocking for integration tests
- **mockall**: Trait mocking

See [DEVELOPMENT.md](DEVELOPMENT.md) for the complete guide.

### Lint

```bash
cargo clippy
cargo fmt
```

### Security

```bash
cargo audit
cargo deny check
```

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting PRs.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Roadmap

### Testing Achievements ✅

- [x] **80.35% Code Coverage** (32% → 80.35%, +48.33%)
- [x] **492 Tests Passing** (104 → 492, +388 tests)
- [x] **Domain Layer**: 100% coverage
- [x] **Error Handling**: 100% coverage
- [x] **Health Handler**: 100% coverage
- [x] **Gateway**: 94.26% coverage
- [x] **Failover**: 86.79% coverage
- [x] **CLI Commands**: 84-85% coverage
- [x] **Chat Handler**: 85.80% coverage
- [x] **Repository**: 80.72% coverage

### Completed ✅

- [x] **Streaming (SSE)** for /v1/chat/completions
- [x] **Endpoint /v1/models** with real model list
- [x] **Anthropic Adapters** (different format)
- [x] **Supported Providers**: Groq, OpenRouter, Mistral, Cerebras, Cloudflare, Anthropic, OpenAI
- [x] **80.35% Code Coverage** with 492 tests
- [x] **Execution Planning Module** with proactive failover

### Pending 🔄

- [ ] Docker + Kubernetes manifests
- [ ] Complete CI/CD pipeline (GitHub Actions)
- [ ] Additional Provider Adapters: Google AI Studio, Cohere, AI21, DeepSeek
- [ ] NVIDIA NIM, Hugging Face inference endpoints

## Project Structure

```
src/
├── domain/           # Domain layer (entities, traits, errors)
│   ├── entities/     # Core business entities
│   ├── traits/       # Ports/interfaces
│   ├── errors/      # Domain errors
│   └── services/    # Domain services
├── app/             # Application layer (use cases, services)
│   ├── services/    # Application services
│   │   ├── account_rotation.rs
│   │   ├── execution_plan/
│   │   ├── failover.rs
│   │   └── auth/
│   ├── router/     # Internal routing
│   └── health.rs   # Health service
├── infrastructure/ # Infrastructure layer (implementations)
│   ├── http_client.rs
│   ├── logging.rs
│   ├── metrics.rs
│   ├── persistence/   # JSON file storage
│   ├── provider/     # Provider adapters
│   ├── gateway/      # LLM gateway
│   └── auth/         # Authentication strategies
├── presentation/   # Presentation layer
│   ├── handlers/   # HTTP handlers
│   ├── routes.rs   # Route definitions
│   ├── state.rs    # Application state
│   └── cli/        # CLI commands
├── config/         # Configuration
├── lib.rs          # Library root
└── main.rs         # Binary entry point
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

Built with Clean Architecture and DDD patterns in Rust.
