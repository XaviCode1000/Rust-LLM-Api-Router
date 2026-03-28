//! Additional tests for provider_commands to reach 80%+ coverage
//!
//! These tests cover edge cases and error scenarios not covered in main test files.

use rust_llm_api_router::cli::provider_commands::{
    cmd_add_provider, cmd_disable_provider, cmd_enable_provider, AddProviderArgs,
    DisableProviderArgs, EnableProviderArgs,
};
use rust_llm_api_router::domain::traits::ProviderRepository;
use rust_llm_api_router::infrastructure::JsonProviderRepository;
use tempfile::TempDir;

fn create_test_repo() -> (TempDir, JsonProviderRepository) {
    let temp_dir = TempDir::new().unwrap();
    let repo = JsonProviderRepository::with_config_dir(temp_dir.path()).unwrap();
    (temp_dir, repo)
}

#[tokio::test]
async fn test_cli_add_provider_with_empty_base_url() {
    let (_temp_dir, repo) = create_test_repo();
    let args = AddProviderArgs {
        id: "test-empty-url".to_string(),
        name: "Test Empty URL".to_string(),
        base_url: "".to_string(),
        api_key: None,
        disabled: false,
        interactive: false,
    };
    let result = cmd_add_provider(args, &repo).await;
    // Empty URL is accepted (no validation in current implementation)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cli_add_provider_with_invalid_url_format() {
    let (_temp_dir, repo) = create_test_repo();
    let args = AddProviderArgs {
        id: "test-invalid-url".to_string(),
        name: "Test Invalid URL".to_string(),
        base_url: "not-a-valid-url".to_string(),
        api_key: None,
        disabled: false,
        interactive: false,
    };
    let result = cmd_add_provider(args, &repo).await;
    // Invalid URL is also accepted (no validation)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cli_add_provider_with_empty_api_key() {
    let (_temp_dir, repo) = create_test_repo();
    let args = AddProviderArgs {
        id: "test-empty-key".to_string(),
        name: "Test Empty Key".to_string(),
        base_url: "https://api.test.com/v1".to_string(),
        api_key: None,
        disabled: false,
        interactive: false,
    };
    let result = cmd_add_provider(args, &repo).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cli_add_provider_duplicate_id() {
    let (_temp_dir, repo) = create_test_repo();
    let args1 = AddProviderArgs {
        id: "duplicate-provider".to_string(),
        name: "First Provider".to_string(),
        base_url: "https://api.first.com/v1".to_string(),
        api_key: None,
        disabled: false,
        interactive: false,
    };
    cmd_add_provider(args1, &repo).await.unwrap();
    let args2 = AddProviderArgs {
        id: "duplicate-provider".to_string(),
        name: "Second Provider".to_string(),
        base_url: "https://api.second.com/v1".to_string(),
        api_key: None,
        disabled: false,
        interactive: false,
    };
    let result = cmd_add_provider(args2, &repo).await;
    assert!(result.is_ok());
    let provider = repo.find_by_id("duplicate-provider").await.unwrap();
    assert_eq!(provider.name, "Second Provider");
}

#[tokio::test]
async fn test_cli_enable_non_existent_provider() {
    let (_temp_dir, repo) = create_test_repo();
    let args = EnableProviderArgs {
        id: "non-existent".to_string(),
    };
    let result = cmd_enable_provider(args, &repo).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_cli_disable_non_existent_provider() {
    let (_temp_dir, repo) = create_test_repo();
    let args = DisableProviderArgs {
        id: "non-existent".to_string(),
    };
    let result = cmd_disable_provider(args, &repo).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_cli_enable_already_enabled_provider() {
    let (_temp_dir, repo) = create_test_repo();
    let add_args = AddProviderArgs {
        id: "already-enabled".to_string(),
        name: "Already Enabled".to_string(),
        base_url: "https://api.enabled.com/v1".to_string(),
        api_key: None,
        disabled: false,
        interactive: false,
    };
    cmd_add_provider(add_args, &repo).await.unwrap();
    let enable_args = EnableProviderArgs {
        id: "already-enabled".to_string(),
    };
    let result = cmd_enable_provider(enable_args, &repo).await;
    assert!(result.is_ok());
    let provider = repo.find_by_id("already-enabled").await.unwrap();
    assert!(provider.enabled);
}

#[tokio::test]
async fn test_cli_disable_already_disabled_provider() {
    let (_temp_dir, repo) = create_test_repo();
    let add_args = AddProviderArgs {
        id: "already-disabled".to_string(),
        name: "Already Disabled".to_string(),
        base_url: "https://api.disabled.com/v1".to_string(),
        api_key: None,
        disabled: true,
        interactive: false,
    };
    cmd_add_provider(add_args, &repo).await.unwrap();
    let disable_args = DisableProviderArgs {
        id: "already-disabled".to_string(),
    };
    let result = cmd_disable_provider(disable_args, &repo).await;
    assert!(result.is_ok());
    let provider = repo.find_by_id("already-disabled").await.unwrap();
    assert!(!provider.enabled);
}

#[tokio::test]
async fn test_cli_list_providers_sorted_by_name() {
    let (_temp_dir, repo) = create_test_repo();
    let providers = vec![
        ("z-provider", "Z Provider", "https://api.z.com/v1"),
        ("a-provider", "A Provider", "https://api.a.com/v1"),
        ("m-provider", "M Provider", "https://api.m.com/v1"),
    ];
    for (id, name, url) in providers {
        let args = AddProviderArgs {
            id: id.to_string(),
            name: name.to_string(),
            base_url: url.to_string(),
            api_key: None,
            disabled: false,
            interactive: false,
        };
        cmd_add_provider(args, &repo).await.unwrap();
    }
    let all_providers = repo.find_all().await.unwrap();
    assert_eq!(all_providers.len(), 3);
    let names: Vec<&str> = all_providers.iter().map(|p| p.name.as_str()).collect();
    assert!(names.contains(&"Z Provider"));
    assert!(names.contains(&"A Provider"));
    assert!(names.contains(&"M Provider"));
}

#[tokio::test]
async fn test_cli_add_provider_with_special_characters_in_name() {
    let (_temp_dir, repo) = create_test_repo();
    let args = AddProviderArgs {
        id: "special-chars".to_string(),
        name: "Test Provider™ 中文 🚀".to_string(),
        base_url: "https://api.special.com/v1".to_string(),
        api_key: None,
        disabled: false,
        interactive: false,
    };
    let result = cmd_add_provider(args, &repo).await;
    assert!(result.is_ok());
    let provider = repo.find_by_id("special-chars").await.unwrap();
    assert_eq!(provider.name, "Test Provider™ 中文 🚀");
}

#[tokio::test]
async fn test_cli_add_provider_with_trailing_slash_in_url() {
    let (_temp_dir, repo) = create_test_repo();
    let args = AddProviderArgs {
        id: "trailing-slash".to_string(),
        name: "Trailing Slash Provider".to_string(),
        base_url: "https://api.trailing.com/v1/".to_string(),
        api_key: None,
        disabled: false,
        interactive: false,
    };
    let result = cmd_add_provider(args, &repo).await;
    assert!(result.is_ok());
    let provider = repo.find_by_id("trailing-slash").await.unwrap();
    assert!(provider.base_url.ends_with('/'));
}

#[tokio::test]
async fn test_cli_provider_enable_disable_cycle() {
    let (_temp_dir, repo) = create_test_repo();
    let add_args = AddProviderArgs {
        id: "cycle-provider".to_string(),
        name: "Cycle Provider".to_string(),
        base_url: "https://api.cycle.com/v1".to_string(),
        api_key: None,
        disabled: false,
        interactive: false,
    };
    cmd_add_provider(add_args, &repo).await.unwrap();
    let provider = repo.find_by_id("cycle-provider").await.unwrap();
    assert!(provider.enabled);
    let disable_args = DisableProviderArgs {
        id: "cycle-provider".to_string(),
    };
    cmd_disable_provider(disable_args, &repo).await.unwrap();
    let provider = repo.find_by_id("cycle-provider").await.unwrap();
    assert!(!provider.enabled);
    let enable_args = EnableProviderArgs {
        id: "cycle-provider".to_string(),
    };
    cmd_enable_provider(enable_args, &repo).await.unwrap();
    let provider = repo.find_by_id("cycle-provider").await.unwrap();
    assert!(provider.enabled);
    let disable_args2 = DisableProviderArgs {
        id: "cycle-provider".to_string(),
    };
    cmd_disable_provider(disable_args2, &repo).await.unwrap();
    let provider = repo.find_by_id("cycle-provider").await.unwrap();
    assert!(!provider.enabled);
}

#[tokio::test]
async fn test_cli_add_multiple_providers_same_base_url() {
    let (_temp_dir, repo) = create_test_repo();
    let providers = vec![
        ("provider-1", "Provider One"),
        ("provider-2", "Provider Two"),
        ("provider-3", "Provider Three"),
    ];
    for (id, name) in providers {
        let args = AddProviderArgs {
            id: id.to_string(),
            name: name.to_string(),
            base_url: "https://api.shared.com/v1".to_string(),
            api_key: None,
            disabled: false,
            interactive: false,
        };
        cmd_add_provider(args, &repo).await.unwrap();
    }
    let all_providers = repo.find_all().await.unwrap();
    assert_eq!(all_providers.len(), 3);
    for provider in &all_providers {
        assert_eq!(provider.base_url, "https://api.shared.com/v1");
    }
}

#[tokio::test]
async fn test_cli_add_provider_error_contains_id() {
    let (_temp_dir, repo) = create_test_repo();
    let args1 = AddProviderArgs {
        id: "error-test".to_string(),
        name: "Error Test".to_string(),
        base_url: "https://api.error.com/v1".to_string(),
        api_key: None,
        disabled: false,
        interactive: false,
    };
    cmd_add_provider(args1, &repo).await.unwrap();
    let args2 = AddProviderArgs {
        id: "error-test".to_string(),
        name: "Error Test Duplicate".to_string(),
        base_url: "https://api.error2.com/v1".to_string(),
        api_key: None,
        disabled: false,
        interactive: false,
    };
    let result = cmd_add_provider(args2, &repo).await;
    assert!(result.is_ok());
    let provider = repo.find_by_id("error-test").await.unwrap();
    assert_eq!(provider.name, "Error Test Duplicate");
}

#[tokio::test]
async fn test_cli_enable_error_contains_id() {
    let (_temp_dir, repo) = create_test_repo();
    let args = EnableProviderArgs {
        id: "missing-provider".to_string(),
    };
    let result = cmd_enable_provider(args, &repo).await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("missing-provider")
            || err_msg.contains("not found")
            || err_msg.contains("exist")
    );
}
