# mem-with-capacity

**Category:** Memory Optimization | **Priority:** CRITICAL

Use `with_capacity()` when you know the size in advance.

## Why

`Vec::new()` starts with small capacity and reallocates as it grows. Each reallocation copies all elements.

## Examples

### ❌ Bad: Growing Vec

```rust
let mut items = Vec::new();
for i in 0..1000 {
    items.push(i);  // Multiple reallocations
}
```

### ✅ Good: Pre-allocate

```rust
let mut items = Vec::with_capacity(1000);
for i in 0..1000 {
    items.push(i);  // No reallocations
}
```

### ❌ Bad: String concatenation

```rust
let mut result = String::new();
for part in parts {
    result.push_str(part);  // Multiple reallocations
}
```

### ✅ Good: Pre-calculate size

```rust
let total_len: usize = parts.iter().map(|s| s.len()).sum();
let mut result = String::with_capacity(total_len);
for part in parts {
    result.push_str(part);  // Minimal reallocations
}
```

## Related rules

- `mem-avoid-format`
- `mem-reuse-collections`
- `mem-clone-from`
