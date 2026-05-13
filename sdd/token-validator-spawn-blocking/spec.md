# Spec: TokenValidator spawn_blocking Migration

## Requirements

### REQ-1: Async Token Validation Entry Point

The `TokenValidator` SHALL provide an async method `validate_async` that executes token counting on a dedicated blocking thread pool, not on the Tokio runtime's I/O threads.

- **Priority**: MUST
- **Rationale**: Prevents executor starvation under load

### REQ-2: Ownership Transfer Without Clone

`validate_async` SHALL take ownership of `ChatRequest` and return it alongside the validation result, avoiding unnecessary heap allocations.

- **Priority**: MUST
- **Rationale**: `spawn_blocking` requires `Send + 'static`. Taking ownership avoids cloning the `Vec<Message>`.

### REQ-3: JoinError Handling

If the blocking task panics (unexpected), `validate_async` SHALL map `JoinError` to a `DomainError::Internal` variant with a descriptive message. The system MUST NOT propagate panics to the caller.

- **Priority**: MUST
- **Rationale**: Panic isolation is critical in async contexts.

### REQ-4: Preserved Sync API

The existing `count_tokens` and `validate` methods SHALL remain unchanged and synchronous. They continue to be used directly in tests and non-async contexts.

- **Priority**: SHOULD
- **Rationale**: Tests and benchmarks don't need async. Avoids unnecessary refactoring of test code.

### REQ-5: Tracing Context Inheritance

The `validate_async` method SHALL inherit the caller's tracing span. The blocking task's execution SHALL be visible in the same trace as the parent request.

- **Priority**: SHOULD
- **Rationale**: Observability must not degrade. The architect specifically requested tracing span continuity.

### REQ-6: Call Site Update

`route_request` in `llm_router.rs` SHALL call `validate_async` instead of the synchronous `validate`, using `.await`.

- **Priority**: MUST
- **Rationale**: This is the production call site that needs the async behavior.

---

## Scenarios

### S-1: Valid Request Within Token Limit

**Given** a `ChatRequest` with 5 messages totaling 200 tokens for `gpt-4` (8192 limit)
**When** `validate_async(request)` is called
**Then** returns `Ok((200, request))` with the original request intact

### S-2: Request Exceeds Token Limit

**Given** a `ChatRequest` with content exceeding `gpt-4`'s 8192 token limit
**When** `validate_async(request)` is called
**Then** returns `Err(DomainError::TokenLimitExceeded { model, tokens, limit })`
**And** the request is consumed (not returned)

### S-3: Unknown Model Skips Validation

**Given** a `ChatRequest` with model `"unknown-model-xyz"`
**When** `validate_async(request)` is called
**Then** returns `Ok((token_count, request))` — validation is skipped for unknown models

### S-4: Blocking Task Panics (Unexpected)

**Given** a hypothetical internal panic in `count_tokens` (should never happen)
**When** `validate_async(request)` is called
**Then** returns `Err(DomainError::Internal("Token validation task panicked: ..."))`
**And** the panic does NOT propagate to the Tokio runtime

### S-5: High Concurrency Does Not Starve I/O

**Given** 100 concurrent requests with large payloads (10K+ tokens each)
**When** all call `validate_async` simultaneously
**Then** the Tokio I/O threads remain responsive (validated by tracing timestamps)
**And** all validations complete without timeout

### S-6: route_request Integration

**Given** a valid `ChatRequest` arriving at `route_request`
**When** the request is processed
**Then** `validate_async` is called instead of synchronous `validate`
**And** the returned request is used in subsequent steps (execution plan, fallback)
**And** existing tracing logs (`token_validation` target) are preserved
