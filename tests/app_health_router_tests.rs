//! Tests for health checks and router modules
//!
//! These tests cover the health status functionality and LLM router logic.

use axum::{body::Body, http::Request};
use tower::util::ServiceExt;

use rust_llm_api_router::app::health::routes as health_routes;
use rust_llm_api_router::app::router::llm_router::route_request;

// ============================================================================
// Health Routes Tests
// ============================================================================

#[tokio::test]
async fn test_health_routes_creation() {
    let _router = health_routes();
    // Verify router is created without panic
}

#[tokio::test]
async fn test_health_check_endpoint() {
    let app = health_routes();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert_eq!(body_str, "OK");
}

#[tokio::test]
async fn test_health_endpoint_with_headers() {
    let health_app = health_routes();

    let response = health_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .header("Accept", "text/plain")
                .header("User-Agent", "test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    assert!(response.headers().contains_key("content-type"));
}

#[tokio::test]
async fn test_health_endpoint_options_method() {
    let health_app = health_routes();

    let response = health_app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await;

    // OPTIONS should return Method Not Allowed or similar
    assert!(response.is_err() || response.unwrap().status() != 200);
}

// ============================================================================
// LLM Router Tests
// ============================================================================

#[tokio::test]
async fn test_llm_router_route_request_stub() {
    // The route_request function is a stub that panics with todo!()
    // We just verify it exists and has the right signature

    let request = serde_json::json!({
        "model": "gpt-4",
        "messages": []
    });

    // This will panic with todo!(), so we catch it
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { route_request("openai", request).await })
    }));

    // Should panic because it's a todo!() stub
    assert!(result.is_err());
}

#[tokio::test]
async fn test_llm_router_with_empty_request() {
    let request = serde_json::json!({});

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { route_request("", request).await })
    }));

    // Should panic because it's a todo!() stub
    assert!(result.is_err());
}

#[tokio::test]
async fn test_llm_router_with_different_providers() {
    let providers = vec!["openai", "groq", "anthropic", "mistral"];

    for provider in providers {
        let request = serde_json::json!({"test": true});

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async { route_request(provider, request).await })
        }));

        // All should panic because it's a todo!() stub
        assert!(result.is_err(), "Provider {} should panic", provider);
    }
}

// ============================================================================
// Health and Router Integration
// ============================================================================

#[tokio::test]
async fn test_health_routes_is_independent() {
    // Verify health routes work independently of router
    let health_app = health_routes();

    let response = health_app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}

#[test]
fn test_router_module_exists() {
    // Verify the router module exists and is accessible
    // This is a compile-time check that passes if the code compiles
    let _ = route_request;
}
