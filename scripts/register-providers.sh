#!/usr/bin/env bash
#
# Bootstrap script to register 12+ LLM providers
#
# Usage: ./scripts/register-providers.sh [--api-key <key>] [--interactive]
#
# This script registers the following providers:
# 1. Groq
# 2. Google AI Studio
# 3. OpenRouter
# 4. Hugging Face
# 5. Mistral
# 6. Cerebras
# 7. NVIDIA NIM
# 8. Cloudflare Workers AI
# 9. Cohere
# 10. AI21
# 11. DeepSeek
# 12. xAI (Grok)
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Build the project first
echo -e "${YELLOW}Building project...${NC}"
cd "$PROJECT_ROOT"
cargo build --release 2>&1 | grep -E "(Compiling|Finished|error)" || true

# Path to binary
BINARY="$PROJECT_ROOT/target/release/llm-router"

if [ ! -f "$BINARY" ]; then
    echo -e "${RED}Error: Binary not found at $BINARY${NC}"
    echo "Please build the project first with: cargo build --release"
    exit 1
fi

# Parse arguments
API_KEY_ARG=""
INTERACTIVE_ARG=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --api-key)
            API_KEY_ARG="--api-key $2"
            shift 2
            ;;
        --interactive)
            INTERACTIVE_ARG="--interactive"
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [--api-key <key>] [--interactive]"
            echo ""
            echo "Options:"
            echo "  --api-key <key>  API key to use for all providers (not recommended)"
            echo "  --interactive    Prompt for API key for each provider"
            echo "  -h, --help       Show this help message"
            echo ""
            echo "Without options, providers are registered without API keys."
            echo "You can add API keys later with: $BINARY provider add --id <id> --api-key <key>"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

echo -e "${GREEN}✓ Build complete${NC}"
echo ""
echo -e "${YELLOW}Registering 12 LLM providers...${NC}"
echo ""

# Provider definitions
# Format: "id|name|base_url"
declare -a PROVIDERS=(
    "groq|Groq|https://api.groq.com/openai/v1"
    "google-ai|Google AI Studio|https://generativelanguage.googleapis.com/v1beta"
    "openrouter|OpenRouter|https://openrouter.ai/api/v1"
    "huggingface|Hugging Face|https://api-inference.huggingface.co/models"
    "mistral|Mistral AI|https://api.mistral.ai/v1"
    "cerebras|Cerebras|https://api.cerebras.ai/v1"
    "nvidia-nim|NVIDIA NIM|https://integrate.api.nvidia.com/v1"
    "cloudflare|Cloudflare Workers AI|https://api.cloudflare.com/client/v4/accounts"
    "cohere|Cohere|https://api.cohere.ai/v1"
    "ai21|AI21 Labs|https://api.ai21.com/studio/v1"
    "deepseek|DeepSeek|https://api.deepseek.com/v1"
    "xai|xAI (Grok)|https://api.x.ai/v1"
)

# Register each provider
for provider_data in "${PROVIDERS[@]}"; do
    IFS='|' read -r id name base_url <<< "$provider_data"
    
    echo -n "Registering: $name ($id)... "
    
    if $BINARY provider add --id "$id" --name "$name" --base-url "$base_url" $API_KEY_ARG $INTERACTIVE_ARG 2>&1; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗ Failed${NC}"
    fi
done

echo ""
echo -e "${YELLOW}Provider registration complete!${NC}"
echo ""
echo -e "${YELLOW}List all providers:${NC}"
echo "  $BINARY provider list"
echo ""
echo -e "${YELLOW}Validate a provider:${NC}"
echo "  $BINARY provider validate --id <provider-id>"
echo ""
echo -e "${YELLOW}Enable/disable providers:${NC}"
echo "  $BINARY provider enable --id <provider-id>"
echo "  $BINARY provider disable --id <provider-id>"
echo ""
echo -e "${YELLOW}Remove a provider:${NC}"
echo "  $BINARY provider remove --id <provider-id>"
echo ""
echo -e "${GREEN}Done!${NC}"
