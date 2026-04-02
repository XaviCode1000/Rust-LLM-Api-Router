# Implementation Tasks: Fix Mutex Async Safety

## Task Checklist

### Phase 1: Production Code (✅ Already Implemented — Unstaged)
- [x] TASK-1.1: Replace `std::sync::Mutex` with `tokio::sync::Mutex` in `FailoverManager`
- [x] TASK-1.2: Make 6 FailoverManager methods async with `.lock().await`
- [x] TASK-1.3: Replace panic with `DomainError::Internal` in `create_no_accounts_error`
- [x] TASK-1.4: Add `From<DomainError>` bound to `execute_with_failover` generic `E`
- [x] TASK-2.1: Replace `std::sync::Mutex` with `tokio::sync::Mutex` in `UserAffinityStrategy`
- [x] TASK-2.2: Make `select_for_user` async with `.lock().await`

### Phase 2: Test Infrastructure (✅ Partially Done — Unstaged)
- [x] TASK-3.1: Create `tests/common/errors.rs` with `TestError` type
- [x] TASK-3.2: Export `errors` module in `tests/common/mod.rs`

### Phase 3: Test Migration (⏳ Pending)
- [ ] TASK-3.3: Update `tests/failover_integration.rs` — Replace `String` with `TestError`
- [ ] TASK-3.4: Update `tests/failover_chaos.rs` — Replace local `TestError` with shared import
- [ ] TASK-3.5: Update `tests/security_tests.rs` — Replace `String` with `TestError`

### Phase 4: Verification (⏳ Pending)
- [ ] TASK-4.1: `cargo check --tests` passes
- [ ] TASK-4.2: `cargo test` passes (all tests)
- [ ] TASK-4.3: `cargo clippy -D warnings` passes
- [ ] TASK-4.4: `cargo fmt --check` passes

### Phase 5: Commit (⏳ Pending)
- [ ] TASK-5.1: Stage and commit all changes with Conventional Commits message

## Task Details

### TASK-3.3: Update failover_integration.rs
**Pattern:** Replace `String` error type with `TestError`
1. Add import: `use crate::common::errors::TestError;`
2. Find all `execute_with_failover::<..., String>` → `execute_with_failover::<..., TestError>`
3. Replace error creation: `"error message".to_string()` → `TestError::new("error message")`
4. Update mock error returns to use `TestError`

### TASK-3.4: Update failover_chaos.rs
**Pattern:** Remove local `TestError` definition, use shared import
1. Remove local `struct TestError(String)` definition
2. Add import: `use crate::common::errors::TestError;`
3. Update any `TestError::new(...)` calls to match shared API

### TASK-3.5: Update security_tests.rs
**Pattern:** Replace `String` error type with `TestError`
1. Add import: `use crate::common::errors::TestError;`
2. Find all `execute_with_failover::<..., String>` → `execute_with_failover::<..., TestError>`
3. Replace error creation patterns
4. Update mock error returns
