# Design: TokenValidator spawn_blocking Migration

## Context

ADR 004 mandates that CPU-bound work be isolated from the Tokio I/O reactor. `TokenValidator::validate` performs BPE encoding synchronously inside `route_request` (async fn). This design details the implementation.

## Decision 1: Wrapper Pattern (Not Refactor)

**Chosen**: Add `validate_async` as an async wrapper around the existing sync `validate`.

**Rejected alternatives**:
- Making `validate` itself async: Breaks test code, adds async trait complexity, pollutes the domain layer with runtime concerns.
- Using `tokio::task::spawn` instead of `spawn_blocking`: Wrong tool — `spawn` goes on the same executor, defeating the purpose.

```rust
// KEEP — unchanged, used by tests and sync callers
pub fn validate(request: &ChatRequest) -> Result<u32, DomainError> { ... }

// NEW — async entry point for production
pub async fn validate_async(request: ChatRequest) -> Result<(u32, ChatRequest), DomainError> {
    tokio::task::spawn_blocking(move || {
        let token_count = Self::count_tokens(&request);
        // ... validation logic ...
        Ok((token_count, request))
    })
    .await
    .map_err(|join_err| DomainError::Internal(format!("Token validation task panicked: {join_err}")))?
}
```

## Decision 2: Ownership Transfer (Not Clone)

**Chosen**: `validate_async` takes `ChatRequest` by value and returns `Result<(u32, ChatRequest), DomainError>`.

**Why**: `spawn_blocking` requires `Send + 'static`. The closure must own all captured data. Two options:
1. Clone the request inside the wrapper → unnecessary `Vec<Message>` allocation
2. Take ownership, return it on success → zero-cost, idiomatic Rust

The caller destructures:
```rust
let (token_count, request) = TokenValidator::validate_async(request).await?;
```

On error (`TokenLimitExceeded`), the request is consumed — this is acceptable because the caller returns early with an error response anyway.

## Decision 3: Error Mapping Strategy

**DomainError variants**:

```rust
pub enum DomainError {
    // ... existing variants ...
    Internal(String),  // NEW — for unexpected failures like JoinError
}
```

The `JoinError` from `spawn_blocking` maps to `DomainError::Internal`. This variant already exists in many Rust projects for catch-all unexpected errors. If the project's `DomainError` doesn't have it, add it.

`TokenLimitExceeded` stays as-is — the blocking task returns the same error variant.

## Decision 4: Tracing Context

`spawn_blocking` does NOT automatically propagate the current tracing span. Two options:

**Option A (Recommended)**: The blocking task runs outside the span. The caller's span already wraps the call:

```rust
// In route_request, the tracing::debug! at line 368 already has the span
let (token_count, request) = TokenValidator::validate_async(request).await?;
tracing::debug!(token_count = token_count, "Token count within limits");
```

**Option B**: Manually instrument with `tracing::Instrument`:

```rust
let span = tracing::Span::current();
tokio::task::spawn_blocking(move || span.in_scope(|| Self::validate(&request)))
```

**Decision**: Option A is sufficient. The caller's tracing context already captures the timing. The internal BPE work doesn't need per-step tracing.

## Decision 5: Test Strategy

Existing tests remain unchanged — they call sync `validate` directly.

Add one integration test:

```rust
#[tokio::test]
async fn test_validate_async_within_limit() {
    let request = ChatRequest::new("gpt-4", vec![Message::user("Hello")]);
    let result = TokenValidator::validate_async(request).await;
    assert!(result.is_ok());
    let (count, returned_request) = result.unwrap();
    assert!(count > 0);
    assert_eq!(returned_request.model, "gpt-4");
}
```

## File Changes

| File | Change |
|------|--------|
| `src/domain/services/token_validator.rs` | Add `validate_async` method, optionally add `Internal` to `DomainError` |
| `src/app/router/llm_router.rs` | Replace `TokenValidator::validate(&request)` with `TokenValidator::validate_async(request).await`, destructure tuple |
| `src/domain/errors.rs` | Add `Internal(String)` variant to `DomainError` if not present |

## Sequence Diagram

```
route_request
  │
  ├─ create_execution_context(&request)
  │
  ├─ TokenValidator::validate_async(request)  ──── moves request
  │     │
  │     ├─ spawn_blocking(move ||)
  │     │     │
  │     │     ├─ count_tokens(&request)  ← runs on blocking pool
  │     │     ├─ validate logic
  │     │     └─ Ok((token_count, request))
  │     │
  │     ├─ .await  ← yields I/O thread back to reactor
  │     │
  │     └─ map_err(JoinError → DomainError::Internal)
  │
  ├─ let (token_count, request) = result?;
  │
  ├─ planner.create_plan(context).await
  │
  └─ execute_with_fallback(&mut plan, &request).await
```

## Performance Characteristics

| Scenario | Before (sync) | After (async) |
|----------|---------------|---------------|
| 5 messages, ~200 tokens | ~50μs on I/O thread | ~50μs on blocking pool + ~5μs spawn overhead |
| 100 messages, ~50K tokens | ~5ms on I/O thread (blocks reactor) | ~5ms on blocking pool (reactor free) |
| 1000 messages, ~500K tokens | ~50ms on I/O thread (severe starvation) | ~50ms on blocking pool (reactor free) |

The overhead for small requests is negligible. The benefit for large requests is reactor protection.
