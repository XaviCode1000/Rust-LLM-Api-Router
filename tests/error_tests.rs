//! Tests for error types

use rust_llm_api_router::Error;
use rust_llm_api_router::domain::errors::DomainError;

#[test]
fn test_error_display_provider_not_found() {
    let error = Error::ProviderNotFound("test-provider".to_string());
    let display = format!("{}", error);
    assert!(display.contains("test-provider"));
}

#[test]
fn test_error_display_internal() {
    let error = Error::Internal("something went wrong".to_string());
    let display = format!("{}", error);
    assert!(display.contains("something went wrong"));
}

#[test]
fn test_error_display_invalid_request() {
    let error = Error::InvalidRequest("bad input".to_string());
    let display = format!("{}", error);
    assert!(display.contains("bad input"));
}

#[test]
fn test_error_from_envy() {
    // Test that we can convert from envy error
    let _error: Error = envy::Error::Custom("test".to_string()).into();
}

#[test]
fn test_error_from_io() {
    // Test that we can convert from io error
    let _error: Error = std::io::Error::new(std::io::ErrorKind::Other, "test").into();
}

#[test]
fn test_error_from_string_utf8() {
    // Test that we can convert from String::from_utf8 error
    let _error: Error = String::from_utf8(vec![0, 159, 146, 150]).unwrap_err().into();
}

#[test]
fn test_error_domain_conversion() {
    // Test that DomainError converts to Error
    let domain_error = DomainError::AccountNotFound("test".to_string());
    let error: Error = domain_error.into();
    assert!(matches!(error, Error::Domain(_)));
}

#[test]
fn test_error_debug() {
    let error = Error::Internal("test".to_string());
    let debug = format!("{:?}", error);
    assert!(debug.contains("Internal"));
}

#[test]
fn test_error_source() {
    use std::error::Error as StdError;
    let error = Error::Internal("test".to_string());
    // Internal errors don't have a source
    assert!(error.source().is_none());
}
