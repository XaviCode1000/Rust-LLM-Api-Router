# Proposal: AUDIT-2026-security-robustness

## Status
`in_progress`

---

## Executive Summary

Este change aborda problemas de robustness y seguridad identificados durante el audit del codebase. El foco principal es arreglar el test `test_anthropic_chat_function_success` que falla por mismatch de header version, agregar instrumentación tracing en los entry points de providers, y evaluar la migración de `derivative` a `educe` por RUSTSEC-2024-0388.

---

## Artifacts

### 1. Intent

**Qué queremos lograr:**
- Corregir el test que falla en el CI (bloquea merge)
- Agregar observabilidad en los entry points de providers para debugging en producción
- Evaluar riesgo de seguridad del advisory sobre `derivative`
- Optimizar serialización de mensajes OpenAI con `#[serde(borrow)]`

**Por qué importa:**
- El test `test_anthropic_chat_function_success` falla con header mismatch: mock espera `"2024-06-20"` pero implementation envía `"2023-06-01"` (anthropic.rs:125) — esto bloquea merges
- Sin `#[tracing::instrument]` en providers, no hay visibility de errores en producción
- RUSTSEC-2024-0388 documenta defecto de seguridad en `derivative` < 0.2.12
- `#[serde(borrow)]` evita allocations innecesarias en hot path

---

### 2. Scope

**INCLUDE (Prioridad Alta):**

| Item | Detalle | Locación |
|------|---------|----------|
| Fix test `test_anthropic_chat_function_success` | Actualizar mock para esperar `"2023-06-01"` (la versión correcta) | `tests/provider_chat_tests.rs:371` |
| Agregar `#[tracing::instrument]` | En método `chat()` de cada provider | `openai.rs`, `groq.rs`, `anthropic.rs` |
| Evaluar `derivative` → `educe` | Buscar uso de `derivative` y verificar versión | `Cargo.toml` y archivos Rust |

**EXCLUDE (Monitoreo):**

| Item | Razón |
|------|-------|
| `unwrap()/expect()` en src/ | 188 usages, mayoría en inicialización — riesgo bajo, contexto aceptable |
| `todo!()` en `planner.rs` | 2 usages son falsos positivos en código de test (MockAccountRepository) |
| Advisory rustls-webpki | Transitivas — correctamente ignoradas según audit |

**NICE TO HAVE:**

| Item | Beneficio |
|------|----------|
| `#[serde(borrow)]` en message structs | Evita clones en hot path de serialización |

---

### 3. Approach

#### Fase 1: Fix Test (Bloqueante)

**Objetivo:** Corregir el test que blocked el CI.

**Steps:**
1. Leer `tests/provider_chat_tests.rs:365-400` para ver el test completo
2. El mock actual espera `header("anthropic-version", "2024-06-20")` (line 371)
3. Cambiar el mock a esperar `"2023-06-01"` - esta ES la versión correcta que usa la API de Anthropic
4. NO modificar la implementación - el código en `anthropic.rs:125` YA usa la versión correcta

**Verificación:**
- `cargo nextest run test_anthropic_chat_function_success` debe pasar

**Evidencia:**
```
# Grep results confirman:
tests/provider_chat_tests.rs:371: .and(header("anthropic-version", "2024-06-20"))
src/infrastructure/provider/anthropic.rs:125: .header("anthropic-version", "2023-06-01")
```

El mock está MAL - debe actualizarse para coincidir con lo que Anthropic API acepta.

---

#### Fase 2: Instrumentación Tracing

**Objetivo:** Agregar observabilidad en entry points de providers.

**Providers a instrumentar:**
| Provider | Archivo | Método |
|----------|---------|--------|
| OpenAiProvider | `src/infrastructure/provider/openai.rs` | `chat()` entry |
| GroqProvider | `src/infrastructure/provider/groq.rs` | `chat()` entry |
| AnthropicProvider | `src/infrastructure/provider/anthropic.rs:110` | `chat()` entry |

**Approach:**
```rust
#[tracing::instrument(skip(self, request), fields(model = %request.model))]
pub async fn chat(&self, request: &OpenAIChatRequest) -> Result<OpenAIChatResponse> {
    // ... existing code
}
```

**Verificación:**
- `cargo check` sin errores
- Los logs muestran el modelname en cada llamada

---

#### Fase 3: Evaluar derivative → educe

**Steps:**
1. Buscar `derivative` en `Cargo.toml` — obtener version exacta
2. Grep `use derivative` en archivos Rust
3. Si `derivative < 0.2.12`, crear tasks para migración a `educe`
4. Si `>= 0.2.12`, documentar que está OK y exclude del scope

**RUSTSEC-2024-0388:**
- Advisory披露 `derivative` < 0.2.12 有未定义行为
- La migration a `educe` es el workaround recomendado

---

#### Fase 4: Optimización serde(borrow) [Nice to Have]

**Structs candidates:**
| Struct | Campo | Tipo actual | Con borrow |
|--------|-------|-------------|------------|
| `OpenAIMessage` | `role` | String | `Cow<str>` o `&str` |
| `OpenAIMessage` | `content` | String | `Cow<str>` o `&str` |

**Approach:**
- Agregar `#[serde(borrow)]` en los campos de tipo String
- Esto evita cloning cuando el input viene de un borrowed source

**Nota:** Esto cambia la API pública — evaluar si breaking change es acceptable. Si no, exclude.

---

## Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|-----------|-----------|
| Test fix no pasa | HIGH | LOW | Verificar mock expects versión correcta `"2023-06-01"` |
| Migration derivative rompe build | MEDIUM | LOW | Si hay breakage, exclude y documentar |
| Breaking change con serde(borrow) | MEDIUM | MEDIUM | Only apply si no hay breaking changes en API |

---

## Next Recommended
`sdd-spec`

La spec detallará cada scenario con Gherkin given/when/then para:
- Test fix con assertions específicas
- Instrumentación con fields esperados
- Evaluación de derivative con criterios de riesgo

---

## Skill Resolution
`injected`

---

## Related Artifacts

- **Exploration**: `.claude/skills/generated/explorations/AUDIT-2026-security-robustness.md`
- **Issue**: `.github/ISSUES/AUDIT-2026-security-robustness.md`
- **GitNexus context**: Repo indexed con 4336 nodes, 9350 edges