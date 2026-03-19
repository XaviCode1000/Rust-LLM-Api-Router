//! Tests for CLI provider commands
//!
//! Tests verify provider management functionality:
//! - add: Add new providers
//! - list: List providers
//! - remove: Delete providers
//! - enable/disable: Toggle provider state
//! - validate: Validate provider credentials

use tempfile::TempDir;

use rust_llm_api_router::cli::provider_commands::{
    cmd_add_provider, cmd_disable_provider, cmd_enable_provider, cmd_list_providers,
    cmd_remove_provider, cmd_validate_provider, AddProviderArgs, DisableProviderArgs,
    EnableProviderArgs, ProviderCommands, RemoveProviderArgs, ValidateProviderArgs,
};
use rust_llm_api_router::domain::traits::ProviderRepository;
use rust_llm_api_router::domain::Provider;
use rust_llm_api_router::infrastructure::{JsonAccountRepository, JsonProviderRepository};

// ============================================================================
// Helper Functions
// ============================================================================

fn create_test_repo() -> (TempDir, JsonProviderRepository, JsonAccountRepository) {
    let temp_dir = TempDir::new().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());
    let repo = JsonProviderRepository::with_config_dir(temp_dir.path()).unwrap();
    let account_repo = JsonAccountRepository::with_config_dir(temp_dir.path()).unwrap();
    (temp_dir, repo, account_repo)
}

// ============================================================================
// Add Provider Tests
// ============================================================================

#[tokio::test]
async fn test_cli_add_provider_success_enabled() {
    let (_temp_dir, repo, account_repo) = create_test_repo();

    let args = AddProviderArgs {
        id: "test-provider".to_string(),
        name: "Test Provider".to_string(),
        base_url: "https://api.test.com/v1".to_string(),
        api_key: Some("test-api-key".to_string()),
        disabled: false,
        interactive: false,
    };

    let result = cmd_add_provider(args, &repo).await;

    assert!(result.is_ok());

    let provider = repo.find_by_id("test-provider").await.unwrap();
    assert_eq!(provider.id, "test-provider");
    assert_eq!(provider.name, "Test Provider");
    assert_eq!(provider.base_url, "https://api.test.com/v1");
    assert!(provider.enabled);
}

#[tokio::test]
async fn test_cli_add_provider_success_disabled() {
    let (_temp_dir, repo, account_repo) = create_test_repo();

    let args = AddProviderArgs {
        id: "disabled-provider".to_string(),
        name: "Disabled Provider".to_string(),
        base_url: "https://api.disabled.com/v1".to_string(),
        api_key: Some("test-key".to_string()),
        disabled: true,
        interactive: false,
    };

    let result = cmd_add_provider(args, &repo).await;

    assert!(result.is_ok());

    let provider = repo.find_by_id("disabled-provider").await.unwrap();
    assert!(!provider.enabled);
}

#[tokio::test]
async fn test_cli_add_provider_empty_api_key_warning() {
    let (_temp_dir, repo, account_repo) = create_test_repo();

    let args = AddProviderArgs {
        id: "no-key-provider".to_string(),
        name: "No Key Provider".to_string(),
        base_url: "https://api.nokey.com/v1".to_string(),
        api_key: None,
        disabled: false,
        interactive: false,
    };

    let result = cmd_add_provider(args, &repo).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cli_add_provider_duplicate_overwrites() {
    let (_temp_dir, repo, account_repo) = create_test_repo();

    // Add first provider
    let args1 = AddProviderArgs {
        id: "dup-provider".to_string(),
        name: "Original".to_string(),
        base_url: "https://original.com/v1".to_string(),
        api_key: Some("key-1".to_string()),
        disabled: false,
        interactive: false,
    };
    cmd_add_provider(args1, &repo).await.unwrap();

    // Add duplicate - should overwrite
    let args2 = AddProviderArgs {
        id: "dup-provider".to_string(),
        name: "Updated".to_string(),
        base_url: "https://updated.com/v1".to_string(),
        api_key: Some("key-2".to_string()),
        disabled: false,
        interactive: false,
    };
    let result = cmd_add_provider(args2, &repo).await;

    assert!(result.is_ok());

    let provider = repo.find_by_id("dup-provider").await.unwrap();
    assert_eq!(provider.name, "Updated");
}

// ============================================================================
// List Providers Tests
// ============================================================================

#[tokio::test]
async fn test_cli_list_providers_empty() {
    let (_temp_dir, repo, account_repo) = create_test_repo();

    let result = cmd_list_providers(&repo, &account_repo).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cli_list_providers_with_data() {
    let (_temp_dir, repo, account_repo) = create_test_repo();

    repo.save(Provider::new("prov-1", "Provider 1", "https://prov1.com"))
        .await
        .unwrap();
    repo.save(Provider::new("prov-2", "Provider 2", "https://prov2.com"))
        .await
        .unwrap();
    repo.save(Provider::disabled(
        "prov-3",
        "Provider 3",
        "https://prov3.com",
    ))
    .await
    .unwrap();

    let result = cmd_list_providers(&repo, &account_repo).await;

    assert!(result.is_ok());

    let providers = repo.find_all().await.unwrap();
    assert_eq!(providers.len(), 3);
}

#[tokio::test]
async fn test_cli_list_providers_displays_enabled_disabled() {
    let (_temp_dir, repo, account_repo) = create_test_repo();

    repo.save(Provider::new(
        "enabled-prov",
        "Enabled",
        "https://enabled.com",
    ))
    .await
    .unwrap();
    repo.save(Provider::disabled(
        "disabled-prov",
        "Disabled",
        "https://disabled.com",
    ))
    .await
    .unwrap();

    let result = cmd_list_providers(&repo, &account_repo).await;

    assert!(result.is_ok());
}

// ============================================================================
// Remove Provider Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_cli_remove_provider_success() {
    let (_temp_dir, repo, account_repo) = create_test_repo();

    repo.save(Provider::new(
        "to-remove",
        "To Remove",
        "https://remove.com",
    ))
    .await
    .unwrap();

    // Verify exists
    assert!(repo.find_by_id("to-remove").await.is_ok());

    // Remove
    let args = RemoveProviderArgs {
        id: "to-remove".to_string(),
    };
    let result = cmd_remove_provider(args, &repo).await;

    assert!(result.is_ok());

    // Verify removed
    assert!(repo.find_by_id("to-remove").await.is_err());
}

#[tokio::test]
async fn test_cli_remove_provider_not_found() {
    let (_temp_dir, repo, account_repo) = create_test_repo();

    let args = RemoveProviderArgs {
        id: "non-existent".to_string(),
    };
    let result = cmd_remove_provider(args, &repo).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        rust_llm_api_router::Error::ProviderNotFound(_)
    ));
}

#[tokio::test]
#[ignore]
async fn test_cli_remove_provider_from_multiple() {
    let (_temp_dir, repo, account_repo) = create_test_repo();

    repo.save(Provider::new("prov-1", "Provider 1", "https://prov1.com"))
        .await
        .unwrap();
    repo.save(Provider::new("prov-2", "Provider 2", "https://prov2.com"))
        .await
        .unwrap();
    repo.save(Provider::new("prov-3", "Provider 3", "https://prov3.com"))
        .await
        .unwrap();

    // Remove middle one
    let args = RemoveProviderArgs {
        id: "prov-2".to_string(),
    };
    cmd_remove_provider(args, &repo).await.unwrap();

    // Verify others remain
    assert!(repo.find_by_id("prov-1").await.is_ok());
    assert!(repo.find_by_id("prov-2").await.is_err());
    assert!(repo.find_by_id("prov-3").await.is_ok());

    let all = repo.find_all().await.unwrap();
    assert_eq!(all.len(), 2);
}

// ============================================================================
// Enable Provider Tests
// ============================================================================

#[tokio::test]
async fn test_cli_enable_provider_success() {
    let (_temp_dir, repo, account_repo) = create_test_repo();

    repo.save(Provider::disabled(
        "disabled-to-enable",
        "Disabled",
        "https://disabled.com",
    ))
    .await
    .unwrap();

    let args = EnableProviderArgs {
        id: "disabled-to-enable".to_string(),
    };
    let result = cmd_enable_provider(args, &repo).await;

    assert!(result.is_ok());

    let provider = repo.find_by_id("disabled-to-enable").await.unwrap();
    assert!(provider.enabled);
}

#[tokio::test]
async fn test_cli_enable_provider_already_enabled() {
    let (_temp_dir, repo, account_repo) = create_test_repo();

    repo.save(Provider::new(
        "already-enabled",
        "Enabled",
        "https://enabled.com",
    ))
    .await
    .unwrap();

    let args = EnableProviderArgs {
        id: "already-enabled".to_string(),
    };
    let result = cmd_enable_provider(args, &repo).await;

    assert!(result.is_ok());

    let provider = repo.find_by_id("already-enabled").await.unwrap();
    assert!(provider.enabled);
}

#[tokio::test]
async fn test_cli_enable_provider_not_found() {
    let (_temp_dir, repo, account_repo) = create_test_repo();

    let args = EnableProviderArgs {
        id: "non-existent".to_string(),
    };
    let result = cmd_enable_provider(args, &repo).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        rust_llm_api_router::Error::ProviderNotFound(_)
    ));
}

// ============================================================================
// Disable Provider Tests
// ============================================================================

#[tokio::test]
async fn test_cli_disable_provider_success() {
    let (_temp_dir, repo, account_repo) = create_test_repo();

    repo.save(Provider::new(
        "enabled-to-disable",
        "Enabled",
        "https://enabled.com",
    ))
    .await
    .unwrap();

    let args = DisableProviderArgs {
        id: "enabled-to-disable".to_string(),
    };
    let result = cmd_disable_provider(args, &repo).await;

    assert!(result.is_ok());

    let provider = repo.find_by_id("enabled-to-disable").await.unwrap();
    assert!(!provider.enabled);
}

#[tokio::test]
async fn test_cli_disable_provider_already_disabled() {
    let (_temp_dir, repo, account_repo) = create_test_repo();

    repo.save(Provider::disabled(
        "already-disabled",
        "Disabled",
        "https://disabled.com",
    ))
    .await
    .unwrap();

    let args = DisableProviderArgs {
        id: "already-disabled".to_string(),
    };
    let result = cmd_disable_provider(args, &repo).await;

    assert!(result.is_ok());

    let provider = repo.find_by_id("already-disabled").await.unwrap();
    assert!(!provider.enabled);
}

#[tokio::test]
async fn test_cli_disable_provider_not_found() {
    let (_temp_dir, repo, account_repo) = create_test_repo();

    let args = DisableProviderArgs {
        id: "non-existent".to_string(),
    };
    let result = cmd_disable_provider(args, &repo).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        rust_llm_api_router::Error::ProviderNotFound(_)
    ));
}

// ============================================================================
// Validate Provider Tests
// ============================================================================

#[tokio::test]
async fn test_cli_validate_provider_success() {
    let (_temp_dir, repo, account_repo) = create_test_repo();

    repo.save(Provider::new(
        "validate-prov",
        "Validate Me",
        "https://httpbin.org",
    ))
    .await
    .unwrap();

    let args = ValidateProviderArgs {
        id: "validate-prov".to_string(),
    };
    let result = cmd_validate_provider(args, &repo).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cli_validate_provider_disabled_warning() {
    let (_temp_dir, repo, account_repo) = create_test_repo();

    repo.save(Provider::disabled(
        "disabled-validate",
        "Disabled",
        "https://disabled.com",
    ))
    .await
    .unwrap();

    let args = ValidateProviderArgs {
        id: "disabled-validate".to_string(),
    };
    let result = cmd_validate_provider(args, &repo).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cli_validate_provider_not_found() {
    let (_temp_dir, repo, account_repo) = create_test_repo();

    let args = ValidateProviderArgs {
        id: "non-existent".to_string(),
    };
    let result = cmd_validate_provider(args, &repo).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        rust_llm_api_router::Error::ProviderNotFound(_)
    ));
}

#[tokio::test]
async fn test_cli_validate_provider_unreachable_url() {
    let (_temp_dir, repo, account_repo) = create_test_repo();

    // Use a URL that will fail
    repo.save(Provider::new(
        "unreachable",
        "Unreachable",
        "http://localhost:1",
    ))
    .await
    .unwrap();

    let args = ValidateProviderArgs {
        id: "unreachable".to_string(),
    };
    let result = cmd_validate_provider(args, &repo).await;

    // Should still succeed (just logs the failure)
    assert!(result.is_ok());
}

// ============================================================================
// Provider Commands Enum Tests
// ============================================================================

#[test]
fn test_provider_commands_add_variant() {
    let cmd = ProviderCommands::Add(AddProviderArgs {
        id: "test".to_string(),
        name: "Test".to_string(),
        base_url: "https://test.com".to_string(),
        api_key: Some("key".to_string()),
        disabled: false,
        interactive: false,
    });

    match cmd {
        ProviderCommands::Add(args) => {
            assert_eq!(args.id, "test");
        }
        _ => panic!("Expected Add command"),
    }
}

#[test]
fn test_provider_commands_list_variant() {
    let cmd = ProviderCommands::List;

    match cmd {
        ProviderCommands::List => {} // OK
        _ => panic!("Expected List command"),
    }
}

#[test]
fn test_provider_commands_remove_variant() {
    let cmd = ProviderCommands::Remove(RemoveProviderArgs {
        id: "to-remove".to_string(),
    });

    match cmd {
        ProviderCommands::Remove(args) => {
            assert_eq!(args.id, "to-remove");
        }
        _ => panic!("Expected Remove command"),
    }
}

#[test]
fn test_provider_commands_enable_variant() {
    let cmd = ProviderCommands::Enable(EnableProviderArgs {
        id: "to-enable".to_string(),
    });

    match cmd {
        ProviderCommands::Enable(args) => {
            assert_eq!(args.id, "to-enable");
        }
        _ => panic!("Expected Enable command"),
    }
}

#[test]
fn test_provider_commands_disable_variant() {
    let cmd = ProviderCommands::Disable(DisableProviderArgs {
        id: "to-disable".to_string(),
    });

    match cmd {
        ProviderCommands::Disable(args) => {
            assert_eq!(args.id, "to-disable");
        }
        _ => panic!("Expected Disable command"),
    }
}

#[test]
fn test_provider_commands_validate_variant() {
    let cmd = ProviderCommands::Validate(ValidateProviderArgs {
        id: "to-validate".to_string(),
    });

    match cmd {
        ProviderCommands::Validate(args) => {
            assert_eq!(args.id, "to-validate");
        }
        _ => panic!("Expected Validate command"),
    }
}

// ============================================================================
// Multiple Providers Integration Tests
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_cli_multiple_providers_workflow() {
    let (_temp_dir, repo, account_repo) = create_test_repo();

    // Add multiple providers
    cmd_add_provider(
        AddProviderArgs {
            id: "prov-a".to_string(),
            name: "Provider A".to_string(),
            base_url: "https://a.com".to_string(),
            api_key: Some("key-a".to_string()),
            disabled: false,
            interactive: false,
        },
        &repo,
    )
    .await
    .unwrap();

    cmd_add_provider(
        AddProviderArgs {
            id: "prov-b".to_string(),
            name: "Provider B".to_string(),
            base_url: "https://b.com".to_string(),
            api_key: Some("key-b".to_string()),
            disabled: true,
            interactive: false,
        },
        &repo,
    )
    .await
    .unwrap();

    // Verify both exist
    let providers = repo.find_all().await.unwrap();
    assert_eq!(providers.len(), 2);

    // Enable provider B
    cmd_enable_provider(
        EnableProviderArgs {
            id: "prov-b".to_string(),
        },
        &repo,
    )
    .await
    .unwrap();

    let prov_b = repo.find_by_id("prov-b").await.unwrap();
    assert!(prov_b.enabled);

    // Disable provider A
    cmd_disable_provider(
        DisableProviderArgs {
            id: "prov-a".to_string(),
        },
        &repo,
    )
    .await
    .unwrap();

    let prov_a = repo.find_by_id("prov-a").await.unwrap();
    assert!(!prov_a.enabled);

    // Remove provider A
    cmd_remove_provider(
        RemoveProviderArgs {
            id: "prov-a".to_string(),
        },
        &repo,
    )
    .await
    .unwrap();

    // Verify final state
    let providers = repo.find_all().await.unwrap();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].id, "prov-b");
}
