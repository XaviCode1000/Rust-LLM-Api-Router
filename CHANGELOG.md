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

### Changed

- Added `Cascading` variant to `ExecutionPlanType` enum
- Extended `PlannedAccount` with `model_id` field for tier-specific models
- Updated execution plan module to support cascading workflows

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