# Testing Guide - Rust-LLM-Api-Router

## 🎯 Overview

Este proyecto alcanza **80.35% de cobertura de código** con **492 tests passing**, siguiendo las mejores prácticas de Rust 2025-26.

## 📦 Stack Óptimo 2025-26

### Herramientas Instaladas

```bash
# Instalar una vez
cargo install cargo-nextest      # Test runner (4x faster)
cargo install cargo-llvm-cov     # Cobertura nativa LLVM (10x faster)
cargo install sccache            # Cache de compilación (6x faster)
cargo install cargo-watch        # Auto-recompilar en cambios
cargo install cargo-binstall     # Binary installs (más rápido)
cargo install cargo-audit        # Security audit
cargo install cargo-deny         # License/dependency check
```

### Verificación

```bash
cargo nextest --version
cargo llvm-cov --version
sccache --version
cargo watch --version
```

## 🚀 Quick Start

### Correr Tests

```bash
# Todos los tests (4x más rápido que cargo test)
cargo nextest run --test-threads 2

# Tests específicos
cargo nextest run --test chat_handler
cargo nextest run --test provider_commands

# Ver solo tests fallidos
cargo nextest run --no-fail-fast
```

### Generar Cobertura

```bash
# Generar reporte HTML
cargo llvm-cov --html --output-dir coverage-llvm

# Abrir reporte en navegador
cargo llvm-cov --html --open

# Ver resumen en terminal
cargo llvm-cov --summary-only
```

### Watch Mode

```bash
# Auto-rerun tests en cambios
./scripts/dev.sh

# O manualmente
cargo watch -x "nextest run --test-threads 2"
```

## 📊 Testing Strategy

### Test Categories

| Tipo | Ubicación | Propósito | Herramientas |
|------|-----------|-----------|--------------|
| **Unit Tests** | `src/**/*.rs` | Lógica de dominio pura | `#[test]`, `proptest` |
| **Integration Tests** | `tests/*.rs` | Componentes + HTTP | `wiremock`, `mockall` |
| **Security Tests** | `tests/security_tests.rs` | Vulnerabilidades | `proptest`, custom |
| **Snapshot Tests** | `tests/error_snapshots.rs` | Golden tests | `insta` |
| **Chaos Tests** | `tests/failover_chaos.rs` | Failover scenarios | `turmoil` |

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature_name() {
        // Arrange
        let input = ...;
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert!(result.is_ok());
    }
}
```

## 🧪 Writing Tests

### Unit Tests (Domain Layer)

```rust
// src/domain/entities/account_health_tests.rs

#[test]
fn test_health_score_with_no_requests() {
    let health = AccountHealth::new("test-account");
    
    // Default score for new accounts
    assert_eq!(health.health_score(), 25.0);
}

#[test]
fn test_circuit_breaker_opens_after_5_failures() {
    let mut health = AccountHealth::new("test-account");
    
    // 4 failures - should still be closed
    for i in 0..4 {
        health.record_failure();
        assert!(!health.circuit_breaker_open);
    }
    
    // 5th failure - should open
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
    
    // Make request and verify response
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
    
    // Snapshot verifies error format
    insta::assert_snapshot!(format!("{:?}", error), @r###"
    Internal("Failed with key: [REDACTED]")
    "###);
    
    // Verify API key is not leaked
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
        
        // Record some requests
        for _ in 0..total_requests.min(1000) {
            if _ < successful.min(total_requests) {
                health.record_success(100);
            } else {
                health.record_failure();
            }
        }
        
        // Score should always be 0-100
        let score = health.health_score();
        prop_assert!(score >= 0.0 && score <= 100.0);
    }
}
```

## 📈 Coverage Goals

### Target Coverage by Layer

| Layer | Target | Current | Status |
|-------|--------|---------|--------|
| **Domain** | >90% | 100% | ✅ |
| **Error Handling** | >90% | 100% | ✅ |
| **Application Services** | >80% | 86-87% | ✅ |
| **Infrastructure** | >80% | 80-94% | ✅ |
| **Presentation** | >80% | 85% | ✅ |
| **CLI** | >80% | 84-85% | ✅ |

### Monitoring Coverage

```bash
# Check coverage by file
cargo llvm-cov --summary-only | grep "src/"

# Check specific file coverage
cargo llvm-cov --open --package rust-llm-api-router -- chat_handler

# Generate coverage for CI
cargo llvm-cov --xml --output-path coverage.xml
```

## 🔧 Troubleshooting

### Tests Failing to Compile

**Problem**: `missing field 'provider_config' in initializer of AppState`

**Solution**:
```rust
// Include provider_config in AppState
let config = ProviderConfig::default();
let gateway = Arc::new(LlmGateway::with_config(config.clone()));

let state = AppState {
    failover_manager: Arc::new(manager),
    llm_gateway: gateway,
    provider_config: Arc::new(HashMap::new()),
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

## 🎯 Best Practices

### DO ✅

- Use `cargo nextest` for running tests (4x faster)
- Use `cargo llvm-cov` for coverage (10x faster)
- Write tests in `Arrange-Act-Assert` pattern
- Use descriptive test names: `test_feature_scenario_expected()`
- Mock external dependencies (HTTP, DB, FS)
- Test error paths, not just happy paths
- Use `#[ignore]` for slow tests (>1s)

### DON'T ❌

- Don't use `cargo test` (slower than nextest)
- Don't use `cargo tarpaulin` (slower than llvm-cov)
- Don't test implementation details
- Don't write tests that depend on external services
- Don't ignore failing tests
- Don't commit coverage directories (`coverage/`, `coverage-llvm/`)

## 📚 Resources

- [cargo-nextest documentation](https://nexte.st/)
- [cargo-llvm-cov documentation](https://github.com/taiki-e/cargo-llvm-cov)
- [wiremock documentation](https://docs.rs/wiremock)
- [insta documentation](https://insta.rs/)
- [proptest documentation](https://altsysrq.github.io/proptest-book/)

## 🏆 Achievements

- ✅ **80.35% Code Coverage** (32% → 80.35%, +48.33%)
- ✅ **492 Tests Passing** (104 → 492, +388 tests)
- ✅ **0 Tests Failing**
- ✅ **100% Domain Coverage**
- ✅ **100% Error Handling Coverage**
- ✅ **94.26% Gateway Coverage**

---

**Last Updated:** 2026-03-14  
**Rust Version:** 1.93.0  
**Testing Stack:** 2025-26 Optimal
