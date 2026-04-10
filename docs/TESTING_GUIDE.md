# Testing Guide - Rust-LLM-Api-Router

## Overview

This project achieves **80.35% code coverage** with **~680+ tests**, following Rust 2025-26 best practices.

## Testing Stack

### Installed Tools

```bash
# Install once
cargo install cargo-nextest      # Test runner (4x faster than cargo test)
cargo install cargo-llvm-cov     # LLVM-native coverage (10x faster than tarpaulin)
cargo install sccache            # Build cache (6x faster)
cargo install cargo-watch        # Auto-recompile on changes
cargo install cargo-audit        # Security audit
cargo install cargo-deny         # License/dependency check
```

### Dev Dependencies

```toml
[dev-dependencies]
mockall = "0.13"           # Trait mocking
tokio-test = "0.4"         # Async testing
tempfile = "3.10"          # Temp directories
proptest = "1.4"           # Property-based testing (19 property tests)
insta = "1.46"             # Snapshot testing (5+ snapshots)
wiremock = "0.6.5"         # HTTP mocking
fs4 = "0.8"                # Advisory file locking with tokio support
```

### CI Security Scanning

```yaml
# .github/workflows/gitleaks.yml — runs on every push/PR
# Scans for hardcoded secrets using gitleaks-action v2
# Config: .gitleaks.toml
```

## Quick Start

### Run Tests

```bash
# All tests (4x faster than cargo test)
cargo nextest run --test-threads 2

# Specific test file
cargo nextest run --test chat_handler
cargo nextest run --test provider_commands

# Live contract tests (requires API keys)
LIVE_TEST=1 cargo test --test live_contract_tests -- --ignored

# Only failing tests
cargo nextest run --no-fail-fast
```

### Generate Coverage

```bash
# HTML report
cargo llvm-cov --html --output-dir coverage-llvm

# Open in browser
cargo llvm-cov --html --open

# Terminal summary
cargo llvm-cov --summary-only

# By-file coverage
cargo llvm-cov --summary-only | grep "src/"
```

### Watch Mode

```bash
# Auto-rerun tests on changes
./scripts/dev.sh

# Or manually
cargo watch -x "nextest run --test-threads 2"
```

## Test Organization

### Test Categories

| Category | Location | Tools | Purpose |
|----------|----------|-------|---------|
| **Unit Tests** | `src/**/*.rs` | `#[test]`, `proptest` | Domain logic, pure functions |
| **Integration Tests** | `tests/*.rs` | `wiremock`, `mockall` | Component + HTTP testing |
| **Snapshot Tests** | `tests/error_snapshots.rs` | `insta` | Golden tests for error formats |
| **Live Contract Tests** | `tests/live_contract_tests.rs` | Real APIs | Provider drift detection |

### Test Files by Functional Area

Based on the current codebase organization (47 test modules, 519 test functions):

| Area | Test Coverage | Key Test Files |
|------|---------------|----------------|
| **Domain Entities** | 100% | Account, Provider, AccountHealth tests |
| **Domain Errors** | 100% | Error type formatting, edge cases |
| **Domain Services** | 86-87% | ModelSelector, QueryClassifier tests |
| **Execution Plan** | 80-90% | Cascading, Planner, Types tests |
| **Quality Evaluator** | 85% | HeuristicQualityEvaluator tests |
| **Failover Manager** | 86.79% | Failover logic, circuit breaker tests |
| **Account Rotation** | 87.31% | Strategy pattern tests |
| **Gateway** | 94.26% | LlmGatewayImpl, ProviderConfig tests |
| **Chat Handler** | 85.80% | HTTP handler integration tests |
| **Health Handler** | 100% | Health endpoint tests |
| **CLI Commands** | 84-85% | Provider, Account, Auth command tests |
| **Persistence** | 80.72% | JSON repository, atomic write tests |

### Test Structure

All tests follow the Arrange-Act-Assert pattern:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_name_scenario_expected() {
        // Arrange
        let input = ...;

        // Act
        let result = function_under_test(input);

        // Assert
        assert!(result.is_ok());
    }
}
```

## Test Patterns

### Unit Tests (Domain Layer)

```rust
// src/domain/entities/account_health_tests.rs

#[test]
fn test_health_score_with_no_requests() {
    let health = AccountHealth::new("test-account");
    assert_eq!(health.health_score(), 25.0); // Default score
}

#[test]
fn test_circuit_breaker_opens_after_5_failures() {
    let mut health = AccountHealth::new("test-account");

    for _ in 0..4 {
        health.record_failure();
        assert!(!health.circuit_breaker_open);
    }

    health.record_failure();
    assert!(health.circuit_breaker_open);
}
```

### Integration Tests (with wiremock)

```rust
// tests/chat_handler_wiremock_tests.rs

use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, header};

#[tokio::test]
async fn test_chat_handler_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "chatcmpl-test",
            "choices": [{"message": {"content": "Test response"}}]
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let response = make_chat_request(&mock_server.uri()).await;
    assert_eq!(response.status(), StatusCode::OK);

    mock_server.verify().await;
}
```

### Snapshot Tests (with insta)

```rust
// tests/error_snapshots.rs

#[test]
fn test_error_format_no_api_key_leak() {
    let api_key = "sk-super-secret-key-12345";
    let error = Error::Internal(format!("Failed with key: {}", api_key));

    insta::assert_snapshot!(format!("{:?}", error), @r###"
    Internal("Failed with key: [REDACTED]")
    "###);

    let error_string = format!("{:?}", error);
    assert!(!error_string.contains(api_key));
}
```

### Property-Based Tests (with proptest)

```rust
// src/domain/entities/account_health_tests.rs

use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_health_score_in_range(total_requests: u64, successful: u64) {
        let mut health = AccountHealth::new("test");

        for _ in 0..total_requests.min(1000) {
            if _ < successful.min(total_requests) {
                health.record_success(100);
            } else {
                health.record_failure();
            }
        }

        let score = health.health_score();
        prop_assert!(score >= 0.0 && score <= 100.0);
    }
}
```

### Live Contract Tests (Real APIs)

```bash
# Enable live tests (requires API keys)
LIVE_TEST=1 GROQ_API_KEY=your-key cargo test --test live_contract_tests -- --ignored test_groq_contract
LIVE_TEST=1 OPENAI_API_KEY=your-key cargo test --test live_contract_tests -- --ignored test_openai_contract
LIVE_TEST=1 ANTHROPIC_API_KEY=your-key cargo test --test live_contract_tests -- --ignored test_anthropic_contract
```

Live contract tests:
- Validate provider API schemas against expected contracts
- Detect provider drift before it breaks production
- Gated behind `LIVE_TEST=1` + provider API key environment variables
- Use insta snapshots with redactions for stable comparisons
- Run on main branch only (not on every CI run)

## Coverage Goals

### Target Coverage by Layer

| Layer | Target | Current | Status |
|-------|--------|---------|--------|
| **Domain** | >90% | 100% | Achieved |
| **Error Handling** | >90% | 100% | Achieved |
| **Application Services** | >80% | 86-87% | Achieved |
| **Infrastructure** | >80% | 80-94% | Achieved |
| **Presentation** | >80% | 85% | Achieved |
| **CLI** | >80% | 84-85% | Achieved |
| **Overall** | >80% | 80.35% | Achieved |

### Coverage Achievement

- **Starting point**: 32.02% coverage, 104 tests
- **Final achievement**: 80.35% coverage, 492 tests
- **Progress**: +48.33% coverage, +388 tests

### Monitoring Coverage

```bash
# Check overall summary
cargo llvm-cov --summary-only

# Check specific module coverage
cargo llvm-cov --summary-only | grep "execution_plan"
cargo llvm-cov --summary-only | grep "quality"

# Generate XML for CI
cargo llvm-cov --xml --output-path coverage.xml
```

## Performance Targets

| Metric | Traditional | Optimized | Improvement |
|--------|-------------|-----------|-------------|
| **Test Execution** | ~31s | ~8s | 4x faster |
| **Coverage Generation** | ~5min | ~30s | 10x faster |
| **Build (cached)** | ~60s | ~10s (with sccache) | 6x faster |

### Test Distribution

```
Unit Tests:           ~200 tests (40%)
Integration Tests:    ~200 tests (40%)
Security Tests:        ~50 tests (10%)
Snapshot Tests:        ~20 tests (5%)
Property-based:        ~12 tests (3%)
Live Contract Tests:    ~3 tests (<1%, gated)
```

## Troubleshooting

### Tests Failing to Compile

**Problem**: `missing field 'provider_config' in initializer of AppState`

**Solution**:
```rust
let config = ProviderConfig::default();
let state = AppState {
    provider_config: Arc::new(HashMap::new()),
    // ... other fields
};
```

### Mock Not Matching

**Problem**: wiremock expects request but doesn't receive it

**Solution**:
- Verify URL path matches exactly
- Check headers (Authorization, Content-Type)
- Use `.expect(1)` to verify mock was called
- Use `mock_server.verify().await` at end of test

### Coverage Not Generating

**Problem**: `cargo llvm-cov` times out or produces no output

**Solution**:
```bash
# Clean build artifacts
cargo clean

# Run with explicit options
cargo llvm-cov --clean --html --output-dir coverage-llvm

# For HDD systems, increase timeout
export CARGO_LLVM_COV_TIMEOUT=300
```

## Best Practices

### Do

- Use `cargo nextest` for running tests (4x faster)
- Use `cargo llvm-cov` for coverage (10x faster)
- Write tests in Arrange-Act-Assert pattern
- Use descriptive test names: `test_feature_scenario_expected()`
- Mock external dependencies (HTTP, DB, FS)
- Test error paths, not just happy paths
- Use `#[ignore]` for slow tests (>1s)
- Keep domain layer pure (no external dependencies except serde)

### Don't

- Use `cargo test` (slower than nextest)
- Use `cargo tarpaulin` (slower than llvm-cov)
- Test implementation details
- Write tests that depend on external services (except live contract tests)
- Ignore failing tests
- Commit coverage directories (`coverage/`, `coverage-llvm/`)
- Use `unwrap()` or `expect()` in production code
- Hold locks across `.await` points

## Achievements

- 80.35% Code Coverage (32% -> 80.35%, +48.33%)
- 492 Tests Passing (104 -> 492, +388 tests)
- 0 Tests Failing
- 100% Domain Coverage
- 100% Error Handling Coverage
- 94.26% Gateway Coverage
- Live contract tests for OpenAI, Anthropic, Groq
- Atomic JSON persistence with fs4 locking
- Property-based testing for edge cases

## Resources

- [cargo-nextest](https://nexte.st/) -- Fast test runner
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) -- Native coverage
- [wiremock](https://docs.rs/wiremock) -- HTTP mocking
- [insta](https://insta.rs/) -- Snapshot testing
- [proptest](https://altsysrq.github.io/proptest-book/) -- Property-based testing
- [Testing Journey](TESTING_JOURNEY.md) -- Historical coverage progress
- [Architecture](architecture.md) -- System architecture

---

**Last Updated:** April 2026
**Rust Version:** 1.93.0
**Test Coverage:** 80.35%
**Tests Passing:** 492/492
