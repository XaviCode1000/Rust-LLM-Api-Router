# CLI Reference

The CLI provides a polished user experience with colored output, professional tables, interactive prompts, and TTY detection.

## Global Options

```bash
llm-router [OPTIONS] [COMMAND]

Options:
      --host <HOST>                      Host to bind to (server mode) [default: 0.0.0.0]
  -p, --port <PORT>                      Port to bind to (server mode) [default: 8080]
      --log-level <LOG_LEVEL>            Log level (trace, debug, info, warn, error) [default: info]
      --routing-strategy <STRATEGY>      Routing strategy: auto, cost-optimized, cascading, failover, load-balanced [default: auto]
      --cascading                        Enable cascading routing (equivalent to --routing-strategy cascading)
      --quality-threshold <THRESHOLD>    Minimum quality score for cascading (0.0-1.0) [default: 0.75]
      --budget-mode                      Enable budget mode for cost optimization
      --max-retries <RETRIES>            Maximum retries per request [default: 3]
      --timeout <SECONDS>                Request timeout in seconds [default: 60]
  -h, --help                             Print help
```

## CLI Commands

```
Commands:
  provider      Provider management (add, list, enable, disable, remove, validate)
  account       Account management (add, list, set-priority, remove, validate)
  auth          Authentication (login, logout)
  completions   Shell completions (bash, zsh, fish) -- requires --features completions
```

## Modern CLI Features (Issue #19)

The CLI provides a polished user experience with:

### Colored Output

- **Success**: Green checkmarks and success messages
- **Error**: Red error messages with details
- **Warning**: Yellow warnings for potential issues
- **Info**: Blue informational messages

```bash
# Example output
✓ Provider 'groq' added successfully
✗ Failed to validate account: Invalid API key
⚠ Warning: Provider already exists
ℹ Use 'llm-router account add --interactive' for secure input
```

### Professional Tables

List commands display formatted tables with alignment:

```
ID                   Name                           Base URL                                 Status
----------------------------------------------------------------------------------------------------
test-provider        Test Provider                  https://api.test.com/v1                  ✓ Enabled
groq                 Groq                           https://api.groq.com/openai/v1           ✓ Enabled
```

### Interactive Prompts

Sensitive operations like API key entry use secure input:

```bash
# Interactive mode (recommended)
llm-router account add --id groq-account --provider groq --interactive
# Prompts for API key with hidden input
```

### Progress Spinners

Long-running operations show spinners:

```
Validating provider... ✓
Fetching models... ✓
```

### TTY Detection

The CLI automatically detects terminal capabilities and provides:

- **Interactive mode**: Full colors, tables, prompts when running in a terminal
- **Non-interactive mode**: Simplified output for scripts/automation

```bash
# Force interactive mode even in scripts
llm-router --force-interactive provider list
```

## Provider Management

### Add Provider

```bash
llm-router provider add --id <id> --name <name> --base-url <url> [--disabled]
```

**Example:**
```bash
llm-router provider add --id groq --name "Groq" --base-url "https://api.groq.com/openai/v1"
llm-router provider add --id openai --name "OpenAI" --base-url "https://api.openai.com/v1"
```

### List Providers

```bash
llm-router provider list
```

**Sample Output:**
```
ID                   Name                           Base URL                                 Status
----------------------------------------------------------------------------------------------------
test-provider        Test Provider                  https://api.test.com/v1                  ✓ Enabled
groq                 Groq                           https://api.groq.com/openai/v1           ✓ Enabled
openrouter           OpenRouter                     https://openrouter.ai/api/v1             ✓ Enabled
mistral              Mistral AI                     https://api.mistral.ai/v1                ✓ Enabled
cerebras             Cerebras                       https://api.cerebras.ai/v1               ✗ Disabled
cloudflare           Cloudflare Workers AI          https://api.cloudflare.com/client/v4/accounts ✗ Disabled
openai               OpenAI                         https://api.openai.com/v1                ✓ Enabled
```

### Enable/Disable Provider

```bash
llm-router provider enable --id <id>
llm-router provider disable --id <id>
```

### Remove Provider

```bash
llm-router provider remove --id <id>
```

### Validate Provider

```bash
llm-router provider validate --id <id>
```

## Account Management

### Add Account

```bash
llm-router account add --id <id> --provider <provider_id> --api-key <key> [--priority <n>] [--interactive]
```

**Examples:**
```bash
# Interactive mode (recommended for security)
llm-router account add --id groq-account-1 --provider groq --interactive

# Direct API key (use with caution)
llm-router account add --id openai-account-1 --provider openai --api-key "<your-api-key>"

# With custom priority (lower = higher priority)
llm-router account add --id groq-account-2 --provider groq --api-key "key" --priority 1
```

### List Accounts

```bash
llm-router account list
```

**Sample Output:**
```
ID                   Provider             Priority   Status   API Key
------------------------------------------------------------------------------------------
openrouter-1         openrouter           0          ✓ Active <your-openrouter-api-key>
openai-1             openai               0          ✓ Active <your-openai-api-key>
openai-2             openai               1          ✓ Active <your-openai-api-key>
groq-2               groq                 0          ✓ Active gsk_DVyb...
openai-account-1     openai               0          ✓ Active ****
```

### Set Account Priority

```bash
llm-router account set-priority --id <id> --priority <n>
```

**Example:**
```bash
llm-router account set-priority --id groq-account-1 --priority 0
```

### Remove Account

```bash
llm-router account remove --id <id>
```

### Validate Account

```bash
llm-router account validate --id <id>
```

## Routing Configuration (Issue #29)

### CLI Flags

Configure routing strategy directly from the command line:

```bash
# Enable cascading routing
llm-router --routing-strategy cascading

# With quality threshold
llm-router --cascading --quality-threshold 0.85

# Budget mode with cost-optimized
llm-router --routing-strategy cost-optimized --budget-mode

# High reliability with failover
llm-router --routing-strategy failover --max-retries 5 --timeout 120
```

| Flag | Description | Example |
|------|-------------|---------|
| `--routing-strategy` | Routing strategy | `--routing-strategy cascading` |
| `--cascading` | Enable cascading | `--cascading` |
| `--quality-threshold` | Quality score (0.0-1.0) | `--quality-threshold 0.85` |
| `--budget-mode` | Enable budget mode | `--budget-mode` |
| `--max-retries` | Max retries | `--max-retries 3` |
| `--timeout` | Timeout in seconds | `--timeout 60` |

### Server Mode with Routing

```bash
# Start server with specific routing
llm-router --port 8080 --routing-strategy cascading --quality-threshold 0.8
```

### Environment Variables

Routing configuration can also be set via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `ROUTING_STRATEGY` | Overall routing strategy | `auto` |
| `CASCADING_ENABLED` | Enable cascading routing | `false` |
| `CASCADING_MIN_QUALITY` | Minimum quality score | `0.75` |
| `CASCADING_MAX_TIERS` | Maximum tiers to try | `3` |
| `CASCADING_PER_TIER_TIMEOUT_MS` | Timeout per tier | `5000` |
| `BUDGET_MODE` | Enable budget mode | `false` |
| `MAX_RETRIES` | Maximum retries per request | `3` |
| `REQUEST_TIMEOUT_SECONDS` | Request timeout in seconds | `60` |

**Priority hierarchy:** CLI flags > Environment variables > Default values

## Auth Commands

### Login

```bash
# Login with API key
llm-router auth login --provider <provider_id>
llm-router auth login -p <provider_id>

# Login with OAuth 2.1 PKCE (opens browser)
llm-router auth login --provider <provider_id> --oauth

# Login with Device Flow (headless environments)
llm-router auth login --provider <provider_id> --device-flow

# Interactive login (prompts for credentials)
llm-router auth login --interactive
```

### Logout

```bash
# Logout from a specific provider
llm-router auth logout --provider <provider_id>

# Logout from all providers
llm-router auth logout --all

# Clear stored credentials
llm-router auth logout --clear-credentials
```

## Shell Completions

Generate shell completions for your terminal (requires `completions` feature flag).

```bash
# Build with completions enabled
cargo build --release --features completions

# Generate completions for your shell
llm-router completions bash > ~/.local/share/bash-completion/completions/llm-router
llm-router completions zsh > ~/.zfunc/_llm-router
llm-router completions fish > ~/.config/fish/completions/llm-router.fish
```

**Supported shells:** `bash`, `zsh`, `fish`

## Working Commands Verified

### Provider Setup
```bash
# Add Groq provider (already configured in default data)
llm-router provider add --id groq --name "Groq" --base-url "https://api.groq.com/openai/v1"

# Add OpenAI provider
llm-router provider add --id openai --name "OpenAI" --base-url "https://api.openai.com/v1"
```

### Account Setup (Working Examples)
```bash
# Add Groq account with verified working key
llm-router account add --id groq-working --provider groq --api-key "<your-groq-api-key>" --priority 0

# Add OpenAI account (requires valid key from platform.openai.com)
llm-router account add --id openai-working --provider openai --api-key "<your-api-key>" --priority 0
```

## Testing the Setup

### Health Check
```bash
curl http://localhost:8080/health
# Returns: {"status":"healthy","timestamp":1773367298,"version":"0.1.0"}
```

### Chat Completion (Working Example)
```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <your-groq-api-key>" \
  -d '{
    "model": "groq:llama-3.3-70b-versatile",
    "messages": [{"role": "user", "content": "Hello, world!"}],
    "max_tokens": 50
  }'
```

## Provider-Specific Notes

### Groq
- Working models: `llama-3.3-70b-versatile`, `llama-3.1-8b-instant`, `groq/compound`, `groq/compound-mini`
- Decommissioned models (do NOT use): `llama3-8b-8192`, `mixtral-8x7b-32768`
- Base URL: `https://api.groq.com/openai/v1`

### OpenAI
- Requires valid API key from https://platform.openai.com/account/api-keys
- Working models: `gpt-3.5-turbo`, `gpt-4`, `gpt-4-turbo` (with valid key)
- Base URL: `https://api.openai.com/v1`

## Troubleshooting

### "No active accounts found for provider"
- **Solution**: Add accounts for the provider using `llm-router account add`

### "Provider returned 401 Unauthorized"
- **Solution**: Verify your API key is valid and has sufficient permissions

### "Model decommissioned" (Groq specific)
- **Solution**: Use supported models like `llama-3.3-70b-versatile` or `llama-3.1-8b-instant`

### Port already in use
- **Solution**: Kill existing process or use different port: `llm-router --port 8081`

## CLI Module Structure

The CLI module consists of 13 files organized into commands and helpers:

**Commands** (`src/presentation/cli/commands/`):
| File | Purpose |
|------|---------|
| `provider.rs` | Provider subcommand handler |
| `account.rs` | Account subcommand handler |
| `auth.rs` | Auth subcommand handler |
| `login.rs` | Login implementation |
| `logout.rs` | Logout implementation |
| `completions.rs` | Shell completion generation |

**Helpers** (`src/presentation/cli/`):
| File | Purpose |
|------|---------|
| `mod.rs` | Cli struct, CliCommands enum, dispatcher |
| `input.rs` | Input helpers |
| `output.rs` | Output formatting |
| `prompt.rs` | Interactive prompts |
| `spinner.rs` | Progress spinners |
| `table.rs` | Table formatting |
| `tty.rs` | TTY detection |

## See Also

- [API Reference](api.md) -- API endpoints and usage
- [Routing Strategies](routing.md) -- Detailed routing configuration
- [Architecture](architecture.md) -- System architecture overview
