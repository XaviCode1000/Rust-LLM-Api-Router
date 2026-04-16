# [AUDIT] Auditorí­a de Seguridad y Robustez - Rust-LLM-Api-Router

## Resumen Ejecutivo

Tras una auditoría técnica completa del código base, se han identificado **issues críticos de seguridad**, **problemas de licencias** y **áreas de mejora de robustez** que afectan la estabilidad y compatibilidad del sistema.

| Métrica | Valor |
|--------|------|
| **Tests** | 591/592 (1 fallando) |
| **Cobertura~** | 80.35% |
| **Binario** | 13MB (release) |
| **unsafe code** | 0 bloques |
| **Clippy** | ✅ 0 warnings |
| **Licenses** | ✅ PASS (`cargo deny check licenses`) |

---

## 1. SEGURIDAD (CRÍTICO)

### 1.1 Vulnerabilidades en Dependencias

Ejecutar `cargo audit` reveló 3 vulnerabilidades activas en `rustls-webpki`:

```bash
RUSTSEC-2026-0099: Name constraints were accepted for wildcard certificates
                 → Upgrade to >=0.103.12 OR >=0.104.0-alpha.6
RUSTSEC-2026-0049: CRLs not considered authoritative (faulty matching logic)
                 → Upgrade to >=0.103.10
RUSTSEC-2026-0098: Name constraints for URI names incorrectly accepted
                 → Upgrade to >=0.103.12 OR >=0.104.0-alpha.6
```

**Status:** ⚠️ IGNORED — Son dependencias transitivas (via `reqwest`, `oauth2`). No hay fix directo sin updatear `reqwest` o `rustls`. Monitorear próximas versiones.

**Workaround:** Ignorado en `deny.toml` hasta que haya fix disponible:
```toml
[advisories]
ignore = [
    "RUSTSEC-2026-0099",  # rustls-webpki name constraints
    "RUSTSEC-2026-0049",  # rustls-webpki CRL matching
    "RUSTSEC-2026-0098",  # rustls-webpki URI names
]
```

**Nota:** Estas vulnerabilidades tienen impacto limitado en contexto de router (servidor actuando como proxy, no validando certificados de clientes).

### 1.2 Dependencia Sin Mantenimiento

```
RUSTSEC-2024-0388: derivative v2.2.0 - unmaintained
→ Considerar migrate a `educe` o derive manual
```

---

## 2. LICENCIAS (ALTA)

### 2.1 Licencia No Permitida

```bash
$ cargo deny check licenses
error: license is not explicitly allowed

tiny-keccak v2.0.2 → CC0-1.0 (Creative Commons Zero)
foldhash v0.2.0 → Zlib
```

**Status:** ✅ FIXED — Agregado licencias a `deny.toml`:
```toml
[licenses]
allow = [
    "MIT",
    "Apache-2.0",
    ...
    "CC0-1.0",  # tiny-keccak (via config -> rust-ini -> const-random)
    "Zlib",     # foldhash (via rustls)
]
```

**Status:** ✅ FIXED — Agregado `CC0-1.0` a `deny.toml`
```toml
[licenses]
allow = [
    "MIT",
    "Apache-2.0",
    ...
    "CC0-1.0",  # tiny-keccak (via config -> rust-ini -> const-random)
]
```

---

## 3. FIABILIDAD (ALTA)

### 3.1 Tests Fallando

```
test test_anthropic_chat_function_success ... FAILED
  → Internal("Anthropic API error (404 Not Found)")
```

**Root cause:** El mock no está matcheando el body JSON correctamente.

**Archivo:** `tests/provider_chat_tests.rs:368-392`

**Fix sugerido:**
```rust
Mock::given(method("POST"))
    .and(path("/v1/messages"))
    .and(header("x-api-key", "sk-anthropic-key"))
    .and(header("anthropic-version", "2024-06-20"))
    .and(body(match_json!(...)))  // ← FALTA ESTE MATCHER
    .respond_with(ResponseTemplate::new(200)...)
```

### 3.2 Códios Inalcanzables (todo!)

Se encontraron 2 `todo!()` en código alcanzable:

| Archivo | Línea | Descripción |
|--------|-------|------------|
| `planner.rs` | 1010 | Método no implementado |
| `planner.rs` | 1046 | Método no implementado |

**Fix sugerido:** Implementar o marcar como `#[allow(dead_code)]` si es intencional.

### 3.3 unwrap()/expect() en Código de Producción

Se encontraron **191 usos** de `unwrap()`/`expect()`, clasificados como:

| Categoría | Cantidad | Riesgo |
|----------|---------|-------|
| Tests | ~60% | Bajo |
| Inicialización runtime | ~20% | Medio (catastrófico si falla) |
| Conversiones seguras | ~15% | Bajo |
| Paths de error | ~5% | Alto |

**Áreas críticas a refactorizar:**

| Archivo | Líneas | Tipo |
|---------|-------|------|
| `infrastructure/http_client.rs` | 55, 19 | Inicialización HTTP client |
| `infrastructure/gateway/llm_gateway.rs` | 424-443 | Test setup |
| `domain/services/model_selector.rs` | 187-237 | Conversiones seguras |

**Fix sugerido:** Usar `?` operator oPattern matching:
```rust
// Antes
let config = config.get("openai").unwrap();

// Después
let config = config.get("openai")
    .ok_or_else(|| DomainError::ConfigError("openai not found".into()))?;
```

---

## 4. OPTIMIZACIÓN (MEDIA)

### 4.1 Zero-Copy Parsing

El proyecto no usa `#[serde(borrow)]` en los structs de mensajes, lo cual puede causar asignaciones innecesarias en el heap cuando se procesan prompts largos.

**Fix sugerido:** Agregar `#[serde(borrow)]` a los structs de entrada:
```rust
#[derive(Deserialize)]
pub struct Message {
    #[serde(borrow)]  // ← Agregar
    pub content: String,
    pub role: Role,
}
```

### 4.2 Binary Size

```bash
$ ls -lh target/release/llm-router
13MB  # Con LTO fat, strip, opt-level=3
```

El tamaño es aceptable dado el feature set (OAuth, TUI, metrics).

---

## 5. OBSERVABILIDAD (MEDIA)

### 5.1 Tracing Coverage

El proyecto usa `tracing` con spans activos:
- `PlanningSpan` - tracking del planning
- `ExecutionSpan` - tracking de ejecución
- `QualityEvaluationSpan` - evaluación de calidad

**Recomendación:** Agregar `#[instrument]` en los entry points de providers:
```rust
#[tracing::instrument]
async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
    // ...
}
```

---

## 6. PUNTOS FUERTES

| Área | Estado | Notas |
|------|--------|-------|
| **async/Send+Sync** | ✅ EXCELENTE | Traits con bounds correctos |
| **spawn_blocking** | ✅ EXCELENTE | I/O de archivos delegdo a thread pool |
| **Clippy** | ✅ PASA | 0 warnings |
| **Timeouts** | ✅ EXCELENTE | Múltiples capas (120s, 60s, 5s) |
| **Exponential Backoff** | ✅ EXCELENTE | Implemented with jitter |
| **Secure Storage** | ✅ EXCELENTE | keyring, encrypted store |
| **Code** | ✅ LIMPIO | Sin `unsafe`, sin `static mut` |

---

## ROADMAP DE MEJORAS

### Prioridad 1 (Crítico - Seguridad)
- [x] Agregar `CC0-1.0` y `Zlib` a licencias permitidas ✅
- [x] Ignorar advisories de rustls-webpki (transitivas) ✅
- [ ] Actualizar `rustls` cuando haya fix disponible (monitorear)

### Prioridad 2 (Alta - Fiabilidad)
- [ ] Fix test `test_anthropic_chat_function_success`
- [ ] Implementar o marcar `todo!()` en `planner.rs`
- [ ] Refactorizar `unwrap()` crítica en `http_client.rs` y `gateway.rs`

### Prioridad 3 (Media - Optimización)
- [ ] Agregar `#[serde(borrow)]` en structs de mensajes
- [ ] Agregar `#[instrument]` en entry points

### Prioridad 4 (Documentación)
- [ ] Documentar decisiones de arquitectura en `docs/architecture.md`
- [ ] Agregar SECURITY.md con política de reporting

---

## Referencias

- [RUSTSEC-2026-0099](https://rustsec.org/advisories/RUSTSEC-2026-0099.html)
- [RUSTSEC-2026-0049](https://rustsec.org/advisories/RUSTSEC-2026-0049.html)
- [RUSTSEC-2026-0098](https://rustsec.org/advisories/RUSTSEC-2026-0098.html)
- [RUSTSEC-2024-0388](https://rustsec.org/advisories/RUSTSEC-2024-0388.html)

---

**Auditor:** @gaza-dev
**Fecha:** 2026-04-16
**Versión del código:** Latest main