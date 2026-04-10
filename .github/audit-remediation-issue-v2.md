# [AUDITORÍA] Remediación de Vulnerabilidades Críticas, Concurrencia y Deuda Arquitectónica

## 📋 Resumen Ejecutivo

El sistema presenta vulnerabilidades de nivel 1 que comprometen la seguridad (credenciales expuestas) y la estabilidad del runtime asíncrono (deadlocks). Además, existen violaciones severas al patrón Clean Architecture que deben ser revertidas para mantener la mantenibilidad del proyecto.

**Datos del audit (GitNexus index):** 4293 nodes | 9259 edges | 146 clusters | 300 flows  
**Skills generados:** 16 (Tests, Execution_plan, Services, Entities, Persistence, Auth, Cli, Handlers, Secure_storage, Provider, Gateway, Quality, Router, Config, Commands, Responses)  
**Test coverage:** 80.35% (492 tests)  
**Clippy status:** 10 errores activos con `-D warnings`

---

## 🔴 1. Vulnerabilidades Críticas (Prioridad Inmediata)

### 1.1 [Seguridad] Credencial Hardcodeada en Production Code

**Archivo:** `src/app/router/llm_router.rs:384-385`  
**Severidad:** 🔴 CRITICAL

```rust
let _account_id = api_key
    .split('_')
    .next()
    .unwrap_or("ex5FpSyn1K5lkyZK6swxSyhpf8DO82BGy");  // ← HARDCODED
format!("{}/ex5FpSyn1K5lkyZK6swxSyhpf8DO82BGy/gateway/ai", provider_config.base_url)  // ← HARDCODED
```

Un **Cloudflare AI Gateway account ID** está inyectado directamente en el código fuente como fallback cuando el parsing de la API key falla. Cualquier persona con acceso al repositorio puede usar esta credencial.

**GitNexus Impact Analysis:**
- `forward_to_provider` tiene **2 upstream callers** (`execute_with_fallback` → `route_request`)
- Afecta **12 execution flows** en el proceso `route_request`
- Módulos impactados: `Execution_plan` (direct), `Entities` (indirect)
- **Risk level:** LOW para el blast radius, pero **CRITICAL** para security

**Acción requerida:**
- Extraer el account ID a `provider_config.custom_attributes` o variable de entorno (`CLOUDFLARE_ACCOUNT_ID`)
- Documentar el formato esperado de la API key de Cloudflare
- Rotar la credencial expuesta inmediatamente tras la corrección
- Agregar regla de pre-commit o CI scan para detectar hardcoded secrets (`git-secrets`, `gitleaks`)

---

### 1.2 [Concurrencia] Bloqueo del Runtime de Tokio vía `block_on`

**Archivo:** `src/app/services/execution_plan/cascading.rs:364`  
**Severidad:** 🔴 CRITICAL

```rust
// execute() es fn sync, pero necesita llamar código async
let quality_score = futures::executor::block_on(
    self.quality_gate.evaluate_quality(&tier_account, response_text, &tier_health_snapshot)
);
```

`futures::executor::block_on` dentro de una cadena de ejecución que puede ser invocada desde un runtime de Tokio causará **deadlocks bajo carga concurrente**. El problema: `block_on` crea un nested executor que compite con el runtime principal por los worker threads del tokio threadpool.

**GitNexus Impact Analysis:**
- `CascadingExecutionPlan` tiene **0 upstream callers** directos (risk: LOW para blast radius)
- Esto significa que la refactorización a `async fn` es **aislada** — no rompe otros módulos
- El cambio solo afectará los tests y el punto de invocación actual

**Acción requerida:**
- Cambiar la firma de `CascadingExecutionPlan::execute` de `fn` a `async fn`
- Reemplazar `block_on(...)` por `.await`
- Actualizar el trait `ExecutionPlan` si es necesario (marcar `execute` como async)
- Actualizar todos los callers de `execute()` para usar `.await`

---

## 🟠 2. Defectos Arquitectónicos y de Robustez

### 2.1 [Clean Architecture] Inversión de Dependencias en la Capa de Dominio

**Archivos:** `src/domain/errors/mod.rs:119-123`, `src/domain/entities/openai_types.rs`  
**Severidad:** 🟠 HIGH

La capa de Dominio — que debe ser **pura** — tiene las siguientes violaciones:

**Violación A: Dependency Inversion**
```rust
// src/domain/errors/mod.rs:119-123
impl From<crate::error::Error> for DomainError {
    fn from(err: crate::error::Error) -> Self {
        DomainError::DomainError(format!("Error: {err}"))
    }
}
```

`DomainError` (capa de dominio) depende de `crate::error::Error` (capa de aplicación). Esto **invierte la dirección de dependencia** de Clean Architecture. El flujo correcto es: `Application → Domain`, nunca `Domain → Application`.

**Violación B: Framework Imports en Dominio**
- `src/domain/errors/mod.rs` importa `reqwest::Error` y `keyring::Error`
- `src/domain/entities/openai_types.rs` importa `axum::http::StatusCode` y `axum::response::{IntoResponse, Response}` — **esto es un concern de presentación**

**GitNexus Impact Analysis:**
- `DomainError` tiene **0 upstream callers** directos (risk: LOW para blast radius)
- Esto significa que la purga de imports es **segura** — no rompe dependencias externas

**Acción requerida:**
- Eliminar `From<crate::error::Error> for DomainError`
- Crear errores de dominio puros (sin dependencias de frameworks) para `reqwest`, `keyring`
- Mover `IntoResponse` implementations de `openai_types.rs` a la capa de Presentation
- Mover definición de rutas de `app/health.rs` a `presentation/routes.rs`

---

### 2.2 [Concurrencia] Mutex Síncrono en Contexto Asíncrono

**Archivo:** `src/infrastructure/secure_storage/mod.rs:78`  
**Severidad:** 🟠 HIGH

```rust
pub struct InsecureStorage {
    store: std::sync::Mutex<std::collections::HashMap<String, String>>,  // ← std::sync::Mutex en async
}
```

`InsecureStorage` usa `std::sync::Mutex` dentro del trait `SecureStorage` que es `Send + Sync`. Aunque actualmente no mantiene el lock cruzando `.await` points, llamar `.lock().unwrap()` en código async es un anti-pattern que dificulta el razonamiento sobre concurrencia y puede volverse peligroso si el código evoluciona.

**GitNexus Impact Analysis:**
- `InsecureStorage` tiene **0 upstream callers** (risk: LOW para blast radius)
- Es un fallback dev-only — usado solo cuando keyring y encrypted storage fallan
- La refactorización es **aislada** — no afecta otros módulos

**Acción requerida:**
- Reemplazar `std::sync::Mutex` por `tokio::sync::Mutex`
- O bien: restringir el scope del lock explícitamente con bloques `{ }` para garantizar que nunca cruce `.await`
- Agregar comentario documentando la decisión de diseño

---

### 2.3 [Trazabilidad] Pérdida de Stack Trace en Error Conversions

**Archivo:** `src/domain/errors/mod.rs:93-117`  
**Severidad:** 🟠 HIGH

Todas las implementaciones `From` para `DomainError` usan `.to_string()` y pierden el error original:

```rust
impl From<reqwest::Error> for DomainError {
    fn from(err: reqwest::Error) -> Self {
        DomainError::DomainError(format!("Request error: {err}"))  // ← Source lost
    }
}

impl From<std::io::Error> for DomainError {
    fn from(err: std::io::Error) -> Self {
        DomainError::Io(format!("IO error: {err}"))  // ← Source lost, no #[source]
    }
}
```

Sin `#[source]`, las herramientas de debugging no pueden reconstruir la cadena de errores. Esto imposibilita identificar la causa raíz de fallos en producción.

**Acción requerida:**
- Reestructurar variantes de error para contener el source typed:
  ```rust
  #[derive(Debug, Error)]
  pub enum DomainError {
      #[error("Request error")]
      Request(#[from] reqwest::Error),  // #[from] auto-implements From + preserves source
      
      #[error("IO error")]
      Io(#[from] std::io::Error),
      // ...
  }
  ```
- Si se prefiere mantener strings, usar `#[source]` manualmente:
  ```rust
  #[error("IO error: {message}")]
  Io { message: String, #[source] source: std::io::Error },
  ```

---

## 🟡 3. Rendimiento y Mantenibilidad

### 3.1 [Hot Path] Alocaciones Innecesarias por Request

**Archivo:** `src/app/router/llm_router.rs:385-413`  
**Severidad:** 🟡 MEDIUM

En el hot path de forwarding (`forward_to_provider`), cada request ejecuta:

```rust
// 4x format!() allocations per request
let url = format!("{}/chat/completions", base_url);
let auth_header = format!("Bearer {}", api_key);

// Runtime JSON tree construction via macro
let body = serde_json::json!({
    "model": request.model,
    "messages": request.messages.iter().map(|m| serde_json::json!({
        "role": m.role,
        "content": m.content
    })).collect::<Vec<_>>(),
    "temperature": request.temperature.unwrap_or(0.7),
    "max_tokens": request.max_tokens.unwrap_or(1024),
    "stream": request.stream.unwrap_or(false)
});
```

**Acción requerida:**
- Crear struct tipado `OptimizedChatBody` con derives de `Serialize`:
  ```rust
  #[derive(Serialize)]
  struct ChatRequestBody<'a> {
      model: &'a str,
      messages: &'a [Message],
      #[serde(skip_serializing_if = "Option::is_none")]
      temperature: Option<f64>,
      #[serde(skip_serializing_if = "Option::is_none")]
      max_tokens: Option<u32>,
      #[serde(skip_serializing_if = "Option::is_none")]
      stream: Option<bool>,
  }
  ```
- Usar `serde_json::to_value(&body)` en lugar de `serde_json::json!()` para zero-cost serialization
- Pre-construir URLs base con `reqwest::Url::parse` una vez, no por request

---

### 3.2 [CI/CD] Incompatibilidad MSRV

**Archivo:** `Cargo.toml` (`rust-version = "1.75"`) vs `src/**/model_context_limits.rs:6`  
**Severidad:** 🟡 MEDIUM

`Cargo.toml` declara MSRV 1.75, pero el código usa `std::sync::LazyLock` que requiere **Rust 1.80**. Esto genera errores `incompatible-msrv` en Clippy.

**Acción requerida:**
- Opción A (recomendada): Actualizar `rust-version = "1.80"` en `Cargo.toml`
- Opción B: Reemplazar `std::sync::LazyLock` por `once_cell::sync::Lazy` (dependencia adicional)

Dado que el builder usa `rust:1.93-slim`, la Opción A es la más simple.

---

### 3.3 [Linter] 10 Errores de Clippy Bloqueando CI

**Severidad:** 🟡 MEDIUM

CI falla con `-D warnings`. Errores activos:

| Error | Archivo | Fix |
|-------|---------|-----|
| `field-reassign-with-default` | `planner.rs:95` | Usar struct literal directo en vez de `Default::default()` + reasignación |
| `unnecessary-cast` | `planner.rs:109` | Eliminar cast `u32 as u32` (no-op) |
| `new-without-default` | `evaluator.rs:108` | Agregar `impl Default for HeuristicQualityEvaluator` |
| `incompatible-msrv` × 3 | `model_context_limits.rs:6` | Ver §3.2 |
| `double-ended-iterator-last` | `token_validator.rs:71` | Usar `.next_back()` en vez de `.last()` |
| +4 más | Varios | Ejecutar `cargo clippy` para listado completo |

**Acción requerida:**
- Ejecutar `cargo clippy --workspace --all-features --all-targets` para ver los 10 errores
- Aplicar fixes automáticos con `cargo clippy --fix`
- Resolver manualmente los que requieran cambios de lógica

---

## 🛠️ Plan de Acción (Checklist)

### Fase 1 — Seguridad y Estabilidad (Hacer YA)
- [ ] Eliminar credencial hardcoded de Cloudflare en `llm_router.rs:384-385`
- [ ] Rotar el account ID `ex5FpSyn1K5lkyZK6swxSyhpf8DO82BGy` en Cloudflare
- [ ] Refactorizar `CascadingExecutionPlan::execute` a `async fn` y reemplazar `block_on` por `.await`
- [ ] Agregar scan de secrets al CI (`gitleaks` o `git-secrets`)

### Fase 2 — Arquitectura (Siguiente sprint)
- [ ] Purgar imports de `axum`, `reqwest`, `keyring` de la capa de Dominio
- [ ] Eliminar `From<crate::error::Error> for DomainError` (inversión de dependencia)
- [ ] Mover `IntoResponse` impls de `domain/entities/openai_types.rs` a Presentation
- [ ] Mover rutas de `app/health.rs` a `presentation/routes.rs`
- [ ] Reemplazar `std::sync::Mutex` por `tokio::sync::Mutex` en `InsecureStorage`
- [ ] Agregar `#[source]` o `#[from]` a todas las variantes de `DomainError`

### Fase 3 — Rendimiento y Linter (Tech debt)
- [ ] Crear `ChatRequestBody` struct tipado y eliminar `serde_json::json!()` del hot path
- [ ] Actualizar MSRV a 1.80 en `Cargo.toml`
- [ ] Resolver 10 errores de Clippy
- [ ] Agregar `proptest` tests para lógica de routing (dep está en dev-dependencies sin uso)
- [ ] Agregar `insta` snapshot tests para respuestas de error (dep está en dev-dependencies sin uso)

---

## 📎 Referencias

- Audit generado por análisis estático + GitNexus (4293 nodes, 9259 edges, 146 clusters, 300 flows)
- 16 skills generados: Tests, Execution_plan, Services, Entities, Persistence, Auth, Cli, Handlers, Secure_storage, Provider, Gateway, Quality, Router, Config, Commands, Responses
- Clean Architecture: [The Clean Code Blog](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- Rust async best practices: [Tokio docs](https://tokio.rs/tokio/topics/async_best_practices)
- MSRV tracking: [RFC 2495](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)

---

*Auditoría ejecutada el 2026-04-09. Próxima revisión: ejecutar `cargo audit` y `cargo deny check` semanalmente (ya configurado en CI).*
