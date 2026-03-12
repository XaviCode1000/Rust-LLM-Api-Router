# mem-avoid-format

**Category:** Memory Optimization | **Priority:** CRITICAL

Avoid `format!()` when string literals or `&str` work.

## Why

`format!()` allocates a new `String` on the heap. String literals are `&'static str` (no allocation).

## Examples

### ❌ Bad: Unnecessary format!

```rust
fn get_message() -> String {
    format!("Hello, World!")  // Allocation for static string
}
```

### ✅ Good: Return &str

```rust
fn get_message() -> &'static str {
    "Hello, World!"  // No allocation
}
```

### ❌ Bad: format! for logging

```rust
log::info!("{}", format!("User {} logged in", user.name));
```

### ✅ Good: Pass args to macro

```rust
log::info!("User {} logged in", user.name);  // Macro handles formatting
```

### ❌ Bad: format! in hot path

```rust
for item in items {
    let key = format!("prefix_{}", item.id);  // Allocation in loop!
    map.insert(key, item);
}
```

### ✅ Good: Reuse buffer

```rust
let mut key = String::with_capacity(32);
for item in items {
    key.clear();
    write!(&mut key, "prefix_{}", item.id).unwrap();  // Reuse allocation
    map.insert(key.clone(), item);
}
```

## Related rules

- `mem-write-over-format`
- `mem-with-capacity`
- `anti-format-hot-path`
