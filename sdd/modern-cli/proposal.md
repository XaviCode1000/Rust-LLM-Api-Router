# Change Proposal: Modern Interactive CLI Experience (Issue #19)

## Intent

Rediseñar completamente la experiencia de usuario de la CLI del LLM API Router, transformándola de una herramienta funcional pero básica en una CLI moderna, interactiva y profesional que inspire confianza y sea intuitiva de usar.

## Problem

La CLI actual es **funcional pero rudimentaria**:

- **Sin colores**: Todo es texto plano, sin distinción visual entre éxito/error/warning
- **Sin tablas reales**: Output formateado con `println!` que se rompe con datos largos
- **Sin confirmaciones**: `provider remove` y `account remove` eliminan sin preguntar
- **Sin feedback visual**: Operaciones de red (validate) sin spinner ni progress bar
- **Mensajes de error básicos**: Solo `eprintln!` sin estructura ni contexto
- **Sin detección de TTY**: No diferencia entre terminal interactiva y piping
- **Sin ejemplos en help**: `--help` muestra opciones básicas sin ejemplos de uso
- **Input básico**: `std::io::stdin().read_line()` sin masking para API keys

**Comparación con CLIs modernas** (turbo, gh, starship, cargo-binstall):
- Todas usan colores semánticos (verde=éxito, rojo=error, amarillo=warning)
- Todas tienen tablas formateadas con bordes y alineación
- Todas confirman acciones destructivas
- Todas muestran spinners durante operaciones de red
- Todas tienen `--help` rico con ejemplos

## Scope

### Incluido
1. **Colores semánticos** — `owo_colors` (zero-dep, minimal) para output con significado
2. **Tablas profesionales** — `comfy-table` para listados de providers/accounts
3. **Prompts interactivos** — `inquire` para confirmaciones, selección, input con masking
4. **Spinners** — `indicatif` para operaciones de red (validate, login)
5. **Confirmaciones destructivas** — `Are you sure?` antes de remove
6. **Error formatting** — Mensajes de error estructurados con contexto y sugerencias
7. **Detección de TTY** — Graceful degradation cuando no hay terminal
8. **Rich help** — Ejemplos de uso en cada subcomando
9. **Input masking** — API keys ocultas al tipear
10. **Barra de progreso** — Para operaciones batch (validate all accounts)

### NO Incluido
- TUI completa con ratatui (fuera de scope, sería Phase 2)
- Modo interactivo tipo REPL/navegación por menús
- Temas personalizables
- Logging a archivo desde CLI

## Approach

### Arquitectura de la CLI Rediseñada

```
src/presentation/cli/
├── mod.rs              # Cli struct + handle_command()
├── output.rs           # NEW: Colored output utilities (success, error, warning, info)
├── spinner.rs          # NEW: Spinner wrapper for async operations
├── table.rs            # NEW: Table formatting utilities
├── prompt.rs           # NEW: Interactive prompt wrappers (inquire)
├── tty.rs              # NEW: TTY detection + graceful degradation
├── input.rs            # UPDATED: Replace stdin with inquire text/masked input
└── commands/
    ├── mod.rs          # Updated imports
    ├── provider.rs     # REDESIGNED: Colored output, tables, confirmations
    ├── account.rs      # REDESIGNED: Colored output, tables, confirmations
    ├── auth.rs         # Updated: Colored output
    ├── login.rs        # REDESIGNED: Spinner during OAuth, colored flow
    ├── logout.rs       # Updated: Colored output
    └── completions.rs  # Unchanged
```

### Dependencias Nuevas

| Crate | Versión | Propósito | Tamaño |
|-------|---------|-----------|--------|
| `owo-colors` | 4.x | Colores zero-dep | ~50KB |
| `comfy-table` | 7.x | Tablas profesionales | ~200KB |
| `inquire` | 0.7.x | Prompts interactivos | ~300KB |
| `indicatif` | 0.17.x | Spinners/progress bars | ~150KB |
| `is-terminal` | 0.4.x | Detección de TTY | ~10KB |

**Total adicional**: ~710KB (mínimo para el impacto UX)

### Decisiones de Diseño Clave

#### 1. Graceful Degradation (TTY Detection)
```rust
// If not a TTY (piped/scripted), fall back to plain output
if !is_tty() {
    // Plain text, no colors, no prompts
    println!("{}", data);
} else {
    // Full interactive experience
    styled_output(data);
}
```

#### 2. Output Module Pattern
```rust
pub fn success(msg: &str) { println!("{}", msg.green().bold()); }
pub fn error(msg: &str) { eprintln!("{}", msg.red().bold()); }
pub fn warning(msg: &str) { eprintln!("{}", msg.yellow()); }
pub fn info(msg: &str) { println!("{}", msg.blue()); }
pub fn dim(msg: &str) { println!("{}", msg.dimmed()); }
```

#### 3. Confirmation Pattern
```rust
// Before destructive action
if !confirm("Are you sure you want to remove provider 'groq'?")? {
    return Ok(()); // User cancelled
}
```

#### 4. Spinner Pattern
```rust
let spinner = start_spinner("Validating provider...");
let result = validate_provider(id).await;
spinner.stop();
```

### Estrategia de Migración

1. **Crear módulos de utilidad** (`output.rs`, `spinner.rs`, `table.rs`, `prompt.rs`, `tty.rs`)
2. **Refactorizar command modules** para usar los nuevos utilities
3. **Mantener compatibilidad** — flags `--no-color`, `--quiet` para scripting
4. **Tests** — Todos los tests existentes deben pasar

### Colores y Estilo Visual

```
✓ Provider 'groq' added successfully
✗ Failed to validate account: invalid API key
⚠ Warning: No API key provided, using interactive mode
ℹ Info: 5 providers configured

┌──────────────────┬──────────────────┬──────────────┬──────────────┐
│ ID               │ Name             │ Base URL     │ Status       │
├──────────────────┼──────────────────┼──────────────┼──────────────┤
│ groq             │ Groq             │ api.groq...  │ ● Enabled    │
│ openai           │ OpenAI           │ api.openai.. │ ● Enabled    │
│ cerebras         │ Cerebras         │ api.cereb... │ ○ Disabled   │
└──────────────────┴──────────────────┴──────────────┴──────────────┘
```

## Impact

### Files Modified
| File | Change |
|------|--------|
| `Cargo.toml` | +5 dependencies |
| `src/presentation/cli/mod.rs` | Updated imports, TTY detection |
| `src/presentation/cli/output.rs` | **NEW** — Colored output utilities |
| `src/presentation/cli/spinner.rs` | **NEW** — Spinner wrapper |
| `src/presentation/cli/table.rs` | **NEW** — Table formatting |
| `src/presentation/cli/prompt.rs` | **NEW** — Interactive prompts |
| `src/presentation/cli/tty.rs` | **NEW** — TTY detection |
| `src/presentation/cli/input.rs` | Updated to use inquire |
| `src/presentation/cli/commands/provider.rs` | Redesigned with new UX |
| `src/presentation/cli/commands/account.rs` | Redesigned with new UX |
| `src/presentation/cli/commands/login.rs` | Spinner + colored flow |
| `src/presentation/cli/commands/logout.rs` | Colored output |
| `src/presentation/cli/commands/auth.rs` | Colored output |
| `docs/cli.md` | Updated documentation |

### Risks
- **inquire + async**: `inquire` usa stdin blocking. Debe usarse fuera de tokio spawn o con `spawn_blocking`.
- **Terminal detection**: Algunos entornos CI/CD pueden no tener TTY — graceful degradation obligatorio.
- **Dependency size**: ~710KB adicionales — aceptable para el impacto UX.

## Alternatives Considered

| Alternative | Why Rejected |
|-------------|-------------|
| **Solo `colored`** | Demasiado mínimo, no resuelve el problema de UX fundamental |
| **`cliclack`** | Menos maduro que `inquire` (331 vs 2543 stars), menos features |
| **`dialoguer`** | API más verbosa, menos mantenida que `inquire` |
| **`ratatui` TUI completa** | Overkill para Phase 1, complejidad significativa |
| **`tabled` vs `comfy-table`** | `comfy-table` tiene mejor API y es más ligero |
| **Sin cambios** | UX actual es inadecuada para producción |
