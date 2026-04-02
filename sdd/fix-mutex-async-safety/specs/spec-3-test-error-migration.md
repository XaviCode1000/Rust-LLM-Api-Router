# SPEC-3: Test Error Type Migration

## Requirement
Update all integration test files to use `TestError` instead of `String` as the error type in `execute_with_failover` calls, since the production code now requires `E: From<DomainError>`.

## Delta Changes

### Shared Error Type (already created)
`tests/common/errors.rs` provides:
```rust
#[derive(Clone, Debug)]
pub struct TestError(String);

impl TestError {
    pub fn new(msg: &str) -> Self { ... }
}

impl fmt::Display for TestError { ... }

impl From<DomainError> for TestError {
    fn from(e: DomainError) -> Self { ... }
}
```

### Test File Changes
Each test file must:
1. Import `TestError` from `common::errors`
2. Replace `String` error type usage with `TestError`
3. Replace `.to_string()` error creation with `TestError::new(...)`
4. Update mock error returns to use `TestError`

### Files Affected
| File | Lines | Pattern to Replace |
|------|-------|-------------------|
| `tests/failover_integration.rs` | 513 | `String` → `TestError` |
| `tests/failover_chaos.rs` | 354 | `TestError(String)` local → `TestError` from common |
| `tests/security_tests.rs` | 677 | `String` → `TestError` |

## Scenarios

### Scenario 1: Integration test with TestError
**Given** a test that calls `execute_with_failover`
**When** the call fails with a `DomainError`
**Then** the error is automatically converted to `TestError` via `From<DomainError>`

### Scenario 2: Chaos test with TestError
**Given** a chaos test using a mock repository
**When** the mock returns `DomainError`
**Then** the test receives `TestError` through the conversion chain

## Acceptance Criteria
- [ ] `cargo check --tests` passes with no errors
- [ ] No `String` used as error type in `execute_with_failover` calls
- [ ] All 3 test files import `TestError` from `common::errors`
- [ ] No duplicate `TestError` definitions in test files
