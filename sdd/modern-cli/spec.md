# Specification: Modern Interactive CLI Experience (Issue #19)

## Requirements

### REQ-1: Colored Output
The CLI SHALL use semantic colors for all output:
- **Green** (✓) for success messages
- **Red** (✗) for error messages
- **Yellow** (⚠) for warnings
- **Blue** (ℹ) for informational messages
- **Dimmed** for secondary/contextual information

### REQ-2: Professional Tables
All list commands (`provider list`, `account list`) SHALL display data using formatted tables with:
- Column headers with bold styling
- Proper column alignment
- Truncated long values with ellipsis
- Status indicators (● Enabled, ○ Disabled, ✓ Active, ✗ Inactive)

### REQ-3: Interactive Confirmation Prompts
All destructive operations SHALL require user confirmation:
- `provider remove` → "Are you sure you want to remove provider '{id}'? This will also remove all associated accounts."
- `account remove` → "Are you sure you want to remove account '{id}'?"
- `auth logout --all` → "Are you sure you want to logout from all providers?"

### REQ-4: Spinners for Network Operations
All operations that perform network requests SHALL show a spinner:
- `provider validate` → "Validating provider '{id}'..."
- `account validate` → "Validating account '{id}'..."
- `auth login --oauth` → "Waiting for browser authentication..."

### REQ-5: Masked Input
API key input SHALL be masked (hidden characters) when using interactive mode.

### REQ-6: TTY Detection
The CLI SHALL detect if running in a TTY and:
- **TTY detected**: Full interactive experience (colors, prompts, tables)
- **No TTY**: Plain text output, no prompts, no colors (for piping/scripting)

### REQ-7: Rich Help Text
All subcommands SHALL include examples in their help text:
```
USAGE:
    llm-router provider add [OPTIONS] --id <ID> --name <NAME> --base-url <URL>

EXAMPLES:
    llm-router provider add --id groq --name "Groq" --base-url "https://api.groq.com/openai/v1"
    llm-router provider add --id groq --name "Groq" --base-url "https://api.groq.com/openai/v1" --interactive
```

### REQ-8: Error Context
Error messages SHALL include:
- What went wrong
- What command was being executed
- Suggested fix (when applicable)

### REQ-9: Backward Compatibility
All existing command signatures SHALL remain unchanged:
- Same flags, same arguments, same behavior
- New interactive features are additive, not breaking

### REQ-10: No-Color Support
The CLI SHALL respect the `NO_COLOR` environment variable and `--no-color` flag to disable colored output.

## Scenarios

### Scenario 1: Add Provider with Interactive Mode
**Given** user runs `llm-router provider add --id groq --name "Groq" --base-url "https://api.groq.com/openai/v1" --interactive`
**When** CLI prompts for API key
**Then** input is masked (hidden characters)
**And** on success: `✓ Provider 'groq' added successfully` (green)

### Scenario 2: List Providers
**Given** user runs `llm-router provider list`
**When** CLI displays providers
**Then** output is a formatted table with headers, alignment, and status indicators

### Scenario 3: Remove Provider (Confirmation)
**Given** user runs `llm-router provider remove --id groq`
**When** CLI prompts "Are you sure you want to remove provider 'groq'? [y/N]"
**And** user types `n`
**Then** operation is cancelled with message: `Cancelled. Provider 'groq' was not removed.`

### Scenario 4: Validate Provider (Spinner)
**Given** user runs `llm-router provider validate --id groq`
**When** CLI shows spinner "⠋ Validating provider 'groq'..."
**And** validation completes
**Then** spinner stops and shows: `✓ Provider 'groq' is valid` (green) or `✗ Provider 'groq' validation failed: invalid API key` (red)

### Scenario 5: Piped Output (No TTY)
**Given** user runs `llm-router provider list | grep groq`
**When** CLI detects no TTY
**Then** output is plain text without colors or table formatting

### Scenario 6: Error with Context
**Given** user runs `llm-router account validate --id nonexistent`
**When** account is not found
**Then** error shows: `✗ Account 'nonexistent' not found. Use 'llm-router account list' to see available accounts.` (red)

### Scenario 7: NO_COLOR Environment
**Given** `NO_COLOR=1 llm-router provider list`
**When** CLI runs
**Then** output has no ANSI color codes

### Scenario 8: Help with Examples
**Given** user runs `llm-router provider add --help`
**When** help is displayed
**Then** output includes usage, options, AND examples section
