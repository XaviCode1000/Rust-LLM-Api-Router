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

- **Multi-Provider Support**: Connect to 29 LLM providers (OpenAI, Anthropic, Groq, Mistral, and 26 more including local and enterprise options)
- **Multi-Account Routing**: Register multiple API keys per provider with automatic rotation
- **Automatic Failover**: Circuit breaker pattern with intelligent retry logic
- **Streaming (SSE)**: Real-time token-by-token streaming responses
- **OpenAI-Compatible API**: Drop-in replacement for OpenAI clients
- **Health Monitoring**: Real-time health checks and metrics
- **Modern Interactive CLI**: Colored output, professional tables, interactive prompts, spinners (#19)
- **Integrated CLI**: Manage providers and accounts from the terminal
- **Shell Completions**: Auto-completion for bash, zsh, and fish (build with `--features completions`)
- **Execution Planning**: Proactive planning with multiple strategies (Standard, Failover, Load Balanced, Cost Optimized, Cascading)
- **Task-Based Routing**: Intelligent routing based on query type (General, Chat, Code, Reasoning, Summarization, Translation) (#26)
- **Cost-Aware Routing**: Static model selection based on query complexity (#23)
- **Cascading Routing**: Dynamic escalation when quality thresholds not met (#24)
- **Routing Configuration**: CLI flags and environment variables for routing strategies (#29)
- **Quality Evaluation**: Heuristic-based response quality checks with structured tracing
- **Token Validation**: Pre-flight context window check prevents wasteful API calls
- **Live Contract Tests**: Real API schema validation detects provider drift before production breaks
- **Atomic Persistence**: File locking + atomic writes prevent data corruption under concurrent access
- **OAuth 2.1 / PKCE**: Secure authentication flow support
- **Secure API Key Storage**: System keyring (macOS Keychain, Windows Credential Manager, Linux Secret Service) or encrypted file fallback (#22)
- **Docker & Containerization**: Multi-stage Dockerfile, docker-compose.yml, GitHub Actions for GHCR (#15)

## Quick Start

### Option 1: Build from Source

```bash
cargo build --release
```

### Option 2: Docker

```bash
# Pull from GHCR
docker pull ghcr.io/xavicode1000/rust-llm-api-router:latest

# Run
docker run -d -p 8080:8080 ghcr.io/xavicode1000/rust-llm-api-router:latest

# Or with docker-compose
docker compose up -d
```

### Register Providers

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

For intelligent routing strategies (Cost-Aware and Cascading), see [docs/routing.md](docs/routing.md).

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

The CLI features modern interactive output with colored feedback, professional tables, and interactive prompts.

### Providers

```bash
# Add provider
llm-router provider add --id <id> --name <name> --base-url <url> [--disabled]

# List providers (shows enabled status + account configuration)
llm-router provider list
# Output:
# ID          Name         Base URL                      Status      Account
# -------------------------------------------------------------------------
# groq        Groq         https://api.groq.com/...      ✓ Enabled   ✓ Configured
# openrouter  OpenRouter   https://openrouter.ai/...     ✓ Enabled   ✗ Not set

# List available models for a provider (requires authenticated account)
llm-router provider models --provider <provider_id>
# Example:
# llm-router provider models --provider groq

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

### Authentication (OAuth 2.1 + PKCE)

The router supports secure authentication via OAuth 2.1 with PKCE (Proof Key for Code Exchange) for enhanced security, and Device Flow for headless environments.

#### Login Commands

```bash
# Login with API key (requires --provider argument)
llm-router auth login --provider groq
llm-router auth login -p openai

# Login with OAuth 2.1 PKCE (opens browser for authentication)
llm-router login --provider <provider_id>

# Login with Device Flow (for headless environments)
llm-router login --provider <provider_id> --device-flow

# Login with specific auth URL (for custom Identity Providers)
llm-router login --provider <provider_id> \
  --auth-url "https://auth.provider.com/authorize" \
  --token-url "https://auth.provider.com/token"

# Interactive login (prompts for credentials)
llm-router login --interactive
```

> **Note**: The `auth login` command requires the `--provider` argument. Use `llm-router provider list` to see available providers.

#### Logout Commands

```bash
# Logout from a specific provider
llm-router logout --provider <provider_id>

# Logout from all providers
llm-router logout --all

# Clear stored credentials
llm-router logout --clear-credentials
```

#### Authentication Flow

1. **PKCE Flow (Default)**:
   - Generates cryptographic verifier/challenge
   - Opens browser for user authorization
   - Handles callback with authorization code
   - Exchanges code for access/refresh tokens
   - Tokens stored securely in system keyring

2. **Device Flow (Headless)**:
   - Activates automatically in headless environments
   - Displays verification URL and user code
   - Polls for authorization completion
   - Supports timeout handling

#### Environment Variables

```bash
# Custom Identity Provider
export OAUTH_CLIENT_ID="your-client-id"
export OAUTH_CLIENT_SECRET="your-client-secret"

# Use device flow explicitly
export NO_BROWSER=true

# Custom CA certificate (corporate proxies)
export CLI_CUSTOM_CA_CERT=/path/to/ca.pem
```

## Supported Providers

### Major AI Providers (Commercial)

| Provider | Base URL | Status |
|----------|----------|--------|
| OpenAI | https://api.openai.com/v1 | ✅ Tested |
| Anthropic | https://api.anthropic.com/v1 | ✅ Tested |
| Mistral AI | https://api.mistral.ai/v1 | ✅ Tested |
| Cohere | https://api.cohere.ai/v1 | ✅ Tested |
| Google AI Studio | https://generativelanguage.googleapis.com/v1 | 🔄 Supported |

### OpenAI-Compatible Platforms

| Provider | Base URL | Status |
|----------|----------|--------|
| Groq | https://api.groq.com/openai/v1 | ✅ Tested |
| OpenRouter | https://openrouter.ai/api/v1 | ✅ Tested |
| Cerebras | https://api.cerebras.ai/v1 | ✅ Tested |
| Cloudflare Workers AI | https://gateway.ai.cloudflare.com/v1 | ✅ Tested |
| DeepSeek | https://api.deepseek.com/v1 | 🔄 Supported |
| Together | https://api.together.xyz/v1 | 🔄 Supported |
| Fireworks AI | https://api.fireworks.ai/inference/v1 | 🔄 Supported |
| xAI (Grok) | https://api.x.ai/v1 | 🔄 Supported |
| Perplexity AI | https://api.perplexity.ai/v1 | 🔄 Supported |
| Replicate | https://api.replicate.com/v1 | 🔄 Supported |
| Anyscale | https://api.endpoints.anyscale.com/v1 | 🔄 Supported |
| DeepInfra | https://api.deepinfra.com/v1 | 🔄 Supported |
| Novita AI | https://api.novita.ai/v1 | 🔄 Supported |
| SambaNova | https://api.sambanova.ai/v1 | 🔄 Supported |
| NVIDIA NIM | https://integrate.api.nvidia.com/v1 | 🔄 Supported |

### Local/On-Premise Servers

| Provider | Base URL | Status |
|----------|----------|--------|
| Ollama | http://localhost:11434/v1 | 🔄 Supported |
| LM Studio | http://localhost:1234/v1 | 🔄 Supported |
| vLLM | http://localhost:8000/v1 | 🔄 Supported |

### Enterprise Cloud Services

| Provider | Base URL | Status |
|----------|----------|--------|
| Azure OpenAI | https://{resource}.openai.azure.com/v1 | 🔄 Supported |
| AWS Bedrock | https://bedrock-runtime.{region}.amazonaws.com | 🔄 Supported |
| Google Vertex AI | https://{region}-aiplatform.googleapis.com/v1 | 🔄 Supported |

### Other Providers

| Provider | Base URL | Status |
|----------|----------|--------|
| Hugging Face | https://api-inference.huggingface.co | 🔄 Supported |
| AI21 Labs | https://api.ai21.com/v1 | 🔄 Supported |
| Aleph Alpha | https://api.aleph-alpha.com/v1 | 🔄 Supported |

✅ = Tested | 🔄 = Supported (pending test)

> **Note**: For Azure, Bedrock, and Vertex AI, replace `{resource}` or `{region}` with your specific resource/region. Enterprise providers may require additional configuration (IAM roles, managed identities, etc.).

### OpenCode Integration

You can use llm-router directly from OpenCode as an OpenAI-compatible proxy:

```bash
# 1. Start the server
./target/release/llm-router --port 8080

# 2. Configure OpenCode (.opencode/opencode.json)
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

See [`.opencode/tools/OPENCODE_INTEGRATION.md`](.opencode/tools/OPENCODE_INTEGRATION.md) for full documentation.

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
| `Cascading` | Quality-based escalation across model tiers |

## Intelligent Routing

The router provides intelligent routing strategies for cost optimization:

- **Cost-Aware Routing** (Issue #23): Static model selection by query complexity
- **Cascading Routing** (Issue #24): Dynamic quality-based escalation
- **Task-Based Routing** (Issue #26): Route by task type (General, Chat, Code, Reasoning, Summarization, Translation)
- **Routing Configuration** (Issue #29): CLI flags and environment variables for all strategies

### Quick Configuration

```bash
# Via environment variables
export ROUTING_STRATEGY=auto           # auto, cost-optimized, cascading, failover, load-balanced
export CASCADING_ENABLED=true
export CASCADING_MIN_QUALITY=0.75
export BUDGET_MODE=true

# Via CLI flags
llm-router --routing-strategy cascading --cascading --quality-threshold 0.85
```

See [docs/routing.md](docs/routing.md) for detailed routing documentation.

## Failover

- **Circuit Breaker**: Opens after 5 consecutive failures
- **Auto-Close**: Retries after 30 seconds
- **Health Scoring**: 0-100 score based on success/latency

## Configuration

### Environment Variables

```bash
# Server configuration
PORT=8080                                    # Server port (default: 8080)
HOST=0.0.0.0                                 # Host (default: 0.0.0.0)
LOG_LEVEL=info                               # Log level (trace, debug, info, warn, error)

# Planning configuration
PLANNING_TIMEOUT_MS=5000                     # Planning timeout (default: 5000ms)
MAX_ACCOUNTS_PER_PAN=3                       # Max accounts per plan (default: 3)

# Routing configuration (Issue #29)
ROUTING_STRATEGY=auto                        # auto, cost-optimized, cascading, failover, load-balanced
CASCADING_ENABLED=false                     # Enable cascading routing
CASCADING_MIN_QUALITY=0.75                   # Minimum quality score (0.0-1.0)
CASCADING_MAX_TIERS=3                       # Maximum tiers to try
CASCADING_PER_TIER_TIMEOUT_MS=5000         # Timeout per tier
BUDGET_MODE=false                           # Enable budget mode
MAX_RETRIES=3                               # Maximum retries per request
REQUEST_TIMEOUT_SECONDS=60                  # Request timeout

# Secure storage (Issue #22)
SECURE_STORAGE=auto                          # auto, keyring, encrypted, disabled

# Live contract tests (set to 1 to enable)
LIVE_TEST=1
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

#### Live Contract Tests

Run real API contract tests against provider APIs (requires API keys):

```bash
# Enable live tests
LIVE_TEST=1 cargo test --test live_contract_tests -- --ignored

# Individual provider tests
LIVE_TEST=1 OPENAI_API_KEY=your-key cargo test --test live_contract_tests -- --ignored test_openai_contract
LIVE_TEST=1 ANTHROPIC_API_KEY=your-key cargo test --test live_contract_tests -- --ignored test_anthropic_contract
LIVE_TEST=1 GROQ_API_KEY=your-key cargo test --test live_contract_tests -- --ignored test_groq_contract
```

> **Note**: Live tests are marked `#[ignore]` and gated behind `LIVE_TEST=1` + provider API key env vars. They run on CI only on `push` to `main`.

#### Optimal Development Stack 2025-26

The project uses the optimal Rust development stack:

- **cargo-nextest**: Test runner (4x faster)
- **cargo-llvm-cov**: Native LLVM coverage (10x faster)
- **sccache**: Build cache (6x faster)
- **cargo-watch**: Auto-rebuild on changes
- **wiremock**: HTTP mocking for integration tests (complemented by live contract tests)
- **mockall**: Trait mocking
- **fs4**: Advisory file locking with tokio support
- **insta**: Snapshot testing for drift detection

See [DEVELOPMENT.md](DEVELOPMENT.md) for the complete guide.

### Lint

```bash
cargo clippy
cargo fmt
```

### Security

### Secure API Key Storage (Issue #22)

The router stores API keys securely using:

- **System Keyring** (default): macOS Keychain, Windows Credential Manager, Linux Secret Service
- **Encrypted File Fallback**: AES-256-GCM with Argon2id key derivation
- **Automatic Migration**: Plaintext keys in `accounts.json` are migrated automatically

```bash
# Configuration via environment variable
export SECURE_STORAGE=auto        # Default: use keyring if available
export SECURE_STORAGE=encrypted   # Force encrypted file storage
export SECURE_STORAGE=disabled    # Dev/testing only (not recommended)

# CLI commands
llm-router account secure-status  # Check storage status
llm-router account migrate         # Migrate to secure storage
```

See [docs/security.md](docs/security.md) for full security documentation.

### OAuth 2.1 / PKCE

```bash
# Login with OAuth 2.1 PKCE (opens browser)
llm-router auth login --provider groq

# Login with Device Flow (headless)
llm-router auth login --provider groq --device-flow

# Logout
llm-router auth logout --provider groq
```

### Security Audit

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
- [x] **Supported Providers**: 29 LLM providers (OpenAI, Anthropic, Groq, Mistral, and 25 more including local and enterprise options)
- [x] **80.35% Code Coverage** with 492 tests
- [x] **Execution Planning Module** with proactive failover
- [x] **Auth Login with --provider argument** (configurable provider)
- [x] **Provider List with account status** (shows configured/not set)
- [x] **Provider Models command** (list available models)
- [x] **Cost-Aware Routing** (#23) - Static model selection by query complexity
- [x] **Cascading Routing** (#24) - Dynamic quality-based escalation
- [x] **Live Contract Tests** for provider API drift detection
- [x] **Atomic JSON Persistence** with file locking
- [x] **Task-Based Routing** (#26) - Route by task type (General, Chat, Code, Reasoning, Summarization, Translation)
- [x] **Modern Interactive CLI** (#19) - Colored output, tables, prompts, spinners
- [x] **Routing Configuration CLI/Env** (#29) - CLI flags and environment variables for routing
- [x] **Docker & Containerization** (#15) - Dockerfile, docker-compose, GitHub Actions
- [x] **Secure API Key Storage** (#22) - System keyring + encrypted file fallback

### Pending 🔄

- [ ] Kubernetes manifests (beyond basic docker-compose)
- [ ] Comprehensive testing of all 29 providers (especially enterprise and specialized platforms)
- [ ] Provider-specific authentication and configuration guides

## Project Structure

```
src/
├── domain/           # Domain layer (entities, traits, errors)
│   ├── entities/     # Core business entities
│   ├── traits/       # Ports/interfaces
│   ├── errors/      # Domain errors
│   └── services/    # Domain services
│       ├── model_selector.rs    # CostAwareSelector (Issue #23)
│       └── query_complexity.rs  # QueryClassifier + TaskType (Issues #23, #26)
├── app/             # Application layer (use cases, services)
│   ├── services/    # Application services
│   │   ├── account_rotation.rs
│   │   ├── execution_plan/
│   │   │   ├── cascading.rs     # CascadingExecutionPlan (Issue #24)
│   │   │   ├── types.rs         # ExecutionPlanType enum
│   │   │   └── planner.rs       # ExecutionPlanner
│   │   ├── quality/
│   │   │   └── evaluator.rs     # HeuristicQualityEvaluator (Issue #24)
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
│   ├── secure_storage/         # Secure API Key Storage (Issue #22)
│   │   ├── mod.rs
│   │   ├── keyring_adapter.rs
│   │   └── encrypted_store.rs
│   └── auth/         # Authentication strategies
├── presentation/   # Presentation layer
│   ├── handlers/   # HTTP handlers
│   ├── routes.rs   # Route definitions
│   ├── state.rs    # Application state
│   └── cli/        # CLI commands
│       ├── mod.rs       # CLI dispatcher
│       ├── commands/    # CLI command implementations
│       │   ├── provider.rs    # Provider CRUD
│       │   ├── account.rs     # Account CRUD
│       │   ├── auth.rs        # Login/logout
│       │   ├── login.rs      # OAuth login
│       │   └── logout.rs     # OAuth logout
│       └── ui/          # Modern CLI UI (Issue #19)
│           ├── output.rs      # Colored output
│           ├── spinner.rs     # Progress spinners
│           ├── table.rs       # Professional tables
│           ├── prompt.rs      # Interactive prompts
│           └── tty.rs        # TTY detection
├── config/         # Configuration
│   ├── mod.rs
│   └── routing.rs   # Routing Configuration (Issue #29)
├── lib.rs          # Library root
└── main.rs         # Binary entry point
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

Built with Clean Architecture and DDD patterns in Rust.
