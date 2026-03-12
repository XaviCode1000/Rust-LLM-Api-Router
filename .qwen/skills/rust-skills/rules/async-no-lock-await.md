# async-no-lock-await

**Category:** Async/Await | **Priority:** CRITICAL

**NUNCA** hold `Mutex` or `RwLock` guards across `.await` points.

## Why

Holding locks across await points can cause:
- **Deadlocks** — Other tasks can't acquire the lock
- **Performance issues** — Serialized execution
- **Race conditions** — Unexpected behavior

## Examples

### ❌ Bad: Lock across await

```rust
async fn process(shared: Arc<Mutex<Data>>) {
    let mut guard = shared.lock().await;  // Lock acquired
    guard.value = 42;
    some_async_fn().await;  // ❌ STILL HOLDING LOCK!
    guard.other = 100;      // Other tasks blocked this whole time
}  // Lock released
```

### ✅ Good: Release before await

```rust
async fn process(shared: Arc<Mutex<Data>>) {
    {
        let mut guard = shared.lock().await;
        guard.value = 42;
        guard.other = 100;
    }  // Lock released BEFORE await
    some_async_fn().await;  // ✅ Other tasks can run
}
```

### ✅ Good: Clone data before await

```rust
async fn process(shared: Arc<Mutex<Data>>) {
    let data = {
        let guard = shared.lock().await;
        guard.clone()  // Clone data
    };  // Lock released
    process_data(data).await;  // ✅ No lock held
}
```

## Related rules

- `async-clone-before-await`
- `async-bounded-channel`
- `anti-lock-across-await`
