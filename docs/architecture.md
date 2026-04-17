# Architecture

System architecture documentation for the LLM API Router.

## Overview

The LLM API Router follows **Clean Architecture** and **Domain-Driven Design (DDD)** principles, providing intelligent request routing across 34 LLM providers with automatic failover, health monitoring, and cost optimization.

```
┌─────────────────────────────────────────────────────────┐
│                 Presentation Layer                       │
│  (HTTP Handlers, Routes, CLI, Middleware, Extractors)   │
├─────────────────────────────────────────────────────────┤
│                 Interfaces Layer                         │
│  (API request/response types, IntoResponse impls,       │
│   SSE events, Error serialization)                      │
├─────────────────────────────────────────────────────────┤
│                 Application Layer                        │
│  (Use Cases, Execution Plans, Quality Evaluation,       │
│   Rotation Strategies, Failover Manager, Internal Router)│
├─────────────────────────────────────────────────────────┤
│                   Domain Layer                           │
│  (Entities, Traits/Ports, Domain Services, Errors)      │
├─────────────────────────────────────────────────────────┤
│              Infrastructure Layer                        │
│  (HTTP Client, Gateway, JSON Persistence, Provider      │
│   Adapters, Auth Strategies, Secure Storage, Metrics)   │
└─────────────────────────────────────────────────────────┘
```

## Dependency Rule

Dependencies point **inward**:

```
Presentation → Interfaces → Application → Domain ← Infrastructure
```

- **Domain** has zero external dependencies (pure business logic) — *enforced since Issue #30 audit*
- **Interfaces** contains wire-format types and HTTP presentation concerns
- **Application** depends only on Domain and Interfaces
- **Infrastructure** implements Domain traits using external crates
- **Presentation** orchestrates Application and Infrastructure

---

## Layers

### Domain Layer

**Location:** `src/domain/`

Core business entities, traits (ports), domain services, and error types.

```
src/domain/
├── entities/           # Core domain models
│   ├── mod.rs
│   ├── account.rs      # Account (API key + provider association)
│   ├── provider.rs     # Provider configuration
│   ├── chat.rs         # ChatRequest, ChatResponse, Message
│   ├── openai_types.rs # OpenAI-compatible types
│   └── account_health.rs # Health scoring, circuit breaker
├── traits/             # Ports (interfaces)
│   ├── mod.rs
│   ├── account_repository.rs  # Account CRUD trait
│   ├── provider_repository.rs # Provider CRUD trait
│   └── llm_gateway.rs         # LLM provider gateway trait
├── services/           # Domain services
│   ├── mod.rs
│   ├── model_selector.rs    # CostAwareSelector, ModelSelector trait
│   ├── query_complexity.rs  # QueryClassifier, QueryComplexity, TaskType
│   ├── token_validator.rs   # Token validation
│   └── auth_strategy.rs     # Auth strategy trait
└── errors/             # Domain error types
    └── mod.rs
```

**Key Entities:**

```rust
// Account - API key associated with a provider
pub struct Account {
    pub id: String,
    pub provider_id: String,
    pub api_key: String,
    pub is_active: bool,
    pub priority: i32,
}

// Provider - LLM provider configuration
pub struct Provider {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub enabled: bool,
}

// ChatRequest - LLM chat request (OpenAI-compatible)
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
}
```

**Traits (Ports):**

```rust
// Repository for accounts
#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn save(&self, account: Account) -> DomainResult<Account>;
    async fn find_all(&self) -> DomainResult<Vec<Account>>;
    async fn find_by_id(&self, id: &str) -> DomainResult<Account>;
    async fn find_active(&self) -> DomainResult<Vec<Account>>;
    async fn find_active_by_provider(&self, provider_id: &str) -> DomainResult<Vec<Account>>;
}

// Gateway for LLM providers
#[async_trait]
pub trait LlmGateway: Send + Sync {
    async fn chat(&self, request: ChatRequest, api_key: &str) -> DomainResult<ChatResponse>;
    async fn list_models(&self, api_key: &str) -> DomainResult<Vec<Model>>;
}

// Model selector trait (Cost-Aware Routing, Issue #23)
pub trait ModelSelector: Send + Sync {
    fn select<'a>(
        &self,
        request: &ChatRequest,
        available_models: &'a [Model],
    ) -> SelectionResult<&'a Model>;
    fn strategy_name(&self) -> &'static str;
}
```

**Domain Services:**

```rust
// Query complexity classification
pub enum QueryComplexity {
    Low = 0,    // Short queries, simple greetings
    Medium = 1, // Conversational, code keywords
    High = 2,   // Complex reasoning, design tasks
}

pub struct QueryClassifier {
    config: ClassifierConfig,  // Thresholds and keywords
}

// Cost-aware model selector
pub struct CostAwareSelector {
    classifier: QueryClassifier,
    max_cost_per_million_tokens: Option<f64>,
}

// Task type classification (Issue #26)
pub enum TaskType {
    General,      // General conversation, Q&A
    Chat,         // Multi-turn dialogue
    Code,         // Code generation, debugging
    Reasoning,    // Logical reasoning, math, analysis
    Summarization,// Text summarization
    Translation,  // Language translation
}
```

---

### Application Layer

**Location:** `src/app/`

Use cases, execution plans, quality evaluation, and business orchestration.

```
src/app/
├── mod.rs
├── health.rs             # App-level health checks
├── services/             # Application services
│   ├── mod.rs
│   ├── account_rotation.rs   # RoundRobin, Weighted, Latency, UserAffinity
│   ├── failover.rs           # Failover manager with circuit breaker
│   ├── execution_plan/       # Execution plan module (10 sub-modules)
│   │   ├── mod.rs
│   │   ├── cascading.rs      # CascadingExecutionPlan (Issue #24)
│   │   ├── context.rs        # Planning context
│   │   ├── execution.rs      # Execution logic
│   │   ├── implementations.rs # Plan implementations
│   │   ├── metrics.rs        # Prometheus metrics
│   │   ├── outcome.rs        # Execution outcomes
│   │   ├── plan.rs           # Plan struct
│   │   ├── planner.rs        # ExecutionPlanner
│   │   ├── status.rs         # Plan status
│   │   ├── tracing.rs        # OpenTelemetry tracing
│   │   └── types.rs          # ExecutionPlanType, PlannedAccount
│   └── quality/              # Quality evaluation (Issue #24)
│       ├── mod.rs
│       └── evaluator.rs      # HeuristicQualityEvaluator, QualityGate trait
└── router/                 # Internal LLM routing
    └── llm_router.rs
```

**Key Services:**

#### Account Rotation

4 rotation strategies:

```rust
// Round-Robin - Sequential
pub struct RoundRobinStrategy {
    index: AtomicUsize,
}

// Weighted - Based on priority
pub struct WeightedStrategy;

// Latency - Lowest latency first
pub struct LatencyStrategy;

// User-Affinity - Same account per user
pub struct UserAffinityStrategy {
    last_selection: tokio::sync::Mutex<HashMap<String, String>>,
}
```

#### Failover Manager

```rust
pub struct FailoverManager {
    account_repo: Arc<JsonAccountRepository>,
    selector: AccountSelector,
    max_retries: u32,
    health_map: tokio::sync::Mutex<HashMap<String, AccountHealth>>,
}
```

**Circuit Breaker:**
- Opens after 5 consecutive failures
- Auto-closes after 30 seconds
- Health scoring 0-100

#### Execution Plan Module (10 Sub-modules)

The execution plan module provides proactive planning for LLM request execution:

| Module | Purpose |
|--------|---------|
| `cascading.rs` | CascadingExecutionPlan with quality-based escalation |
| `context.rs` | Planning context (request ID, model, options) |
| `execution.rs` | Core execution logic |
| `implementations.rs` | Concrete plan implementations |
| `metrics.rs` | Prometheus metrics export |
| `outcome.rs` | Execution result types |
| `plan.rs` | Plan data structure |
| `planner.rs` | ExecutionPlanner orchestrator |
| `status.rs` | Plan status tracking |
| `tracing.rs` | OpenTelemetry distributed tracing |
| `types.rs` | ExecutionPlanType enum, PlannedAccount struct |

**Execution Plan Types:**

```rust
pub enum ExecutionPlanType {
    Standard,      // Single account execution
    Failover,      // Sequential fallback on failure
    LoadBalanced,  // Health-weighted distribution across accounts
    CostOptimized, // Cheapest provider selection
    Cascading,     // Quality-based escalation (Issue #24)
}
```

#### Cascading Execution Plan (Issue #24)

Attempts cheapest tier first, escalates if quality insufficient.

```rust
pub struct CascadingExecutionPlan {
    inner: ExecutionPlanImpl,
    tiers: Vec<CascadingTier>,
    quality_config: QualityConfig,
    total_cost_microdollars: u64,
    tiers_attempted: u32,
    quality_gate: Arc<dyn QualityGate>,
}

pub struct CascadingTier {
    pub account: PlannedAccount,
    pub model_id: String,
    pub tier_order: u32,
}
```

**Cascading Flow:**
1. Execute with cheapest tier (Tier 0)
2. Evaluate response quality (4 heuristic checks)
3. If quality >= threshold (default 0.75) → return success
4. If quality < threshold → escalate to next tier
5. Repeat until success or tiers exhausted
6. Track accumulated cost in microdollars

**Streaming Guard:**
- Cascading automatically disabled for streaming requests
- Quality cannot be evaluated until stream completes
- Falls back to Standard execution plan

#### Quality Gate (Issue #24)

Trait for evaluating response quality without additional LLM calls.

```rust
#[async_trait]
pub trait QualityGate: Send + Sync {
    async fn evaluate_quality(
        &self,
        account: &PlannedAccount,
        response: &str,
        health: &AccountHealth,
    ) -> QualityScore;
}

pub struct QualityScore {
    pub score: f64,              // 0.0 to 1.0
    pub is_acceptable: bool,     // score >= min_quality_score
    pub checks_failed: Vec<String>, // Failed check names
}
```

**HeuristicQualityEvaluator — 4 Checks:**

| Check | What It Measures | Failure Condition |
|-------|------------------|-------------------|
| **Completeness** | Response not truncated | Ends with `,`, `:`, `;`, `-`, `{`, `[`, or whitespace |
| **Length** | Minimum response size | < 10 characters (configurable) |
| **Structure** | Valid JSON when expected | Unmatched `{`/`}` or `[`/`]` |
| **Coherence** | No error patterns | Contains "I cannot", "As an AI", repeated words (4+) |

**Quality Configuration:**

```rust
pub struct QualityConfig {
    pub min_quality_score: f64,     // Default: 0.75
    pub min_response_length: usize, // Default: 10
    pub max_tiers: u32,             // Default: 3
    pub per_tier_timeout_ms: u64,   // Default: 5000
}
```

---

### Infrastructure Layer

**Location:** `src/infrastructure/`

Concrete implementations of Domain traits, external integrations.

```
src/infrastructure/
├── mod.rs
├── http_client.rs          # Shared HTTP client (reqwest, rustls-tls)
├── logging.rs              # Logging (tracing)
├── metrics.rs              # Prometheus metrics
├── gateway/                # LLM Gateway
│   ├── mod.rs
│   ├── llm_gateway.rs      # LlmGatewayImpl, ProviderConfig, default_providers()
│   └── README.md       # Module removed during docs consolidation (2026-04-17)
├── provider/               # Provider adapters
│   ├── mod.rs
│   ├── openai.rs           # OpenAI-compatible format
│   ├── anthropic.rs        # Anthropic format (API v2023-06-01)
│   └── groq.rs             # Groq format
├── persistence/            # JSON file persistence
│   ├── mod.rs
│   ├── json_provider_repository.rs
│   └── json_account_repository.rs  # Atomic writes + fs4 locking
├── auth/                   # Authentication strategies
│   ├── mod.rs
│   ├── api_key_strategy.rs # API Key authentication
│   ├── pkce_strategy.rs    # OAuth 2.1 PKCE
│   ├── device_flow_strategy.rs # OAuth Device Flow (headless)
│   └── README.md       # Module removed during docs consolidation (2026-04-17)
└── secure_storage/         # Encrypted credential storage
    ├── mod.rs
    ├── encrypted_store.rs  # Encrypted key-value store
    └── keyring_adapter.rs  # OS keyring integration
```

**LLM Gateway:**

The gateway aggregates requests across multiple LLM providers with caching and routing:

```rust
pub struct LlmGatewayImpl {
    http_client: SharedHttpClient,
    account_repo: Arc<dyn AccountRepository>,
    provider_config: HashMap<String, ProviderConfig>,
    cache: Arc<RwLock<HashMap<String, CachedResponse>>>,
    cache_ttl_secs: u64,
}
```

**JsonAccountRepository:**

```rust
pub struct JsonAccountRepository {
    file_path: PathBuf,  // ~/.config/rust-llm-api-router/accounts.json
}

// Atomic writes: write-to-temp-then-rename (eliminates TOCTOU race)
// Advisory locking: fs4 with tokio support
// Shared read locks, exclusive write locks, 5-second lock timeout
// Stale temp file cleanup on init
```

**Authentication Strategies:**

| Strategy | Location | Use Case |
|----------|----------|----------|
| API Key | `api_key_strategy.rs` | Traditional API key auth |
| OAuth 2.1 PKCE | `pkce_strategy.rs` | Browser-based OAuth with PKCE |
| Device Flow | `device_flow_strategy.rs` | Headless environments (no browser) |

**Secure Storage:**

| Component | Location | Purpose |
|-----------|----------|---------|
| Encrypted Store | `encrypted_store.rs` | Encrypted key-value credential storage |
| Keyring Adapter | `keyring_adapter.rs` | OS-native keyring integration (macOS Keychain, Linux Secret Service, Windows Credential Manager) |

---

### Presentation Layer

**Location:** `src/presentation/` + `src/interfaces/`

HTTP handlers, routes, CLI commands, middleware, and extractors.

```
src/presentation/
├── mod.rs
├── routes.rs                  # Route definitions
├── state.rs                   # AppState
└── cli/                       # CLI (13 files: 6 commands + 7 helpers)
    ├── mod.rs                 # Cli struct, CliCommands enum, dispatcher
    ├── input.rs               # Input helpers
    ├── output.rs              # Output helpers
    ├── prompt.rs              # Interactive prompts
    ├── spinner.rs             # Progress spinners
    ├── table.rs               # Table formatting
    ├── tty.rs                 # TTY detection
    └── commands/              # CLI commands (6)
        ├── mod.rs
        ├── provider.rs        # Provider subcommands (add, list, enable, disable, remove, validate)
        ├── account.rs         # Account subcommands (add, list, set-priority, remove, validate)
        ├── auth.rs            # Auth subcommands (login, logout)
        ├── login.rs           # Login implementation
        ├── logout.rs          # Logout implementation
        └── completions.rs     # Shell completions (feature-gated: --features completions)

src/interfaces/
├── mod.rs
├── handlers/                  # HTTP handlers
│   ├── mod.rs
│   ├── chat.rs                # POST /v1/chat/completions
│   └── metrics.rs             # GET /metrics
├── extractors/                # Custom Axum extractors
├── middleware/                # HTTP middleware
└── responses/                 # SSE event streaming (4 functions)
```

**Health handlers** are also in `src/interfaces/handlers/`:

```rust
// Health check handlers
pub async fn health(...) -> impl IntoResponse { ... }
pub async fn health_detail(...) -> impl IntoResponse { ... }
```

**AppState:**

```rust
pub struct AppState {
    pub config: Settings,
    pub http_client: Arc<HttpClient>,
    pub metrics: Arc<Metrics>,
    pub account_repo: Arc<dyn AccountRepository>,
    pub llm_gateway: Arc<LlmGatewayImpl>,
    pub provider_config: Arc<HashMap<String, ProviderConfig>>,
    pub llm_router: Arc<LlmRouter<dyn AccountRepository>>,
}
```

**Routes:**

```rust
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        // Health checks
        .route("/health", get(health))
        .route("/health/detail", get(health_detail))
        // OpenAI-compatible API
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/models", get(list_models))
        // Account management
        .route("/accounts", get(list_accounts))
        // Metrics
        .route("/metrics", get(metrics))
}
```

**CLI Structure:**

```bash
llm-router [OPTIONS] <COMMAND>

Commands:
  provider      Provider management (add, list, enable, disable, remove, validate)
  account       Account management (add, list, set-priority, remove, validate)
  auth          Authentication (login, logout)
  completions   Shell completions (bash, zsh, fish) -- requires --features completions

Options:
      --host <HOST>                      Host to bind [default: 0.0.0.0]
  -p, --port <PORT>                      Port to bind [default: 8080]
      --log-level <LEVEL>                Log level [default: info]
      --routing-strategy <STRATEGY>      auto, cost-optimized, cascading, failover, load-balanced
      --cascading                        Enable cascading routing
      --quality-threshold <SCORE>        Min quality score [default: 0.75]
      --budget-mode                      Enable budget mode
      --max-retries <N>                  Max retries [default: 3]
      --timeout <SECONDS>                Request timeout [default: 60]
```

---

## Design Patterns

### Repository Pattern

Separates domain from persistence:

```rust
// Domain defines the trait
#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn save(&self, account: Account) -> DomainResult<Account>;
    // ...
}

// Infrastructure implements
pub struct JsonAccountRepository { ... }

#[async_trait]
impl AccountRepository for JsonAccountRepository { ... }
```

### Strategy Pattern

Interchangeable rotation strategies:

```rust
pub trait RotationStrategy: Send + Sync {
    fn select<'a>(&self, accounts: &'a [Account]) -> Option<&'a Account>;
}

pub struct RoundRobinStrategy { ... }
impl RotationStrategy for RoundRobinStrategy { ... }
```

### Circuit Breaker Pattern

Automatic failover with health tracking:

```rust
pub struct AccountHealth {
    pub consecutive_failures: u32,
    pub circuit_breaker_open: bool,
    pub circuit_breaker_opened_at: Option<u64>,
}

impl AccountHealth {
    pub fn record_failure(&mut self) {
        self.consecutive_failures += 1;
        if self.consecutive_failures >= 5 {
            self.open_circuit_breaker();
        }
    }

    pub fn can_make_request(&mut self) -> bool {
        if !self.circuit_breaker_open {
            return true;
        }
        // Auto-close after 30s
        if let Some(opened_at) = self.circuit_breaker_opened_at {
            if current_timestamp() - opened_at > 30 {
                self.circuit_breaker_open = false;
                return true;
            }
        }
        false
    }
}
```

---

## Request Flow

```
1. HTTP Request → POST /v1/chat/completions
       ↓
2. chat_completions handler (src/interfaces/handlers/chat.rs)
       ↓
3. Parse model: "groq:llama-3.1-8b-instant"
       ↓
4. account_repo.find_active_by_provider("groq")
       ↓
5. ExecutionPlanner creates optimal plan
       ↓
6. Execute plan (Standard/Failover/LoadBalanced/CostOptimized/Cascading)
       ↓
7. HTTP POST → https://api.groq.com/openai/v1/chat/completions
       ↓
8. Response → Convert to OpenAI format
       ↓
9. JSON Response → Client
```

---

## Community Breakdown (GitNexus Analysis)

| Functional Area | Functions | Modules | Description |
|-----------------|-----------|---------|-------------|
| **Tests** | 519 | 47 | Test functions across all modules |
| **Execution Plan** | 180 | 10 | Complex routing logic (cascading, failover, load balancing) |
| **Services** | 136 | 6 | Business logic (rotation, selection, quality evaluation) |
| **Entities** | 97 | 5 | Domain models (Account, Provider, Chat, Health) |
| **Persistence** | 40 | 3 | JSON file storage with atomic writes |
| **Auth** | 31 | 5 | OAuth 2.1/PKCE, API Key, Device Flow strategies |
| **CLI** | 27 | 13 | Command-line interface with colored output, TTY detection |
| **Handlers** | 18 | 4 | HTTP handlers (chat, health, metrics) |
| **Secure Storage** | 17 | 3 | Encrypted store, keyring adapter |
| **Provider** | 12 | 4 | OpenAI, Anthropic, Groq adapters |
| **Quality** | 11 | 2 | HeuristicQualityEvaluator with 4 checks |
| **Gateway** | 11 | 3 | LLM gateway router |
| **Router** | 10 | 1 | Main routing logic |
| **Config** | 5 | 2 | Settings, routing configuration |
| **Responses** | 4 | 1 | SSE event streaming |
| **Infrastructure** | 2 | 3 | Logging, metrics, HTTP client |

---

## Design Decisions

### Why Clean Architecture?

- **Testable**: Domain tested without infrastructure
- **Maintainable**: Infrastructure changes don't affect domain
- **Flexible**: Easy to add new providers

### Why JSON for Persistence?

- **Simple**: No database dependencies
- **Portable**: Easy backup/migrate
- **Sufficient**: For < 1000 accounts

### Why Round-Robin by Default?

- **Fair**: Distributes load evenly
- **Simple**: No complex state
- **Predictable**: Easy to debug

---

## Extensions

### Add a New Provider

1. Register via CLI:
```bash
llm-router provider add --id nuevo --name "Nuevo" --base-url "https://api.nuevo.com/v1"
```

2. (Optional) Specific adapter in `src/infrastructure/provider/nuevo.rs`

### Add a New Rotation Strategy

```rust
pub struct MiEstrategia;

impl RotationStrategy for MiEstrategia {
    fn select<'a>(&self, accounts: &'a [Account]) -> Option<&'a Account> {
        // Your logic here
    }
}
```

### Add a New Endpoint

1. Handler in `src/interfaces/handlers/mi_handler.rs`
2. Route in `src/presentation/routes.rs`

---

## Testing

### Unit Tests (Domain)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_health_score() {
        let mut health = AccountHealth::new("test-1");
        health.record_success(100);
        health.record_success(150);

        assert!(health.health_score() > 80.0);
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_chat_completions_endpoint() {
    let app = create_test_app();
    let response = app
        .oneshot(post("/v1/chat/completions").json(&request))
        .await;

    assert_eq!(response.status(), StatusCode::OK);
}
```

---

## References

- [Clean Architecture - Robert C. Martin](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Domain-Driven Design - Eric Evans](https://domainlanguage.com/ddd/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Routing Strategies](routing.md) — Cost-Aware, Cascading, Task-Based routing
- [Testing Guide](TESTING_GUIDE.md) — How to run tests, coverage goals
