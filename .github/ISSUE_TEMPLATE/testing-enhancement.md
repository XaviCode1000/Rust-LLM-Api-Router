## Testing Enhancement: Complete Test Suite Audit and Improvements

### Summary

Enhance the existing test suite to follow professional Rust testing best practices. The project currently has 492 tests with 80.35% coverage but critical gaps: **zero CLI binary testing**, **no timeout/retry behavior verification**, and **limited concurrency safety tests**.

---

## Problem Statement

### The "Mock Tunnel Effect"

The project has high test coverage (80.35%) but suffers from the "Mock Tunnel Effect" — tests pass in CI but real usage reveals bugs:

1. **main.rs (103 lines) is completely untested** — no verification of CLI exit codes, error messages, or flag behavior
2. **Timeout and retry logic not tested** — wiremock is used but timeout scenarios aren't covered
3. **No concurrency/race condition tests** — code uses `Arc<TokioMutex>` but data loss under concurrent writes isn't verified

This mirrors findings from industry audits where projects with 500+ tests missed critical bugs because tests only covered happy paths with mocks.

---

## Goals

1. **Add CLI binary testing** using `assert_cmd` to verify exit codes, stdout/stderr, and flag behavior
2. **Add timeout and error path tests** using wiremock to test 404, 500, 429 scenarios
3. **Add concurrency safety tests** to verify no data loss under concurrent operations
4. **Maintain test execution speed** — continue using cargo-nextest (4x faster)
5. **Keep coverage above 80%** — current is 80.35%

---

## Technical Approach

### Phase 1: CLI Binary Testing (HIGH PRIORITY)

Add `assert_cmd` tests in `tests/cli_binary_tests.rs`:

```rust
// Example: Verify --help shows all commands
#[test]
fn test_help_shows_commands() {
    Command::cargo_bin("llm-router")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("provider"))
        .stdout(predicate::str::contains("account"));
}

// Example: Verify invalid provider shows error
#[test]
fn test_invalid_provider_error() {
    Command::cargo_bin("llm-router")
        .unwrap()
        .args(["provider", "validate", "--id", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}
```

**Files to create:**
- `tests/cli_binary_tests.rs` — ~10 tests

**Dependencies needed (verify in Cargo.toml):**
- `assert_cmd = "2"` 
- `predicates = "3"`
- `assert_fs = "1"`

---

### Phase 2: Wiremock Error Path Testing (MEDIUM PRIORITY)

Add timeout and HTTP error scenario tests in `tests/http_error_tests.rs`:

```rust
// Test: Client timeout behavior
#[tokio::test]
async fn test_request_timeout_returns_error() {
    let server = MockServer::start().await;
    
    // Slow response that exceeds timeout
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).delay(Duration::from_secs(60)))
        .mount(&server)
        .await;
    
    let result = client.post(&server.url(), request).await;
    assert!(result.is_err());
}

// Test: 404 error handling
#[tokio::test]
async fn test_404_returns_proper_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;
    
    let result = client.post(&server.url(), request).await;
    assert!(matches!(result, Err(Error::NotFound)));
}

// Test: Rate limiting (429) triggers retry
#[tokio::test]
async fn test_rate_limit_triggers_retry() {
    let server = MockServer::start().await;
    
    // First call returns 429, second succeeds
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(429))
        .mount(&server)
        .await;
    
    // Verify retry logic works
    let result = client.post(&server.url(), request).await;
    // Should have retried or returned appropriate error
}
```

**Files to create:**
- `tests/http_error_tests.rs` — ~8 tests

---

### Phase 3: Concurrency Safety Tests (MEDIUM PRIORITY)

Add race condition tests in `tests/concurrency_tests.rs`:

```rust
// Test: No data loss under concurrent writes
#[tokio::test]
async fn test_concurrent_writes_no_data_loss() {
    let repo = Arc::new(JsonAccountRepository::new().unwrap());
    let barrier = Arc::new(Arc::new(Barrier::new(10)));
    
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let repo = repo.clone();
            let barrier = barrier.clone();
            tokio::spawn(async move {
                barrier.wait().await;
                let account = Account::new(format!("account-{}", i), "test", "key");
                repo.save(account).await
            })
        })
        .collect();
    
    let results = join_all(handles).await;
    
    // All writes should succeed
    assert!(results.iter().all(|r| r.is_ok()));
    
    // Verify all accounts exist
    let all = repo.find_all().await.unwrap();
    assert_eq!(all.len(), 10, "Should have all 10 accounts");
}

// Test: No deadlock with parallel operations
#[tokio::test]
async fn test_parallel_operations_no_deadlock() {
    // Spawn multiple operations that use shared state
    let state = Arc::new(TokioMutex::new(HashMap::new()));
    
    let handles = (0..100)
        .map(|i| {
            let state = state.clone();
            tokio::spawn(async move {
                let mut guard = state.lock().await;
                guard.insert(i.to_string(), i);
            })
        })
        .collect::<Vec<_>>();
    
    // Should complete within reasonable time
    let timeout = tokio::time::timeout(std::time::Duration::from_secs(5), join_all(handles));
    assert!(timeout.await.is_ok(), "Operations should not deadlock");
}
```

**Files to create:**
- `tests/concurrency_tests.rs` — ~5 tests

---

### Phase 4: Cleanup Dead Dependencies (LOW PRIORITY)

Verify and remove unused dev-dependencies:

```bash
# Check which dev-deps are actually used
cargo tree -i mockall  # If unused, remove from Cargo.toml
cargo tree -i tokio-test
```

---

## Implementation Plan

### Tasks

- [ ] **T1.1** Create `tests/cli_binary_tests.rs` with assert_cmd tests (~10 tests)
- [ ] **T1.2** Add `assert_cmd`, `predicates`, `assert_fs` to dev-dependencies if missing
- [ ] **T2.1** Create `tests/http_error_tests.rs` with wiremock error path tests (~8 tests)
- [ ] **T2.2** Test timeout scenarios (client timeout exceeded)
- [ ] **T2.3** Test 404/500 error responses
- [ ] **T2.4** Test rate limiting (429) behavior
- [ ] **T3.1** Create `tests/concurrency_tests.rs` with race condition tests (~5 tests)
- [ ] **T3.2** Test concurrent writes don't lose data
- [ ] **T3.3** Test parallel operations don't deadlock
- [ ] **T4.1** Audit dev-dependencies and remove unused ones
- [ ] **T4.2** Run full test suite and verify all pass
- [ ] **T4.3** Run coverage and verify maintained above 80%

---

## Definition of Done

All of the following must pass:

- [ ] `cargo fmt --check` exits 0
- [ ] `cargo clippy -D warnings` exits 0  
- [ ] `cargo nextest run --test-threads 2` exits 0 (all tests pass)
- [ ] `cargo llvm-cov --summary-only` shows coverage ≥ 80%
- [ ] New CLI binary tests verify exit codes and error messages
- [ ] New HTTP error tests verify timeout, 404, 500, 429 handling
- [ ] New concurrency tests verify no data loss under parallel writes
- [ ] No unused dev-dependencies remain in Cargo.toml

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| New tests add execution time | Low | Low | Keep using cargo-nextest (4x faster) |
| Concurrency tests flaky | Medium | Medium | Run multiple iterations, use proper barriers |
| Break existing tests | Low | High | Run full suite before commit |

---

## Related Issues

- Related to: #23 (Cost-Aware Routing) — tests should cover routing logic
- Related to: #24 (Cascading Routing) — tests should cover escalation behavior
- Supersedes: Previous coverage-focused efforts (now focusing on test quality)

---

## References

- [Effective Rust - Item 30: Write more than unit tests](https://lurklurk.org/effective-rust/testing.html)
- [Rust Testing Best Practices](https://medium.com/@ashusk_1790/rust-testing-best-practices-unit-to-integration-965b39a8212f)
- [Software Patterns Lexicon - Unit Testing](https://softwarepatternslexicon.com/rust/testing-and-quality-assurance/unit-testing-with-cargo-test/)
- [OneUptime - Integration Tests Guide](https://oneuptime.com/blog/post/2026-01-26-rust-integration-tests/view)
- Project's existing testing docs: `docs/TESTING_GUIDE.md`, `docs/TESTING_JOURNEY.md`