# Tasks: TokenValidator spawn_blocking Migration

## Task 0: Fix validation bypass bug in route_request (CRITICAL) ✅

**File**: `src/app/router/llm_router.rs` (lines 391-393)

- [x] The current `Err(e)` catch-all logs but does NOT return — validation failure is silently ignored
- [x] Change to return early: `return Err(Error::Internal(format!("Token validation failed: {e}")));`
- [x] This must be done BEFORE adding `validate_async` — the bug exists in the sync path too
- [x] Verify: `cargo check` passes

---

## Task 1: Add `validate_async` to TokenValidator ✅

**File**: `src/domain/services/token_validator.rs`

- [x] Add `pub async fn validate_async(request: ChatRequest) -> Result<(u32, ChatRequest), DomainError>`
- [x] Implement using `tokio::task::spawn_blocking(move || { ... })`
- [x] Map `JoinError` to `DomainError::Internal` with descriptive message
- [x] Keep existing `validate` and `count_tokens` unchanged
- [x] Verify: `cargo check` passes

---

## Task 2: Update `route_request` call site ✅

**File**: `src/app/router/llm_router.rs`

- [x] Replace `TokenValidator::validate(&request)` with `TokenValidator::validate_async(request).await`
- [x] Destructure the result: `let (_token_count, request) = match ...`
- [x] Update the `Ok` branch to use `request.model` from the returned request
- [x] Update the `Err` branch — request is consumed, returns early
- [x] Verify: `cargo check` passes

---

## Task 3: Add async integration tests ✅

**File**: `src/domain/services/token_validator.rs` (inside `#[cfg(test)]`)

- [x] `test_validate_async_within_limit` — valid request, request returned intact
- [x] `test_validate_async_exceeds_limit` — TokenLimitExceeded error
- [x] `test_validate_async_unknown_model` — unknown model skips validation
- [x] Verify: 10/10 tests pass (7 existing + 3 new)

---

## Task 4: Quality gates ✅

- [x] `cargo check` — PASS
- [x] `cargo test --lib token_validator` — 10/10 PASS
- [x] No new `unwrap()` or `expect()` added in production code paths
- [x] `fmt` and `clippy` issues are pre-existing (not introduced by this change)

---

## Files Changed

| File | Lines Changed | Description |
|------|--------------|-------------|
| `src/domain/services/token_validator.rs` | +35 lines | Added `validate_async` method + 3 async tests |
| `src/app/router/llm_router.rs` | ~30 lines refactored | Fixed bypass bug + migrated to `validate_async` |
