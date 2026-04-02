# Technical Design: Fix Mutex Async Safety

## Architecture Decision

### Why `tokio::sync::Mutex` over `std::sync::Mutex`

In async Rust, holding a `std::sync::Mutex` lock across any `.await` point blocks the entire tokio worker thread. Since tokio uses a work-stealing scheduler with a fixed thread pool (typically equal to CPU cores), blocking one thread reduces available parallelism. Under high contention, this causes **runtime starvation** — other async tasks cannot make progress.

`tokio::sync::Mutex` yields the task back to the scheduler when the lock is unavailable, allowing other tasks to run. The tradeoff is slightly higher per-lock overhead (atomic operations + task parking), but this is negligible for lock scopes measured in microseconds.

## Implementation Details

### 1. failover.rs

**Field migration:**
```rust
health_map: Mutex<std::collections::HashMap<String, AccountHealth>>
```
Uses `use tokio::sync::Mutex;` import, type alias `Mutex` for brevity.

**Constructor migration:**
Both `new()` and `with_backoff()` use `Mutex::new(...)` — same API, different type.

**Method async migration (6 methods):**
Each method follows the same pattern:
```rust
async fn method(&self, ...) {
    let mut guard = self.health_map.lock().await;
    // operate on guard
} // guard dropped here, lock released
```

**Panic elimination:**
`create_no_accounts_error` returns `DomainError::Internal(...)` instead of panicking. The caller uses `.into()` to convert to the generic error type `E`.

**Generic bound:**
`execute_with_failover<F, Fut, T, E>` adds `E: From<DomainError>` so the no-accounts error can be converted.

### 2. account_rotation.rs

**Field migration:**
```rust
last_selection: tokio::sync::Mutex<std::collections::HashMap<String, String>>
```
Uses fully qualified `tokio::sync::Mutex` to avoid import conflicts.

**Method async migration:**
`select_for_user` becomes `async fn`. Lock scopes are minimized:
```rust
// Read lock — released before branching
let last = {
    let selection = self.last_selection.lock().await;
    selection.get(user_id).cloned()
};

// Write lock — only if needed
let mut selection = self.last_selection.lock().await;
selection.insert(user_id.to_string(), account.id.clone());
```

**Design decision: Keep as inherent method**
`select_for_user` is NOT part of the `RotationStrategy` trait. It's an inherent method on `UserAffinityStrategy`. This avoids forcing all 5 strategy implementations to become async.

### 3. Test Error Type

**Problem:** `execute_with_failover` requires `E: From<DomainError>`. Tests used `String` which doesn't implement this.

**Solution:** `TestError` in `tests/common/errors.rs`:
```rust
#[derive(Clone, Debug)]
pub struct TestError(String);

impl From<DomainError> for TestError {
    fn from(e: DomainError) -> Self {
        TestError(format!("DomainError: {}", e))
    }
}
```

**Migration pattern per test file:**
1. Add `use crate::common::errors::TestError;` (or `mod common;` + path)
2. Replace `String` with `TestError` in type annotations
3. Replace `format!(...)` error creation with `TestError::new(&format!(...))`
4. Remove local `TestError` struct definitions (failover_chaos.rs has one)

## Async Boundaries

All `.lock().await` calls follow the pattern:
```rust
{
    let mut guard = mutex.lock().await;
    // minimal operations
} // guard dropped, lock released
// other async work here
```

No lock is held across any `.await` point other than the lock acquisition itself.

## Migration Strategy

**Already done (unstaged):**
- Production code in failover.rs and account_rotation.rs
- `tests/common/errors.rs` created
- `tests/common/mod.rs` updated to export `errors` module

**Pending:**
- Update 3 test files to use `TestError`
- Verify compilation and test suite

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| `tokio::sync::Mutex` overhead | ~50ns per lock vs ~5ns for std | Negligible compared to network I/O (ms) |
| Test file size (1544 lines) | Manual changes error-prone | Mechanical pattern — search and replace |
| Compilation cascade | Other callers may break | `cargo check --tests` catches all |
