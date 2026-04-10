//! Tests for LLM router backward compatibility
//!
//! These tests cover the backward-compatible route_request function.

use rust_llm_api_router::app::router::llm_router::route_request;

// ============================================================================
// LLM Router Backward Compatibility Tests
// ============================================================================

#[tokio::test]
async fn test_llm_router_route_request_stub() {
    // The route_request function returns an error indicating full router is needed
    let request = serde_json::json!({"model": "gpt-4", "messages": []});
    let result = route_request("openai", request).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(format!("{err}").contains("full routing"));
}

#[tokio::test]
async fn test_llm_router_with_empty_request() {
    let request = serde_json::json!({});
    let result = route_request("", request).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_llm_router_with_different_providers() {
    let providers = vec!["openai", "anthropic", "groq"];

    for provider in providers {
        let request = serde_json::json!({
            "model": "test-model",
            "messages": []
        });
        let result = route_request(provider, request).await;
        assert!(result.is_err(), "Should error for provider: {provider}");
    }
}
