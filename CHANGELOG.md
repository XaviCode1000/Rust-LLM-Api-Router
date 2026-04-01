# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### Cost-Aware Routing (Issue #23)

- **`CostAwareSelector`** in `src/domain/services/model_selector.rs`
  - Routes queries to the cheapest model capable of handling estimated complexity
  - Classifies queries into Low/Medium/High complexity tiers
  - Configurable cost ceiling via `max_cost_per_million_tokens`
  - Automatic fallback to best available when no tier matches
  
- **`QueryClassifier`** in `src/domain/services/query_complexity.rs`
  - Heuristic-based complexity classification
  - Considers message length, conversation length, and keywords
  - Customizable thresholds via `ClassifierConfig`
  - Keywords for high-complexity tasks (design, architect, analyze, etc.)
  - Keywords for code-related tasks (code, function, algorithm, etc.)

- **`ModelSelector` trait** for pluggable selection strategies
  - Domain-level port following hexagonal architecture
  - Synchronous execution (pure domain operation)
  - `strategy_name()` for logging and metrics

#### Cascading Routing (Issue #24)

- **`CascadingExecutionPlan`** in `src/app/services/execution_plan/cascading.rs`
  - Tries cheapest tier first, escalates based on response quality
  - Quality-based escalation with configurable threshold (default: 0.75)
  - Cost budget enforcement across tiers
  - Streaming guard: disables cascading for streaming requests
  - Automatic cost tracking in microdollars

- **`HeuristicQualityEvaluator`** in `src/app/services/quality/evaluator.rs`
  - Implements `QualityGate` trait with 4 heuristic checks:
    1. **Completeness**: Response not truncated
    2. **Length**: Meets minimum character threshold
    3. **Structure**: Valid JSON structure when expected
    4. **Coherence**: No error patterns or excessive repetition
  - Configurable thresholds via `QualityConfig`
  - Score calculation: passed_checks / total_checks

- **`QualityGate` trait** for extensible quality evaluation
  - Async interface for future LLM-as-Judge integration
  - Returns `QualityScore` with detailed check results
  - Can be implemented with custom evaluation logic

#### QA Resilience Testing

- **Live Contract Tests** in `tests/live_contract_tests.rs`
  - Tests that hit REAL provider APIs (OpenAI, Anthropic, Groq) to detect schema drift
  - Gated behind `LIVE_TEST=1` env var + individual provider API key env vars
  - Marked `#[ignore]` so they never run in normal `cargo test`
  - Use insta snapshots with redactions for variable fields (id, timestamps, content)
  - CI job runs only on `push` to `main` branch when API key secrets are configured

- **Atomic JSON Persistence** in `src/infrastructure/persistence/json_account_repository.rs`
  - Write-to-temp-then-rename pattern eliminates TOCTOU race condition
  - `fs4` advisory file locking (shared for reads, exclusive for writes)
  - 5-second lock timeout with `DomainError::LockTimeout`
  - Stale temp file cleanup on initialization
  - Prevents data corruption under concurrent writes

- **Dependency Cleanup**
  - Removed unused `turmoil` and `testcontainers` dev-dependencies
  - Updated Anthropic API version header from `2023-06-01` to `2024-06-20` in all test files
  - Added `CASCADING_MIN_QUALITY_SCORE` env var (default: `0.75`) to Settings

#### Token Validation (Pre-flight Check)

- **`TokenValidator`** in `src/domain/services/token_validator.rs`
  - Counts tokens using `tiktoken-rs` (cl100k_base encoding) before sending to providers
  - Validates request doesn't exceed model's context window
  - Considers both prompt tokens and `max_tokens` parameter
  - Gracefully skips validation for unknown models
  - Supports model name prefixes (e.g., `openai:gpt-4`, `groq:llama-3.1-8b-instant`)

- **`ModelContextLimits`** registry in `src/domain/services/model_context_limits.rs`
  - Static registry covering 30+ models across 8 providers
  - OpenAI (gpt-4o, gpt-4, gpt-3.5-turbo, o1, o3-mini)
  - Anthropic (Claude 3.5 Sonnet, Claude 3 Opus/Sonnet/Haiku, Claude 2)
  - Groq (Llama 3.x, Mixtral, Gemma)
  - Mistral (Large, Small, Codestral, Nemo)
  - Google (Gemini 2.0 Flash, Gemini 1.5 Pro/Flash)
  - Cohere (Command R/R+), DeepSeek (Chat, Reasoner)
  - Prefix-based fallback matching for versioned model IDs

- **`TokenLimitExceeded`** error variant in `DomainError`
  - Returns clear error with model name, token count, and limit
  - Integrated into `LlmRouter::route_request()` as pre-flight check
  - Prevents wasteful API calls that would fail anyway

#### Structured Logging & Quality Evaluation

- **`QualityEvaluationSpan`** in `src/app/services/execution_plan/tracing.rs`
  - Structured tracing for quality evaluation lifecycle
  - Tracks individual check results (completeness, length, structure, coherence)
  - Logs score, acceptability, failed checks, and duration
  - Integrates with existing `PlanningSpan`/`ExecutionSpan` patterns

- **Structured logging in `HeuristicQualityEvaluator`**
  - `tracing::debug!` at evaluation start (response length, account ID)
  - `tracing::info!` at completion (score, checks passed/failed, acceptability)
  - Enables production observability of quality escalation decisions

- **Real quality evaluation wired in `CascadingExecutionPlan`**
  - Replaced hardcoded `Some(0.85)` score with actual `quality_gate.evaluate_quality()` call
  - QualityEvaluationSpan traces each tier's evaluation
  - Enables real quality-based escalation (previously simulated)

### Changed

- Added `Cascading` variant to `ExecutionPlanType` enum
- Extended `PlannedAccount` with `model_id` field for tier-specific models
- Updated execution plan module to support cascading workflows
- `fs4` crate added for advisory file locking
- `JsonAccountRepository` now uses atomic writes and file locking
- Live contract tests added to CI pipeline (main branch only)
- Anthropic API version updated to `2024-06-20` across all tests
- `tiktoken-rs` crate added for token counting
- `CascadingExecutionPlan.execute()` now calls real quality evaluation (was simulated)
- `tiktoken-rs` crate added for token counting
- `CascadingExecutionPlan.execute()` now calls real quality evaluation (was simulated)

### Technical Details

**Issue #23 (Cost-Aware Routing)**:
- Domain layer implementation (no infrastructure dependencies)
- Pure functions for model selection and complexity classification
- Comprehensive test coverage (15+ test cases)
- Zero external dependencies added

**Issue #24 (Cascading Routing)**:
- Application layer implementation (uses domain services)
- Integration with existing execution plan infrastructure
- Quality evaluation without additional LLM calls
- Cost tracking across multiple tier attempts

## Previous Releases

See git history for changes prior to this changelog.