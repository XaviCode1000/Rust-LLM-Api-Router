## Exploration: fix-mutex-async-safety

### Current State

**Project**: rust-llm-api-router
**Stack**: Rust 1.75+, tokio async runtime, Axum web framework, reqwest HTTP client, thiserror errors
**Test Runner**: cargo test (built-in) with tokio::test, mockall, wiremock, proptest, insta

### Affected Area 1: `src/app/services/failover.rs` — CRITICAL

`FailoverManager` uses `std::sync::Mutex<HashMap<String, AccountHealth>>` in an async struct. All methods calling `.lock().unwrap()` are invoked from `execute_with_failover()` which is async. This risks tokio runtime starvation.

**Exact issues:**

| Line | Pattern | Risk |
|------|---------|------|
| 24 | `health_map: std::sync::Mutex<...>` | Field declaration |
| 62 | `std::sync::Mutex::new(...)` in `new()` | Constructor |
| 78 | `std::sync::Mutex::new(...)` in `with_backoff()` | Constructor |
| 251 | `.lock().unwrap()` in `can_use_account` | Blocking inside async |
| 261 | `.lock().unwrap()` in `record_success` | Blocking inside async |
| 270 | `.lock().unwrap()` in `record_failure` | Blocking inside async |
| 285 | `.lock().unwrap()` in `update_rate_limits` | Blocking inside async |
| 306 | `.lock().unwrap()` in `get_health` | Sync accessor |
| 312 | `.lock().unwrap()` in `get_all_health` | Sync accessor |
| 299-301 | `panic!("No available accounts...")` | Panic instead of DomainError |

### Affected Area 2: `src/app/services/account_rotation.rs` — CRITICAL

`UserAffinityStrategy` uses `std::sync::Mutex<HashMap<String, String>>` for `last_selection`.

**Exact issues:**

| Line | Pattern | Risk |
|------|---------|------|
| 418 | `last_selection: std::sync::Mutex<...>` | Field declaration |
| 424 | `std::sync::Mutex::new(...)` in `new()` | Constructor |
| 440 | `.lock().ok()?` | Silent error handling |
| 454 | `if let Ok(mut) = .lock()` | Silent failure on lock error |

### Affected Area 3: `src/app/services/auth/service.rs` — NO PRODUCTION CHANGES NEEDED

**CRITICAL FINDING**: The audit context claimed 10+ `.lock().unwrap()` calls in auth/service.rs. ALL of them are inside `#[cfg(test)] mod tests` (lines 273+). The production code (lines 1-270) contains ZERO `std::sync::Mutex` usage. It only uses `Arc<dyn Repository>`.

Since the mandate says DO NOT MODIFY tests, this file requires zero changes.

### Out of Scope: `src/infrastructure/secure_storage/mod.rs`
Uses `std::sync::Mutex` at lines 78, 84, 100, 106, 111. Not in fix mandate.

---

### Dependency Status

**tokio `sync` feature ALREADY ENABLED** in `Cargo.toml` (line 24):
```toml
tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync", "time", "fs", "io-util"] }
```
No dependency changes needed. `tokio::sync::Mutex` ready to use.

---

### DomainError Variants for ? Conversions

| Variant | Signature | Use in this fix |
|---------|-----------|-----------------|
| `LockTimeout(String)` | For try_lock failures | If using try_lock |
| `Internal(String)` | For panic replacement | `create_no_accounts_error` |
| `ConfigError(String)` | Generic config issue | Alternative for panic replacement |
| `AccountNotFound(String)` | Existing | N/A |
| `DomainError(String)` | Generic fallback | N/A |

Type alias: `DomainResult<T> = Result<T, DomainError>`

---

### std::sync::Mutex vs tokio::sync::Mutex Trade-offs

| Aspect | std::sync::Mutex | tokio::sync::Mutex |
|--------|-----------------|-------------------|
| Lock return type | `Result<MutexGuard, PoisonError>` | `MutexGuard` (no poison) |
| Lock usage | `.lock().unwrap()` | `.lock().await` |
| Blocking | Thread blocking | Async yield |
| Fairness | Unfair | Fair (FIFO) |
| Poisons | Yes | No |

---

### Recommended Approach (Approach 1 — Minimal Scope)

Replace only in 2 files:

#### failover.rs changes:
1. Replace `std::sync::Mutex` → `tokio::sync::Mutex` (field + 2 constructors)
2. Replace 6 × `.lock().unwrap()` → `.lock().await`
3. Methods become async: `can_use_account`, `record_success`, `record_failure`, `update_rate_limits`, `get_health`, `get_all_health`
4. Replace `create_no_accounts_error` panic → `DomainError::Internal(...)` or `DomainError::AccountDisabled(...)`

#### account_rotation.rs changes:
1. Replace `std::sync::Mutex` → `tokio::sync::Mutex` (field + constructor)
2. `.lock().ok()?` → `.lock().await`
3. `if let Ok(mut) = .lock()` → `.lock().await` with `?`
4. **DESIGN ISSUE**: `select_for_user` may need async if inherent call changes

#### auth/service.rs changes:
NONE. All mutex usage in test mocks only.

---

### Complexity Ranking

1. failover.rs — LOW (direct async context, methods already called from async)
2. account_rotation.rs — MEDIUM (trait async cascade risk)
3. auth/service.rs — ZERO (no production changes)

### Risks

- **Trait async cascade**: If `select_for_user` becomes async through trait, all 5 strategy implementations need updating
- **Sync accessor breakage**: `get_health()` / `get_all_health()` becoming async may break callers
- **Scope correction needed**: Parent agent should confirm auth/service.rs has no production work

### Ready for Proposal

**Yes** — with caveats:
1. auth/service.rs needs NO production changes (test-only code)
2. account_rotation.rs trait impact needs design resolution
3. Recommendation: keep `select_for_user` as inherent method to avoid trait async cascade