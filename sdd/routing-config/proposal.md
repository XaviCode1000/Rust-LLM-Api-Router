# Change Proposal: CLI and Configuration Options for Routing Strategies (Issue #29)

## Intent

Agregar CLI flags, variables de entorno y soporte de headers HTTP para configurar las estrategias de routing (Cost-Aware, Cascading, Failover, Load Balancing) sin necesidad de cambiar código.

## Problem

Las estrategias de routing Cost-Aware (#23) y Cascading (#24) están implementadas pero **requieren cambios en código para habilitarse**. No hay forma de que un usuario configure:

- Qué estrategia de routing usar
- Umbral de calidad para cascading
- Modo budget
- Número máximo de retries
- Timeout por request
- Fallback chain de modelos

**Lo que ya existe:**
- `PlanningOptions` struct con presets (reliability, cost_optimized, low_latency, cascading)
- `ExecutionPlannerConfig` con flags internos
- Algunas env vars básicas (`EXECUTION_PLAN_TYPE`, `EXECUTION_AUTO_SELECTION`, etc.)
- `Settings.cascading_min_quality_score` — existe pero NO está wired al planner
- **CERO** CLI flags relacionados con routing

## Scope

### Incluido (Phase 1 + 2 — HIGH/MEDIUM priority)
1. **CLI flags** para server start: `--routing-strategy`, `--cascading`, `--quality-threshold`, `--budget-mode`, `--max-retries`, `--timeout`
2. **Environment variables** adicionales: `ROUTING_STRATEGY`, `CASCADING_ENABLED`, `CASCADING_MIN_QUALITY`, `MAX_TIERS`, `PER_TIER_TIMEOUT_MS`, `BUDGET_MODE`
3. **Wire up** `Settings.cascading_min_quality_score` al planner
4. **RoutingConfig struct** centralizado que unifica CLI + env + defaults
5. **Rich help** con ejemplos en cada flag
6. **Logging** que muestra qué estrategia se usó en cada request

### NO Incluido (Future Phases)
- Headers HTTP para per-request overrides (Phase 3)
- Request body parameters (Phase 4)
- Config file YAML/TOML
- UI/web dashboard para configuración

## Approach

### Arquitectura de Configuración

```
Config Hierarchy (highest → lowest priority):
1. CLI flags (--routing-strategy, --cascading, etc.)
2. Environment variables (ROUTING_STRATEGY, CASCADING_ENABLED, etc.)
3. RoutingConfig defaults
```

### Nuevo `RoutingConfig` Struct

```rust
// src/config/routing.rs
pub struct RoutingConfig {
    pub strategy: RoutingStrategy,       // auto, cost_optimized, cascading, failover, load_balanced
    pub cascading_enabled: bool,
    pub cascading_min_quality: f64,      // 0.0-1.0, default 0.75
    pub cascading_max_tiers: u32,        // default 3
    pub cascading_per_tier_timeout_ms: u64, // default 5000
    pub budget_mode: bool,
    pub max_cost_per_million: Option<f64>,
    pub max_retries: u32,                // default 3
    pub timeout_seconds: u64,            // default 60
}
```

### CLI Flags en `Cli` Struct

```rust
// src/presentation/cli/mod.rs
#[derive(Debug, Parser)]
pub struct Cli {
    // ... existing fields ...
    
    /// Routing strategy: auto, cost-optimized, cascading, failover, load-balanced
    #[arg(long, default_value = "auto")]
    pub routing_strategy: String,
    
    /// Enable cascading (quality-based escalation)
    #[arg(long)]
    pub cascading: bool,
    
    /// Minimum quality score for cascading (0.0-1.0)
    #[arg(long, default_value = "0.75")]
    pub quality_threshold: f64,
    
    /// Enable budget mode (select cheapest model)
    #[arg(long)]
    pub budget_mode: bool,
    
    /// Maximum retries per request
    #[arg(long, default_value = "3")]
    pub max_retries: u32,
    
    /// Request timeout in seconds
    #[arg(long, default_value = "60")]
    pub timeout: u64,
}
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ROUTING_STRATEGY` | "auto" | auto, cost-optimized, cascading, failover, load-balanced |
| `CASCADING_ENABLED` | "false" | Enable cascading routing |
| `CASCADING_MIN_QUALITY` | "0.75" | Minimum quality score (0.0-1.0) |
| `CASCADING_MAX_TIERS` | "3" | Maximum tiers to try |
| `CASCADING_PER_TIER_TIMEOUT_MS` | "5000" | Timeout per tier in ms |
| `BUDGET_MODE` | "false" | Enable budget mode |
| `MAX_RETRIES` | "3" | Maximum retries per request |
| `REQUEST_TIMEOUT_SECONDS` | "60" | Request timeout |

### Integration Flow

```
CLI flags + Env vars
    ↓
RoutingConfig::from_cli_and_env()
    ↓
ExecutionPlannerConfig (existing)
    ↓
select_plan_type() (existing logic, already honors flags)
    ↓
ExecutionContext.planning_options (existing)
    ↓
Plan execution
```

### Logging de Estrategia Usada

```rust
// In LlmRouter::route_request()
info!(
    request_id = %context.request_id,
    strategy = %plan.plan_type().name(),
    accounts = %plan.account_count(),
    "Routing request"
);
```

## Impact

### Files Modified
| File | Change |
|------|--------|
| `src/config/routing.rs` | **NEW** — RoutingConfig struct with CLI + env loading |
| `src/config/mod.rs` | Re-export RoutingConfig |
| `src/presentation/cli/mod.rs` | Add routing flags to Cli struct |
| `src/app/services/execution_plan/planner.rs` | Wire RoutingConfig to planner |
| `src/app/router/llm_router.rs` | Pass routing config to planner |
| `src/main.rs` | Initialize RoutingConfig from CLI + env |
| `docs/cli.md` | Document new flags |
| `docs/routing.md` | Document configuration options |

### Risks
- **Config priority**: CLI > Env > Defaults must be clear and consistent
- **Backward compat**: Existing env vars (`EXECUTION_PLAN_TYPE`, etc.) must still work
- **Validation**: Quality threshold must be 0.0-1.0, strategy must be valid enum value

## Alternatives Considered

| Alternative | Why Rejected |
|-------------|-------------|
| **Solo env vars** | CLI flags son más discoverables y documentables con --help |
| **Config file YAML** | Overkill para Phase 1, se puede agregar después |
| **Modificar ExecutionPlannerConfig directamente** | Mejor crear RoutingConfig como capa de abstracción |
| **Headers HTTP en Phase 1** | Complejidad adicional, mejor enfocarse en CLI/env primero |
