# own-slice-over-vec

**Category:** Ownership & Borrowing | **Priority:** CRITICAL

Accept `&[T]` instead of `&Vec<T>`, and `&str` instead of `&String`.

## Why

`&Vec<T>` forces the caller to have a `Vec`, but your function probably just needs a slice. Same for `&String` vs `&str`.

## Examples

### ❌ Bad: Accepting &Vec

```rust
fn process_items(items: &Vec<String>) {
    for item in items {
        println!("{}", item);
    }
}
```

### ✅ Good: Accepting slice

```rust
fn process_items(items: &[String]) {
    for item in items {
        println!("{}", item);
    }
}
```

### ❌ Bad: Accepting &String

```rust
fn greet(name: &String) {
    println!("Hello, {}", name);
}
```

### ✅ Good: Accepting &str

```rust
fn greet(name: &str) {
    println!("Hello, {}", name);
}
```

## Why this matters

- `&[T]` accepts: slices, `Vec`, arrays, `&[T]`
- `&str` accepts: `String`, `&str`, string literals
- More flexible API = happier users

## Related rules

- `own-borrow-over-clone`
- `api-impl-asref`
- `anti-vec-for-slice`
- `anti-string-for-str`
