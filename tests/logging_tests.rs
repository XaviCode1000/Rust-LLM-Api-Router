//! Tests for logging infrastructure

#[test]
fn test_logging_init() {
    // Test that logging initialization compiles and has correct signature
    // Note: Can only initialize once per process
    rust_llm_api_router::infrastructure::logging::init_logging("info");
}
