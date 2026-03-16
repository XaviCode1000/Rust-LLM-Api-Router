//! Tests for domain error types
//!
//! Comprehensive tests for all error types in the domain layer.

#![allow(clippy::unnecessary_literal_unwrap)]

use crate::domain::errors::{DomainError, DomainResult};

// ============================================================================
// DOMAIN ERROR DISPLAY TESTS
// ============================================================================

#[test]
fn test_domain_error_invalid_request_display() {
    let err = DomainError::InvalidRequest("bad request".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("invalid request"));
    assert!(msg.contains("bad request"));
}

#[test]
fn test_domain_error_provider_not_found_display() {
    let err = DomainError::ProviderNotFound("provider-123".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("provider not found"));
    assert!(msg.contains("provider-123"));
}

#[test]
fn test_domain_error_provider_disabled_display() {
    let err = DomainError::ProviderDisabled("disabled-provider".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("provider disabled"));
    assert!(msg.contains("disabled-provider"));
}

#[test]
fn test_domain_error_account_not_found_display() {
    let err = DomainError::AccountNotFound("acc-456".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("account not found"));
    assert!(msg.contains("acc-456"));
}

#[test]
fn test_domain_error_account_inactive_display() {
    let err = DomainError::AccountInactive("inactive-acc".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("account inactive"));
    assert!(msg.contains("inactive-acc"));
}

#[test]
fn test_domain_error_no_available_accounts_display() {
    let err = DomainError::NoAvailableAccounts("provider-x".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("no available accounts"));
    assert!(msg.contains("provider-x"));
}

#[test]
fn test_domain_error_model_not_found_display() {
    let err = DomainError::ModelNotFound("model-789".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("model not found"));
    assert!(msg.contains("model-789"));
}

#[test]
fn test_domain_error_gateway_error_display() {
    let err = DomainError::GatewayError("connection timeout".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("gateway error"));
    assert!(msg.contains("connection timeout"));
}

#[test]
fn test_domain_error_authentication_error_display() {
    let err = DomainError::AuthenticationError("invalid token".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("authentication error"));
    assert!(msg.contains("invalid token"));
}

#[test]
fn test_domain_error_rate_limited_display() {
    let err = DomainError::RateLimited("too many requests".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("rate limited"));
    assert!(msg.contains("too many requests"));
}

#[test]
fn test_domain_error_validation_error_display() {
    let err = DomainError::ValidationError("email is invalid".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("validation error"));
    assert!(msg.contains("email is invalid"));
}

#[test]
fn test_domain_error_io_display() {
    let err = DomainError::Io("file not found".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("I/O error"));
    assert!(msg.contains("file not found"));
}

#[test]
fn test_domain_error_serialization_display() {
    let err = DomainError::Serialization("invalid JSON".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("serialization error"));
    assert!(msg.contains("invalid JSON"));
}

#[test]
fn test_domain_error_external_service_error_display() {
    let err = DomainError::ExternalServiceError("API down".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("external service error"));
    assert!(msg.contains("API down"));
}

#[test]
fn test_domain_error_not_implemented_display() {
    let err = DomainError::NotImplemented("feature X".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("not implemented"));
    assert!(msg.contains("feature X"));
}

#[test]
fn test_domain_error_internal_display() {
    let err = DomainError::Internal("something went wrong".to_string());
    let msg = format!("{}", err);
    assert!(msg.contains("internal error"));
    assert!(msg.contains("something went wrong"));
}

// ============================================================================
// DOMAIN ERROR CONSTRUCTOR TESTS
// ============================================================================

#[test]
fn test_domain_error_invalid_request_constructor() {
    let err = DomainError::invalid_request("test message");
    assert!(matches!(err, DomainError::InvalidRequest(_)));
}

#[test]
fn test_domain_error_provider_not_found_constructor() {
    let err = DomainError::provider_not_found("test-provider");
    assert!(matches!(err, DomainError::ProviderNotFound(_)));
}

#[test]
fn test_domain_error_provider_disabled_constructor() {
    let err = DomainError::provider_disabled("test-provider");
    assert!(matches!(err, DomainError::ProviderDisabled(_)));
}

#[test]
fn test_domain_error_account_not_found_constructor() {
    let err = DomainError::account_not_found("test-account");
    assert!(matches!(err, DomainError::AccountNotFound(_)));
}

#[test]
fn test_domain_error_account_inactive_constructor() {
    let err = DomainError::account_inactive("test-account");
    assert!(matches!(err, DomainError::AccountInactive(_)));
}

#[test]
fn test_domain_error_no_available_accounts_constructor() {
    let err = DomainError::no_available_accounts("test-provider");
    assert!(matches!(err, DomainError::NoAvailableAccounts(_)));
}

#[test]
fn test_domain_error_model_not_found_constructor() {
    let err = DomainError::model_not_found("test-model");
    assert!(matches!(err, DomainError::ModelNotFound(_)));
}

#[test]
fn test_domain_error_gateway_error_constructor() {
    let err = DomainError::gateway_error("timeout");
    assert!(matches!(err, DomainError::GatewayError(_)));
}

#[test]
fn test_domain_error_authentication_error_constructor() {
    let err = DomainError::authentication_error("bad token");
    assert!(matches!(err, DomainError::AuthenticationError(_)));
}

#[test]
fn test_domain_error_rate_limited_constructor() {
    let err = DomainError::rate_limited("exceeded limit");
    assert!(matches!(err, DomainError::RateLimited(_)));
}

#[test]
fn test_domain_error_validation_error_constructor() {
    let err = DomainError::validation_error("invalid format");
    assert!(matches!(err, DomainError::ValidationError(_)));
}

#[test]
fn test_domain_error_io_constructor() {
    let err = DomainError::io("disk full");
    assert!(matches!(err, DomainError::Io(_)));
}

#[test]
fn test_domain_error_serialization_constructor() {
    let err = DomainError::serialization("bad JSON");
    assert!(matches!(err, DomainError::Serialization(_)));
}

#[test]
fn test_domain_error_external_service_error_constructor() {
    let err = DomainError::external_service_error("503");
    assert!(matches!(err, DomainError::ExternalServiceError(_)));
}

#[test]
fn test_domain_error_not_implemented_constructor() {
    let err = DomainError::not_implemented("streaming");
    assert!(matches!(err, DomainError::NotImplemented(_)));
}

#[test]
fn test_domain_error_internal_constructor() {
    let err = DomainError::internal("unexpected");
    assert!(matches!(err, DomainError::Internal(_)));
}

// ============================================================================
// DOMAIN RESULT TYPE TESTS
// ============================================================================

#[test]
fn test_domain_result_ok_variant() {
    let result: DomainResult<i32> = Ok(42);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_domain_result_err_variant() {
    let result: DomainResult<i32> = Err(DomainError::Internal("error".to_string()));
    assert!(result.is_err());
    assert!(matches!(result, Err(DomainError::Internal(_))));
}

#[test]
fn test_domain_result_map_ok() {
    let result: DomainResult<i32> = Ok(42);
    let mapped = result.map(|x| x * 2);
    assert_eq!(mapped.unwrap(), 84);
}

#[test]
fn test_domain_result_map_err() {
    let result: DomainResult<i32> = Err(DomainError::Internal("error".to_string()));
    let mapped = result.map_err(|e| DomainError::ValidationError(e.to_string()));
    assert!(matches!(mapped, Err(DomainError::ValidationError(_))));
}

#[test]
fn test_domain_result_and() {
    let ok1: DomainResult<i32> = Ok(1);
    let ok2: DomainResult<i32> = Ok(2);

    assert!(ok1.and(Ok(2)).is_ok());
    assert!(Ok::<_, DomainError>(1).and(ok2).is_ok());
    assert!(Err::<i32, _>(DomainError::Internal("e".to_string()))
        .and(Ok(1))
        .is_err());
}

#[test]
fn test_domain_result_or() {
    let ok1: DomainResult<i32> = Ok(1);
    let ok2: DomainResult<i32> = Ok(2);

    assert_eq!(ok1.unwrap_or(2), 1);
    assert!(Err::<i32, _>(DomainError::Internal("e".to_string()))
        .or(ok2)
        .is_ok());
    assert!(Err::<i32, _>(DomainError::Internal("e1".to_string()))
        .or(Err(DomainError::Internal("e2".to_string())))
        .is_err());
}

#[test]
fn test_domain_result_is_ok() {
    let ok: DomainResult<i32> = Ok(42);
    let err: DomainResult<i32> = Err(DomainError::Internal("error".to_string()));

    assert!(ok.is_ok());
    assert!(err.is_err());
}

#[test]
fn test_domain_result_is_err() {
    let ok: DomainResult<i32> = Ok(42);
    let err: DomainResult<i32> = Err(DomainError::Internal("error".to_string()));

    assert!(ok.is_ok());
    assert!(err.is_err());
}

// ============================================================================
// DEBUG TRAIT TESTS
// ============================================================================

#[test]
fn test_domain_error_debug_format() {
    let err = DomainError::InvalidRequest("test".to_string());
    let debug = format!("{:?}", err);
    assert!(debug.contains("InvalidRequest"));
    assert!(debug.contains("test"));
}

#[test]
fn test_domain_error_debug_all_variants() {
    // Verify all variants have proper Debug implementation
    let errors = vec![
        DomainError::InvalidRequest("test".to_string()),
        DomainError::ProviderNotFound("test".to_string()),
        DomainError::ProviderDisabled("test".to_string()),
        DomainError::AccountNotFound("test".to_string()),
        DomainError::AccountInactive("test".to_string()),
        DomainError::NoAvailableAccounts("test".to_string()),
        DomainError::ModelNotFound("test".to_string()),
        DomainError::GatewayError("test".to_string()),
        DomainError::AuthenticationError("test".to_string()),
        DomainError::RateLimited("test".to_string()),
        DomainError::ValidationError("test".to_string()),
        DomainError::Io("test".to_string()),
        DomainError::Serialization("test".to_string()),
        DomainError::ExternalServiceError("test".to_string()),
        DomainError::NotImplemented("test".to_string()),
        DomainError::Internal("test".to_string()),
    ];

    for err in errors {
        let debug = format!("{:?}", err);
        assert!(!debug.is_empty(), "Debug format should not be empty");
    }
}
