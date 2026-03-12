# err-no-unwrap-prod

**Category:** Error Handling | **Priority:** CRITICAL

**NUNCA** use `.unwrap()` in production code.

## Why

`.unwrap()` panics on error. Production code should handle errors gracefully.

## Examples

### ❌ Bad: unwrap in production

```rust
let file = File::open("config.json").unwrap();  // PANIC if file missing!
```

### ✅ Good: Propagate error

```rust
let file = File::open("config.json")?;  // Returns error to caller
```

### ✅ Good: Handle error

```rust
let file = match File::open("config.json") {
    Ok(f) => f,
    Err(e) => {
        log::warn!("Config not found, using defaults: {}", e);
        return Ok(default_config());
    }
};
```

### ✅ Good: expect with context (bugs only)

```rust
// Only for programming errors (should never happen)
let config = load_config().expect("Config must be valid");
```

## When unwrap is acceptable

- Examples and tutorials
- Test code
- Prototypes (not production!)
- `expect()` for bugs only (invariants)

## Related rules

- `err-expect-bugs-only`
- `err-question-mark`
- `err-result-over-panic`
- `anti-unwrap-abuse`
