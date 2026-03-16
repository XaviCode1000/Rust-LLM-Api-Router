# Domain Traits (Ports)

Abstractions that define contracts for infrastructure implementations.

## Overview

Following Clean Architecture's Ports and Adapters pattern, these traits define the interfaces (ports) that the domain uses to interact with external systems. The infrastructure layer provides concrete implementations (adapters).

## Traits

### LlmGateway

Primary port for sending chat requests to LLM providers.

```rust
use rust_llm_api_router::domain::traits::LlmGateway;
use rust_llm_api_router::domain::{ChatRequest, ChatResponse};

#[async_trait]
impl LlmGateway for MyLlmGateway {
    async fn chat(&self, request: ChatRequest, api_key: &str) -> DomainResult<ChatResponse> {
        // Implementation
    }

    async fn list_models(&self, api_key: &str) -> DomainResult<Vec<Model>> {
        // Implementation
    }
}
```

### ProviderRepository

Contract for provider persistence and retrieval.

```rust
use rust_llm_api_router::domain::traits::ProviderRepository;
use rust_llm_api_router::domain::Provider;

#[async_trait]
impl ProviderRepository for JsonProviderRepository {
    async fn save(&self, provider: Provider) -> DomainResult<Provider> { /* ... */ }
    async fn find_all(&self) -> DomainResult<Vec<Provider>> { /* ... */ }
    async fn find_by_id(&self, id: &str) -> DomainResult<Provider> { /* ... */ }
    async fn find_enabled_by_id(&self, id: &str) -> DomainResult<Provider> { /* ... */ }
    async fn delete(&self, id: &str) -> DomainResult<()> { /* ... */ }
}
```

### AccountRepository

Contract for account persistence and retrieval.

```rust
use rust_llm_api_router::domain::traits::AccountRepository;
use rust_llm_api_router::domain::Account;

#[async_trait]
impl AccountRepository for JsonAccountRepository {
    async fn save(&self, account: Account) -> DomainResult<Account> { /* ... */ }
    async fn find_all(&self) -> DomainResult<Vec<Account>> { /* ... */ }
    async fn find_by_id(&self, id: &str) -> DomainResult<Account> { /* ... */ }
    async fn find_active(&self) -> DomainResult<Vec<Account>> { /* ... */ }
    async fn find_active_by_provider(&self, provider_id: &str) -> DomainResult<Vec<Account>> { /* ... */ }
    async fn delete(&self, id: &str) -> DomainResult<()> { /* ... */ }
}
```

### LlmProvider

Legacy trait for backward compatibility. New code should use `LlmGateway`.

## DomainResult

A type alias for domain operations:

```rust
pub type DomainResult<T> = Result<T, DomainError>;
```

## Design Principles

1. **Dependency Inversion**: Domain defines traits, infrastructure implements
2. **Async Trait**: All traits use `async_trait` for async support
3. **No Implementation**: Traits only define contracts, no default methods

## Implementations

See the infrastructure layer for concrete implementations:
- [`JsonAccountRepository`](../../infrastructure/persistence/json_account_repository.rs)
- [`JsonProviderRepository`](../../infrastructure/persistence/json_provider_repository.rs)
- [`LlmGatewayImpl`](../../infrastructure/gateway/mod.rs)

## See Also

- [Domain Entities](../entities/mod.rs)
- [Infrastructure Layer](../../infrastructure/mod.rs)
- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
