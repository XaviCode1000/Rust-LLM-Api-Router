# Design: Models Endpoint Implementation

## Context
Issue #10 - Future Improvements
Current state: `/v1/models` endpoint returns empty list `{"data": []}`

## Architecture Analysis

### Current Flow
```
Router (routes.rs) 
  → Controller (models_controller.rs) 
    → Service (LlmGateway::list_models())
      → Provider Trait (LlmProvider::get_models())
        → HTTP Call (reqwest)
```

### Key Components

1. **Entity**: `src/domain/entities/model.rs`
   - Already defined with `id`, `object`, `created`, `owned_by`

2. **Service Interface**: `src/application/services/llm_gateway.rs`
   - Method exists: `pub async fn list_models(&self) -> Result<Vec<Model>>`
   - Currently returns `Ok(vec![])` stub

3. **Provider Trait**: `src/infrastructure/providers/traits.rs` (or similar)
   - Need to verify if `get_models()` is defined in trait
   - Each provider (OpenAI, Groq, etc.) needs implementation

4. **Controller**: `src/interfaces/controllers/models_controller.rs`
   - Exists and calls gateway correctly

## Implementation Plan

### Step 1: Define/Verify Provider Trait Method
Add to provider trait:
```rust
async fn get_models(&self, api_key: &str) -> Result<Vec<Model>>;
```

### Step 2: Implement for OpenAI Provider
- Endpoint: `GET https://api.openai.com/v1/models`
- Headers: `Authorization: Bearer {api_key}`
- Parse response to `Vec<Model>`

### Step 3: Implement for Other Providers
- Groq: `https://api.groq.com/openai/v1/models`
- Others as needed (can return empty or subset)

### Step 4: Update LlmGateway
- Iterate through configured providers
- Aggregate models from active providers
- Add provider prefix to model IDs if needed

### Step 5: Error Handling
- If provider fails, log warning but continue with others
- Return combined list even if some providers fail

## Files to Modify

1. `src/domain/entities/model.rs` - Verify structure
2. `src/infrastructure/providers/traits.rs` - Add trait method
3. `src/infrastructure/providers/openai_provider.rs` - Implement
4. `src/infrastructure/providers/groq_provider.rs` - Implement
5. `src/application/services/llm_gateway.rs` - Remove stub, add real logic
6. `src/config/provider_config.rs` - May need provider capabilities flag

## Testing Strategy

1. Unit tests for Model entity parsing
2. Integration test with mock HTTP responses
3. Manual test: `curl http://localhost:8080/v1/models`

## Constraints

- Hardware: Haswell/HDD/8GB - Keep builds incremental
- Must maintain backward compatibility
- No breaking changes to existing routes

## Next Phase

After design approval, proceed to **Implement** phase with @rust-api agent.
