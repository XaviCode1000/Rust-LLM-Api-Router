# Architecture — GitNexus Stats

> GitNexus code knowledge graph statistics for Rust-LLM-Api-Router

## Repository Statistics

From GitNexus index (fresh re-index after latest changes):

| Metric | Value |
|--------|-------|
| **Files** | 205 |
| **Nodes** | 4,416 |
| **Edges** | 9,652 |
| **Communities** | 155 |
| **Processes** | 300 |
| **Indexed At** | 2026-04-17 (fresh re-index) |
| **Last Commit** | Latest main (2026-04-16) |

## Architecture Breakdown

### Communities (Functional Areas)

The codebase is organized into **149 communities** detected via Leiden algorithm:

| Community | Description | Key Files |
|-----------|-------------|-----------|
| Auth | Authentication strategies | `src/infrastructure/auth/` |
| Execution Plan | Request execution planning | `src/app/services/execution_plan/` |
| Providers | LLM provider integrations | `src/infrastructure/provider/` |
| Router | Request routing | `src/app/router/` |
| Handlers | HTTP handlers | `src/interfaces/handlers/` |
| Persistence | Data storage | `src/infrastructure/persistence/` |
| Entities | Domain models | `src/domain/entities/` |
| Traits | Domain interfaces | `src/domain/traits/` |

### Process Flow

**300 processes** detected representing execution flows:

- **Cross-community**: Processes spanning multiple functional areas (e.g., chat request → provider)
- **Intra-community**: Processes within a single area (e.g., provider validation tests)

### Key Relationships

| Relationship Type | Count | Description |
|-------------------|-------|-------------|
| `CALLS` | ~3,000+ | Function/method calls |
| `IMPORTS` | ~2,000+ | Module imports |
| `DEFINES` | ~1,500+ | Type/function definitions |
| `HAS_METHOD` | ~800+ | Class methods |
| `HAS_PROPERTY` | ~500+ | Struct fields |
| `ACCESSES` | ~300+ | Field access patterns |

## Code Structure

### Domain Layer (`src/domain/`)

- **Entities**: Core business models (Account, Provider, ChatRequest, etc.)
- **Traits**: Port interfaces (LlmGateway, AccountRepository, ProviderRepository)
- **Services**: Domain services (ModelSelector, QueryClassifier, TokenValidator)

### Application Layer (`src/app/`)

- **Router**: Request routing logic (`LlmRouter`)
- **Services**: Execution plans, quality evaluation
- **CLI**: Command-line interface

### Infrastructure Layer (`src/infrastructure/`)

- **Gateway**: LLM provider communication
- **Persistence**: JSON file-based storage
- **Auth**: Authentication strategies (API key, OAuth, Device Flow)
- **Metrics**: Prometheus metrics

### Presentation Layer (`src/presentation/`)

- **Handlers**: Axum HTTP handlers
- **Routes**: Route definitions
- **State**: Application state management

## GitNexus Usage

### Query Execution Flows

```bash
# Find processes related to chat handling
gitnexus query --repo "Rust-LLM-Api-Router" "chat handler provider router"
```

### Analyze Symbol Impact

```bash
# See what breaks if you change LlmRouter
gitnexus impact --repo "Rust-LLM-Api-Router" --target "LlmRouter" --direction upstream
```

### Check Shape Mismatches

```bash
# Verify API responses match consumer expectations
gitnexus shape-check --repo "Rust-LLM-Api-Router"
```

## Integration Points

### Router → Execution Plan

```
LlmRouter::route_request()
    ↓
ExecutionPlanner::create_plan()
    ↓
ExecutionPlanImpl / CascadingExecutionPlan
    ↓
LlmGateway::chat()
```

### Handler → Router

```
chat_handler::chat_completions()
    ↓
AppState::llm_router.route_request()
    ↓
LlmRouter
```

## Maintenance

To refresh the index after code changes:

```bash
gitnexus analyze --repo "Rust-LLM-Api-Router"
```

## See Also

- [docs/PROCESSES.md](PROCESSES.md) — Detected execution processes
- [docs/architecture.md](architecture.md) — Detailed architecture documentation
- [docs/routing.md](docs/routing.md) — Routing strategies

---

**Note**: These stats are from the GitNexus index. For real-time analysis, use the GitNexus CLI tools.