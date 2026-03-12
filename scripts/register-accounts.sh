#!/usr/bin/env bash
#
# Register providers and accounts using API keys from copyq SECRETS tab
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BINARY="$PROJECT_ROOT/target/debug/llm-router"

echo "Building project..."
cd "$PROJECT_ROOT"
cargo build 2>&1 | grep -E "(Compiling|Finished|error)" || true

if [ ! -f "$BINARY" ]; then
    echo "Error: Binary not found at $BINARY"
    exit 1
fi

echo ""
echo "=== Registering Providers ==="
echo ""

# Register providers
$BINARY provider add --id groq --name "Groq" --base-url "https://api.groq.com/openai/v1" --disabled
$BINARY provider add --id openrouter --name "OpenRouter" --base-url "https://openrouter.ai/api/v1" --disabled
$BINARY provider add --id mistral --name "Mistral AI" --base-url "https://api.mistral.ai/v1" --disabled
$BINARY provider add --id cerebras --name "Cerebras" --base-url "https://api.cerebras.ai/v1" --disabled
$BINARY provider add --id cloudflare --name "Cloudflare Workers AI" --base-url "https://api.cloudflare.com/client/v4/accounts" --disabled

echo ""
echo "=== Providers Registered ==="
$BINARY provider list

echo ""
echo "=== Registering Accounts with API Keys ==="
echo ""

# Get API keys from copyq SECRETS tab and register accounts
# Item 2: sk-or-v1-* (OpenRouter)
OPENROUTER_KEY=$(copyq tab SECRETS read 2 2>/dev/null | tr -d '\n')
if [ -n "$OPENROUTER_KEY" ]; then
    echo "Registering OpenRouter account..."
    $BINARY account add --id openrouter-1 --provider openrouter --api-key "$OPENROUTER_KEY" --priority 0
fi

# Item 3: sk-user-* (OpenAI compatible)
OPENAI_KEY=$(copyq tab SECRETS read 3 2>/dev/null | tr -d '\n')
if [ -n "$OPENAI_KEY" ]; then
    echo "Registering OpenAI account..."
    $BINARY account add --id openai-1 --provider openai --api-key "$OPENAI_KEY" --priority 0
fi

# Item 4: sk-* (OpenAI compatible)
OPENAI_KEY2=$(copyq tab SECRETS read 4 2>/dev/null | tr -d '\n')
if [ -n "$OPENAI_KEY2" ]; then
    echo "Registering OpenAI account 2..."
    $BINARY account add --id openai-2 --provider openai --api-key "$OPENAI_KEY2" --priority 1
fi

# Item 5: vFZbOkqX* (Groq?)
GROQ_KEY=$(copyq tab SECRETS read 5 2>/dev/null | tr -d '\n')
if [ -n "$GROQ_KEY" ]; then
    echo "Registering Groq account..."
    $BINARY account add --id groq-1 --provider groq --api-key "$GROQ_KEY" --priority 0
fi

# Item 7: gsk_* (Groq)
GROQ_KEY2=$(copyq tab SECRETS read 7 2>/dev/null | tr -d '\n')
if [ -n "$GROQ_KEY2" ]; then
    echo "Registering Groq account 2..."
    $BINARY account add --id groq-2 --provider groq --api-key "$GROQ_KEY2" --priority 1
fi

echo ""
echo "=== Accounts Registered ==="
$BINARY account list

echo ""
echo "=== Enabling Providers ==="
$BINARY provider enable --id groq
$BINARY provider enable --id openrouter
$BINARY provider enable --id mistral

echo ""
echo "=== Done! ==="
echo ""
echo "Test the API with:"
echo "  curl http://localhost:8080/v1/chat/completions \\"
echo "    -H 'Content-Type: application/json' \\"
echo "    -d '{\"model\": \"openrouter:llama-3.2-3b-instruct:free\", \"messages\": [{\"role\": \"user\", \"content\": \"Hello!\"}]}'"
echo ""
