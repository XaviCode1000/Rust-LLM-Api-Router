# SPEC-1: FailoverManager Mutex Migration

## Requirement
Replace `std::sync::Mutex` with `tokio::sync::Mutex` in `FailoverManager` to eliminate blocking calls in async context.

## Delta Changes

### Field Type Change
```rust
// BEFORE
health_map: std::sync::Mutex<std::collections::HashMap<String, AccountHealth>>

// AFTER
health_map: tokio::sync::Mutex<std::collections::HashMap<String, AccountHealth>>
```

### Constructor Changes
- `new()` — Replace `std::sync::Mutex::new(...)` with `tokio::sync::Mutex::new(...)`
- `with_backoff()` — Same replacement

### Method Signature Changes (6 methods become async)
| Method | Before | After |
|--------|--------|-------|
| `can_use_account` | `fn(&self, &str) -> bool` | `async fn(&self, &str) -> bool` |
| `record_success` | `fn(&self, &str, u64)` | `async fn(&self, &str, u64)` |
| `record_failure` | `fn(&self, &str)` | `async fn(&self, &str)` |
| `update_rate_limits` | `fn(&self, &str, &[(String, String)])` | `async fn(&self, &str, &[(String, String)])` |
| `get_health` | `fn(&self, &str) -> Option<AccountHealth>` | `async fn(&self, &str) -> Option<AccountHealth>` |
| `get_all_health` | `fn(&self) -> Vec<AccountHealth>` | `async fn(&self) -> Vec<AccountHealth>` |

### Lock Pattern Changes
```rust
// BEFORE
let mut health_map = self.health_map.lock().unwrap();

// AFTER
let mut health_map = self.health_map.lock().await;
```

### Panic Elimination
```rust
// BEFORE
fn create_no_accounts_error(&self, provider_id: &str) -> E {
    panic!("No available accounts for provider: {}", provider_id)
}

// AFTER
fn create_no_accounts_error(&self, provider_id: &str) -> DomainError {
    DomainError::Internal(format!("No available accounts for provider: {}", provider_id))
}
```

### Generic Bound Change
```rust
// BEFORE
E: std::fmt::Debug + Clone

// AFTER
E: std::fmt::Debug + Clone + From<crate::domain::errors::DomainError>
```

## Scenarios

### Scenario 1: Lock acquisition in async context
**Given** a `FailoverManager` with `tokio::sync::Mutex`
**When** `can_use_account` is called from an async function
**Then** the lock is acquired with `.lock().await` without blocking the tokio runtime

### Scenario 2: No accounts available returns error instead of panic
**Given** a `FailoverManager` with no active accounts for a provider
**When** `execute_with_failover` is called
**Then** it returns `Err(DomainError::Internal(...))` instead of panicking

### Scenario 3: Error type conversion
**Given** `execute_with_failover` requires `E: From<DomainError>`
**When** no accounts are available
**Then** the `DomainError` is converted to `E` via `.into()`

## Acceptance Criteria
- [ ] `cargo check` passes with no errors
- [ ] No `std::sync::Mutex` remains in `failover.rs` production code
- [ ] All 6 methods are `async fn`
- [ ] `create_no_accounts_error` returns `DomainError` instead of panicking
- [ ] `execute_with_failover` has `From<DomainError>` bound on `E`
