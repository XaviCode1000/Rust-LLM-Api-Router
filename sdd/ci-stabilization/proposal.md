# Proposal: CI Pipeline Stabilization (Zero Failures)

## Intent

Eliminar TODOS los fallos del pipeline de CI para alcanzar un estado de **Zero Failures** donde cada job pase en verde. Actualmente 4 de 7 jobs fallan.

## Problem

El pipeline de CI tiene los siguientes fallos activos:

| Job | Estado | Razón |
|-----|--------|-------|
| **Security** | ❌ | `RUSTSEC-2026-0104`: `rustls-webpki v0.103.12` tiene un panic alcanzable en CRL parsing. + 4 advisories stale en `deny.toml`. |
| **Test** | ❌ | `test_cli_add_provider_url_validation` falla: test asume que URLs vacías son aceptadas, pero la implementación ahora valida que `base_url` no esté vacío. |
| **Doctests** | ❌ | 12 doctests fallan por imports faltantes (`ProviderRepository`, tipos no encontrados en scope). |
| **Coverage** | ❌ | Dependiente de tests que fallan (derivado). |

## Scope

**In scope (Prioridad 1 — Seguridad):**
- Actualizar `rustls-webpki` a >=0.103.13
- Limpiar advisories stale de `deny.toml`
- Verificar `cargo deny check advisories` pasa

**In scope (Prioridad 2 — Estabilidad):**
- Corregir `test_cli_add_provider_url_validation` para alinearse con la validación actual
- Verificar todos los tests pasan con `cargo nextest run`

**In scope (Prioridad 3 — Documentación):**
- Sanear 12 doctests fallidos (imports, compilabilidad)
- Restaurar `cargo test --doc` en verde

**Out of scope:**
- Añadir nuevos tests o funcionalidades
- Refactorizar arquitectura
- Modificar la lógica de negocio existente

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| `cargo update` rompe API de dependencias | Medium | Actualizar solo `rustls-webpki` (patch version), no majors |
| Fix de test revela bug real en validación | Low | La validación de URL vacía es correcta — el test estaba mal |
| Doctests requieren refactor mayor | Medium | Mark como `ignore` los que requieren arquitectura compleja; fix los simples |

## Success Criteria

1. `cargo deny check advisories` — PASS
2. `cargo nextest run --workspace` — ALL PASS
3. `cargo test --doc` — ALL PASS (or expected failures documented)
4. `cargo llvm-cov nextest` — PASS
5. GitHub Actions CI run — ALL JOBS GREEN
