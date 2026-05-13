# Proposal: TokenValidator spawn_blocking Migration

## Intent

Migrate the CPU-bound token counting logic (`TokenValidator::count_tokens` and `TokenValidator::validate`) off the Tokio runtime's I/O threads by wrapping calls in `tokio::task::spawn_blocking`.

## Problem

`TokenValidator::validate` is called synchronously inside `route_request` (an `async fn`). The BPE encoding via `tiktoken-rs` is CPU-bound with no `.await` points. Under sustained load, this monopolizes a Tokio worker thread, starving other I/O-bound tasks on that thread.

While typical request sizes (5-20 messages) complete in sub-millisecond time, the system must handle:
- Requests with large context windows (100K+ token models)
- High concurrency where every microsecond of executor time matters
- Future tokenizer changes that may be more expensive

## Scope

**In scope:**
- `src/domain/services/token_validator.rs` — add async wrapper using `spawn_blocking`
- `src/app/router/llm_router.rs` — update call site to use async validate
- Tests — ensure existing tests pass, add async test variant

**Out of scope:**
- Changes to the `TokenValidator` internal logic (BPE counting algorithm)
- Changes to other callers of `count_tokens` (tests only)
- New token counting strategies (different encodings per provider)

## Approach

Keep `count_tokens` and `validate` as synchronous functions (they're pure computation). Add a `validate_async` wrapper that:
1. Takes ownership of `ChatRequest` (avoids clone)
2. Spawns the sync validate on the blocking thread pool
3. Returns `Result<(u32, ChatRequest), DomainError>` — the token count AND the request back

The caller in `llm_router.rs` destructures the tuple and continues with the request.

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| `spawn_blocking` overhead for small requests | Low | Pool thread reuse makes this negligible (~microseconds). Correctness > micro-optimization. |
| `JoinError` from panicked blocking task | Low | The sync `validate` has no panic paths (only `expect` on encoding load which is infallible after first call). Map `JoinError` to `DomainError::Internal`. |
| API surface change | Medium | `validate` stays sync for tests. `validate_async` is the new async entry point. No breaking changes to existing sync callers. |

## Success Criteria

1. `cargo check` passes
2. `cargo clippy --deny warnings` passes
3. All existing `TokenValidator` tests pass unchanged
4. New async test validates `validate_async` returns correct token count
5. `route_request` calls `validate_async` and correctly handles the returned request
