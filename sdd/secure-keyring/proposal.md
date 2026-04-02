# Change Proposal: Secure API Key Storage with System Keyring (Issue #22)

## Intent

Reemplazar el almacenamiento de API keys en texto plano por almacenamiento seguro usando System Keyring (macOS Keychain, Windows Credential Manager, Linux Secret Service) con fallback encriptado para entornos sin keyring.

## Problem

Las API keys se almacenan **en texto plano** en `~/.config/rust-llm-api-router/accounts.json`. Esto significa:

- **Cualquier usuario con acceso al sistema** puede leer las API keys
- **Backup/sync de archivos** expone las keys accidentalmente
- **Documentación dice "secure"** pero la realidad es plaintext
- `keyring` y `secrecy` están en `Cargo.toml` pero **NO se usan**

**Gap crítico**: `README.md` dice "Tokens stored securely in system keyring" — esto es **FALSO**.

## Scope

### Incluido
1. **Crear `SecureStorage` trait** — Abstracción para almacenamiento seguro
2. **Implementar `KeyringStorage`** — Usa `keyring` crate para system keyring nativo
3. **Implementar `EncryptedFileStorage`** — Fallback con AES-GCM para entornos sin keyring
4. **Modificar `JsonAccountRepository`** — Usar `SecureStorage` para API keys
5. **Migración automática** — Detectar keys en plaintext y migrar a keyring al iniciar
6. **Graceful fallback** — Si keyring no está disponible, usar encrypted file
7. **`secrecy::SecretString`** para API keys en memoria

### NO Incluido
- UI para gestionar credenciales
- Rotación automática de API keys
- Auditoría de acceso a credenciales
- Soporte para HSMs hardware

## Approach

### Arquitectura Híbrida

```
Account Repository
    ↓
SecureStorage (trait)
    ├── KeyringStorage (primary) — system keyring nativo
    └── EncryptedFileStorage (fallback) — AES-GCM con machine-derived key
```

### Keyring Service Naming

```
Service: "rust-llm-api-router"
Account: "{account_id}"  → keyring entry per account
```

### JSON File Changes

**Antes:**
```json
{
  "id": "groq-1",
  "provider_id": "groq",
  "api_key": "gsk_abc123..."  // ← INSEGURO
}
```

**Después:**
```json
{
  "id": "groq-1",
  "provider_id": "groq",
  "api_key_ref": "keyring:groq-1"  // ← Referencia, no el valor
}
```

### Migration Strategy

1. Al iniciar, detectar si hay `api_key` en plaintext en JSON
2. Si existe, mover a keyring y reemplazar con referencia
3. Si keyring falla, cifrar con AES-GCM y guardar en archivo separado
4. Log de migración para debugging

### Backward Compatibility

- Lectura de formato anterior (plaintext) → migración automática
- Si keyring no está disponible → fallback transparente
- Variable de entorno `SECURE_STORAGE=disabled` para forzar plaintext (dev/testing)

## Impact

### Files Created
| File | Description |
|------|-------------|
| `src/infrastructure/secure_storage/mod.rs` | SecureStorage trait + factory |
| `src/infrastructure/secure_storage/keyring_adapter.rs` | Keyring implementation |
| `src/infrastructure/secure_storage/encrypted_store.rs` | AES-GCM fallback |

### Files Modified
| File | Change |
|------|--------|
| `src/infrastructure/persistence/json_account_repository.rs` | Use SecureStorage for keys |
| `src/domain/entities/account.rs` | Wrap api_key in SecretString |
| `src/infrastructure/mod.rs` | Re-export secure_storage |
| `docs/security.md` | **NEW** — Security documentation |

### Risks
- **Linux keyring**: Requiere libsecret/Secret Service — fallback automático
- **Migración**: Keys existentes se migran automáticamente, pero loguear para debugging
- **Containers**: Keyring puede no estar disponible — fallback a encrypted file

## Alternatives Considered

| Alternative | Why Rejected |
|-------------|-------------|
| **Solo keyring** | No funciona en containers/sin GUI — necesita fallback |
| **Solo AES-GCM** | Menos seguro que system keyring nativo |
| **Vault/HashiCorp** | Overkill para CLI local |
| **Sin cambios** | Security risk crítico — inaceptable |
