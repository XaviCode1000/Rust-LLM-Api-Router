# SPEC-2: UserAffinityStrategy Mutex Migration

## Requirement
Replace `std::sync::Mutex` with `tokio::sync::Mutex` in `UserAffinityStrategy` to eliminate blocking calls in async context.

## Delta Changes

### Field Type Change
```rust
// BEFORE
last_selection: std::sync::Mutex<std::collections::HashMap<String, String>>

// AFTER
last_selection: tokio::sync::Mutex<std::collections::HashMap<String, String>>
```

### Constructor Change
```rust
// BEFORE
last_selection: std::sync::Mutex::new(std::collections::HashMap::new())

// AFTER
last_selection: tokio::sync::Mutex::new(std::collections::HashMap::new())
```

### Method Signature Change
```rust
// BEFORE
pub fn select_for_user<'a>(&self, accounts: &'a [Account], user_id: &str) -> Option<&'a Account>

// AFTER
pub async fn select_for_user<'a>(&self, accounts: &'a [Account], user_id: &str) -> Option<&'a Account>
```

### Lock Pattern Changes
```rust
// BEFORE
let selection = self.last_selection.lock().ok()?;
if let Ok(mut selection) = self.last_selection.lock() { ... }

// AFTER
let selection = self.last_selection.lock().await;
let mut selection = self.last_selection.lock().await;
```

## Scenarios

### Scenario 1: User affinity selection
**Given** a `UserAffinityStrategy` with `tokio::sync::Mutex`
**When** `select_for_user` is called with a known user_id
**Then** the last selected account is returned without blocking the tokio runtime

### Scenario 2: Fallback to first account
**Given** a `UserAffinityStrategy` where the user's last account is no longer available
**When** `select_for_user` is called
**Then** the first available account is selected and stored

## Acceptance Criteria
- [ ] `cargo check` passes with no errors
- [ ] No `std::sync::Mutex` remains in `UserAffinityStrategy`
- [ ] `select_for_user` is `async fn`
- [ ] No `.lock().ok()?` or `if let Ok(mut) = .lock()` patterns remain
- [ ] `RotationStrategy` trait is NOT changed (avoid async cascade)
