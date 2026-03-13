# CLI Reference

## Global Options

```bash
llm-router [OPTIONS] [COMMAND]

Options:
      --host <HOST>            Host to bind to (server mode) [default: 0.0.0.0]
  -p, --port <PORT>            Port to bind to (server mode) [default: 8080]
      --log-level <LOG_LEVEL>  Log level (trace, debug, info, warn, error) [default: info]
  -h, --help                   Print help
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
llm-router account add --id openai-working --provider openai --api-key "<your-api-key>r-valid-key-here" --priority 0
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
- Solution: Add accounts for the provider using `llm-router account add`

### "Provider returned 401 Unauthorized"
- Solution: Verify your API key is valid and has sufficient permissions

### "Model decommissioned" (Groq specific)
- Solution: Use supported models like `llama-3.3-70b-versatile` or `llama-3.1-8b-instant`

### Port already in use
- Solution: Kill existing process or use different port: `llm-router --port 8081`