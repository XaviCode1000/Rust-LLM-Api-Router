# Security

The LLM API Router implements multiple security measures to protect your API keys and credentials.

## Secure API Key Storage (Issue #22)

The router provides secure storage for API keys using system keyrings or encrypted local storage.

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Secure Storage Layer                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌──────────────────┐     ┌───────────────────────────────┐  │
│   │  SecureStorage  │────▶│    KeyringStorage             │  │
│   │     Trait       │     │    (Primary - System Keyring)  │  │
│   └──────────────────┘     └───────────────────────────────┘  │
│          │                              │                       │
│          │                              ▼                       │
│          │                  ┌───────────────────────────────┐  │
│          │                  │  macOS Keychain               │  │
│          │                  │  Windows Credential Manager   │  │
│          │                  │  Linux Secret Service         │  │
│          └────────────────▶│  (DBus/Secret Service)        │  │
│                             └─────────────────────────────────┘  │
│                                    │                            │
│                                    ▼                            │
│                             ┌───────────────────────────────┐   │
│                             │  EncryptedFileStorage        │   │
│                             │  (Fallback - AES-256-GCM)    │   │
│                             └───────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Keyring Storage

The primary storage uses system keyrings:

| Platform | Keyring | Library |
|----------|---------|---------|
| **macOS** | Keychain | `security-framework` |
| **Windows** | Credential Manager | `windows-credentials` |
| **Linux** | Secret Service (DBus) | `keyring` crate |

**How it works:**
- API keys are stored in the OS-native credential store
- Keys are encrypted by the OS and protected by user authentication
- Automatic retrieval on startup for authenticated users

### Encrypted File Storage (Fallback)

When system keyring is unavailable, the router falls back to encrypted file storage:

- **Algorithm**: AES-256-GCM
- **Key Derivation**: Argon2id from machine-specific secret
- **Location**: `~/.config/rust-llm-api-router/secrets.enc`

```rust
// Encrypted file storage configuration
let storage = EncryptedFileStorage::new()
    .with_path(PathBuf::from("~/.config/rust-llm-api-router/secrets.enc"));
```

### Automatic Migration

The router automatically migrates plaintext API keys to secure storage:

```
Migration Flow:
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  accounts.json  │────▶│  Detect plaintext │────▶│  Keyring/       │
│  (old format)   │     │  API keys         │     │  Encrypted store│
└─────────────────┘     └──────────────────┘     └─────────────────┘
```

**Migration triggers:**
- On startup if plaintext keys exist in `accounts.json`
- On `account add` command
- Manual trigger via CLI: `llm-router account migrate`

### Configuration

#### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SECURE_STORAGE` | `auto` | Storage mode: `auto`, `keyring`, `encrypted`, `disabled` |

```bash
# Use system keyring (default)
export SECURE_STORAGE=auto

# Force encrypted file storage
export SECURE_STORAGE=encrypted

# Disable secure storage (dev/testing only!)
export SECURE_STORAGE=disabled
```

> **Warning**: Setting `SECURE_STORAGE=disabled` is not recommended for production. Use only for local development or testing.

#### CLI Commands

```bash
# Check secure storage status
llm-router account secure-status

# Migrate existing accounts to secure storage
llm-router account migrate

# Export keys to keyring (if not already done)
llm-router account sync-secure
```

### Security Best Practices

1. **Use System Keyring** (default) - Keys are protected by OS authentication
2. **Never commit API keys** - Use environment variables or secure storage
3. **Enable secure storage in production** - Set `SECURE_STORAGE=keyring`
4. **Rotate API keys periodically** - Use `llm-router account validate` to verify
5. **Use OAuth 2.1 / PKCE** when available - Token-based auth is more secure

### Encryption Details

#### Key Derivation

The encryption key is derived using Argon2id:

```rust
let secret = machine_uid()?;  // Unique machine identifier
let key = argon2::Argon2::default()
    .hash_password(secret.as_bytes(), salt)
    .map(|h| h.hash.unwrap())?;
```

#### File Format

```
┌────────────────────────────────────────────┐
│  Magic Bytes (4)    │  "LLRK"             │
├────────────────────────────────────────────┤
│  Version (1)       │  0x01               │
├────────────────────────────────────────────┤
│  Salt (16 bytes)   │  Random salt        │
├────────────────────────────────────────────┤
│  Nonce (12 bytes)  │  Random nonce       │
├────────────────────────────────────────────┤
│  Ciphertext        │  AES-256-GCM        │
│  (variable)        │  encrypted data     │
├────────────────────────────────────────────┤
│  Tag (16 bytes)    │  GCM authentication │
└────────────────────────────────────────────┘
```

## OAuth 2.1 / PKCE

The router supports OAuth 2.1 with PKCE for enhanced security:

### Authentication Flow

1. **Generate Verifier**: Cryptographically random string (43-128 chars)
2. **Generate Challenge**: SHA-256 hash of verifier (Base64URL encoded)
3. **Authorization URL**: User authenticates in browser
4. **Callback**: Authorization code received
5. **Token Exchange**: Verifier + code → access/refresh tokens

### CLI Commands

```bash
# Login with PKCE (opens browser)
llm-router auth login --provider groq

# Login with Device Flow (headless)
llm-router auth login --provider groq --device-flow

# Logout
llm-router auth logout --provider groq
```

### Token Storage

OAuth tokens are also stored in secure storage:
- Access tokens
- Refresh tokens
- Token expiration timestamps

## Input Validation & Sanitization

All API inputs are validated:

- **Model IDs**: Whitelist of known providers/models
- **Message Length**: Configurable max tokens
- **Content Type**: JSON-only for chat completions
- **Rate Limiting**: Per-account request limits

## Security Checklist for Production

- [ ] Use system keyring storage (`SECURE_STORAGE=keyring`)
- [ ] Enable TLS/HTTPS (reverse proxy)
- [ ] Restrict network access
- [ ] Monitor authentication logs
- [ ] Rotate API keys regularly
- [ ] Use OAuth 2.1 / PKCE when available
- [ ] Set appropriate rate limits
- [ ] Enable audit logging

## See Also

- [Configuration](deployment.md) - Environment variables
- [CLI Reference](cli.md) - CLI commands
- [Architecture](architecture.md) - System architecture