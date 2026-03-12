# own-borrow-over-clone

**Category:** Ownership & Borrowing | **Priority:** CRITICAL

Prefer borrowing (`&T`) over cloning (`.clone()`).

## Why

Cloning allocates memory and copies data. Borrowing just creates a reference.

## Examples

### ❌ Bad: Unnecessary clone

```rust
fn process(data: Vec<String>) {
    let cloned = data.clone();  // Expensive allocation
    // use cloned
}
```

### ✅ Good: Borrow instead

```rust
fn process(data: &[String]) {
    // use data directly, no clone
}
```

## When cloning is acceptable

- You need ownership for async tasks
- The data is `Copy` (cheap to copy)
- You're storing data in a collection long-term

## Related rules

- `own-slice-over-vec`
- `own-clone-explicit`
- `mem-clone-from`
