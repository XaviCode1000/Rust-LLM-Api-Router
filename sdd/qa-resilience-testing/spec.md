# QA Resilience Testing — Detailed Specifications

## Spec 1: Cascading Real HTTP Execution

### Overview
Replace the stub `CascadingExecutionPlan::execute()` with real HTTP calls through the existing `SharedHttpClient` and provider infrastructure. Each tier must make an actual HTTP request, parse the response, and evaluate quality against real response text.

### Architecture Decision
The `CascadingExecutionPlan` receives a `SharedHttpClient` and a `ProviderExecutor` trait (new) at construction. The plan delegates HTTP work to the executor, which knows how to build provider-specific requests (OpenAI-compatible vs Anthropic-native).

### New Types

```rust
/// Executes a single LLM request for a given tier.
#[async_trait::async_trait]
pub trait TierExecutor: Send + Sync {
    /// Execute a chat request against a specific provider/account.
    /// Returns the response text, token counts, and raw cost.
    async fn execute_tier(
        &self,
        tier: &CascadingTier,
        messages: Vec<ChatMessage>,
        model: &str,
        timeout: Duration,
    ) -> TierExecutionResult;
}

pub struct TierExecutionResult {
    pub response_text: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_microdollars: u64,
    pub provider_id: String,
}
```

### Requirements

**R-1.1 — HTTP Client Injection into CascadingExecutionPlan**
- `CascadingExecutionPlan::new()` gains a `SharedHttpClient` parameter and a `Arc<dyn TierExecutor>` parameter.
- The existing `quality_gate: Arc<dyn QualityGate>` is no longer `#[allow(dead_code)]` — it is actively used.
- Backward-compatible constructor: existing tests that don't pass an executor get a `MockTierExecutor` in test mode.

**Scenario**: Given a CascadingExecutionPlan is constructed with a SharedHttpClient and TierExecutor, When execute() is called, Then the plan uses the executor to make real HTTP calls instead of simulating cost 1000.

**R-1.2 — Per-Tier HTTP Request Construction**
- For each tier, the executor builds the HTTP request using the tier's `account.provider_id`, `account.provider_base_url`, and `tier.model_id`.
- Provider-specific headers: OpenAI-compatible providers use `Authorization: Bearer {api_key}` and `Content-Type: application/json`. Anthropic uses `x-api-key: {api_key}`, `anthropic-version: 2024-06-20` (updated from stale 2023-06-01), and `content-type: application/json`.
- The API key is fetched from the `AccountRepository` at execution time (not cached at plan construction).

**Scenario**: Given a tier with provider_id "anthropic" and a valid API key, When the executor builds the request, Then it includes the x-api-key header and anthropic-version 2024-06-20.

**R-1.3 — Response Parsing and Quality Evaluation**
- After each tier's HTTP call succeeds, the response body is parsed into a `TierExecutionResult`.
- The `HeuristicQualityEvaluator.evaluate_quality()` is called with the real response text (not a hardcoded 0.85).
- The evaluator receives the `PlannedAccount`, the `response_text`, and the `AccountHealth` snapshot.
- Quality score is computed from 4 checks: completeness, length, structure, coherence.

**Scenario**: Given a tier returns a valid response "The capital of France is Paris.", When quality is evaluated, Then HeuristicQualityEvaluator checks all 4 heuristics against the actual text and returns a real QualityScore.

**R-1.4 — Error Handling: Timeout**
- Each tier's HTTP request uses a timeout derived from `min(per_tier_timeout_ms, remaining_global_budget)`.
- If a tier times out, it is treated as a quality failure: the plan escalates to the next tier.
- Timeout errors are logged with `tracing::warn!` including tier_order, provider_id, and elapsed time.

**Scenario**: Given a tier with per_tier_timeout_ms=5000 and the HTTP call takes 6000ms, When the timeout fires, Then the tier is marked failed and the plan escalates to the next tier.

**R-1.5 — Error Handling: Network Errors**
- Network errors (connection refused, DNS failure, TLS error) are caught and treated as tier failure.
- The error is recorded in `ExecutionResult` as a diagnostic (new field: `tier_errors: Vec<String>`).
- The plan escalates to the next tier on any network error.

**Scenario**: Given a tier's provider endpoint is unreachable, When the HTTP call fails with a connection error, Then the error is recorded and the plan escalates to the next tier.

**R-1.6 — Error Handling: Provider HTTP Errors (4xx/5xx)**
- HTTP 4xx errors: treated as permanent failure for that tier. The plan escalates.
- HTTP 429 (rate limit): treated as tier failure with special logging. The plan escalates.
- HTTP 5xx errors: treated as transient. The plan escalates.
- The HTTP status code and error body are captured in the tier error log.

**Scenario**: Given a tier returns HTTP 401 with body `{"error": "Invalid API key"}`, When the response is processed, Then the tier is marked failed with the error recorded and the plan escalates.

**R-1.7 — Streaming Mode Bypass (Documented Behavior)**
- When `config.stream == true`, cascading is skipped entirely (existing behavior preserved).
- Only the first (cheapest) tier is executed.
- No quality evaluation is performed in streaming mode.
- This is documented in the module doc comment with a `## Streaming Behavior` section.

**Scenario**: Given streaming mode is enabled, When execute() is called, Then only the first tier is executed and no quality evaluation occurs.

**R-1.8 — Cost Calculation from Real Token Usage**
- Cost is no longer hardcoded to 1000.
- Cost is calculated using `ProviderPricing` with actual `input_tokens` and `output_tokens` from the response.
- Formula: `cost = (input_tokens / 1_000_000) * input_price + (output_tokens / 1_000_000) * output_price`, converted to microdollars.
- If pricing is unavailable for a provider, cost defaults to 0 with a `tracing::warn!`.

**Scenario**: Given a response with 500 input tokens and 300 output tokens, and pricing of $10/M input, $30/M output, When cost is calculated, Then cost = (500/1M * 10 + 300/1M * 30) * 1M = 14000 microdollars.

### Files Affected
- `src/app/services/execution_plan/cascading.rs` — main changes to execute()
- `src/app/services/execution_plan/execution.rs` — add `TierExecutor` trait, `TierExecutionResult`
- `src/app/services/quality/evaluator.rs` — no changes needed (already has the interface)

---

## Spec 2: Live Contract Tests

### Overview
Add environment-gated integration tests that hit real provider APIs. These tests detect schema drift before it reaches production. All tests are `#[ignore]` by default and only run with explicit opt-in.

### Requirements

**R-2.1 — Environment-Gated Test Execution**
- Tests are gated behind environment variables: `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GROQ_API_KEY`.
- Tests are marked `#[ignore]` so `cargo test` skips them by default.
- To run: `LIVE_TEST=1 OPENAI_API_KEY=sk-xxx cargo test -- --ignored` or `cargo test -- --ignored` with env vars set.
- A helper function `skip_if_no_env_var(var_name)` returns early with `eprintln!` if the env var is missing.

**Scenario**: Given OPENAI_API_KEY is not set, When a live test runs, Then the test is skipped with a clear message.

**R-2.2 — Live OpenAI Contract Test**
- File: `tests/live_contract_tests.rs`
- Sends a minimal POST to `https://api.openai.com/v1/chat/completions` with model `gpt-4o-mini`.
- Validates required fields in response:
  - `id`: non-empty string
  - `model`: non-empty string matching requested model prefix
  - `choices`: non-empty array
  - `choices[0].message.content`: non-empty string
  - `choices[0].message.role`: equals "assistant"
  - `choices[0].finish_reason`: present
  - `usage.prompt_tokens`: positive integer
  - `usage.completion_tokens`: non-negative integer
  - `usage.total_tokens`: positive integer

**Scenario**: Given a valid OPENAI_API_KEY, When the live test sends a chat request, Then the response contains all required fields with valid types.

**R-2.3 — Live Anthropic Contract Test**
- Sends a minimal POST to `https://api.anthropic.com/v1/messages` with model `claude-3-haiku-20240307`.
- Uses correct header: `anthropic-version: 2024-06-20` (NOT the stale 2023-06-01).
- Validates required fields:
  - `id`: non-empty string
  - `type`: equals "message"
  - `role`: equals "assistant"
  - `content`: non-empty array with at least one text block
  - `content[0].type`: equals "text"
  - `content[0].text`: non-empty string
  - `model`: non-empty string
  - `stop_reason`: present
  - `usage.input_tokens`: positive integer
  - `usage.output_tokens`: non-negative integer

**Scenario**: Given a valid ANTHROPIC_API_KEY, When the live test sends a messages request with version 2024-06-20, Then the response contains all required fields.

**R-2.4 — Live Groq Contract Test**
- Sends a minimal POST to `https://api.groq.com/openai/v1/chat/completions` with model `llama-3.1-8b-instant`.
- Validates the same OpenAI-compatible schema as R-2.2.

**Scenario**: Given a valid GROQ_API_KEY, When the live test sends a chat request, Then the response matches the OpenAI-compatible schema.

**R-2.5 — Insta Snapshot Tests for Schema Drift**
- Each live test captures the full JSON response and compares it against an `insta` snapshot.
- Snapshots stored in `tests/snapshots/live_contract_tests__{provider}_schema.snap.json`.
- Snapshot redactions: redact `id`, `created` timestamps, and `content` text (keep structure).
- If a provider changes its response schema, the snapshot diff fails and alerts the team.
- Snapshots are committed to the repository as living documentation of expected schemas.

**Scenario**: Given a saved snapshot of OpenAI's response schema, When the live test runs and the provider adds a new field, Then insta detects the diff and the test fails with a clear diff output.

**R-2.6 — CI Integration for Live Tests**
- CI workflow `.github/workflows/ci.yml` gains a new job `live-contract-tests`.
- Runs only on `main` branch pushes (not on PRs) OR when `LIVE_TEST=1` env var is set.
- API keys are injected from GitHub Secrets: `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GROQ_API_KEY`.
- Job has `continue-on-error: true` — live test failures do not block CI (providers may be down).
- Timeout: 5 minutes per provider.

**Scenario**: Given a push to main branch, When CI runs, Then the live contract test job executes with secrets and reports results without blocking the pipeline.

**R-2.7 — Wiremock Test Header Update**
- All existing wiremock tests using `anthropic-version: 2023-06-01` are updated to `2024-06-20`.
- This is a mechanical find-and-replace across all test files.

**Scenario**: Given the file tests/provider_chat_tests.rs, When the Anthropic version header is checked, Then it reads `2024-06-20` not `2023-06-01`.

### Files Affected
- `tests/live_contract_tests.rs` — new file
- `tests/snapshots/` — new directory with snapshot files
- `tests/provider_chat_tests.rs` — update Anthropic version header
- `.github/workflows/ci.yml` — add live contract test job

---

## Spec 3: Atomic JSON Persistence

### Overview
Fix the TOCTOU race condition in `JsonAccountRepository` by implementing atomic writes (write-to-temp + rename) and advisory file locking using the `fs4` crate.

### Requirements

**R-3.1 — Atomic Write Pattern**
- `write_accounts()` writes to a temporary file `{file_path}.tmp` first.
- After successful write and `sync_all()`, the temp file is renamed to the target path using `fs::rename()`.
- `fs::rename()` is atomic on POSIX systems — either the old file or the new file exists, never a partial state.
- If the temp file write fails, the original file is untouched.

**Scenario**: Given two concurrent write operations, When both attempt to write, Then at least one succeeds atomically and the file is never in a partially-written state.

**R-3.2 — Advisory File Locking with fs4**
- Add `fs4` crate to dependencies: `fs4 = { version = "0.12", features = ["tokio"] }`.
- Read operations acquire a shared (read) lock: `file.lock_shared().await`.
- Write operations acquire an exclusive (write) lock: `file.lock_exclusive().await`.
- Lock timeout: 5 seconds. If lock cannot be acquired within timeout, return `DomainError::LockTimeout`.
- Locks are advisory — all access to the file must go through the repository.

**Scenario**: Given a write lock is held, When a read operation attempts to acquire a shared lock, Then it waits up to 5 seconds and returns LockTimeout if the write lock is not released.

**R-3.3 — Read-Write Concurrency**
- Multiple readers can hold shared locks simultaneously (fs4 supports this).
- A writer requires an exclusive lock — no other readers or writers can proceed.
- The lock is released immediately after the operation completes (RAII via `File` drop).

**Scenario**: Given 3 concurrent read operations, When all 3 call find_all(), Then all 3 proceed simultaneously with shared locks.

**R-3.4 — Error Handling**
- Lock timeout: returns `DomainError::LockTimeout(message)` — new variant added to `DomainError`.
- File not found on read: `ensure_file_exists()` creates the file before attempting to lock.
- Parse error: returns `DomainError::Serialization(message)` with the JSON error.
- Write error: the temp file is cleaned up on failure (best effort, logged with `tracing::warn!`).

**Scenario**: Given the JSON file contains invalid JSON, When read_accounts() is called, Then it returns DomainError::Serialization with the parse error details.

**R-3.5 — Backward Compatibility**
- Existing JSON files without the new locking metadata continue to work.
- The file format (JSON array of AccountData) is unchanged.
- The `.tmp` file extension is reserved and cleaned up on startup if found from a previous crash.

**Scenario**: Given an existing accounts.json file from before this change, When the repository reads it, Then it works without migration or format changes.

**R-3.6 — Stale Temp File Cleanup**
- On repository initialization, if a `{file_path}.tmp` file exists, it is deleted (it represents a crashed write).
- This is logged with `tracing::info!` for observability.

**Scenario**: Given a crashed write left a stale .tmp file, When the repository is initialized, Then the .tmp file is removed and a log entry is created.

### Files Affected
- `src/infrastructure/persistence/json_account_repository.rs` — main changes
- `Cargo.toml` — add `fs4` dependency
- `src/domain/errors.rs` — add `DomainError::LockTimeout` variant

---

## Spec 4: Global Cascading Deadline

### Overview
Add a wall-clock budget for the entire cascading operation to prevent unbounded execution time when multiple tiers each have their own timeout.

### Requirements

**R-4.1 — New Field: total_timeout_ms in ExecutionConfig**
- `ExecutionConfig` gains a new field: `total_timeout_ms: Option<u64>`.
- Default: `Some(30000)` (30 seconds).
- `None` means no global deadline (backward compatible).
- Constructor: `ExecutionConfig::with_total_timeout_ms(ms)`.

**Scenario**: Given an ExecutionConfig with total_timeout_ms = Some(30000), When cascading starts, Then the entire operation must complete within 30 seconds.

**R-4.2 — Per-Tier Timeout as Minimum of Budget and Config**
- For each tier, the effective timeout is: `min(per_tier_timeout_ms, remaining_budget_ms)`.
- Remaining budget is calculated as: `total_timeout_ms - elapsed_since_start`.
- If remaining budget <= 0, no more tiers are attempted.

**Scenario**: Given total_timeout_ms=10000, per_tier_timeout_ms=5000, and tier 1 took 7000ms, When tier 2 is attempted, Then its timeout is min(5000, 3000) = 3000ms.

**R-4.3 — Global Deadline Exceeded During Tier**
- If the global deadline is exceeded before starting a tier, that tier is skipped.
- If the global deadline is exceeded during a tier's HTTP call, the call is cancelled (via tokio timeout).
- The best available result is returned if any tier succeeded.
- If no tier succeeded before the deadline, return an error with timeout details.

**Scenario**: Given total_timeout_ms=5000 and 3 tiers each needing 3000ms, When tier 2 starts at 4500ms elapsed, Then tier 2 gets a 500ms timeout and if it fails, the result from tier 1 is returned.

**R-4.4 — Timeout Details in ExecutionResult**
- `ExecutionResult` gains a new field: `timeout_details: Option<TimeoutDetails>`.
- `TimeoutDetails` contains: `total_budget_ms`, `elapsed_ms`, `tiers_completed`, `tiers_skipped`.
- Populated only when the global deadline is hit.

**Scenario**: Given the global deadline is exceeded after 2 of 3 tiers, When the result is returned, Then timeout_details shows total_budget_ms=30000, elapsed_ms=30001, tiers_completed=2, tiers_skipped=1.

**R-4.5 — Configuration via Environment Variable**
- `CASCADING_TOTAL_TIMEOUT_MS` environment variable overrides the default.
- Parsed as `u64`, validated to be > 0.
- Falls back to 30000 if not set or invalid.

**Scenario**: Given CASCADING_TOTAL_TIMEOUT_MS=60000, When the execution plan is created, Then the total timeout is 60 seconds.

### Files Affected
- `src/app/services/execution_plan/execution.rs` — add `total_timeout_ms` to `ExecutionConfig`
- `src/app/services/execution_plan/cascading.rs` — implement deadline tracking in execute()
- `src/app/services/quality/evaluator.rs` — add `total_timeout_ms` to `QualityConfig`
- `src/config/mod.rs` — add env var parsing for `CASCADING_TOTAL_TIMEOUT_MS`

---

## Spec 5: Dependency Cleanup & Quality Evaluator Wiring

### Overview
Remove dead dependencies from Cargo.toml and wire the `HeuristicQualityEvaluator` to actually evaluate real responses in the cascading execution flow.

### Requirements

**R-5.1 — Remove turmoil from dev-dependencies**
- `turmoil = "0.7.1"` is removed from `[dev-dependencies]` in `Cargo.toml`.
- No code references turmoil — confirmed by grep across the entire codebase.

**Scenario**: Given Cargo.toml contains turmoil in dev-dependencies, When the dependency is removed, Then `cargo build` and `cargo test` both succeed without turmoil.

**R-5.2 — Remove testcontainers from dev-dependencies**
- `testcontainers = "0.27"` is removed from `[dev-dependencies]` in `Cargo.toml`.
- No code references testcontainers — confirmed by grep across the entire codebase.

**Scenario**: Given Cargo.toml contains testcontainers in dev-dependencies, When the dependency is removed, Then `cargo build` and `cargo test` both succeed without testcontainers.

**R-5.3 — Quality Evaluator Wired to Real Responses**
- In `CascadingExecutionPlan::execute()`, after a tier's HTTP call succeeds:
  1. The response text is extracted from the parsed response.
  2. `self.quality_gate.evaluate_quality(&tier.account, &response_text, &tier.account.health_snapshot)` is called.
  3. The resulting `QualityScore` replaces the hardcoded `Some(0.85)`.
  4. If `score.is_acceptable`, the tier succeeds and cascading stops.
  5. If `!score.is_acceptable`, cascading escalates to the next tier.

**Scenario**: Given a tier returns a response "I cannot help with that", When quality is evaluated, Then the coherence check fails (contains "I cannot"), quality score < 0.75, and the plan escalates.

**R-5.4 — Quality Evaluator: JSON Validity Check (Actual Parsing)**
- `check_structure()` is renamed to `check_json_validity()` and performs actual JSON parsing.
- If the response starts with `{` or `[`, it is parsed with `serde_json::from_str::<serde_json::Value>()`.
- If parsing succeeds, the check passes. If it fails, the check fails with the parse error logged.
- If the response does not look like JSON (doesn't start with `{` or `[`), the check passes (not applicable).

**Scenario**: Given a response `{"key": "value"` (missing closing brace), When check_json_validity() runs, Then it returns false because serde_json fails to parse it.

**R-5.5 — Quality Evaluator: Completeness Check Enhanced**
- The existing `check_completeness()` is enhanced to also check for common truncation patterns:
  - Response ends mid-word (last character is alphanumeric, not whitespace/punctuation)
  - Response contains `[...truncated...]` or similar markers
  - Response is cut off mid-sentence (ends with comma, colon, or open bracket)

**Scenario**: Given a response "The answer is 42 and the reason is", When check_completeness() runs, Then it returns false because it ends mid-sentence.

**R-5.6 — Quality Evaluator: Refusal Pattern Detection**
- The existing `check_coherence()` already detects refusal patterns.
- Enhanced to also detect:
  - "I'm sorry but I can't"
  - "I don't have the ability"
  - "I'm not able to"
  - "I cannot assist with"
  - Markdown-formatted refusals: `**I cannot**`, `**I'm unable**`

**Scenario**: Given a response "**I cannot** provide that information", When check_coherence() runs, Then it detects the refusal pattern and returns false.

**R-5.7 — Minimum Quality Threshold Configurable**
- `QualityConfig.min_quality_score` is already configurable (default 0.75).
- Exposed via environment variable `CASCADING_MIN_QUALITY_SCORE`.
- Parsed as `f64`, validated to be in range [0.0, 1.0].
- Falls back to 0.75 if not set or invalid.

**Scenario**: Given CASCADING_MIN_QUALITY_SCORE=0.90, When a response scores 0.85, Then it is rejected as below threshold and cascading escalates.

### Files Affected
- `Cargo.toml` — remove turmoil and testcontainers
- `src/app/services/execution_plan/cascading.rs` — wire quality evaluator to real responses
- `src/app/services/quality/evaluator.rs` — enhance checks (JSON validity, completeness, refusal)
- `src/config/mod.rs` — add env var parsing for `CASCADING_MIN_QUALITY_SCORE`

---

## Cross-Cutting Concerns

### Tracing & Observability
All phases add structured tracing at `info` and `warn` levels:
- Cascading: tier attempts, quality scores, timeouts, escalations
- Live tests: response times, schema diffs
- Persistence: lock acquisitions, atomic write operations
- Configuration: env var values at startup (redacted for secrets)

### Testing Strategy
- Unit tests: all new logic covered with unit tests (mock HTTP, mock locks)
- Integration tests: live contract tests (env-gated, `#[ignore]`)
- E2E tests: existing `tests/cascading_routing_e2e_tests.rs` updated to use real HTTP mocks

### Backward Compatibility
- All new fields have defaults matching current behavior
- Environment variables are optional with sensible defaults
- Existing tests continue to pass without modification (except header update)
- No breaking changes to public API
