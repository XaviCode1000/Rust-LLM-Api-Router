# Specification: Secure API Key Storage with System Keyring (Issue #22)

## Requirements

### REQ-1: SecureStorage Trait
The system SHALL provide a `SecureStorage` trait with:
- `store(account_id: &str, key: &str) -> Result<()>` — Store an API key
- `retrieve(account_id: &str) -> Result<Option<String>>` — Retrieve an API key
- `delete(account_id: &str) -> Result<()>` — Delete an API key
- `is_available() -> bool` — Check if storage backend is available

### REQ-2: Keyring Implementation
The `KeyringStorage` implementation SHALL:
- Use the `keyring` crate with service name `"rust-llm-api-router"`
- Store one entry per account ID
- Return `false` from `is_available()` if keyring is not accessible

### REQ-3: Encrypted File Fallback
The `EncryptedFileStorage` implementation SHALL:
- Use AES-256-GCM encryption
- Derive encryption key from machine ID using argon2
- Store encrypted data in `~/.local/share/rust-llm-api-router/credentials.enc`
- Be used automatically when keyring is not available

### REQ-4: Automatic Migration
On startup, the system SHALL:
- Detect API keys stored in plaintext in `accounts.json`
- Migrate them to the secure storage backend
- Replace plaintext keys with references (`keyring:{account_id}`)
- Log migration actions

### REQ-5: SecretString in Memory
API keys in memory SHALL be wrapped in `secrecy::SecretString` to prevent accidental logging or display.

### REQ-6: Graceful Degradation
If secure storage is unavailable:
- Log a warning
- Fall back to encrypted file storage
- If `SECURE_STORAGE=disabled`, allow plaintext (for dev/testing)

### REQ-7: Zeroize on Drop
All sensitive data structures SHALL implement `Zeroize` and `ZeroizeOnDrop` to clear memory when dropped.

### REQ-8: No Plaintext in Logs
API keys SHALL NEVER appear in log output, error messages, or debug output.

## Scenarios

### Scenario 1: Store API Key via Keyring
**Given** user adds an account with `llm-router account add --id groq-1 --provider groq --interactive`
**When** API key is entered
**Then** key is stored in system keyring (not in JSON file)
**And** JSON file contains only `"api_key_ref": "keyring:groq-1"`

### Scenario 2: Retrieve API Key
**Given** account exists with keyring reference
**When** system needs the API key for a request
**Then** key is retrieved from system keyring
**And** key is wrapped in `SecretString` in memory

### Scenario 3: Keyring Not Available (Container)
**Given** running in a container without Secret Service
**When** system tries to store API key
**Then** keyring returns "not available"
**And** system falls back to encrypted file storage
**And** warning is logged: "Keyring not available, using encrypted file storage"

### Scenario 4: Migrate Existing Plaintext Keys
**Given** `accounts.json` has plaintext API keys from before this change
**When** application starts
**Then** keys are automatically migrated to secure storage
**And** plaintext keys are removed from JSON
**And** log shows: "Migrated 3 API keys to secure storage"

### Scenario 5: Delete Account
**Given** user runs `llm-router account remove --id groq-1`
**When** account is deleted
**Then** API key is deleted from secure storage (keyring or encrypted file)
**And** account record is removed from JSON

### Scenario 6: Dev Mode (Plaintext Allowed)
**Given** `SECURE_STORAGE=disabled` is set
**When** application starts
**Then** warning is logged: "Secure storage disabled — API keys stored in plaintext"
**And** keys are stored in JSON as before (for development only)
