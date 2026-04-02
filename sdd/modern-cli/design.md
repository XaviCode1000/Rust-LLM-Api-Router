# Technical Design: Modern Interactive CLI Experience (Issue #19)

## Architecture

### Module Structure

```
src/presentation/cli/
├── mod.rs              # Cli struct + handle_command() + TTY detection
├── output.rs           # Colored output utilities
├── spinner.rs          # Spinner wrapper for async operations
├── table.rs            # Table formatting utilities
├── prompt.rs           # Interactive prompt wrappers (inquire)
├── tty.rs              # TTY detection + NO_COLOR support
├── input.rs            # Updated: masked input via inquire
└── commands/
    ├── mod.rs          # Updated imports
    ├── provider.rs     # Redesigned with new UX
    ├── account.rs      # Redesigned with new UX
    ├── auth.rs         # Colored output
    ├── login.rs        # Spinner + colored flow
    ├── logout.rs       # Colored output + confirmation
    └── completions.rs  # Unchanged
```

### 1. TTY Detection (`tty.rs`)

```rust
use std::sync::atomic::{AtomicBool, Ordering};

static NO_COLOR: AtomicBool = AtomicBool::new(false);
static IS_TTY: AtomicBool = AtomicBool::new(false);

/// Initialize TTY detection. Call once at startup.
pub fn init() {
    IS_TTY.store(is_terminal::is_terminal(std::io::stdout()), Ordering::Relaxed);
    NO_COLOR.store(
        std::env::var_os("NO_COLOR").is_some(),
        Ordering::Relaxed,
    );
}

#[must_use]
pub fn is_tty() -> bool {
    IS_TTY.load(Ordering::Relaxed)
}

#[must_use]
pub fn should_use_color() -> bool {
    is_tty() && !NO_COLOR.load(Ordering::Relaxed)
}
```

### 2. Colored Output (`output.rs`)

```rust
use owo_colors::OwoColorize;
use crate::presentation::cli::tty::should_use_color;

pub fn success(msg: &str) {
    if should_use_color() {
        println!("{} {}", "✓".green().bold(), msg.green());
    } else {
        println!("✓ {msg}");
    }
}

pub fn error(msg: &str) {
    if should_use_color() {
        eprintln!("{} {}", "✗".red().bold(), msg.red());
    } else {
        eprintln!("✗ {msg}");
    }
}

pub fn warning(msg: &str) {
    if should_use_color() {
        eprintln!("{} {}", "⚠".yellow().bold(), msg.yellow());
    } else {
        eprintln!("⚠ {msg}");
    }
}

pub fn info(msg: &str) {
    if should_use_color() {
        println!("{} {}", "ℹ".blue().bold(), msg.blue());
    } else {
        println!("ℹ {msg}");
    }
}

pub fn dim(msg: &str) {
    if should_use_color() {
        println!("{}", msg.dimmed());
    } else {
        println!("{msg}");
    }
}

pub fn bold(msg: &str) -> String {
    if should_use_color() {
        msg.bold().to_string()
    } else {
        msg.to_string()
    }
}
```

### 3. Table Formatting (`table.rs`)

```rust
use comfy_table::{Table, Row, Cell, ContentArrangement, presets};
use crate::presentation::cli::tty::should_use_color;

pub fn provider_table(providers: &[Provider]) -> String {
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("ID"),
            Cell::new("Name"),
            Cell::new("Base URL"),
            Cell::new("Status"),
        ]);

    for p in providers {
        let status = if p.is_enabled {
            "● Enabled"
        } else {
            "○ Disabled"
        };
        table.add_row(vec![
            Cell::new(&p.id),
            Cell::new(&p.name),
            Cell::new(&truncate(&p.base_url, 30)),
            Cell::new(status),
        ]);
    }

    table.to_string()
}

pub fn account_table(accounts: &[Account]) -> String {
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("ID"),
            Cell::new("Provider"),
            Cell::new("Priority"),
            Cell::new("Status"),
            Cell::new("API Key"),
        ]);

    for a in accounts {
        let status = if a.is_active { "✓ Active" } else { "✗ Inactive" };
        let api_key = mask_api_key(&a.api_key);
        table.add_row(vec![
            Cell::new(&a.id),
            Cell::new(&a.provider_id),
            Cell::new(a.priority.to_string()),
            Cell::new(status),
            Cell::new(api_key),
        ]);
    }

    table.to_string()
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}

fn mask_api_key(key: &str) -> String {
    if key.len() <= 8 {
        "****".to_string()
    } else {
        format!("{}...{}", &key[..4], &key[key.len() - 4..])
    }
}
```

### 4. Interactive Prompts (`prompt.rs`)

```rust
use inquire::{Confirm, Text, Select};
use crate::presentation::cli::tty::is_tty;
use crate::Result;

/// Ask for confirmation. Returns false if not TTY (non-interactive).
pub fn confirm(message: &str) -> Result<bool> {
    if !is_tty() {
        return Ok(true); // Auto-confirm in non-interactive mode
    }
    Confirm::new(message)
        .with_default(false)
        .prompt()
        .map_err(|e| crate::Error::Internal(format!("Prompt error: {e}")))
}

/// Prompt for text input (e.g., API key with masking).
pub fn prompt_text(message: &str) -> Result<String> {
    if !is_tty() {
        return Err(crate::Error::Internal(
            "Interactive input requires a terminal".to_string(),
        ));
    }
    Text::new(message)
        .prompt()
        .map_err(|e| crate::Error::Internal(format!("Prompt error: {e}")))
}

/// Prompt for masked text input (e.g., API key).
pub fn prompt_secret(message: &str) -> Result<String> {
    if !is_tty() {
        return Err(crate::Error::Internal(
            "Interactive input requires a terminal".to_string(),
        ));
    }
    Text::new(message)
        .with_display_mode(inquire::DisplayMode::Masked)
        .prompt()
        .map_err(|e| crate::Error::Internal(format!("Prompt error: {e}")))
}

/// Select from a list of options.
pub fn select<'a>(message: &str, options: Vec<&'a str>) -> Result<&'a str> {
    if !is_tty() {
        return Err(crate::Error::Internal(
            "Interactive input requires a terminal".to_string(),
        ));
    }
    Select::new(message, options)
        .prompt()
        .map_err(|e| crate::Error::Internal(format!("Prompt error: {e}")))
}
```

### 5. Spinner (`spinner.rs`)

```rust
use indicatif::{ProgressBar, ProgressStyle};
use crate::presentation::cli::tty::is_tty;

pub struct CliSpinner {
    pb: Option<ProgressBar>,
}

impl CliSpinner {
    pub fn new(message: &str) -> Self {
        if !is_tty() {
            return Self { pb: None };
        }
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner} {msg}")
                .unwrap(),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(80));
        Self { pb: Some(pb) }
    }

    pub fn finish_with_message(&self, message: &str) {
        if let Some(ref pb) = self.pb {
            pb.finish_with_message(message);
        }
    }

    pub fn abandon(&self) {
        if let Some(ref pb) = self.pb {
            pb.abandon();
        }
    }
}

impl Drop for CliSpinner {
    fn drop(&mut self) {
        if let Some(ref pb) = self.pb {
            pb.finish_and_clear();
        }
    }
}
```

### 6. Updated `input.rs`

```rust
use crate::presentation::cli::prompt::prompt_secret;
use crate::Result;

/// Read API key interactively with masked input.
pub fn read_api_key_interactive() -> Result<String> {
    prompt_secret("Enter API key:")
}
```

### 7. Command Module Pattern (provider.rs example)

```rust
// Before: plain println
println!("Provider '{}' added successfully", args.id);

// After: colored output
crate::presentation::cli::output::success(&format!("Provider '{}' added successfully", args.id));

// Before: no confirmation
repo.remove(&args.id).await?;

// After: confirmation
if crate::presentation::cli::prompt::confirm(
    &format!("Are you sure you want to remove provider '{}'? This will also remove all associated accounts.", args.id)
)? {
    repo.remove(&args.id).await?;
    output::success(&format!("Provider '{}' removed", args.id));
} else {
    output::info(&format!("Cancelled. Provider '{}' was not removed.", args.id));
}

// Before: plain table
println!("{:<20} {:<30} {:<40} {}", "ID", "Name", "Base URL", "Status");
// ...

// After: comfy-table
let table = crate::presentation::cli::table::provider_table(&providers);
println!("{table}");

// Before: no spinner
let result = validate_provider(&args.id).await?;

// After: spinner
let spinner = crate::presentation::cli::spinner::CliSpinner::new(
    &format!("Validating provider '{}'...", args.id)
);
let result = validate_provider(&args.id).await?;
spinner.finish_with_message(&format!("✓ Provider '{}' is valid", args.id));
```

### 8. Error Context Pattern

```rust
// In command handlers
match repo.find(&args.id).await? {
    Some(account) => { /* ... */ }
    None => {
        output::error(&format!(
            "Account '{}' not found. Use 'llm-router account list' to see available accounts.",
            args.id
        ));
        return Err(crate::Error::NotFound(args.id));
    }
}
```

### 9. Rich Help Text

```rust
#[derive(Debug, Args)]
#[command(after_help = r#"
EXAMPLES:
    llm-router provider add --id groq --name "Groq" --base-url "https://api.groq.com/openai/v1"
    llm-router provider add --id groq --name "Groq" --base-url "https://api.groq.com/openai/v1" --interactive
"#)]
pub struct AddProviderArgs {
    // ...
}
```

### 10. Cargo.toml Dependencies

```toml
# CLI - Interactive experience
owo-colors = "4"
comfy-table = "7"
inquire = "0.7"
indicatif = "0.17"
is-terminal = "0.4"
```

## Files Modified

| File | Lines Changed | Description |
|------|---------------|-------------|
| `Cargo.toml` | +5 deps | New CLI dependencies |
| `src/presentation/cli/mod.rs` | +20 | TTY detection init, updated imports |
| `src/presentation/cli/output.rs` | +60 | **NEW** — Colored output utilities |
| `src/presentation/cli/spinner.rs` | +50 | **NEW** — Spinner wrapper |
| `src/presentation/cli/table.rs` | +80 | **NEW** — Table formatting |
| `src/presentation/cli/prompt.rs` | +60 | **NEW** — Interactive prompts |
| `src/presentation/cli/tty.rs` | +30 | **NEW** — TTY detection |
| `src/presentation/cli/input.rs` | ~5 | Updated to use inquire |
| `src/presentation/cli/commands/provider.rs` | ~200 | Redesigned with new UX |
| `src/presentation/cli/commands/account.rs` | ~150 | Redesigned with new UX |
| `src/presentation/cli/commands/login.rs` | ~50 | Spinner + colored flow |
| `src/presentation/cli/commands/logout.rs` | ~30 | Colored output + confirmation |
| `src/presentation/cli/commands/auth.rs` | ~20 | Colored output |
| `docs/cli.md` | ~50 | Updated documentation |

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| `inquire` blocking stdin in async | Use `tokio::task::spawn_blocking` for prompt calls |
| Non-TTY environments (CI/CD) | Graceful degradation via `is_tty()` checks |
| `NO_COLOR` not respected | Check env var + `--no-color` flag in `tty.rs` |
| Table width on narrow terminals | `ContentArrangement::Dynamic` in comfy-table |
| Spinner flicker on slow terminals | 80ms tick interval, disable if not TTY |
