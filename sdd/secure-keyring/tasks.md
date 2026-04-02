# Tasks: Secure API Key Storage with System Keyring (Issue #22)

## Implementation Tasks

### Phase 1: Secure Storage Module
- [ ] **T1: Create `src/infrastructure/secure_storage/mod.rs`** — SecureStorage trait, SecureStorageError, factory function
- [ ] **T2: Create `src/infrastructure/secure_storage/keyring_adapter.rs`** — KeyringStorage implementation
- [ ] **T3: Create `src/infrastructure/secure_storage/encrypted_store.rs`** — EncryptedFileStorage fallback

### Phase 2: Integration
- [ ] **T4: Update `src/infrastructure/mod.rs`** — Re-export secure_storage module
- [ ] **T5: Update `src/infrastructure/persistence/json_account_repository.rs`** — Use SecureStorage for API keys, add migration logic
- [ ] **T6: Update `src/domain/entities/account.rs`** — Wrap api_key in SecretString where applicable

### Phase 3: Verification
- [ ] **T7: Verify compilation** — `cargo check` passes
- [ ] **T8: Run tests** — All tests pass
- [ ] **T9: Format and lint** — `cargo fmt --check` and `cargo clippy -- -D warnings` clean
- [ ] **T10: Create `docs/security.md`** — Document security architecture

## Dependencies

```
T1 → T2 → T3
T4 (after T1-T3)
T5 (after T1-T4)
T6 (after T5)
T7-T10 (after T6)
```

## Notes

- `keyring`, `secrecy`, and `zeroize` are already in Cargo.toml
- Migration is automatic — plaintext keys detected and moved to secure storage
- Fallback chain: Keyring → Encrypted File → Plaintext (with warning)
- `SECURE_STORAGE=disabled` env var for dev/testing
