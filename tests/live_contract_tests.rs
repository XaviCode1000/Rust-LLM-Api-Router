//! Live contract tests for provider APIs.
//!
//! These tests hit the REAL provider APIs (OpenAI, Anthropic, Groq, Mistral, DeepSeek,
//! Google, Cohere, Ollama) to detect schema drift that wiremock static mocks cannot catch.
//! They are:
//! - Marked `#[ignore]` so they never run in normal `cargo test`
//! - Gated by `LIVE_TEST=1` env var
//! - Require individual provider API key env vars
//! - Use insta snapshots with redactions for variable fields
//!
//! Run with:
//! ```bash
//! LIVE_TEST=1 OPENAI_API_KEY=sk-xxx ANTHROPIC_API_KEY=sk-xxx GROQ_API_KEY=gsk-xxx \
//! MISTRAL_API_KEY=xxx DEEPSEEK_API_KEY=xxx GOOGLE_API_KEY=xxx COHERE_API_KEY=xxx \
//!   cargo test --test live_contract_tests -- --ignored
//! ```
//!
//! For Ollama (local), ensure Ollama is running with a model pulled:
//! ```bash
//! ollama pull llama3.2
//! LIVE_TEST=1 cargo test --test live_contract_tests -- --ignored
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
        Ok("1") | Ok("true") => {}
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
    assert!(
        body.get("id").and_then(Value::as_str).is_some(),
        "missing 'id' (string)"
    );
    assert!(
        body.get("model").and_then(Value::as_str).is_some(),
        "missing 'model' (string)"
    );
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
    assert!(
        !choices.is_empty(),
        "'choices' array must have at least 1 element"
    );

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
    settings.add_redaction(".choices", "[choices-redacted]");
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
        .header("anthropic-version", "2023-06-01")
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
    assert!(
        body.get("id").and_then(Value::as_str).is_some(),
        "missing 'id' (string)"
    );

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

    assert!(
        body.get("model").and_then(Value::as_str).is_some(),
        "missing 'model' (string)"
    );

    let content = body
        .get("content")
        .and_then(Value::as_array)
        .expect("missing 'content' (array)");
    assert!(
        !content.is_empty(),
        "'content' array must have at least 1 element"
    );

    let first = &content[0];
    assert!(
        first.get("type").and_then(Value::as_str).is_some(),
        "missing 'content[0].type'"
    );
    assert!(
        first.get("text").and_then(Value::as_str).is_some(),
        "missing 'content[0].text'"
    );

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
    // Note: Live API responses have variable fields (timestamps, tokens, content).
    // We redact these to ensure snapshot stability across runs.
    let mut settings = insta::Settings::clone_current();
    settings.add_redaction(".id", "[redacted]");
    settings.add_redaction(".created", "[redacted]");
    settings.add_redaction(".choices", "[choices-redacted]");
    settings.add_redaction(".usage", "[usage-redacted]");
    settings.add_redaction(".system_fingerprint", "[redacted]");
    settings.add_redaction(".x_groq", "[x_groq-redacted]");
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
    assert!(
        body.get("id").and_then(Value::as_str).is_some(),
        "missing 'id' (string)"
    );
    assert!(
        body.get("model").and_then(Value::as_str).is_some(),
        "missing 'model' (string)"
    );
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
    assert!(
        !choices.is_empty(),
        "'choices' array must have at least 1 element"
    );

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
    // Note: Live API responses have variable fields (timestamps, tokens, content).
    // We redact these to ensure snapshot stability across runs.
    let mut settings = insta::Settings::clone_current();
    settings.add_redaction(".id", "[redacted]");
    settings.add_redaction(".created", "[redacted]");
    settings.add_redaction(".choices", "[choices-redacted]");
    settings.add_redaction(".system_fingerprint", "[redacted]");
    settings.add_redaction(".usage", "[usage-redacted]");
    settings.add_redaction(".x_groq", "[x_groq-redacted]");
    settings.bind(|| {
        insta::assert_json_snapshot!("groq_contract", body);
    });
}

// ============================================================================
// Mistral Contract Test
// ============================================================================

/// Live contract test against the Mistral Chat Completions API.
///
/// Validates that the response schema contains all expected fields.
#[tokio::test]
#[ignore]
async fn test_mistral_contract() {
    require_live_test();
    let api_key =
        require_env_var("MISTRAL_API_KEY").expect("MISTRAL_API_KEY env var not set or empty");

    let client = http_client();
    let response = client
        .post("https://api.mistral.ai/v1/chat/completions")
        .bearer_auth(&api_key)
        .header("content-type", "application/json")
        .json(&json!({
            "model": "mistral-small-latest",
            "messages": [{"role": "user", "content": "Say hello"}],
            "max_tokens": 10
        }))
        .send()
        .await
        .expect("Mistral request failed");

    let status = response.status();
    assert!(status.is_success(), "Mistral returned status {}", status);

    let body: Value = response
        .json()
        .await
        .expect("failed to parse Mistral response");

    // ── Schema assertions (OpenAI-compatible format) ─────────────────────
    assert!(
        body.get("id").and_then(Value::as_str).is_some(),
        "missing 'id' (string)"
    );
    assert!(
        body.get("model").and_then(Value::as_str).is_some(),
        "missing 'model' (string)"
    );
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
    assert!(
        !choices.is_empty(),
        "'choices' array must have at least 1 element"
    );

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
    settings.add_redaction(".choices", "[choices-redacted]");
    settings.bind(|| {
        insta::assert_json_snapshot!("mistral_contract", body);
    });
}

// ============================================================================
// DeepSeek Contract Test
// ============================================================================

/// Live contract test against the DeepSeek Chat Completions API.
///
/// DeepSeek uses an OpenAI-compatible format.
#[tokio::test]
#[ignore]
async fn test_deepseek_contract() {
    require_live_test();
    let api_key =
        require_env_var("DEEPSEEK_API_KEY").expect("DEEPSEEK_API_KEY env var not set or empty");

    let client = http_client();
    let response = client
        .post("https://api.deepseek.com/chat/completions")
        .bearer_auth(&api_key)
        .header("content-type", "application/json")
        .json(&json!({
            "model": "deepseek-chat",
            "messages": [{"role": "user", "content": "Say hello"}],
            "max_tokens": 10
        }))
        .send()
        .await
        .expect("DeepSeek request failed");

    let status = response.status();
    assert!(status.is_success(), "DeepSeek returned status {}", status);

    let body: Value = response
        .json()
        .await
        .expect("failed to parse DeepSeek response");

    // ── Schema assertions (OpenAI-compatible format) ─────────────────────
    assert!(
        body.get("id").and_then(Value::as_str).is_some(),
        "missing 'id' (string)"
    );
    assert!(
        body.get("model").and_then(Value::as_str).is_some(),
        "missing 'model' (string)"
    );
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
    assert!(
        !choices.is_empty(),
        "'choices' array must have at least 1 element"
    );

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
    settings.add_redaction(".choices", "[choices-redacted]");
    settings.bind(|| {
        insta::assert_json_snapshot!("deepseek_contract", body);
    });
}

// ============================================================================
// Google Contract Test (Gemini via OpenAI compatibility)
// ============================================================================

/// Live contract test against the Google Gemini API via OpenAI compatibility.
///
/// Uses the `gemini-2.0-flash` or similar model.
#[tokio::test]
#[ignore]
async fn test_google_contract() {
    require_live_test();
    let api_key =
        require_env_var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY env var not set or empty");

    let client = http_client();
    let response = client
        .post("https://generativelanguage.googleapis.com/v1beta/openai/chat/completions")
        .query(&[("key", api_key.as_str())])
        .header("content-type", "application/json")
        .json(&json!({
            "model": "gemini-2.0-flash",
            "messages": [{"role": "user", "content": "Say hello"}],
            "max_tokens": 10
        }))
        .send()
        .await
        .expect("Google request failed");

    let status = response.status();
    assert!(status.is_success(), "Google returned status {}", status);

    let body: Value = response
        .json()
        .await
        .expect("failed to parse Google response");

    // ── Schema assertions (OpenAI-compatible format) ─────────────────────
    assert!(
        body.get("id").and_then(Value::as_str).is_some(),
        "missing 'id' (string)"
    );
    assert!(
        body.get("model").and_then(Value::as_str).is_some(),
        "missing 'model' (string)"
    );
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
    assert!(
        !choices.is_empty(),
        "'choices' array must have at least 1 element"
    );

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

    // Google may not always return usage in non-streaming mode
    if let Some(usage) = body.get("usage") {
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
    }

    // ── Snapshot with redactions ───────────────────────────────────────
    let mut settings = insta::Settings::clone_current();
    settings.add_redaction(".id", "[redacted]");
    settings.add_redaction(".created", "[redacted]");
    settings.add_redaction(".choices", "[choices-redacted]");
    settings.bind(|| {
        insta::assert_json_snapshot!("google_contract", body);
    });
}

// ============================================================================
// Cohere Contract Test
// ============================================================================

/// Live contract test against the Cohere Chat API.
///
/// Cohere uses its own format, not OpenAI-compatible.
#[tokio::test]
#[ignore]
async fn test_cohere_contract() {
    require_live_test();
    let api_key =
        require_env_var("COHERE_API_KEY").expect("COHERE_API_KEY env var not set or empty");

    let client = http_client();
    let response = client
        .post("https://api.cohere.com/v2/chat")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&json!({
            "model": "command-r-plus-08-2024",
            "message": "Say hello",
            "max_tokens": 10
        }))
        .send()
        .await
        .expect("Cohere request failed");

    let status = response.status();
    assert!(status.is_success(), "Cohere returned status {}", status);

    let body: Value = response
        .json()
        .await
        .expect("failed to parse Cohere response");

    // ── Schema assertions ────────────────────────────────────────────────
    assert!(
        body.get("message").and_then(Value::as_str).is_some(),
        "missing 'message' (string)"
    );

    let chat_history = body.get("chat_history");
    if let Some(history) = chat_history.and_then(Value::as_array) {
        assert!(
            !history.is_empty(),
            "'chat_history' array must have at least 1 element"
        );
    }

    // ── Snapshot with redactions ───────────────────────────────────────
    let mut settings = insta::Settings::clone_current();
    settings.add_redaction(".message", "[redacted]");
    settings.add_redaction(".chat_history", "[chat_history-redacted]");
    settings.bind(|| {
        insta::assert_json_snapshot!("cohere_contract", body);
    });
}

// ============================================================================
// Ollama Contract Test (Local)
// ============================================================================

/// Live contract test against the local Ollama API.
///
/// Requires Ollama to be running locally with a model.
/// Run `ollama pull llama3.2` first.
#[tokio::test]
#[ignore]
async fn test_ollama_contract() {
    require_live_test();

    // Ollama doesn't require an API key for local usage
    let client = http_client();
    let response = client
        .post("http://localhost:11434/api/chat")
        .json(&json!({
            "model": "llama3.2",
            "messages": [{"role": "user", "content": "Say hello"}],
            "stream": false
        }))
        .send()
        .await
        .expect("Ollama request failed - is Ollama running? Run 'ollama pull llama3.2' first");

    let status = response.status();
    assert!(status.is_success(), "Ollama returned status {}", status);

    let body: Value = response
        .json()
        .await
        .expect("failed to parse Ollama response");

    // ── Schema assertions ────────────────────────────────────────────────
    assert!(
        body.get("message").and_then(Value::as_str).is_some(),
        "missing 'message' (string)"
    );

    let msg_content = body.get("message");
    if let Some(msg) = msg_content {
        assert!(msg.get("content").is_some(), "missing 'message.content'");
    }

    // ── Snapshot with redactions ───────────────────────────────────────
    let mut settings = insta::Settings::clone_current();
    settings.add_redaction(".message", "[message-redacted]");
    settings.bind(|| {
        insta::assert_json_snapshot!("ollama_contract", body);
    });
}
