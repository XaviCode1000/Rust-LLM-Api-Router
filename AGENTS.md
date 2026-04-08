# AGENTS.md — Rust-LLM-Api-Router

> **README for AI Agents** — Project-specific context, commands, and constraints for AI coding agents.

---

## Project Overview

**LLM API Router**: High-performance LLM proxy built with Clean Architecture in Rust. Routes requests across 34 providers with automatic failover, health monitoring, and intelligent routing.

**Key Features**:
- Multi-provider support (OpenAI, Anthropic, Groq, 31 more)
- Cost-Aware Routing (Issue #23) — static model selection by query complexity
- Cascading Routing (Issue #24) — dynamic quality-based escalation
- Live Contract Tests — real API schema validation
- Atomic JSON Persistence — file locking + atomic writes
- OAuth 2.1 / PKCE authentication
- 80.35% test coverage (492 tests passing)

---

## Tech Stack (2025-26 Optimal)

| Component | Technology | Version |
|-----------|------------|---------|
| **Language** | Rust | 1.93.0 (MSRV: 1.75) |
| **Edition** | 2021 | — |
| **Web Framework** | Axum | 0.7 |
| **HTTP Client** | reqwest | 0.12 (rustls-tls) |
| **Async Runtime** | tokio | 1.x (rt-multi-thread) |
| **Error Handling** | thiserror + anyhow | 1.0 |
| **Serialization** | serde + serde_json | 1.0 |
| **Testing** | cargo-nextest | 0.9.130 (4x faster) |
| **Coverage** | cargo-llvm-cov | LLVM-native (10x faster) |
| **Build Cache** | sccache | 0.14.0 (6x faster) |
| **File Locking** | fs4 | 0.8 (tokio support) |
| **Mocking** | mockall + wiremock | 0.13 + 0.6.5 |
| **Snapshot Testing** | insta | 1.46.3 |
| **Property Testing** | proptest | 1.4 |
| **Metrics** | prometheus | 0.14 |
| **CLI** | clap | 4.4 (derive) |
| **Auth** | oauth2 | 5.0 (PKCE) + keyring |

---

## Build and Test Commands

### Development Workflow (Low-Resource Hardware)

**Hardware Context**: Intel i5-4590 (4C/4T), 8GB RAM, HDD

```bash
# Install tools (one-time)
cargo install cargo-nextest cargo-llvm-cov sccache cargo-watch

# Start development server (watch mode)
./scripts/dev.sh

# Run tests (4x faster than cargo test)
cargo nextest run --test-threads 2

# Run specific test file
cargo nextest run --test chat_handler
cargo nextest run --test live_contract_tests -- --ignored

# Generate coverage report (10x faster than tarpaulin)
cargo llvm-cov --html --output-dir coverage-llvm
cargo llvm-cov --summary-only

# Watch mode (auto-rerun on changes)
cargo watch -x "nextest run --test-threads 2"
```

### Build Commands

```bash
# Standard build
cargo build --release

# With sccache (6x faster)
sccache --show-stats

# Check only (fastest verification)
cargo check
```

### Linting and Formatting

```bash
# Clippy with warnings as errors
cargo clippy -D warnings

# Auto-fix clippy
cargo clippy --fix

# Format code
cargo fmt

# Check format
cargo fmt --check
```

### Security

```bash
# Audit dependencies
cargo audit

# Check licenses and policies
cargo deny check
```

### Live Contract Tests (Real API)

```bash
# Enable live tests (requires API keys)
LIVE_TEST=1 cargo test --test live_contract_tests -- --ignored

# Specific provider tests
LIVE_TEST=1 GROQ_API_KEY=your-key cargo test --test live_contract_tests -- --ignored test_groq_contract
LIVE_TEST=1 OPENAI_API_KEY=your-key cargo test --test live_contract_tests -- --ignored test_openai_contract
```

---

## Definition of Done

A task is complete when **ALL** of the following pass:

1. ✅ `cargo fmt --check` exits 0
2. ✅ `cargo clippy -D warnings` exits 0
3. ✅ `cargo nextest run --test-threads 2` exits 0 (all 492 tests pass)
4. ✅ `cargo llvm-cov --summary-only` shows coverage maintained or improved (current: 80.35%)
5. ✅ Changed files are staged and committed
6. ✅ Commit message follows Conventional Commits: `type(scope): description`

---

## Code Style

### Naming Conventions

- **Files**: snake_case (`chat_handler.rs`, `execution_plan.rs`)
- **Modules**: snake_case (matches file names)
- **Structs/Classes**: PascalCase (`CascadingExecutionPlan`, `HeuristicQualityEvaluator`)
- **Traits**: PascalCase (`QualityGate`, `ModelSelector`, `RotationStrategy`)
- **Functions/Methods**: snake_case (`select_model`, `evaluate_quality`)
- **Constants**: UPPER_SNAKE_CASE (`DEFAULT_TIMEOUT_MS`, `MAX_RETRIES`)
- **Enums**: PascalCase (`QueryComplexity`, `ExecutionPlanType`)
- **Enum Variants**: PascalCase (`Low`, `Medium`, `High`, `Cascading`, `Failover`)

### Project Structure

```
src/
├── domain/              # Domain layer (pure business logic)
│   ├── entities/        # Core entities (Account, Provider, Chat)
│   ├── traits/          # Ports/interfaces (Repository traits)
│   ├── errors/          # Domain errors
│   └── services/        # Domain services (CostAwareSelector, QueryClassifier)
├── app/                 # Application layer (use cases)
│   ├── services/        # App services
│   │   ├── execution_plan/  # Execution plans (Cascading, Failover)
│   │   ├── quality/         # Quality evaluation (HeuristicQualityEvaluator)
│   │   ├── failover.rs      # Failover manager
│   │   └── auth/            # Authentication
│   └── router/          # Internal routing
├── infrastructure/      # Infrastructure layer (implementations)
│   ├── http_client.rs   # Shared HTTP client
│   ├── persistence/     # JSON file storage (atomic writes)
│   ├── provider/        # Provider adapters (OpenAI, Anthropic)
│   ├── gateway/         # LLM gateway
│   └── auth/            # OAuth 2.1 / PKCE
├── presentation/        # Presentation layer
│   ├── handlers/        # HTTP handlers
│   ├── routes.rs        # Route definitions
│   ├── state.rs         # AppState
│   └── cli/             # CLI commands
└── main.rs / lib.rs
```

### Clean Architecture Rules

**Dependency Rule**: Dependencies point inward

```
Presentation → Application → Domain ← Infrastructure
```

- **Domain**: Pure Rust, no external dependencies (except serde for serialization)
- **Application**: Only depends on Domain
- **Infrastructure**: Implements Domain traits, depends on external crates
- **Presentation**: Uses Application + Infrastructure

### Code Example

```rust
// ✅ Good - Clean Architecture, proper error handling
#[async_trait]
pub trait ModelSelector: Send + Sync {
    fn select<'a>(
        &self,
        request: &ChatRequest,
        available_models: &'a [Model],
    ) -> SelectionResult<&'a Model>;
    
    fn strategy_name(&self) -> &'static str;
}

pub struct CostAwareSelector {
    classifier: QueryClassifier,
    max_cost_per_million_tokens: Option<f64>,
}

impl ModelSelector for CostAwareSelector {
    fn select<'a>(
        &self,
        request: &ChatRequest,
        available_models: &'a [Model],
    ) -> SelectionResult<&'a Model> {
        let complexity = self.classifier.classify(&request.messages);
        
        available_models
            .iter()
            .filter(|m| m.capability_tier() >= complexity)
            .min_by(|a, b| a.pricing().avg_cost().partial_cmp(&b.pricing().avg_cost()).unwrap())
            .ok_or(SelectionError::NoSuitableModel)
    }
}

// ❌ Bad - Leaks infrastructure into domain
use reqwest::Client;  // Domain should not depend on HTTP client

pub struct BadService {
    client: Client,  // WRONG - infrastructure dependency
}
```

### Async Patterns (Critical for Rust)

```rust
// ✅ Good - No locks across await
pub struct GoodService {
    data: Arc<TokioMutex<HashMap<String, Data>>>,
}

impl GoodService {
    async fn process(&self) {
        {
            // Lock scope is minimal, NOT across await
            let mut guard = self.data.lock().await;
            guard.insert("key".to_string(), Data::new());
        } // Lock released here
        
        // Other work can happen concurrently
        self.do_something_else().await;
    }
}

// ❌ Bad - Lock held across await (deadlock risk)
pub struct BadService {
    data: Arc<Mutex<HashMap<String, Data>>>,  // std::sync::Mutex in async context
}

impl BadService {
    async fn process(&self) {
        let mut guard = self.data.lock().unwrap();  // DEADLOCK RISK
        guard.insert("key".to_string(), Data::new());
        self.do_something_else().await;  // Lock held during await!
    }
}
```

---

## Project-Specific Knowledge

### Intelligent Routing Strategies

#### 1. Cost-Aware Routing (Issue #23)

**Purpose**: Select cheapest capable model **before** request based on query complexity.

```rust
// Complexity levels
pub enum QueryComplexity {
    Low = 0,    // <100 chars, simple greetings
    Medium = 1, // 100-500 chars, code keywords, 4+ messages
    High = 2,   // >500 chars, analysis keywords, 8+ messages
}

// Usage
let selector = CostAwareSelector::new();
let model = selector.select(&request, &available_models)?;
```

**When to use**: Budget-critical, predictable queries, latency-sensitive

#### 2. Cascading Routing (Issue #24)

**Purpose**: Start cheap, evaluate quality, escalate only if quality < threshold.

```rust
// Quality evaluation (4 heuristic checks)
pub struct HeuristicQualityEvaluator;

impl QualityGate for HeuristicQualityEvaluator {
    async fn evaluate_quality(&self, response: &str) -> QualityScore {
        // 1. Completeness - not truncated
        // 2. Length - >= 10 chars
        // 3. Structure - valid JSON when expected
        // 4. Coherence - no error patterns
    }
}

// Configuration
let config = QualityConfig {
    min_quality_score: 0.75,  // Default threshold
    max_tiers: 3,              // Try up to 3 tiers
    per_tier_timeout_ms: 5000,
};
```

**Streaming Guard**: Cascading is **disabled** for streaming requests (quality can't be evaluated until stream completes).

### Execution Plan Types

```rust
pub enum ExecutionPlanType {
    Standard,      // Single account
    Failover,      // Sequential fallback on failure
    LoadBalanced,  // Health-weighted distribution
    CostOptimized, // Cheapest provider selection
    Cascading,     // Quality-based escalation (Issue #24)
}
```

### File Persistence (Atomic Writes)

```rust
// JSON repositories use atomic writes + file locking
pub struct JsonAccountRepository {
    file_path: PathBuf,  // ~/.config/rust-llm-api-router/accounts.json
}

// Uses fs4 for advisory locking with tokio support
// Prevents data corruption under concurrent access
```

### Provider Support

**34 Providers** across categories:

- **Major AI**: OpenAI, Anthropic, Mistral, Cohere, Google AI Studio
- **OpenAI-Compatible**: Groq, OpenRouter, Cerebras, Cloudflare, DeepSeek, Together, Fireworks, xAI, Perplexity
- **Local**: Ollama, LM Studio, vLLM
- **Enterprise**: Azure OpenAI, AWS Bedrock, Google Vertex AI
- **Free Tier**: Zhipu AI, GitHub Models, Kluster AI, LLM7.io, SiliconFlow

---

## Boundaries

### ✅ Always Do

- Use `cargo nextest` for tests (4x faster)
- Use `cargo llvm-cov` for coverage (10x faster)
- Run `cargo fmt` before commit (pre-commit hook does this automatically)
- Use `Arc<TokioMutex>` for shared state in async contexts (NOT `std::sync::Mutex`)
- Keep Domain layer pure (no external dependencies except serde)
- Use traits for dependency injection (Repository pattern)
- Test error paths, not just happy paths
- Use `thiserror` for domain errors, `anyhow` for application errors
- Follow Clean Architecture dependency rule

### ⚠️ Ask First

- Adding new dependencies (check `cargo deny` first)
- Modifying execution plan types (core architecture)
- Changing quality evaluation thresholds (affects routing behavior)
- Modifying live contract tests (require API keys)
- Refactoring Domain layer entities (breaking changes)
- Changing file persistence format (backward compatibility)

### 🚫 Never Do

- **NEVER** use `unwrap()` or `expect()` in production code — use `thiserror` + proper error handling
- **NEVER** hold locks (`Mutex`/`RwLock`) across `.await` points — deadlock risk
- **NEVER** use `&Vec<T>` or `&String` when `&[T]` / `&str` suffices
- **NEVER** use `format!()` in hot paths — allocation overhead
- **NEVER** commit secrets or API keys — use environment variables
- **NEVER** modify `target/` or `.cargo/` directories
- **NEVER** skip tests before commit — all 492 must pass
- **NEVER** hardcode provider URLs or API keys
- **NEVER** use `cargo test` (slower than nextest) or `cargo tarpaulin` (slower than llvm-cov)

---

## When Blocked

### Escalation Rules

1. **If tests fail after 3 attempts**: STOP and report full error output
2. **If dependency missing**: Check `Cargo.toml` first, then ask
3. **If merge conflicts**: STOP and show conflicting files
4. **If CI fails**: Check formatting (`cargo fmt --check`) and clippy (`cargo clippy -D warnings`)
5. **If coverage drops**: Identify uncovered files, add targeted tests

### Never Do When Blocked

- **NEVER** delete files to resolve errors
- **NEVER** force push without approval
- **NEVER** skip tests or disable them to make CI pass
- **NEVER** ignore clippy warnings with `#[allow(...)]` without justification

---

## Git Workflow

### Branch Naming

- `feature/issue-23-cost-aware-routing`
- `fix/chat-handler-panic`
- `refactor/execution-plan-module`
- `docs/update-routing-guide`

### Commit Messages (Conventional Commits)

```
feat(routing): add cost-aware model selection
fix(handler): prevent panic on empty messages
refactor(execution): extract cascading logic
docs(architecture): update layer diagram
test(coverage): add quality evaluator tests
```

### Pre-Commit Hooks

**Automatic**: Pre-commit hook runs `cargo fmt --all` and stages formatted files

**Pre-Push**: Verifies formatting with `cargo fmt --all -- --check`

```bash
# Install hooks (one-time)
./scripts/setup-githooks.sh
```

---

## Good and Bad Examples

### ✅ Good Files to Copy

- `src/domain/services/model_selector.rs` — Clean trait implementation
- `src/app/services/execution_plan/cascading.rs` — Complex async logic done right
- `src/app/services/quality/evaluator.rs` — Quality gate pattern
- `src/infrastructure/persistence/json_account_repository.rs` — Atomic writes + locking
- `tests/error_snapshots.rs` — Snapshot testing with insta

### ❌ Bad Files to Avoid (Legacy)

- None currently — codebase is well-refactored

---

## API Documentation

### Key Endpoints

```
POST /v1/chat/completions    # OpenAI-compatible chat API
GET  /health                 # Basic health check
GET  /health/detail          # Detailed system status
GET  /v1/models              # List available models
GET  /accounts               # List registered accounts
GET  /metrics                # Prometheus metrics
```

### CLI Commands

```bash
# Provider management
llm-router provider add --id <id> --name <name> --base-url <url>
llm-router provider list
llm-router provider enable --id <id>

# Account management
llm-router account add --id <id> --provider <provider> --api-key <key>
llm-router account list
llm-router account validate --id <id>

# Auth (OAuth 2.1 / PKCE)
llm-router auth login --provider <provider>
llm-router auth logout --all
```

---

## Testing Strategy

### Test Categories

| Type | Location | Tools | Purpose |
|------|----------|-------|---------|
| **Unit** | `src/**/*.rs` | `#[test]`, `proptest` | Domain logic |
| **Integration** | `tests/*.rs` | `wiremock`, `mockall` | Component + HTTP |
| **Snapshot** | `tests/error_snapshots.rs` | `insta` | Golden tests |
| **Live Contract** | `tests/live_contract_tests.rs` | Real APIs | Provider drift detection |

### Test Pattern (Arrange-Act-Assert)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_score_with_no_requests() {
        // Arrange
        let health = AccountHealth::new("test-account");

        // Act
        let score = health.health_score();

        // Assert
        assert_eq!(score, 25.0);  // Default score for new accounts
    }

    #[tokio::test]
    async fn test_chat_handler_success() {
        // Arrange
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        // Act
        let response = make_chat_request(&mock_server.uri()).await;

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
    }
}
```

---

## Performance Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| **Test Execution** | <10s | ~8s | ✅ |
| **Coverage Generation** | <60s | ~30s | ✅ |
| **Build (clean)** | <60s | ~10s (with sccache) | ✅ |
| **Code Coverage** | >80% | 80.35% | ✅ |
| **Tests Passing** | All | 492/492 | ✅ |

---

## Documentation

### Key Documentation Files

- [`README.md`](../README.md) — Project overview and quick start
- [`DEVELOPMENT.md`](../DEVELOPMENT.md) — Development workflow guide
- [`docs/architecture.md`](../docs/architecture.md) — Clean Architecture details
- [`docs/routing.md`](../docs/routing.md) — Cost-Aware and Cascading routing
- [`docs/TESTING_GUIDE.md`](../docs/TESTING_GUIDE.md) — Testing strategy
- [`docs/TESTING_JOURNEY.md`](../docs/TESTING_JOURNEY.md) — Coverage progress story
- [`docs/cli.md`](../docs/cli.md) — CLI reference
- [`sdd/qa-resilience-testing/spec.md`](../sdd/qa-resilience-testing/spec.md) — QA resilience specs

---

## Environment Variables

```bash
# Server configuration
PORT=8080
HOST=0.0.0.0
LOG_LEVEL=info

# Testing
LIVE_TEST=1                    # Enable live contract tests
GROQ_API_KEY=your-key          # For live Groq tests
OPENAI_API_KEY=your-key        # For live OpenAI tests
ANTHROPIC_API_KEY=your-key     # For live Anthropic tests

# Cascading routing
CASCADING_MIN_QUALITY_SCORE=0.75

# Timeouts
PLANNING_TIMEOUT_MS=5000
MAX_ACCOUNTS_PER_PLAN=3
```

---

## Resources

- [cargo-nextest](https://nexte.st/) — Fast test runner
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) — Native coverage
- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Tokio Best Practices](https://tokio.rs/tokio/topics/async_best_practices)

---

## Quick Reference Card

```bash
# Daily workflow
./scripts/dev.sh                    # Watch mode
cargo nextest run --test-threads 2  # Run tests
cargo llvm-cov --summary-only       # Check coverage

# Before commit
cargo fmt                           # Auto-formatted by pre-commit hook
cargo clippy -D warnings            # Lint
cargo nextest run --test-threads 2  # Test

# Verify
cargo check                         # Fast compilation check
cargo audit                         # Security audit
cargo deny check                    # License check
```

---

**Last Updated**: April 2026
**Rust Version**: 1.93.0
**Test Coverage**: 80.35% (492 tests)
**Project Status**: Active development

<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **Rust-LLM-Api-Router** (4356 symbols, 9302 relationships, 300 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

> If any GitNexus tool warns the index is stale, run `npx gitnexus analyze` in terminal first.

## Always Do

- **MUST run impact analysis before editing any symbol.** Before modifying a function, class, or method, run `gitnexus_impact({target: "symbolName", direction: "upstream"})` and report the blast radius (direct callers, affected processes, risk level) to the user.
- **MUST run `gitnexus_detect_changes()` before committing** to verify your changes only affect expected symbols and execution flows.
- **MUST warn the user** if impact analysis returns HIGH or CRITICAL risk before proceeding with edits.
- When exploring unfamiliar code, use `gitnexus_query({query: "concept"})` to find execution flows instead of grepping. It returns process-grouped results ranked by relevance.
- When you need full context on a specific symbol — callers, callees, which execution flows it participates in — use `gitnexus_context({name: "symbolName"})`.

## When Debugging

1. `gitnexus_query({query: "<error or symptom>"})` — find execution flows related to the issue
2. `gitnexus_context({name: "<suspect function>"})` — see all callers, callees, and process participation
3. `READ gitnexus://repo/Rust-LLM-Api-Router/process/{processName}` — trace the full execution flow step by step
4. For regressions: `gitnexus_detect_changes({scope: "compare", base_ref: "main"})` — see what your branch changed

## When Refactoring

- **Renaming**: MUST use `gitnexus_rename({symbol_name: "old", new_name: "new", dry_run: true})` first. Review the preview — graph edits are safe, text_search edits need manual review. Then run with `dry_run: false`.
- **Extracting/Splitting**: MUST run `gitnexus_context({name: "target"})` to see all incoming/outgoing refs, then `gitnexus_impact({target: "target", direction: "upstream"})` to find all external callers before moving code.
- After any refactor: run `gitnexus_detect_changes({scope: "all"})` to verify only expected files changed.

## Never Do

- NEVER edit a function, class, or method without first running `gitnexus_impact` on it.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis.
- NEVER rename symbols with find-and-replace — use `gitnexus_rename` which understands the call graph.
- NEVER commit changes without running `gitnexus_detect_changes()` to check affected scope.

## Tools Quick Reference

| Tool | When to use | Command |
|------|-------------|---------|
| `query` | Find code by concept | `gitnexus_query({query: "auth validation"})` |
| `context` | 360-degree view of one symbol | `gitnexus_context({name: "validateUser"})` |
| `impact` | Blast radius before editing | `gitnexus_impact({target: "X", direction: "upstream"})` |
| `detect_changes` | Pre-commit scope check | `gitnexus_detect_changes({scope: "staged"})` |
| `rename` | Safe multi-file rename | `gitnexus_rename({symbol_name: "old", new_name: "new", dry_run: true})` |
| `cypher` | Custom graph queries | `gitnexus_cypher({query: "MATCH ..."})` |

## Impact Risk Levels

| Depth | Meaning | Action |
|-------|---------|--------|
| d=1 | WILL BREAK — direct callers/importers | MUST update these |
| d=2 | LIKELY AFFECTED — indirect deps | Should test |
| d=3 | MAY NEED TESTING — transitive | Test if critical path |

## Resources

| Resource | Use for |
|----------|---------|
| `gitnexus://repo/Rust-LLM-Api-Router/context` | Codebase overview, check index freshness |
| `gitnexus://repo/Rust-LLM-Api-Router/clusters` | All functional areas |
| `gitnexus://repo/Rust-LLM-Api-Router/processes` | All execution flows |
| `gitnexus://repo/Rust-LLM-Api-Router/process/{name}` | Step-by-step execution trace |

## Self-Check Before Finishing

Before completing any code modification task, verify:
1. `gitnexus_impact` was run for all modified symbols
2. No HIGH/CRITICAL risk warnings were ignored
3. `gitnexus_detect_changes()` confirms changes match expected scope
4. All d=1 (WILL BREAK) dependents were updated

## Keeping the Index Fresh

After committing code changes, the GitNexus index becomes stale. Re-run analyze to update it:

```bash
npx gitnexus analyze
```

If the index previously included embeddings, preserve them by adding `--embeddings`:

```bash
npx gitnexus analyze --embeddings
```

To check whether embeddings exist, inspect `.gitnexus/meta.json` — the `stats.embeddings` field shows the count (0 means no embeddings). **Running analyze without `--embeddings` will delete any previously generated embeddings.**

> Claude Code users: A PostToolUse hook handles this automatically after `git commit` and `git merge`.

## CLI

| Task | Read this skill file |
|------|---------------------|
| Understand architecture / "How does X work?" | `.claude/skills/gitnexus/gitnexus-exploring/SKILL.md` |
| Blast radius / "What breaks if I change X?" | `.claude/skills/gitnexus/gitnexus-impact-analysis/SKILL.md` |
| Trace bugs / "Why is X failing?" | `.claude/skills/gitnexus/gitnexus-debugging/SKILL.md` |
| Rename / extract / split / refactor | `.claude/skills/gitnexus/gitnexus-refactoring/SKILL.md` |
| Tools, resources, schema reference | `.claude/skills/gitnexus/gitnexus-guide/SKILL.md` |
| Index, status, clean, wiki CLI commands | `.claude/skills/gitnexus/gitnexus-cli/SKILL.md` |

<!-- gitnexus:end -->
