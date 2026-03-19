//! Integration tests para provider commands con wiremock
//!
//! Estos tests verifican la funcionalidad completa de los comandos CLI
//! para gestión de providers, incluyendo validación y persistencia.

use rust_llm_api_router::cli::provider_commands::*;
use rust_llm_api_router::domain::ProviderRepository;
use rust_llm_api_router::infrastructure::{JsonAccountRepository, JsonProviderRepository};
use rust_llm_api_router::Error;
use tempfile::TempDir;

/// Helper para crear repository en directorio temporal
fn create_test_repo() -> (JsonProviderRepository, JsonAccountRepository, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonProviderRepository::with_config_dir(temp_dir.path()).unwrap();
    let account_repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();
    (repo, account_repo, temp_dir)
}

/// Helper para crear AddProviderArgs
fn create_add_args(id: &str, name: &str, base_url: &str, api_key: Option<&str>) -> AddProviderArgs {
    AddProviderArgs {
        id: id.to_string(),
        name: name.to_string(),
        base_url: base_url.to_string(),
        api_key: api_key.map(String::from),
        disabled: false,
        interactive: false,
    }
}

#[tokio::test]
async fn test_cli_add_provider_with_real_validation() {
    let (repo, account_repo, _temp_dir) = create_test_repo();

    // Valid provider with all required fields
    let args = create_add_args(
        "test-provider",
        "Test Provider",
        "https://api.test.com/v1",
        Some("sk-test-key"),
    );

    let result = cmd_add_provider(args, &repo).await;

    assert!(result.is_ok(), "Should add provider successfully");

    // Verify provider was persisted
    let provider = repo.find_by_id("test-provider").await.unwrap();
    assert_eq!(provider.name, "Test Provider");
    assert_eq!(provider.base_url, "https://api.test.com/v1");
    assert!(provider.enabled);
}

#[tokio::test]
async fn test_cli_add_provider_url_validation() {
    let (repo, account_repo, _temp_dir) = create_test_repo();

    // Invalid URL - should still add (validation is minimal)
    let args = create_add_args(
        "bad-url",
        "Bad URL Provider",
        "not-a-valid-url",
        Some("sk-key"),
    );

    // Current implementation doesn't validate URL format strictly
    let result = cmd_add_provider(args, &repo).await;
    assert!(
        result.is_ok(),
        "Should add provider even with invalid URL format"
    );

    // Empty URL - should add (no validation)
    let args2 = create_add_args("empty-url", "Empty URL Provider", "", Some("sk-key"));

    let result2 = cmd_add_provider(args2, &repo).await;
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_cli_add_provider_without_api_key() {
    let (repo, account_repo, _temp_dir) = create_test_repo();

    // Provider without API key (warning should be printed)
    let args = create_add_args(
        "no-key-provider",
        "No Key Provider",
        "https://api.nokey.com",
        None,
    );

    let result = cmd_add_provider(args, &repo).await;

    // Should succeed but print warning
    assert!(result.is_ok());

    // Verify provider was added
    let provider = repo.find_by_id("no-key-provider").await.unwrap();
    assert_eq!(provider.name, "No Key Provider");
}

#[tokio::test]
async fn test_cli_add_provider_disabled_flag() {
    let (repo, account_repo, _temp_dir) = create_test_repo();

    let mut args = create_add_args(
        "disabled-provider",
        "Disabled Provider",
        "https://api.disabled.com",
        Some("sk-key"),
    );
    args.disabled = true;

    let result = cmd_add_provider(args, &repo).await;
    assert!(result.is_ok());

    // Verify provider is disabled
    let provider = repo.find_by_id("disabled-provider").await.unwrap();
    assert!(!provider.enabled);
}

#[tokio::test]
async fn test_cli_add_duplicate_provider() {
    let (repo, account_repo, _temp_dir) = create_test_repo();

    // Add provider first time
    let args1 = create_add_args(
        "duplicate",
        "Duplicate Provider",
        "https://api.dup1.com",
        Some("sk-key1"),
    );
    let result1 = cmd_add_provider(args1, &repo).await;
    assert!(result1.is_ok());

    // Add same provider ID again - should overwrite
    let args2 = create_add_args(
        "duplicate",
        "Duplicate Provider Updated",
        "https://api.dup2.com",
        Some("sk-key2"),
    );
    let result2 = cmd_add_provider(args2, &repo).await;
    assert!(result2.is_ok());

    // Verify updated values
    let provider = repo.find_by_id("duplicate").await.unwrap();
    assert_eq!(provider.name, "Duplicate Provider Updated");
    assert_eq!(provider.base_url, "https://api.dup2.com");
}

#[tokio::test]
async fn test_cli_list_providers_formatting() {
    let (repo, account_repo, _temp_dir) = create_test_repo();

    // Add multiple providers
    let args1 = create_add_args("prov-1", "Provider 1", "https://api.1.com", Some("key-1"));
    let args2 = create_add_args("prov-2", "Provider 2", "https://api.2.com", Some("key-2"));
    let args3 = create_add_args("prov-3", "Provider 3", "https://api.3.com", Some("key-3"));

    cmd_add_provider(args1, &repo).await.unwrap();
    cmd_add_provider(args2, &repo).await.unwrap();
    cmd_add_provider(args3, &repo).await.unwrap();

    // List providers - should not error
    let result = cmd_list_providers(&repo, &account_repo).await;
    assert!(result.is_ok());

    // Verify all providers exist
    let providers = repo.find_all().await.unwrap();
    assert_eq!(providers.len(), 3);

    let provider_ids: Vec<&String> = providers.iter().map(|p| &p.id).collect();
    assert!(provider_ids.contains(&&"prov-1".to_string()));
    assert!(provider_ids.contains(&&"prov-2".to_string()));
    assert!(provider_ids.contains(&&"prov-3".to_string()));
}

#[tokio::test]
async fn test_cli_list_empty_providers() {
    let (repo, account_repo, _temp_dir) = create_test_repo();

    // List with no providers
    let result = cmd_list_providers(&repo, &account_repo).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cli_enable_disable_provider_workflow() {
    let (repo, account_repo, _temp_dir) = create_test_repo();

    // Add provider
    let args = create_add_args(
        "test-enable",
        "Test Enable",
        "https://api.enable.com",
        Some("sk-key"),
    );
    cmd_add_provider(args, &repo).await.unwrap();

    // Verify initially enabled
    let provider = repo.find_by_id("test-enable").await.unwrap();
    assert!(provider.enabled);

    // Disable provider
    let disable_args = DisableProviderArgs {
        id: "test-enable".to_string(),
    };
    let result = cmd_disable_provider(disable_args, &repo).await;
    assert!(result.is_ok());

    // Verify disabled
    let provider = repo.find_by_id("test-enable").await.unwrap();
    assert!(!provider.enabled);

    // Enable provider
    let enable_args = EnableProviderArgs {
        id: "test-enable".to_string(),
    };
    let result = cmd_enable_provider(enable_args, &repo).await;
    assert!(result.is_ok());

    // Verify enabled again
    let provider = repo.find_by_id("test-enable").await.unwrap();
    assert!(provider.enabled);
}

#[tokio::test]
async fn test_cli_enable_nonexistent_provider() {
    let (repo, account_repo, _temp_dir) = create_test_repo();

    let args = EnableProviderArgs {
        id: "nonexistent".to_string(),
    };

    let result = cmd_enable_provider(args, &repo).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        Error::ProviderNotFound(id) => assert_eq!(id, "nonexistent"),
        _ => panic!("Should return ProviderNotFound error"),
    }
}

#[tokio::test]
async fn test_cli_disable_nonexistent_provider() {
    let (repo, account_repo, _temp_dir) = create_test_repo();

    let args = DisableProviderArgs {
        id: "nonexistent".to_string(),
    };

    let result = cmd_disable_provider(args, &repo).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        Error::ProviderNotFound(id) => assert_eq!(id, "nonexistent"),
        _ => panic!("Should return ProviderNotFound error"),
    }
}

#[tokio::test]
async fn test_cli_remove_provider_workflow() {
    let (repo, account_repo, _temp_dir) = create_test_repo();

    // Add provider
    let args = create_add_args(
        "to-remove",
        "To Remove",
        "https://api.remove.com",
        Some("sk-key"),
    );
    cmd_add_provider(args, &repo).await.unwrap();

    // Verify exists
    let provider = repo.find_by_id("to-remove").await.unwrap();
    assert_eq!(provider.name, "To Remove");

    // Remove provider
    let remove_args = RemoveProviderArgs {
        id: "to-remove".to_string(),
    };
    let result = cmd_remove_provider(remove_args, &repo).await;
    assert!(result.is_ok());

    // Verify removed - should return error now
    let result = repo.find_by_id("to-remove").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_cli_remove_nonexistent_provider() {
    let (repo, account_repo, _temp_dir) = create_test_repo();

    let args = RemoveProviderArgs {
        id: "nonexistent".to_string(),
    };

    let result = cmd_remove_provider(args, &repo).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        Error::ProviderNotFound(id) => assert_eq!(id, "nonexistent"),
        _ => panic!("Should return ProviderNotFound error"),
    }
}

#[tokio::test]
async fn test_cli_remove_and_readd_provider() {
    let (repo, account_repo, _temp_dir) = create_test_repo();

    // Add provider
    let args1 = create_add_args(
        "readd",
        "Readd Provider",
        "https://api.readd1.com",
        Some("sk-key1"),
    );
    cmd_add_provider(args1, &repo).await.unwrap();

    // Remove provider
    let remove_args = RemoveProviderArgs {
        id: "readd".to_string(),
    };
    cmd_remove_provider(remove_args, &repo).await.unwrap();

    // Re-add with same ID but different data
    let args2 = create_add_args(
        "readd",
        "Readd Provider New",
        "https://api.readd2.com",
        Some("sk-key2"),
    );
    cmd_add_provider(args2, &repo).await.unwrap();

    // Verify new data
    let provider = repo.find_by_id("readd").await.unwrap();
    assert_eq!(provider.name, "Readd Provider New");
    assert_eq!(provider.base_url, "https://api.readd2.com");
}

#[tokio::test]
async fn test_cli_provider_persistence_across_repo_reloads() {
    let temp_dir = TempDir::new().unwrap();

    // Create repo and add provider
    let repo1 = JsonProviderRepository::with_config_dir(temp_dir.path()).unwrap();
    let args = create_add_args(
        "persistent",
        "Persistent Provider",
        "https://api.persistent.com",
        Some("sk-key"),
    );
    cmd_add_provider(args, &repo1).await.unwrap();

    // Create new repo instance (simulates restart)
    let repo2 = JsonProviderRepository::with_config_dir(temp_dir.path()).unwrap();

    // Verify provider persists
    let provider = repo2.find_by_id("persistent").await.unwrap();
    assert_eq!(provider.name, "Persistent Provider");
    assert_eq!(provider.base_url, "https://api.persistent.com");
}

#[tokio::test]
async fn test_cli_provider_with_special_characters() {
    let (repo, account_repo, _temp_dir) = create_test_repo();

    // Provider with special characters in name
    let args = create_add_args(
        "special-chars",
        "Provider with spaces & symbols!",
        "https://api.special.com",
        Some("sk-key"),
    );

    let result = cmd_add_provider(args, &repo).await;
    assert!(result.is_ok());

    // Verify provider was added with special characters
    let provider = repo.find_by_id("special-chars").await.unwrap();
    assert_eq!(provider.name, "Provider with spaces & symbols!");
}

#[tokio::test]
async fn test_cli_provider_with_long_name() {
    let (repo, account_repo, _temp_dir) = create_test_repo();

    let long_name = "A".repeat(200); // 200 character name
    let args = create_add_args(
        "long-name",
        &long_name,
        "https://api.long.com",
        Some("sk-key"),
    );

    let result = cmd_add_provider(args, &repo).await;
    assert!(result.is_ok());

    // Verify long name was stored
    let provider = repo.find_by_id("long-name").await.unwrap();
    assert_eq!(provider.name.len(), 200);
}

#[tokio::test]
async fn test_cli_provider_url_with_different_schemes() {
    let (repo, account_repo, _temp_dir) = create_test_repo();

    // HTTP URL
    let args1 = create_add_args(
        "http-provider",
        "HTTP Provider",
        "http://api.http.com",
        Some("sk-key"),
    );
    cmd_add_provider(args1, &repo).await.unwrap();

    // HTTPS URL
    let args2 = create_add_args(
        "https-provider",
        "HTTPS Provider",
        "https://api.https.com",
        Some("sk-key"),
    );
    cmd_add_provider(args2, &repo).await.unwrap();

    // URL with port
    let args3 = create_add_args(
        "port-provider",
        "Port Provider",
        "https://api.port.com:8080/v1",
        Some("sk-key"),
    );
    cmd_add_provider(args3, &repo).await.unwrap();

    // Verify all URLs stored correctly
    let providers = repo.find_all().await.unwrap();
    assert_eq!(providers.len(), 3);

    let http_provider = providers.iter().find(|p| p.id == "http-provider").unwrap();
    assert_eq!(http_provider.base_url, "http://api.http.com");

    let https_provider = providers.iter().find(|p| p.id == "https-provider").unwrap();
    assert_eq!(https_provider.base_url, "https://api.https.com");

    let port_provider = providers.iter().find(|p| p.id == "port-provider").unwrap();
    assert_eq!(port_provider.base_url, "https://api.port.com:8080/v1");
}

#[tokio::test]
async fn test_cli_provider_enable_disable_multiple_times() {
    let (repo, account_repo, _temp_dir) = create_test_repo();

    // Add provider
    let args = create_add_args(
        "toggle",
        "Toggle Provider",
        "https://api.toggle.com",
        Some("sk-key"),
    );
    cmd_add_provider(args, &repo).await.unwrap();

    // Toggle multiple times
    for i in 0..5 {
        if i % 2 == 0 {
            // Disable
            let disable_args = DisableProviderArgs {
                id: "toggle".to_string(),
            };
            cmd_disable_provider(disable_args, &repo).await.unwrap();

            let provider = repo.find_by_id("toggle").await.unwrap();
            assert!(
                !provider.enabled,
                "Should be disabled after iteration {}",
                i
            );
        } else {
            // Enable
            let enable_args = EnableProviderArgs {
                id: "toggle".to_string(),
            };
            cmd_enable_provider(enable_args, &repo).await.unwrap();

            let provider = repo.find_by_id("toggle").await.unwrap();
            assert!(provider.enabled, "Should be enabled after iteration {}", i);
        }
    }
}

#[tokio::test]
async fn test_cli_provider_validate_command() {
    let (repo, account_repo, _temp_dir) = create_test_repo();

    // Add provider
    let args = create_add_args(
        "validate-test",
        "Validate Test Provider",
        "https://httpbin.org", // Use httpbin for testing
        Some("sk-key"),
    );
    cmd_add_provider(args, &repo).await.unwrap();

    // Validate provider (should attempt to reach URL)
    let validate_args = ValidateProviderArgs {
        id: "validate-test".to_string(),
    };
    let result = cmd_validate_provider(validate_args, &repo).await;

    // Should succeed (may be reachable or not, but command should complete)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cli_provider_validate_nonexistent() {
    let (repo, account_repo, _temp_dir) = create_test_repo();

    let args = ValidateProviderArgs {
        id: "nonexistent".to_string(),
    };

    let result = cmd_validate_provider(args, &repo).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        Error::ProviderNotFound(id) => assert_eq!(id, "nonexistent"),
        _ => panic!("Should return ProviderNotFound error"),
    }
}

#[tokio::test]
async fn test_cli_provider_validate_disabled_provider() {
    let (repo, account_repo, _temp_dir) = create_test_repo();

    // Add and disable provider
    let mut args = create_add_args(
        "disabled-validate",
        "Disabled Validate",
        "https://api.disabled.com",
        Some("sk-key"),
    );
    args.disabled = true;
    cmd_add_provider(args, &repo).await.unwrap();

    // Try to validate disabled provider
    let validate_args = ValidateProviderArgs {
        id: "disabled-validate".to_string(),
    };
    let result = cmd_validate_provider(validate_args, &repo).await;

    // Should succeed but print warning
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cli_provider_crud_complete_workflow() {
    let (repo, account_repo, _temp_dir) = create_test_repo();

    // CREATE
    let add_args = create_add_args(
        "crud-test",
        "CRUD Test Provider",
        "https://api.crud.com",
        Some("sk-key"),
    );
    cmd_add_provider(add_args, &repo).await.unwrap();

    // Verify created
    let provider = repo.find_by_id("crud-test").await.unwrap();
    assert_eq!(provider.name, "CRUD Test Provider");

    // UPDATE (disable)
    let disable_args = DisableProviderArgs {
        id: "crud-test".to_string(),
    };
    cmd_disable_provider(disable_args, &repo).await.unwrap();

    let provider = repo.find_by_id("crud-test").await.unwrap();
    assert!(!provider.enabled);

    // UPDATE (enable)
    let enable_args = EnableProviderArgs {
        id: "crud-test".to_string(),
    };
    cmd_enable_provider(enable_args, &repo).await.unwrap();

    let provider = repo.find_by_id("crud-test").await.unwrap();
    assert!(provider.enabled);

    // DELETE
    let remove_args = RemoveProviderArgs {
        id: "crud-test".to_string(),
    };
    cmd_remove_provider(remove_args, &repo).await.unwrap();

    // Verify deleted - should return error
    let result = repo.find_by_id("crud-test").await;
    assert!(result.is_err());
}
