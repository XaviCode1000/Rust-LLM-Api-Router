//! Extended tests para CLI provider commands
//!
//! Tests adicionales para cubrir casos edge y error handling
//! que no están en cli_provider_commands_tests.rs

use tempfile::TempDir;

use rust_llm_api_router::cli::provider_commands::{
    cmd_add_provider, cmd_disable_provider, cmd_enable_provider, cmd_list_providers,
    cmd_remove_provider, cmd_validate_provider, AddProviderArgs, DisableProviderArgs,
    EnableProviderArgs, ProviderCommands, RemoveProviderArgs, ValidateProviderArgs,
};
use rust_llm_api_router::domain::traits::ProviderRepository;
use rust_llm_api_router::domain::Provider;
use rust_llm_api_router::infrastructure::JsonProviderRepository;

// ============================================================================
// Helper Functions
// ============================================================================

fn create_test_repo() -> (TempDir, JsonProviderRepository) {
    let temp_dir = TempDir::new().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());
    let repo = JsonProviderRepository::with_config_dir(temp_dir.path()).unwrap();
    (temp_dir, repo)
}

// ============================================================================
// REMOVE PROVIDER - Bug Fix Tests (Previously Ignored)
// ============================================================================

#[tokio::test]
async fn test_cli_remove_provider_from_single_provider_list() {
    // This test was failing before the bug fix
    // Bug: cmd_remove_provider didn't properly handle empty list after deletion
    let (temp_dir, repo) = create_test_repo();

    // Add single provider
    let add_args = AddProviderArgs {
        id: "only-provider".to_string(),
        name: "Only Provider".to_string(),
        base_url: "https://api.only.com".to_string(),
        api_key: Some("key".to_string()),
        disabled: false,
        interactive: false,
    };
    cmd_add_provider(add_args, &repo).await.unwrap();

    // Verify exists
    let providers_before = repo.find_all().await.unwrap();
    assert_eq!(providers_before.len(), 1);

    // Remove the only provider (this was failing before)
    let args = RemoveProviderArgs {
        id: "only-provider".to_string(),
    };
    let result = cmd_remove_provider(args, &repo).await;

    assert!(result.is_ok());

    // Verify empty list
    let providers_after = repo.find_all().await.unwrap();
    assert!(providers_after.is_empty());

    // Verify persisted - create new repo instance
    let repo2 = JsonProviderRepository::with_config_dir(temp_dir.path()).unwrap();
    let providers2 = repo2.find_all().await.unwrap();
    assert!(providers2.is_empty());
}

#[tokio::test]
async fn test_cli_remove_provider_last_one_with_error_handling() {
    // Test that removing last provider doesn't cause errors
    let (temp_dir, repo) = create_test_repo();

    // Add single provider
    let add_args = AddProviderArgs {
        id: "last-one".to_string(),
        name: "Last One".to_string(),
        base_url: "https://api.last.com".to_string(),
        api_key: Some("key".to_string()),
        disabled: false,
        interactive: false,
    };
    cmd_add_provider(add_args, &repo).await.unwrap();

    // Remove and verify no errors with empty list
    let args = RemoveProviderArgs {
        id: "last-one".to_string(),
    };
    cmd_remove_provider(args, &repo).await.unwrap();

    // Try to remove again (should fail gracefully)
    let result = cmd_remove_provider(
        RemoveProviderArgs {
            id: "last-one".to_string(),
        },
        &repo,
    )
    .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_cli_remove_provider_from_multiple() {
    // Test removing one provider from a list of multiple
    let (_temp_dir, repo) = create_test_repo();

    // Add multiple providers
    cmd_add_provider(
        AddProviderArgs {
            id: "prov-1".to_string(),
            name: "Provider 1".to_string(),
            base_url: "https://prov1.com".to_string(),
            api_key: Some("key-1".to_string()),
            disabled: false,
            interactive: false,
        },
        &repo,
    )
    .await
    .unwrap();

    cmd_add_provider(
        AddProviderArgs {
            id: "prov-2".to_string(),
            name: "Provider 2".to_string(),
            base_url: "https://prov2.com".to_string(),
            api_key: Some("key-2".to_string()),
            disabled: false,
            interactive: false,
        },
        &repo,
    )
    .await
    .unwrap();

    cmd_add_provider(
        AddProviderArgs {
            id: "prov-3".to_string(),
            name: "Provider 3".to_string(),
            base_url: "https://prov3.com".to_string(),
            api_key: Some("key-3".to_string()),
            disabled: false,
            interactive: false,
        },
        &repo,
    )
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

#[tokio::test]
async fn test_cli_remove_provider_first_of_many() {
    let (_temp_dir, repo) = create_test_repo();

    // Add three providers
    cmd_add_provider(
        AddProviderArgs {
            id: "first".to_string(),
            name: "First".to_string(),
            base_url: "https://first.com".to_string(),
            api_key: Some("key".to_string()),
            disabled: false,
            interactive: false,
        },
        &repo,
    )
    .await
    .unwrap();

    cmd_add_provider(
        AddProviderArgs {
            id: "second".to_string(),
            name: "Second".to_string(),
            base_url: "https://second.com".to_string(),
            api_key: Some("key".to_string()),
            disabled: false,
            interactive: false,
        },
        &repo,
    )
    .await
    .unwrap();

    cmd_add_provider(
        AddProviderArgs {
            id: "third".to_string(),
            name: "Third".to_string(),
            base_url: "https://third.com".to_string(),
            api_key: Some("key".to_string()),
            disabled: false,
            interactive: false,
        },
        &repo,
    )
    .await
    .unwrap();

    // Remove first one
    let args = RemoveProviderArgs {
        id: "first".to_string(),
    };
    cmd_remove_provider(args, &repo).await.unwrap();

    let providers = repo.find_all().await.unwrap();
    assert_eq!(providers.len(), 2);
    assert_eq!(providers[0].id, "second");
    assert_eq!(providers[1].id, "third");
}

#[tokio::test]
async fn test_cli_remove_provider_last_of_many() {
    let (_temp_dir, repo) = create_test_repo();

    // Add three providers
    cmd_add_provider(
        AddProviderArgs {
            id: "first".to_string(),
            name: "First".to_string(),
            base_url: "https://first.com".to_string(),
            api_key: Some("key".to_string()),
            disabled: false,
            interactive: false,
        },
        &repo,
    )
    .await
    .unwrap();

    cmd_add_provider(
        AddProviderArgs {
            id: "second".to_string(),
            name: "Second".to_string(),
            base_url: "https://second.com".to_string(),
            api_key: Some("key".to_string()),
            disabled: false,
            interactive: false,
        },
        &repo,
    )
    .await
    .unwrap();

    cmd_add_provider(
        AddProviderArgs {
            id: "third".to_string(),
            name: "Third".to_string(),
            base_url: "https://third.com".to_string(),
            api_key: Some("key".to_string()),
            disabled: false,
            interactive: false,
        },
        &repo,
    )
    .await
    .unwrap();

    // Remove last one
    let args = RemoveProviderArgs {
        id: "third".to_string(),
    };
    cmd_remove_provider(args, &repo).await.unwrap();

    let providers = repo.find_all().await.unwrap();
    assert_eq!(providers.len(), 2);
    assert_eq!(providers[0].id, "first");
    assert_eq!(providers[1].id, "second");
}

#[tokio::test]
async fn test_cli_remove_provider_middle_of_many() {
    let (_temp_dir, repo) = create_test_repo();

    // Add three providers
    cmd_add_provider(
        AddProviderArgs {
            id: "first".to_string(),
            name: "First".to_string(),
            base_url: "https://first.com".to_string(),
            api_key: Some("key".to_string()),
            disabled: false,
            interactive: false,
        },
        &repo,
    )
    .await
    .unwrap();

    cmd_add_provider(
        AddProviderArgs {
            id: "second".to_string(),
            name: "Second".to_string(),
            base_url: "https://second.com".to_string(),
            api_key: Some("key".to_string()),
            disabled: false,
            interactive: false,
        },
        &repo,
    )
    .await
    .unwrap();

    cmd_add_provider(
        AddProviderArgs {
            id: "third".to_string(),
            name: "Third".to_string(),
            base_url: "https://third.com".to_string(),
            api_key: Some("key".to_string()),
            disabled: false,
            interactive: false,
        },
        &repo,
    )
    .await
    .unwrap();

    // Remove middle one
    let args = RemoveProviderArgs {
        id: "second".to_string(),
    };
    cmd_remove_provider(args, &repo).await.unwrap();

    let providers = repo.find_all().await.unwrap();
    assert_eq!(providers.len(), 2);
    assert_eq!(providers[0].id, "first");
    assert_eq!(providers[1].id, "third");
}

#[tokio::test]
async fn test_cli_remove_provider_persists_across_instances() {
    let (temp_dir, repo) = create_test_repo();

    // Add provider
    cmd_add_provider(
        AddProviderArgs {
            id: "to-remove".to_string(),
            name: "Remove Me".to_string(),
            base_url: "https://remove.com".to_string(),
            api_key: Some("key".to_string()),
            disabled: false,
            interactive: false,
        },
        &repo,
    )
    .await
    .unwrap();

    // Remove provider
    let args = RemoveProviderArgs {
        id: "to-remove".to_string(),
    };
    cmd_remove_provider(args, &repo).await.unwrap();

    // New instance should also not have it
    let repo2 = JsonProviderRepository::with_config_dir(temp_dir.path()).unwrap();
    let providers = repo2.find_all().await.unwrap();
    assert!(providers.is_empty());
}

// ============================================================================
// Additional Edge Cases
// ============================================================================

#[tokio::test]
async fn test_cli_remove_provider_with_special_characters_in_id() {
    let (_temp_dir, repo) = create_test_repo();

    // Add provider with special characters
    cmd_add_provider(
        AddProviderArgs {
            id: "test-provider-123".to_string(),
            name: "Test Provider".to_string(),
            base_url: "https://test.com".to_string(),
            api_key: Some("key".to_string()),
            disabled: false,
            interactive: false,
        },
        &repo,
    )
    .await
    .unwrap();

    // Remove it
    let args = RemoveProviderArgs {
        id: "test-provider-123".to_string(),
    };
    let result = cmd_remove_provider(args, &repo).await;

    assert!(result.is_ok());

    // Verify removed
    assert!(repo.find_by_id("test-provider-123").await.is_err());
}

#[tokio::test]
async fn test_cli_remove_provider_case_sensitive() {
    let (_temp_dir, repo) = create_test_repo();

    // Add provider
    cmd_add_provider(
        AddProviderArgs {
            id: "MyProvider".to_string(),
            name: "My Provider".to_string(),
            base_url: "https://myprovider.com".to_string(),
            api_key: Some("key".to_string()),
            disabled: false,
            interactive: false,
        },
        &repo,
    )
    .await
    .unwrap();

    // Try to remove with different case (should fail)
    let args = RemoveProviderArgs {
        id: "myprovider".to_string(),
    };
    let result = cmd_remove_provider(args, &repo).await;

    assert!(result.is_err());

    // Verify original still exists
    assert!(repo.find_by_id("MyProvider").await.is_ok());
}

// ============================================================================
// Workflow Tests
// ============================================================================

#[tokio::test]
async fn test_cli_workflow_add_remove_multiple_sequential() {
    let (_temp_dir, repo) = create_test_repo();

    // Add 5 providers
    for i in 0..5 {
        cmd_add_provider(
            AddProviderArgs {
                id: format!("provider-{}", i),
                name: format!("Provider {}", i),
                base_url: format!("https://provider{}.com", i),
                api_key: Some("key".to_string()),
                disabled: false,
                interactive: false,
            },
            &repo,
        )
        .await
        .unwrap();
    }

    // Verify all exist
    let providers = repo.find_all().await.unwrap();
    assert_eq!(providers.len(), 5);

    // Remove them one by one
    for i in 0..5 {
        let args = RemoveProviderArgs {
            id: format!("provider-{}", i),
        };
        cmd_remove_provider(args, &repo).await.unwrap();
    }

    // Verify all removed
    let providers = repo.find_all().await.unwrap();
    assert!(providers.is_empty());
}

#[tokio::test]
async fn test_cli_workflow_add_disable_remove() {
    let (_temp_dir, repo) = create_test_repo();

    // Add provider
    cmd_add_provider(
        AddProviderArgs {
            id: "workflow-provider".to_string(),
            name: "Workflow Provider".to_string(),
            base_url: "https://workflow.com".to_string(),
            api_key: Some("key".to_string()),
            disabled: false,
            interactive: false,
        },
        &repo,
    )
    .await
    .unwrap();

    // Disable it
    cmd_disable_provider(
        DisableProviderArgs {
            id: "workflow-provider".to_string(),
        },
        &repo,
    )
    .await
    .unwrap();

    let provider = repo.find_by_id("workflow-provider").await.unwrap();
    assert!(!provider.enabled);

    // Remove it
    cmd_remove_provider(
        RemoveProviderArgs {
            id: "workflow-provider".to_string(),
        },
        &repo,
    )
    .await
    .unwrap();

    // Verify removed
    assert!(repo.find_by_id("workflow-provider").await.is_err());
}

#[tokio::test]
async fn test_cli_workflow_add_enable_remove() {
    let (_temp_dir, repo) = create_test_repo();

    // Add disabled provider
    cmd_add_provider(
        AddProviderArgs {
            id: "disabled-provider".to_string(),
            name: "Disabled Provider".to_string(),
            base_url: "https://disabled.com".to_string(),
            api_key: Some("key".to_string()),
            disabled: true,
            interactive: false,
        },
        &repo,
    )
    .await
    .unwrap();

    let provider = repo.find_by_id("disabled-provider").await.unwrap();
    assert!(!provider.enabled);

    // Enable it
    cmd_enable_provider(
        EnableProviderArgs {
            id: "disabled-provider".to_string(),
        },
        &repo,
    )
    .await
    .unwrap();

    let provider = repo.find_by_id("disabled-provider").await.unwrap();
    assert!(provider.enabled);

    // Remove it
    cmd_remove_provider(
        RemoveProviderArgs {
            id: "disabled-provider".to_string(),
        },
        &repo,
    )
    .await
    .unwrap();

    // Verify removed
    assert!(repo.find_by_id("disabled-provider").await.is_err());
}
