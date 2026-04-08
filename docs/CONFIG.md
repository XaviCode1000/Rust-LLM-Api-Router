# Configuration

Complete reference for all environment variables, configuration files, and settings priority.

---

## Table of Contents

- [Settings Priority](#settings-priority)
- [Server Configuration](#server-configuration)
- [Routing Configuration](#routing-configuration)
- [Quality Evaluation Configuration](#quality-evaluation-configuration)
- [Security Configuration](#security-configuration)
- [Configuration Files](#configuration-files)
- [Environment File Example](#environment-file-example)

---

## Settings Priority

Configuration is resolved in this order (highest to lowest priority):

```
1. CLI flags          (highest)
2. Environment variables
3. Default values     (lowest)
```

**Example:** If you set `--timeout 120` via CLI and `REQUEST_TIMEOUT_SECONDS=30` via env var, the CLI value (120) wins.

---

## Server Configuration

| Variable | CLI Flag | Default | Description |
|----------|----------|---------|-------------|
| `PORT` | `--port` / `-p` | `8080` | Server port |
| `HOST` | `--host` | `0.0.0.0` | Bind address |
| `LOG_LEVEL` | `--log-level` | `info` | Log level: `trace`, `debug`, `info`, `warn`, `error` |

### Examples

```bash
# Via CLI
./target/release/llm-router --host 127.0.0.1 --port 3000 --log-level debug

# Via environment
export HOST=127.0.0.1
export PORT=3000
export LOG_LEVEL=warn
./target/release/llm-router
```

---

## Routing Configuration

### Routing Strategy

| Variable | CLI Flag | Default | Valid Values |
|----------|----------|---------|--------------|
| — | `--routing-strategy` | `auto` | `auto`, `cost-optimized`, `cascading`, `failover`, `load-balanced` |

**Strategy descriptions:**

| Strategy | When to Use |
|----------|-------------|
| `auto` | General use — let the planner decide based on context |
| `cost-optimized` | Budget-critical — always pick cheapest capable model |
| `cascading` | Quality-critical — start cheap, escalate if quality is low |
| `failover` | Reliability — sequential fallback on failure |
| `load-balanced` | Throughput — health-weighted distribution |

### Cascading Routing

| Variable | CLI Flag | Default | Range | Description |
|----------|----------|---------|-------|-------------|
| `CASCADING_ENABLED` | `--cascading` | `false` | `true`, `false` | Enable cascading routing |
| `CASCADING_MIN_QUALITY` | `--quality-threshold` | `0.75` | `0.0`–`1.0` | Minimum quality score to accept a response |
| `CASCADING_MAX_TIERS` | — | `3` | `1`–`10` | Maximum model tiers to try |
| `CASCADING_PER_TIER_TIMEOUT_MS` | — | `5000` | `1000`–`30000` | Timeout per tier in milliseconds |

### Request Behavior

| Variable | CLI Flag | Default | Range | Description |
|----------|----------|---------|-------|-------------|
| `BUDGET_MODE` | `--budget-mode` | `false` | `true`, `false` | Enable budget mode (prefer cheapest models) |
| `MAX_RETRIES` | `--max-retries` | `3` | `1`–`10` | Maximum retries per request |
| `REQUEST_TIMEOUT_SECONDS` | `--timeout` | `60` | `10`–`300` | Request timeout in seconds |

### Examples

```bash
# Cascading with custom quality threshold
./target/release/llm-router \
  --routing-strategy cascading \
  --quality-threshold 0.85

# High reliability with failover
./target/release/llm-router \
  --routing-strategy failover \
  --max-retries 5 \
  --timeout 120

# Via environment variables
export ROUTING_STRATEGY=cascading
export CASCADING_MIN_QUALITY=0.8
export CASCADING_MAX_TIERS=4
export BUDGET_MODE=true
./target/release/llm-router
```

---

## Quality Evaluation Configuration

These settings control how cascading routing evaluates response quality.

| Setting | Default | Description |
|---------|---------|-------------|
| `min_quality_score` | `0.75` | Minimum score (0.0–1.0) to accept a response |
| `min_response_length` | `10` | Minimum characters to consider a response valid |
| `max_tiers` | `3` | Maximum model tiers to attempt |
| `per_tier_timeout_ms` | `5000` | Timeout per tier in milliseconds |

### Quality Checks

The `HeuristicQualityEvaluator` performs 4 checks:

| Check | What It Measures | Failure Condition |
|-------|------------------|-------------------|
| **Completeness** | Response not truncated | Ends with `,`, `:`, `;`, `-`, `{`, `[`, or whitespace |
| **Length** | Minimum response size | < 10 characters |
| **Structure** | Valid JSON when expected | Unmatched `{`/`}` or `[`/`]` |
| **Coherence** | No error patterns | Contains "I cannot", "As an AI", repeated words (4+) |

> ⚠️ **Streaming Guard:** Cascading is automatically disabled for streaming requests since quality can't be evaluated until the stream completes.

---

## Security Configuration

### Secure Storage

| Variable | Default | Valid Values | Description |
|----------|---------|--------------|-------------|
| `SECURE_STORAGE` | `auto` | `auto`, `keyring`, `encrypted`, `disabled` | How API keys are stored |

**Storage modes:**

| Mode | Behavior |
|------|----------|
| `auto` | Use system keyring if available, fallback to encrypted file |
| `keyring` | Force system keyring (macOS Keychain, Windows Credential Manager, Linux Secret Service) |
| `encrypted` | Force AES-256-GCM encrypted file storage |
| `disabled` | ⚠️ Store in plaintext (dev/testing only) |

### OAuth Configuration

These environment variables apply to OAuth 2.1 / PKCE authentication:

| Variable | Description |
|----------|-------------|
| `OAUTH_CLIENT_ID` | OAuth client ID for custom identity providers |
| `OAUTH_CLIENT_SECRET` | OAuth client secret |
| `NO_BROWSER` | Set to `true` to force device flow (headless environments) |
| `CLI_CUSTOM_CA_CERT` | Path to custom CA certificate (corporate proxies) |

---

## Configuration Files

### Location

Configuration files are stored in the XDG config directory:

```
~/.config/rust-llm-api-router/
├── providers.json    # Registered providers and their settings
└── accounts.json     # Accounts with API keys (keys are masked/encrypted)
```

### providers.json

```json
[
  {
    "id": "groq",
    "name": "Groq",
    "base_url": "https://api.groq.com/openai/v1",
    "enabled": true
  },
  {
    "id": "openai",
    "name": "OpenAI",
    "base_url": "https://api.openai.com/v1",
    "enabled": true
  }
]
```

### accounts.json

```json
[
  {
    "id": "groq-1",
    "provider_id": "groq",
    "api_key_masked": "gsk_DVyb...",
    "is_active": true,
    "priority": 0
  }
]
```

> 🔒 API keys are stored securely — either in the system keyring or encrypted with AES-256-GCM.

---

## Environment File Example

Copy this as `.env` for a production-ready setup:

```bash
# ── Server ──────────────────────────────────────────────
HOST=0.0.0.0
PORT=8080
LOG_LEVEL=info

# ── Routing ─────────────────────────────────────────────
ROUTING_STRATEGY=auto
# ROUTING_STRATEGY=cascading
# ROUTING_STRATEGY=cost-optimized
# ROUTING_STRATEGY=failover
# ROUTING_STRATEGY=load-balanced

# ── Cascading (only if ROUTING_STRATEGY=cascading) ──────
CASCADING_ENABLED=false
CASCADING_MIN_QUALITY=0.75
CASCADING_MAX_TIERS=3
CASCADING_PER_TIER_TIMEOUT_MS=5000

# ── Request Behavior ────────────────────────────────────
BUDGET_MODE=false
MAX_RETRIES=3
REQUEST_TIMEOUT_SECONDS=60

# ── Security ────────────────────────────────────────────
SECURE_STORAGE=auto
# SECURE_STORAGE=keyring
# SECURE_STORAGE=encrypted
# SECURE_STORAGE=disabled

# ── OAuth (optional, for custom identity providers) ─────
# OAUTH_CLIENT_ID=your-client-id
# OAUTH_CLIENT_SECRET=your-client-secret
# NO_BROWSER=false
# CLI_CUSTOM_CA_CERT=/path/to/ca.pem
```

---

## Quick Reference

### All Environment Variables

| Variable | Default | Section |
|----------|---------|---------|
| `HOST` | `0.0.0.0` | [Server](#server-configuration) |
| `PORT` | `8080` | [Server](#server-configuration) |
| `LOG_LEVEL` | `info` | [Server](#server-configuration) |
| `ROUTING_STRATEGY` | `auto` | [Routing](#routing-configuration) |
| `CASCADING_ENABLED` | `false` | [Routing](#routing-configuration) |
| `CASCADING_MIN_QUALITY` | `0.75` | [Routing](#routing-configuration) |
| `CASCADING_MAX_TIERS` | `3` | [Routing](#routing-configuration) |
| `CASCADING_PER_TIER_TIMEOUT_MS` | `5000` | [Routing](#routing-configuration) |
| `BUDGET_MODE` | `false` | [Routing](#routing-configuration) |
| `MAX_RETRIES` | `3` | [Routing](#routing-configuration) |
| `REQUEST_TIMEOUT_SECONDS` | `60` | [Routing](#routing-configuration) |
| `SECURE_STORAGE` | `auto` | [Security](#security-configuration) |
| `OAUTH_CLIENT_ID` | — | [Security](#security-configuration) |
| `OAUTH_CLIENT_SECRET` | — | [Security](#security-configuration) |
| `NO_BROWSER` | `false` | [Security](#security-configuration) |
| `CLI_CUSTOM_CA_CERT` | — | [Security](#security-configuration) |

---

## See Also

- [CLI Reference](cli.md) — All CLI commands and flags
- [Usage Guide](USAGE.md) — Practical examples and workflows
- [Routing Strategies](routing.md) — Detailed routing documentation
- [Security](security.md) — Auth, OAuth, secure storage details
- [Deployment Guide](deployment.md) — Docker, systemd, Kubernetes
