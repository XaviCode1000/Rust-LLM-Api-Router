//! Round-trip validation pipeline for identity newtypes (AccountId, ProviderId).
//!
//! Ensures that the String → Newtype migration preserves serialization compatibility
//! and handles edge cases without panics or data corruption.
//!
//! ## Pipeline
//! 1. **Insta Snapshots**: Verify JSON output is byte-identical to pre-refactor format
//! 2. **Proptest Invariants**: Serialize → Deserialize must be identity for any input
//! 3. **Edge Cases**: Empty strings, unicode, special chars, max-length inputs

use insta::assert_json_snapshot;
use proptest::prelude::*;
use rust_llm_api_router::domain::entities::{Account, AccountHealth, AccountId, Provider};
use rust_llm_api_router::domain::providers::ProviderId;

// =============================================================================
// INSTA SNAPSHOTS — JSON format stability
// =============================================================================

/// Verify Account serializes as plain strings (not objects).
/// This is the critical invariant: serde(transparent) must produce `"id": "acc1"`,
/// NOT `"id": {"inner": "acc1"}`.
#[test]
fn test_account_json_snapshot() {
    let account = Account::new_api_key("acc-001", "openai", "sk-test-key-12345").with_priority(10);

    let json = serde_json::to_value(&account).unwrap();

    assert_json_snapshot!("account_json_format", json, {
        ".created_at" => "[timestamp]",
        ".last_used_at" => "[timestamp]",
        ".updated_at" => "[timestamp]",
    });
}

/// Verify Provider serializes as plain strings.
#[test]
fn test_provider_json_snapshot() {
    let provider = Provider::new("openai", "OpenAI", "https://api.openai.com/v1");

    let json = serde_json::to_value(&provider).unwrap();

    assert_json_snapshot!("provider_json_format", json);
}

/// Verify AccountHealth serializes with AccountId as plain string.
#[test]
fn test_account_health_json_snapshot() {
    let health = AccountHealth::new("acc-001");

    let json = serde_json::to_value(&health).unwrap();

    assert_json_snapshot!("account_health_json_format", json, {
        ".last_success_at" => "[timestamp]",
        ".last_failure_at" => "[timestamp]",
        ".circuit_breaker_reset_at" => "[timestamp]",
    });
}

/// Round-trip: serialize then deserialize must produce identical Account.
#[test]
fn test_account_roundtrip_snapshot() {
    let original = Account::new_oauth(
        "acc-oauth-1",
        "anthropic",
        "at-xxxxx",
        Some("rt-yyyyy"),
        Some("idt-zzzzz"),
        None,
    )
    .with_priority(5);

    let json = serde_json::to_string(&original).unwrap();
    let restored: Account = serde_json::from_str(&json).unwrap();

    assert_eq!(
        original, restored,
        "Account round-trip must preserve equality"
    );
}

/// Round-trip: serialize then deserialize must produce identical Provider.
#[test]
fn test_provider_roundtrip_snapshot() {
    let original = Provider::new("groq", "Groq", "https://api.groq.com/v1");

    let json = serde_json::to_string(&original).unwrap();
    let restored: Provider = serde_json::from_str(&json).unwrap();

    assert_eq!(
        original, restored,
        "Provider round-trip must preserve equality"
    );
}

/// Verify legacy JSON (pre-refactor format) loads correctly via persistence bridge.
/// This simulates loading an existing database file through the AccountData → Account path.
#[test]
fn test_legacy_account_json_loads() {
    // Legacy JSON uses flat fields (not auth_method enum).
    // This is how accounts.json looks on disk.
    let legacy_json = r#"{
        "id": "legacy-acc-42",
        "provider_id": "openai",
        "api_key": "sk-old-key",
        "is_active": true,
        "priority": 0,
        "auth_strategy_type": "api_key",
        "access_token": null,
        "refresh_token": null,
        "id_token": null,
        "token_expires_at": null,
        "created_at": 1700000000,
        "last_used_at": null
    }"#;

    // Deserialize as AccountData (flat format), then convert to Account (enum format)
    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct AccountData {
        id: String,
        provider_id: String,
        api_key: Option<String>,
        is_active: bool,
        priority: i32,
        auth_strategy_type: String,
        access_token: Option<String>,
        refresh_token: Option<String>,
        id_token: Option<String>,
        token_expires_at: Option<u64>,
        created_at: Option<u64>,
        last_used_at: Option<u64>,
    }

    let data: AccountData = serde_json::from_str(legacy_json).unwrap();

    // Convert to Account using the same logic as the persistence bridge
    let auth_method = if data.auth_strategy_type == "api_key" || data.api_key.is_some() {
        rust_llm_api_router::domain::entities::AuthMethod::ApiKey {
            api_key: data.api_key.unwrap_or_default(),
        }
    } else {
        rust_llm_api_router::domain::entities::AuthMethod::OAuth {
            access_token: data.access_token.unwrap_or_default(),
            refresh_token: data.refresh_token,
            id_token: data.id_token,
            token_expires_at: data.token_expires_at,
        }
    };
    let account = Account::new_api_key(
        data.id,
        data.provider_id,
        match &auth_method {
            rust_llm_api_router::domain::entities::AuthMethod::ApiKey { api_key } => {
                api_key.clone()
            }
            _ => String::new(),
        },
    );

    assert_eq!(account.id, "legacy-acc-42");
    assert_eq!(account.provider_id, "openai");
    assert!(account.is_active);
    assert_eq!(account.auth_method.api_key(), Some("sk-old-key"));
}

/// Verify legacy Provider JSON loads correctly.
#[test]
fn test_legacy_provider_json_loads() {
    let legacy_json = r#"{
        "id": "anthropic",
        "name": "Anthropic",
        "base_url": "https://api.anthropic.com",
        "enabled": true,
        "oauth_client_id": null,
        "oauth_client_secret": null,
        "oauth_redirect_uri": null,
        "oauth_scopes": null
    }"#;

    let provider: Provider = serde_json::from_str(legacy_json).unwrap();

    assert_eq!(provider.id, "anthropic");
    assert_eq!(provider.name, "Anthropic");
    assert!(provider.enabled);
}

// =============================================================================
// PROptest INVARIANTS — Property-based round-trip validation
// =============================================================================

proptest! {
    /// INVARIANT: AccountId Serialize → Deserialize must be identity.
    /// For any string input, the round-trip must preserve the value exactly.
    #[test]
    fn account_id_roundtrip_is_identity(input in r"[a-zA-Z0-9_\-\.@]{1,256}") {
        let id = AccountId::from(input.as_str());
        let json = serde_json::to_string(&id).unwrap();
        let restored: AccountId = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(id, restored);
    }

    /// INVARIANT: ProviderId Serialize → Deserialize must be identity.
    #[test]
    fn provider_id_roundtrip_is_identity(input in r"[a-zA-Z0-9_\-\.]{1,128}") {
        let id = ProviderId::from(input.as_str());
        let json = serde_json::to_string(&id).unwrap();
        let restored: ProviderId = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(id, restored);
    }

    /// INVARIANT: AccountId JSON must be a plain string, never an object.
    #[test]
    fn account_id_serializes_as_plain_string(input in r"[a-zA-Z0-9_\-]{1,128}") {
        let id = AccountId::from(input.as_str());
        let json = serde_json::to_value(&id).unwrap();
        prop_assert!(json.is_string(), "AccountId must serialize as string, got: {:?}", json);
        prop_assert_eq!(json.as_str().unwrap(), input);
    }

    /// INVARIANT: ProviderId JSON must be a plain string, never an object.
    #[test]
    fn provider_id_serializes_as_plain_string(input in r"[a-zA-Z0-9_\-]{1,64}") {
        let id = ProviderId::from(input.as_str());
        let json = serde_json::to_value(&id).unwrap();
        prop_assert!(json.is_string(), "ProviderId must serialize as string, got: {:?}", json);
        prop_assert_eq!(json.as_str().unwrap(), input);
    }

    /// INVARIANT: PartialEq must be consistent with inner string equality.
    #[test]
    fn partialeq_consistent_with_string(input in r"[a-zA-Z0-9_\-]{1,128}") {
        let id = AccountId::from(input.as_str());
        prop_assert_eq!(id.clone(), input.as_str());
        prop_assert_eq!(id.clone(), input.clone());
        prop_assert_eq!(id.as_str(), input.as_str());
    }

    /// INVARIANT: Account round-trip preserves all fields for any valid input.
    #[test]
    fn account_full_roundtrip(
        id in r"[a-z0-9\-]{1,64}",
        provider in r"(openai|anthropic|groq|mistral)",
        api_key in r"sk-[a-zA-Z0-9]{10,64}",
        priority in 0i32..100,
    ) {
        let original = Account::new_api_key(id.as_str(), provider.as_str(), api_key.as_str())
            .with_priority(priority);

        let json = serde_json::to_string(&original).unwrap();
        let restored: Account = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(original, restored.clone());
        prop_assert_eq!(&restored.id, id.as_str());
        prop_assert_eq!(&restored.provider_id, provider.as_str());
        prop_assert_eq!(restored.priority, priority);
    }

    /// INVARIANT: Zeroize clears AccountId inner string.
    #[test]
    fn account_id_zeroize_clears(input in r"[a-zA-Z0-9_\-]{1,256}") {
        let mut id = AccountId::from(input.as_str());
        use zeroize::Zeroize;
        id.zeroize();
        prop_assert_eq!(id.as_str(), "");
        prop_assert!(id.as_str().is_empty());
    }

    /// INVARIANT: Zeroize clears ProviderId inner string.
    #[test]
    fn provider_id_zeroize_clears(input in r"[a-zA-Z0-9_\-]{1,128}") {
        let mut id = ProviderId::from(input.as_str());
        use zeroize::Zeroize;
        id.zeroize();
        prop_assert_eq!(id.as_str(), "");
        prop_assert!(id.as_str().is_empty());
    }
}

// =============================================================================
// EDGE CASES — Boundary conditions and malformed inputs
// =============================================================================

/// Empty string: AccountId and ProviderId must accept empty strings
/// (validation is a separate concern — types must not panic).
#[test]
fn test_empty_string_ids() {
    let empty_account = AccountId::from("");
    let empty_provider = ProviderId::from("");

    assert_eq!(empty_account.as_str(), "");
    assert_eq!(empty_provider.as_str(), "");

    // Round-trip must work even for empty strings
    let json = serde_json::to_string(&empty_account).unwrap();
    let restored: AccountId = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.as_str(), "");
}

/// Unicode: IDs with non-ASCII characters must round-trip correctly.
#[test]
fn test_unicode_ids() {
    let unicode_id = AccountId::from("café-日本-🆔");
    let json = serde_json::to_string(&unicode_id).unwrap();
    let restored: AccountId = serde_json::from_str(&json).unwrap();
    assert_eq!(restored, unicode_id);
    assert_eq!(restored.as_str(), "café-日本-🆔");
}

/// Special characters: IDs with JSON-sensitive chars must not break.
#[test]
fn test_special_char_ids() {
    let special_ids = vec![
        "id-with-\"quotes\"",
        "id-with-\\backslash",
        "id-with-\nnewline",
        "id-with-\ttab",
        "id-with-unicode-\u{1F600}",
    ];

    for raw in special_ids {
        let id = AccountId::from(raw);
        // Must not panic during serialization
        let json = serde_json::to_string(&id).unwrap();
        // Must round-trip correctly
        let restored: AccountId = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, id, "Round-trip failed for: {:?}", raw);
    }
}

/// Long strings: Verify no truncation or buffer overflow.
#[test]
fn test_long_ids() {
    let long_str = "a".repeat(10_000);
    let id = AccountId::from(long_str.as_str());

    assert_eq!(id.as_str().len(), 10_000);

    let json = serde_json::to_string(&id).unwrap();
    let restored: AccountId = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.as_str().len(), 10_000);
    assert_eq!(restored, id);
}

/// PartialEq edge cases: cross-type comparisons must be consistent.
#[test]
fn test_partialeq_edge_cases() {
    let id = AccountId::from("test-id");

    // All these must be equal
    assert_eq!(id, "test-id");
    assert_eq!(id, "test-id".to_string());
    assert_eq!(id, *"test-id");
    assert_eq!("test-id", id); // reverse &str

    // These must NOT be equal
    assert_ne!(id, "other-id");
    assert_ne!(id, "test-id-different");
    assert_ne!(id, "TEST-ID"); // case sensitive
}

/// FromStr: parsing must work like From<&str>.
#[test]
fn test_from_str_consistency() {
    let input = "parse-test-123";

    let from_str: AccountId = input.parse().unwrap();
    let from_into = AccountId::from(input);

    assert_eq!(from_str, from_into);
    assert_eq!(from_str.as_str(), input);
}

/// Display: output must match inner string.
#[test]
fn test_display_matches_inner() {
    let id = AccountId::from("display-test");
    assert_eq!(format!("{}", id), "display-test");
    assert_eq!(id.to_string(), "display-test");
}

/// Serde transparent: JSON must be indistinguishable from plain String.
#[test]
fn test_serde_transparent_indistinguishable() {
    let id = AccountId::from("serde-test");

    // Serialize as AccountId
    let newtype_json = serde_json::to_string(&id).unwrap();

    // Serialize as plain String
    let string_json = serde_json::to_string("serde-test").unwrap();

    // Must be identical
    assert_eq!(
        newtype_json, string_json,
        "serde(transparent) must produce identical JSON to plain String"
    );
}
