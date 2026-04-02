# Proposal: Fix Mutex Async Safety

## Intent

Replace `std::sync::Mutex` with `tokio::sync::Mutex` in production async code to prevent tokio runtime starvation, and update all affected tests to compile with the new async signatures.

## Problem

`std::sync::Mutex` blocks the OS thread when `.lock()` is called. When used inside async functions, this blocks the tokio worker thread, preventing other tasks from executing. Under high concurrency with lock contention, this can cause **tokio runtime starvation** — effectively a denial-of-service.

Two production files have this issue:
1. `src/app/services/failover.rs` — 6 `.lock().unwrap()` calls + 1 panic
2. `src/app/services/account_rotation.rs` — 2 `.lock()` calls with error suppression

## Scope

### In Scope
- `src/app/services/failover.rs` — Migrate to `tokio::sync::Mutex`, make methods async, replace panic with `DomainError`
- `src/app/services/account_rotation.rs` — Migrate `UserAffinityStrategy` to `tokio::sync::Mutex`, make `select_for_user` async
- `tests/failover_integration.rs` — Update to use `TestError` instead of `String` as error type
- `tests/failover_chaos.rs` — Update to use `TestError` instead of `String`
- `tests/security_tests.rs` — Update to use `TestError` instead of `String`
- `src/app/services/failover.rs` — Add `From<DomainError>` bound to `execute_with_failover` generic `E`

### Out of Scope
- `src/app/services/auth/service.rs` — All `std::sync::Mutex` usage is inside `#[cfg(test)]` blocks only
- `src/infrastructure/secure_storage/mod.rs` — Not in fix mandate
- No changes to `RotationStrategy` trait (avoid async cascade to all 5 implementations)

## Impact Analysis

### Breaking Changes
- `FailoverManager::can_use_account` → becomes `async fn`
- `FailoverManager::record_success` → becomes `async fn`
- `FailoverManager::record_failure` → becomes `async fn`
- `FailoverManager::update_rate_limits` → becomes `async fn`
- `FailoverManager::get_health` → becomes `async fn`
- `FailoverManager::get_all_health` → becomes `async fn`
- `UserAffinityStrategy::select_for_user` → becomes `async fn`
- `execute_with_failover` error type `E` requires `From<DomainError>` bound

### Test Impact
- 3 test files (1544 lines) need error type migration from `String` to `TestError`
- `tests/common/errors.rs` already provides `TestError` with `From<DomainError>`
- All mock repositories need no changes (they return `DomainError` already)

### No User-Facing API Changes
- All changed methods are internal to the routing system
- HTTP endpoints remain unchanged
- CLI commands remain unchanged

## Approach

1. **Production code** (already implemented, unstaged): Replace `std::sync::Mutex` → `tokio::sync::Mutex` in 2 files
2. **Error type** (already created): `TestError` in `tests/common/errors.rs` implements `From<DomainError>`
3. **Test migration** (pending): Update 3 test files to use `TestError` instead of `String`
4. **Verification**: `cargo check --tests`, `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check`

## Risks

| Risk | Mitigation |
|------|-----------|
| Lock contention with `tokio::sync::Mutex` (has overhead) | Acceptable — correctness over micro-optimization; lock scopes are small |
| Test file changes are large (1544 lines) | Mechanical change — replace `String` with `TestError` + import |
| Callers of async methods not updated | `cargo check --tests` catches all |
