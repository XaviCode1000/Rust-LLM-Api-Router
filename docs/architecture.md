# Arquitectura

Documentación de la arquitectura del sistema.

## Visión General

LLM API Router sigue los principios de **Clean Architecture** y **Domain-Driven Design (DDD)**.

```
┌─────────────────────────────────────────────────────────┐
│                 Presentation Layer                       │
│  (HTTP Handlers, Routes, CLI, Middleware)               │
├─────────────────────────────────────────────────────────┤
│                 Application Layer                        │
│  (Use Cases, Services, Rotation Strategies)             │
├─────────────────────────────────────────────────────────┤
│                   Domain Layer                           │
│  (Entities, Traits/Ports, Domain Errors)                │
├─────────────────────────────────────────────────────────┤
│              Infrastructure Layer                        │
│  (HTTP Client, JSON Persistence, Provider Adapters)     │
└─────────────────────────────────────────────────────────┘
```

## Regla de Dependencias

Las dependencias apuntan **hacia adentro**:

```
Presentation → Application → Domain ← Infrastructure
```

- **Domain** no depende de nada (puro)
- **Application** solo depende de Domain
- **Infrastructure** implementa traits de Domain
- **Presentation** usa Application e Infrastructure

---

## Capas

### Domain Layer

**Ubicación:** `src/domain/`

Entidades y traits core del negocio.

```
src/domain/
├── entities/           # Entidades
│   ├── mod.rs
│   ├── account.rs      # Account (API keys)
│   ├── provider.rs     # Provider
│   ├── chat.rs         # ChatRequest, ChatResponse
│   ├── openai_types.rs # Tipos OpenAI-compatible
│   └── account_health.rs # Health scoring
├── traits/             # Traits (ports)
│   ├── mod.rs
│   ├── account_repository.rs
│   ├── provider_repository.rs
│   └── llm_gateway.rs
├── services/           # Domain services
│   ├── mod.rs
│   ├── model_selector.rs    # CostAwareSelector, ModelSelector trait
│   └── query_complexity.rs  # QueryClassifier, QueryComplexity
└── errors/             # Errores de dominio
    └── mod.rs
```

**Entidades Principales:**

```rust
// Account - API key de un proveedor
pub struct Account {
    pub id: String,
    pub provider_id: String,
    pub api_key: String,
    pub is_active: bool,
    pub priority: i32,
}

// Provider - Configuración de proveedor
pub struct Provider {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub enabled: bool,
}

// ChatRequest - Request a LLM
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
// Repository para cuentas
#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn save(&self, account: Account) -> DomainResult<Account>;
    async fn find_all(&self) -> DomainResult<Vec<Account>>;
    async fn find_by_id(&self, id: &str) -> DomainResult<Account>;
    async fn find_active(&self) -> DomainResult<Vec<Account>>;
    async fn find_active_by_provider(&self, provider_id: &str) -> DomainResult<Vec<Account>>;
}

// Gateway para LLM providers
#[async_trait]
pub trait LlmGateway: Send + Sync {
    async fn chat(&self, request: ChatRequest, api_key: &str) -> DomainResult<ChatResponse>;
    async fn list_models(&self, api_key: &str) -> DomainResult<Vec<Model>>;
}

// Model selector trait (Issue #23: Cost-Aware Routing)
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
// Query complexity classification (Issue #23)
pub enum QueryComplexity {
    Low = 0,    // Short queries, simple questions
    Medium = 1, // Conversational, code keywords
    High = 2,   // Complex reasoning, design tasks
}

pub struct QueryClassifier {
    config: ClassifierConfig,  // Thresholds and keywords
}

// Cost-aware model selector (Issue #23)
pub struct CostAwareSelector {
    classifier: QueryClassifier,
    max_cost_per_million_tokens: Option<f64>,
}
```

---

### Application Layer

**Ubicación:** `src/app/`

Casos de uso y lógica de negocio.

```
src/app/
├── mod.rs
├── services/           # Servicios de aplicación
│   ├── mod.rs
│   ├── account_rotation.rs  # Estrategias de rotación
│   ├── failover.rs          # Failover manager
│   ├── execution_plan/      # Execution plan module
│   │   ├── mod.rs
│   │   ├── cascading.rs     # CascadingExecutionPlan (Issue #24)
│   │   ├── types.rs         # ExecutionPlanType enum
│   │   └── planner.rs       # ExecutionPlanner
│   └── quality/             # Quality evaluation (Issue #24)
│       ├── mod.rs
│       └── evaluator.rs     # HeuristicQualityEvaluator, QualityGate trait
└── router/             # Enrutamiento interno
    └── llm_router.rs
```

**Servicios Principales:**

#### Account Rotation

4 estrategias de rotación:

```rust
// Round-Robin - Secuencial
pub struct RoundRobinStrategy {
    index: AtomicUsize,
}

// Weighted - Basado en prioridad
pub struct WeightedStrategy;

// Latency - Menor latencia
pub struct LatencyStrategy;

// User-Affinity - Misma cuenta por usuario
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

- Se abre tras 5 fallos consecutivos
- Auto-cierre después de 30 segundos
- Health scoring 0-100

#### Cascading Execution Plan (Issue #24)

Estrategia de ejecución que intenta modelos más baratos y escala si la calidad es insuficiente.

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

**Flujo de Cascading:**

1. Ejecuta con el tier más barato (Tier 0)
2. Evalúa calidad de la respuesta (4 checks heurísticos)
3. Si calidad ≥ umbral (default 0.75) → retorna éxito
4. Si calidad < umbral → escala al siguiente tier
5. Repite hasta éxito o agotar tiers
6. Rastrea costo acumulado en microdólares

**Streaming Guard:**

- Cascading se deshabilita automáticamente para requests streaming
- La calidad no se puede evaluar hasta completar el stream
- Fallback a plan `Standard` en ese caso

#### Quality Gate (Issue #24)

Trait para evaluar calidad de respuestas sin llamadas adicionales al LLM.

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
    pub score: f64,              // 0.0 a 1.0
    pub is_acceptable: bool,     // score >= min_quality_score
    pub checks_failed: Vec<String>, // Nombres de checks fallidos
}
```

**HeuristicQualityEvaluator:**

Evaluación basada en 4 checks:

| Check | Qué evalúa | Condición de fallo |
|-------|------------|-------------------|
| **Completeness** | Respuesta no truncada | Termina con puntuación abierta |
| **Length** | Longitud mínima | < 10 caracteres |
| **Structure** | JSON válido cuando esperado | Llaves/brackets sin cerrar |
| **Coherence** | Sin patrones de error | Contiene "I cannot", repetición excesiva |

---

### Infrastructure Layer

**Ubicación:** `src/infrastructure/`

Implementaciones concretas.

```
src/infrastructure/
├── mod.rs
├── http_client.rs          # HTTP client (reqwest)
├── logging.rs              # Logging (tracing)
├── metrics.rs              # Métricas (prometheus)
├── persistence/            # Persistencia
│   ├── mod.rs
│   ├── json_provider_repository.rs
│   └── json_account_repository.rs
└── provider/               # Adapters de providers
    ├── mod.rs
    ├── openai.rs           # OpenAI-compatible
    └── anthropic.rs        # Anthropic (WIP)
```

**JsonAccountRepository:**

```rust
pub struct JsonAccountRepository {
    file_path: PathBuf,  // ~/.config/rust-llm-api-router/accounts.json
}

#[async_trait]
impl AccountRepository for JsonAccountRepository {
    async fn save(&self, account: Account) -> DomainResult<Account> { ... }
    async fn find_all(&self) -> DomainResult<Vec<Account>> { ... }
    async fn find_active(&self) -> DomainResult<Vec<Account>> { ... }
    // ...
}
```

---

### Presentation Layer

**Ubicación:** `src/presentation/` + `src/interfaces/`

HTTP handlers, routes y CLI commands.

```
src/presentation/
├── mod.rs
├── routes.rs                  # Definición de routes
├── state.rs                   # AppState
└── cli/
    ├── mod.rs                 # Cli struct, CliCommands enum, dispatcher
    └── commands/
        ├── mod.rs
        ├── provider.rs        # Provider subcommands
        ├── account.rs         # Account subcommands
        ├── auth.rs            # Auth (login/logout)
        ├── completions.rs     # Shell completions (feature-gated: --features completions)
        └── input.rs           # Shared input helpers

src/interfaces/
├── mod.rs
└── handlers/
    ├── mod.rs
    ├── chat_handler.rs        # /v1/chat/completions
    └── health_handler.rs      # /health, /health/detail
```

**Trait-based DI en CLI:**

Los comandos CLI reciben repositorios vía traits, no instancias concretas:

```rust
// Los comandos aceptan &impl ProviderRepository / &impl AccountRepository
pub fn handle_provider_add(
    repo: &impl ProviderRepository,
    args: ProviderAddArgs,
) -> Result<(), DomainError> { ... }
```

Esto permite testear los comandos con mocks sin dependencia de infraestructura.

**AppState:**

```rust
pub struct AppState {
    pub config: Settings,
    pub http_client: Arc<HttpClient>,
    pub metrics: Arc<Metrics>,
    pub account_repo: Arc<JsonAccountRepository>,
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

---

## Patrones de Diseño

### Repository Pattern

Separa dominio de persistencia:

```rust
// Domain define el trait
#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn save(&self, account: Account) -> DomainResult<Account>;
    // ...
}

// Infrastructure implementa
pub struct JsonAccountRepository { ... }

#[async_trait]
impl AccountRepository for JsonAccountRepository { ... }
```

### Strategy Pattern

Rotación de cuentas intercambiable:

```rust
pub trait RotationStrategy: Send + Sync {
    fn select<'a>(&self, accounts: &'a [Account]) -> Option<&'a Account>;
}

pub struct RoundRobinStrategy { ... }
impl RotationStrategy for RoundRobinStrategy { ... }

pub struct WeightedStrategy { ... }
impl RotationStrategy for WeightedStrategy { ... }
```

### Circuit Breaker Pattern

Failover automático:

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

## Flujo de Request

```
1. HTTP Request → POST /v1/chat/completions
       ↓
2. chat_completions handler
       ↓
3. Parse model: "groq:llama-3.1-8b-instant"
       ↓
4. account_repo.find_active_by_provider("groq")
       ↓
5. selector.select(accounts) → Account
       ↓
6. failover_manager.execute_with_failover(...)
       ↓
7. HTTP POST → https://api.groq.com/openai/v1/chat/completions
       ↓
8. Response → Convert to OpenAI format
       ↓
9. JSON Response → Client
```

---

## Decisiones de Diseño

### ¿Por qué Clean Architecture?

- **Testeable**: Domain se testea sin infraestructura
- **Mantenible**: Cambios en infra no afectan dominio
- **Flexible**: Fácil agregar nuevos providers

### ¿Por qué JSON para persistencia?

- **Simple**: Sin dependencias de DB
- **Portable**: Fácil backup/migrate
- **Suficiente**: Para < 1000 cuentas

### ¿Por qué round-robin por defecto?

- **Justo**: Distribuye carga equitativamente
- **Simple**: Sin estado complejo
- **Predecible**: Fácil de debuggear

---

## Extensiones

### Agregar Nuevo Provider

1. Registrar con CLI:
```bash
llm-router provider add --id nuevo --name "Nuevo" --base-url "https://api.nuevo.com/v1"
```

2. (Opcional) Adapter específico en `src/infrastructure/provider/nuevo.rs`

### Agregar Nueva Estrategia de Rotación

```rust
pub struct MiEstrategia;

impl RotationStrategy for MiEstrategia {
    fn select<'a>(&self, accounts: &'a [Account]) -> Option<&'a Account> {
        // Tu lógica aquí
    }
}
```

### Agregar Nuevo Endpoint

1. Handler en `src/interfaces/handlers/mi_handler.rs`
2. Route en `src/presentation/routes.rs`

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

## Referencias

- [Clean Architecture - Robert C. Martin](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Domain-Driven Design - Eric Evans](https://domainlanguage.com/ddd/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
