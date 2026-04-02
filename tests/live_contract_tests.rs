//! Live contract tests for provider APIs.
//!
//! These tests hit the REAL provider APIs (OpenAI, Anthropic, Groq) to detect
//! schema drift that wiremock static mocks cannot catch. They are:
//! - Marked `#[ignore]` so they never run in normal `cargo test`
//! - Gated by `LIVE_TEST=1` env var
//! - Require individual provider API key env vars
//! - Use insta snapshots with redactions for variable fields
//!
//! Run with:
//! ```bash
//! LIVE_TEST=1 OPENAI_API_KEY=sk-xxx ANTHROPIC_API_KEY=sk-xxx GROQ_API_KEY=gsk-xxx \
//!   cargo test --test live_contract_tests -- --ignored
//! ```

use serde_json::{json, Value};

// ============================================================================
// Helpers
// ============================================================================

/// Skip the test if a required environment variable is not set or is empty.
fn require_env_var(key: &str) -> Option<String> {
    match std::env::var(key) {
        Ok(val) if !val.is_empty() => Some(val),
        _ => None,
    }
}

/// Skip the test unless `LIVE_TEST=1` is set.
fn require_live_test() {
    match std::env::var("LIVE_TEST").as_deref() {
        Ok("1") | Ok("true") => {},
        _ => panic!("Skipping: set LIVE_TEST=1 to run live contract tests"),
    }
}

/// Build a reqwest client with a reasonable timeout.
fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("failed to build reqwest client")
}

// ============================================================================
// OpenAI Contract Test
// ============================================================================

/// Live contract test against the OpenAI Chat Completions API.
///
/// Validates that the response schema contains all expected fields
/// and captures a snapshot (with redactions) for drift detection.
#[tokio::test]
#[ignore]
async fn test_openai_contract() {
    require_live_test();
    let api_key =
        require_env_var("OPENAI_API_KEY").expect("OPENAI_API_KEY env var not set or empty");

    let client = http_client();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .bearer_auth(&api_key)
        .header("content-type", "application/json")
        .json(&json!({
            "model": "gpt-3.5-turbo",
            "messages": [{"role": "user", "content": "Say hello"}],
            "max_tokens": 10
        }))
        .send()
        .await
        .expect("OpenAI request failed");

    let status = response.status();
    assert!(status.is_success(), "OpenAI returned status {}", status);

    let body: Value = response
        .json()
        .await
        .expect("failed to parse OpenAI response");

    // ── Schema assertions ──────────────────────────────────────────────
    assert!(body.get("id").and_then(Value::as_str).is_some(), "missing 'id' (string)");
    assert!(body.get("model").and_then(Value::as_str).is_some(), "missing 'model' (string)");
    assert!(
        body.get("object").and_then(Value::as_str).is_some(),
        "missing 'object' (string)"
    );
    assert!(
        body.get("created").and_then(Value::as_u64).is_some(),
        "missing 'created' (number)"
    );

    let choices = body
        .get("choices")
        .and_then(Value::as_array)
        .expect("missing 'choices' (array)");
    assert!(!choices.is_empty(), "'choices' array must have at least 1 element");

    let first = &choices[0];
    assert!(first.get("index").is_some(), "missing 'choices[0].index'");

    let message = first.get("message").expect("missing 'choices[0].message'");
    assert!(
        message.get("role").and_then(Value::as_str).is_some(),
        "missing 'choices[0].message.role'"
    );
    assert!(
        message.get("content").and_then(Value::as_str).is_some(),
        "missing 'choices[0].message.content'"
    );

    let usage = body.get("usage").expect("missing 'usage'");
    assert!(
        usage.get("prompt_tokens").and_then(Value::as_u64).is_some(),
        "missing 'usage.prompt_tokens'"
    );
    assert!(
        usage
            .get("completion_tokens")
            .and_then(Value::as_u64)
            .is_some(),
        "missing 'usage.completion_tokens'"
    );
    assert!(
        usage.get("total_tokens").and_then(Value::as_u64).is_some(),
        "missing 'usage.total_tokens'"
    );

    // ── Snapshot with redactions ───────────────────────────────────────
    let mut settings = insta::Settings::clone_current();
    settings.add_redaction(".id", "[redacted]");
    settings.add_redaction(".created", "[redacted]");
    settings.add_redaction(".choices[*].finish_reason", "[redacted]");
    settings.bind(|| {
        insta::assert_json_snapshot!("openai_contract", body);
    });
}

// ============================================================================
// Anthropic Contract Test
// ============================================================================

/// Live contract test against the Anthropic Messages API.
///
/// Validates that the response schema contains all expected fields
/// and captures a snapshot (with redactions) for drift detection.
#[tokio::test]
#[ignore]
async fn test_anthropic_contract() {
    require_live_test();
    let api_key =
        require_env_var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY env var not set or empty");

    let client = http_client();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2024-06-20")
        .header("content-type", "application/json")
        .json(&json!({
            "model": "claude-3-haiku-20240307",
            "max_tokens": 10,
            "messages": [{"role": "user", "content": "Say hello"}]
        }))
        .send()
        .await
        .expect("Anthropic request failed");

    let status = response.status();
    assert!(status.is_success(), "Anthropic returned status {}", status);

    let body: Value = response
        .json()
        .await
        .expect("failed to parse Anthropic response");

    // ── Schema assertions ──────────────────────────────────────────────
    assert!(body.get("id").and_then(Value::as_str).is_some(), "missing 'id' (string)");

    let r#type = body
        .get("type")
        .and_then(Value::as_str)
        .expect("missing 'type' (string)");
    assert_eq!(r#type, "message", "'type' should be 'message'");

    let role = body
        .get("role")
        .and_then(Value::as_str)
        .expect("missing 'role' (string)");
    assert_eq!(role, "assistant", "'role' should be 'assistant'");

    assert!(body.get("model").and_then(Value::as_str).is_some(), "missing 'model' (string)");

    let content = body
        .get("content")
        .and_then(Value::as_array)
        .expect("missing 'content' (array)");
    assert!(!content.is_empty(), "'content' array must have at least 1 element");

    let first = &content[0];
    assert!(first.get("type").and_then(Value::as_str).is_some(), "missing 'content[0].type'");
    assert!(first.get("text").and_then(Value::as_str).is_some(), "missing 'content[0].text'");

    let usage = body.get("usage").expect("missing 'usage'");
    assert!(
        usage.get("input_tokens").and_then(Value::as_u64).is_some(),
        "missing 'usage.input_tokens'"
    );
    assert!(
        usage.get("output_tokens").and_then(Value::as_u64).is_some(),
        "missing 'usage.output_tokens'"
    );

    // ── Snapshot with redactions ───────────────────────────────────────
    let mut settings = insta::Settings::clone_current();
    settings.add_redaction(".id", "[redacted]");
    settings.add_redaction(".content[*].text", "[redacted-content]");
    settings.add_redaction(".model", "[redacted-model]");
    settings.bind(|| {
        insta::assert_json_snapshot!("anthropic_contract", body);
    });
}

// ============================================================================
// Groq Contract Test
// ============================================================================

/// Live contract test against the Groq Chat Completions API.
///
/// Groq uses an OpenAI-compatible format, so the schema assertions
/// mirror the OpenAI test.
#[tokio::test]
#[ignore]
async fn test_groq_contract() {
    require_live_test();
    let api_key = require_env_var("GROQ_API_KEY").expect("GROQ_API_KEY env var not set or empty");

    let client = http_client();
    let response = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .bearer_auth(&api_key)
        .header("content-type", "application/json")
        .json(&json!({
            "model": "llama-3.1-8b-instant",
            "messages": [{"role": "user", "content": "Say hello"}],
            "max_tokens": 10
        }))
        .send()
        .await
        .expect("Groq request failed");

    let status = response.status();
    assert!(status.is_success(), "Groq returned status {}", status);

    let body: Value = response
        .json()
        .await
        .expect("failed to parse Groq response");

    // ── Schema assertions (OpenAI-compatible format) ───────────────────
    assert!(body.get("id").and_then(Value::as_str).is_some(), "missing 'id' (string)");
    assert!(body.get("model").and_then(Value::as_str).is_some(), "missing 'model' (string)");
    assert!(
        body.get("object").and_then(Value::as_str).is_some(),
        "missing 'object' (string)"
    );
    assert!(
        body.get("created").and_then(Value::as_u64).is_some(),
        "missing 'created' (number)"
    );

    let choices = body
        .get("choices")
        .and_then(Value::as_array)
        .expect("missing 'choices' (array)");
    assert!(!choices.is_empty(), "'choices' array must have at least 1 element");

    let first = &choices[0];
    assert!(first.get("index").is_some(), "missing 'choices[0].index'");

    let message = first.get("message").expect("missing 'choices[0].message'");
    assert!(
        message.get("role").and_then(Value::as_str).is_some(),
        "missing 'choices[0].message.role'"
    );
    assert!(
        message.get("content").and_then(Value::as_str).is_some(),
        "missing 'choices[0].message.content'"
    );

    let usage = body.get("usage").expect("missing 'usage'");
    assert!(
        usage.get("prompt_tokens").and_then(Value::as_u64).is_some(),
        "missing 'usage.prompt_tokens'"
    );
    assert!(
        usage
            .get("completion_tokens")
            .and_then(Value::as_u64)
            .is_some(),
        "missing 'usage.completion_tokens'"
    );
    assert!(
        usage.get("total_tokens").and_then(Value::as_u64).is_some(),
        "missing 'usage.total_tokens'"
    );

    // ── Snapshot with redactions ───────────────────────────────────────
    let mut settings = insta::Settings::clone_current();
    settings.add_redaction(".id", "[redacted]");
    settings.add_redaction(".created", "[redacted]");
    settings.add_redaction(".choices[*].finish_reason", "[redacted]");
    settings.bind(|| {
        insta::assert_json_snapshot!("groq_contract", body);
    });
}
