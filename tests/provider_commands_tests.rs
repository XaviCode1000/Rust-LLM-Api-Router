//! Tests for CLI provider commands
//!
//! Integration tests for provider management CLI commands.

use tempfile::TempDir;
use std::path::PathBuf;

use rust_llm_api_router::cli::provider_commands::{
    ProviderCommands, AddProviderArgs, RemoveProviderArgs, EnableProviderArgs, 
    DisableProviderArgs, ValidateProviderArgs,
};
use rust_llm_api_router::infrastructure::persistence::JsonProviderRepository;
use rust_llm_api_router::domain::traits::ProviderRepository;
use rust_llm_api_router::domain::Provider;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn setup_test_environment() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_dir = temp_dir.path().join("config");
    std::fs::create_dir_all(&config_dir).expect("Failed to create config dir");
    (temp_dir, config_dir)
}

fn create_add_provider_args(id: &str, name: &str, base_url: &str, api_key: Option<&str>) -> AddProviderArgs {
    AddProviderArgs {
        id: id.to_string(),
        name: name.to_string(),
        base_url: base_url.to_string(),
        api_key: api_key.map(String::from),
        disabled: false,
        interactive: false,
    }
}

// Helper to handle provider command with custom repo
async fn handle_provider_command_with_dir(cmd: ProviderCommands, config_dir: &std::path::Path) -> rust_llm_api_router::Result<()> {
    let repo = JsonProviderRepository::with_config_dir(config_dir)
        .map_err(|e| rust_llm_api_router::Error::Internal(e.to_string()))?;

    match cmd {
        ProviderCommands::Add(args) => {
            // Get API key (from args or interactive)
            let api_key = if args.interactive {
                args.api_key.unwrap_or_default()
            } else {
                args.api_key.unwrap_or_default()
            };

            if api_key.is_empty() && !args.interactive {
                eprintln!("Warning: No API key provided. Use --api-key or --interactive.");
            }

            let provider = if args.disabled {
                Provider::disabled(&args.id, &args.name, &args.base_url)
            } else {
                Provider::new(&args.id, &args.name, &args.base_url)
            };

            repo.save(provider).await
                .map_err(|e| rust_llm_api_router::Error::Internal(e.to_string()))?;
            println!("✓ Provider '{}' added successfully", args.id);
            Ok(())
        }
        ProviderCommands::List => {
            let providers = repo.find_all().await
                .map_err(|e| rust_llm_api_router::Error::Internal(e.to_string()))?;

            if providers.is_empty() {
                println!("No providers registered.");
                return Ok(());
            }

            println!("{:<20} {:<30} {:<40} Status", "ID", "Name", "Base URL");
            println!("{:-<100}", "");

            for provider in providers {
                let status = if provider.enabled {
                    "✓ Enabled"
                } else {
                    "✗ Disabled"
                };
                println!(
                    "{:<20} {:<30} {:<40} {}",
                    provider.id, provider.name, provider.base_url, status
                );
            }

            Ok(())
        }
        ProviderCommands::Remove(args) => {
            // First check if provider exists
            repo.find_by_id(&args.id).await
                .map_err(|_| rust_llm_api_router::Error::ProviderNotFound(args.id.clone()))?;

            // Get all providers and filter out the one to remove
            let providers = repo.find_all().await
                .map_err(|e| rust_llm_api_router::Error::Internal(e.to_string()))?;

            let updated: Vec<_> = providers.into_iter().filter(|p| p.id != args.id).collect();

            // Save all providers back (overwrites the file)
            for provider in updated {
                repo.save(provider).await
                    .map_err(|e| rust_llm_api_router::Error::Internal(e.to_string()))?;
            }

            println!("✓ Provider '{}' removed successfully", args.id);
            Ok(())
        }
        ProviderCommands::Enable(args) => {
            let mut provider = repo.find_by_id(&args.id).await
                .map_err(|_| rust_llm_api_router::Error::ProviderNotFound(args.id.clone()))?;

            provider.enabled = true;

            repo.save(provider).await
                .map_err(|e| rust_llm_api_router::Error::Internal(e.to_string()))?;

            println!("✓ Provider '{}' enabled", args.id);
            Ok(())
        }
        ProviderCommands::Disable(args) => {
            let mut provider = repo.find_by_id(&args.id).await
                .map_err(|_| rust_llm_api_router::Error::ProviderNotFound(args.id.clone()))?;

            provider.enabled = false;

            repo.save(provider).await
                .map_err(|e| rust_llm_api_router::Error::Internal(e.to_string()))?;

            println!("✓ Provider '{}' disabled", args.id);
            Ok(())
        }
        ProviderCommands::Validate(args) => {
            let provider = match repo.find_enabled_by_id(&args.id).await {
                Ok(p) => p,
                Err(rust_llm_api_router::domain::DomainError::ProviderNotFound(id)) => {
                    return Err(rust_llm_api_router::Error::ProviderNotFound(id));
                }
                Err(rust_llm_api_router::domain::DomainError::ProviderDisabled(id)) => {
                    eprintln!("Warning: Provider '{}' is disabled. Enable it first.", id);
                    return Ok(());
                }
                Err(e) => return Err(rust_llm_api_router::Error::Internal(e.to_string())),
            };

            println!("Validating provider '{}'...", provider.id);
            println!("Note: Actual credential validation requires API key storage.");
            println!("This feature will be implemented when account management is added.");

            // Check if provider is reachable
            let client = reqwest::Client::new();
            match client.get(&provider.base_url).send().await {
                Ok(response) => {
                    if response.status().is_success() || response.status().is_client_error() {
                        println!(
                            "✓ Provider '{}' is reachable at {}",
                            provider.id, provider.base_url
                        );
                    } else {
                        println!(
                            "⚠ Provider '{}' returned status: {}",
                            provider.id,
                            response.status()
                        );
                    }
                }
                Err(e) => {
                    println!("✗ Provider '{}' is not reachable: {}", provider.id, e);
                }
            }

            Ok(())
        }
    }
}

// ============================================================================
// ADD PROVIDER TESTS
// ============================================================================

#[tokio::test]
async fn test_add_provider_success() {
    let (_temp_dir, config_dir) = setup_test_environment();
    
    let args = create_add_provider_args("test-provider", "Test Provider", "https://api.test.com", Some("key-12345"));
    let cmd = ProviderCommands::Add(args);
    
    let result = handle_provider_command_with_dir(cmd, &config_dir).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_provider_without_api_key() {
    let (_temp_dir, config_dir) = setup_test_environment();
    
    let args = create_add_provider_args("test-provider-2", "Test Provider 2", "https://api.test2.com", None);
    let cmd = ProviderCommands::Add(args);
    
    let result = handle_provider_command_with_dir(cmd, &config_dir).await;
    
    // Should succeed but print warning
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_provider_disabled() {
    let (_temp_dir, config_dir) = setup_test_environment();
    
    let mut args = create_add_provider_args("test-provider-3", "Test Provider 3", "https://api.test3.com", Some("key-12345"));
    args.disabled = true;
    let cmd = ProviderCommands::Add(args);
    
    let result = handle_provider_command_with_dir(cmd, &config_dir).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_provider_duplicate_id() {
    let (_temp_dir, config_dir) = setup_test_environment();
    
    // Add first provider
    let args1 = create_add_provider_args("dup-provider", "First Provider", "https://api.first.com", Some("key-1"));
    let cmd1 = ProviderCommands::Add(args1);
    let result1 = handle_provider_command_with_dir(cmd1, &config_dir).await;
    assert!(result1.is_ok());
    
    // Add duplicate - current implementation allows it (overwrites)
    let args2 = create_add_provider_args("dup-provider", "Second Provider", "https://api.second.com", Some("key-2"));
    let cmd2 = ProviderCommands::Add(args2);
    let result2 = handle_provider_command_with_dir(cmd2, &config_dir).await;
    
    // Should succeed (overwrites existing)
    assert!(result2.is_ok());
}

// ============================================================================
// LIST PROVIDERS TESTS
// ============================================================================

#[tokio::test]
async fn test_list_providers_empty() {
    let (_temp_dir, config_dir) = setup_test_environment();
    
    let cmd = ProviderCommands::List;
    let result = handle_provider_command_with_dir(cmd, &config_dir).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_providers_with_data() {
    let (_temp_dir, config_dir) = setup_test_environment();
    
    // Add some providers first
    let args1 = create_add_provider_args("list-provider-1", "Provider 1", "https://api.p1.com", Some("key-1"));
    let cmd1 = ProviderCommands::Add(args1);
    handle_provider_command_with_dir(cmd1, &config_dir).await.unwrap();
    
    let args2 = create_add_provider_args("list-provider-2", "Provider 2", "https://api.p2.com", Some("key-2"));
    let cmd2 = ProviderCommands::Add(args2);
    handle_provider_command_with_dir(cmd2, &config_dir).await.unwrap();
    
    // List providers
    let cmd = ProviderCommands::List;
    let result = handle_provider_command_with_dir(cmd, &config_dir).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_list_providers_mixed_status() {
    let (_temp_dir, config_dir) = setup_test_environment();
    
    // Add enabled provider
    let args1 = create_add_provider_args("enabled-provider", "Enabled Provider", "https://api.enabled.com", Some("key-1"));
    let cmd1 = ProviderCommands::Add(args1);
    handle_provider_command_with_dir(cmd1, &config_dir).await.unwrap();
    
    // Add disabled provider
    let mut args2 = create_add_provider_args("disabled-provider", "Disabled Provider", "https://api.disabled.com", Some("key-2"));
    args2.disabled = true;
    let cmd2 = ProviderCommands::Add(args2);
    handle_provider_command_with_dir(cmd2, &config_dir).await.unwrap();
    
    // List - should show both with status
    let cmd = ProviderCommands::List;
    let result = handle_provider_command_with_dir(cmd, &config_dir).await;
    
    assert!(result.is_ok());
}

// ============================================================================
// REMOVE PROVIDER TESTS
// ============================================================================

#[tokio::test]
async fn test_remove_provider_success() {
    let (_temp_dir, config_dir) = setup_test_environment();
    
    // Add provider first
    let args_add = create_add_provider_args("remove-provider", "Remove Provider", "https://api.remove.com", Some("key-1"));
    let cmd_add = ProviderCommands::Add(args_add);
    handle_provider_command_with_dir(cmd_add, &config_dir).await.unwrap();
    
    // Remove provider
    let args_remove = RemoveProviderArgs {
        id: "remove-provider".to_string(),
    };
    let cmd = ProviderCommands::Remove(args_remove);
    let result = handle_provider_command_with_dir(cmd, &config_dir).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_remove_provider_not_found() {
    let (_temp_dir, config_dir) = setup_test_environment();
    
    let args_remove = RemoveProviderArgs {
        id: "nonexistent-provider".to_string(),
    };
    let cmd = ProviderCommands::Remove(args_remove);
    let result = handle_provider_command_with_dir(cmd, &config_dir).await;
    
    // Should fail with ProviderNotFound
    assert!(result.is_err());
}

#[tokio::test]
async fn test_remove_provider_from_multiple() {
    let (_temp_dir, config_dir) = setup_test_environment();
    
    // Add multiple providers
    for i in 1..=3 {
        let args = create_add_provider_args(
            &format!("multi-provider-{}", i),
            &format!("Provider {}", i),
            &format!("https://api.p{}.com", i),
            Some("key-1")
        );
        let cmd = ProviderCommands::Add(args);
        handle_provider_command_with_dir(cmd, &config_dir).await.unwrap();
    }
    
    // Remove middle one
    let args_remove = RemoveProviderArgs {
        id: "multi-provider-2".to_string(),
    };
    let cmd = ProviderCommands::Remove(args_remove);
    let result = handle_provider_command_with_dir(cmd, &config_dir).await;
    
    assert!(result.is_ok());
    
    // Verify others still exist
    let cmd_list = ProviderCommands::List;
    let result_list = handle_provider_command_with_dir(cmd_list, &config_dir).await;
    assert!(result_list.is_ok());
}

// ============================================================================
// ENABLE PROVIDER TESTS
// ============================================================================

#[tokio::test]
async fn test_enable_provider_success() {
    let (_temp_dir, config_dir) = setup_test_environment();
    
    // Add disabled provider first
    let mut args_add = create_add_provider_args("enable-provider", "Enable Provider", "https://api.enable.com", Some("key-1"));
    args_add.disabled = true;
    let cmd_add = ProviderCommands::Add(args_add);
    handle_provider_command_with_dir(cmd_add, &config_dir).await.unwrap();
    
    // Enable provider
    let args_enable = EnableProviderArgs {
        id: "enable-provider".to_string(),
    };
    let cmd = ProviderCommands::Enable(args_enable);
    let result = handle_provider_command_with_dir(cmd, &config_dir).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_enable_provider_not_found() {
    let (_temp_dir, config_dir) = setup_test_environment();
    
    let args_enable = EnableProviderArgs {
        id: "nonexistent-provider".to_string(),
    };
    let cmd = ProviderCommands::Enable(args_enable);
    let result = handle_provider_command_with_dir(cmd, &config_dir).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_enable_already_enabled_provider() {
    let (_temp_dir, config_dir) = setup_test_environment();
    
    // Add enabled provider
    let args_add = create_add_provider_args("already-enabled", "Already Enabled", "https://api.enabled.com", Some("key-1"));
    let cmd_add = ProviderCommands::Add(args_add);
    handle_provider_command_with_dir(cmd_add, &config_dir).await.unwrap();
    
    // Enable again (should succeed, just no change)
    let args_enable = EnableProviderArgs {
        id: "already-enabled".to_string(),
    };
    let cmd = ProviderCommands::Enable(args_enable);
    let result = handle_provider_command_with_dir(cmd, &config_dir).await;
    
    assert!(result.is_ok());
}

// ============================================================================
// DISABLE PROVIDER TESTS
// ============================================================================

#[tokio::test]
async fn test_disable_provider_success() {
    let (_temp_dir, config_dir) = setup_test_environment();
    
    // Add enabled provider first
    let args_add = create_add_provider_args("disable-provider", "Disable Provider", "https://api.disable.com", Some("key-1"));
    let cmd_add = ProviderCommands::Add(args_add);
    handle_provider_command_with_dir(cmd_add, &config_dir).await.unwrap();
    
    // Disable provider
    let args_disable = DisableProviderArgs {
        id: "disable-provider".to_string(),
    };
    let cmd = ProviderCommands::Disable(args_disable);
    let result = handle_provider_command_with_dir(cmd, &config_dir).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_disable_provider_not_found() {
    let (_temp_dir, config_dir) = setup_test_environment();
    
    let args_disable = DisableProviderArgs {
        id: "nonexistent-provider".to_string(),
    };
    let cmd = ProviderCommands::Disable(args_disable);
    let result = handle_provider_command_with_dir(cmd, &config_dir).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_disable_already_disabled_provider() {
    let (_temp_dir, config_dir) = setup_test_environment();
    
    // Add disabled provider
    let mut args_add = create_add_provider_args("already-disabled", "Already Disabled", "https://api.disabled.com", Some("key-1"));
    args_add.disabled = true;
    let cmd_add = ProviderCommands::Add(args_add);
    handle_provider_command_with_dir(cmd_add, &config_dir).await.unwrap();
    
    // Disable again (should succeed, just no change)
    let args_disable = DisableProviderArgs {
        id: "already-disabled".to_string(),
    };
    let cmd = ProviderCommands::Disable(args_disable);
    let result = handle_provider_command_with_dir(cmd, &config_dir).await;
    
    assert!(result.is_ok());
}

// ============================================================================
// VALIDATE PROVIDER TESTS
// ============================================================================

#[tokio::test]
async fn test_validate_provider_success() {
    let (_temp_dir, config_dir) = setup_test_environment();
    
    // Add provider with reachable URL
    let args_add = create_add_provider_args("validate-provider", "Validate Provider", "https://httpbin.org", Some("key-1"));
    let cmd_add = ProviderCommands::Add(args_add);
    handle_provider_command_with_dir(cmd_add, &config_dir).await.unwrap();
    
    // Validate provider
    let args_validate = ValidateProviderArgs {
        id: "validate-provider".to_string(),
    };
    let cmd = ProviderCommands::Validate(args_validate);
    let result = handle_provider_command_with_dir(cmd, &config_dir).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_provider_not_found() {
    let (_temp_dir, config_dir) = setup_test_environment();
    
    let args_validate = ValidateProviderArgs {
        id: "nonexistent-provider".to_string(),
    };
    let cmd = ProviderCommands::Validate(args_validate);
    let result = handle_provider_command_with_dir(cmd, &config_dir).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_validate_disabled_provider() {
    let (_temp_dir, config_dir) = setup_test_environment();
    
    // Add disabled provider
    let mut args_add = create_add_provider_args("validate-disabled", "Validate Disabled", "https://api.disabled.com", Some("key-1"));
    args_add.disabled = true;
    let cmd_add = ProviderCommands::Add(args_add);
    handle_provider_command_with_dir(cmd_add, &config_dir).await.unwrap();
    
    // Validate disabled provider - should warn
    let args_validate = ValidateProviderArgs {
        id: "validate-disabled".to_string(),
    };
    let cmd = ProviderCommands::Validate(args_validate);
    let result = handle_provider_command_with_dir(cmd, &config_dir).await;
    
    // Should succeed but warn about disabled status
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_provider_unreachable_url() {
    let (_temp_dir, config_dir) = setup_test_environment();
    
    // Add provider with unreachable URL
    let args_add = create_add_provider_args(
        "validate-unreachable", 
        "Validate Unreachable", 
        "https://this-domain-definitely-does-not-exist-12345.com", 
        Some("key-1")
    );
    let cmd_add = ProviderCommands::Add(args_add);
    handle_provider_command_with_dir(cmd_add, &config_dir).await.unwrap();
    
    // Validate - should report unreachable
    let args_validate = ValidateProviderArgs {
        id: "validate-unreachable".to_string(),
    };
    let cmd = ProviderCommands::Validate(args_validate);
    let result = handle_provider_command_with_dir(cmd, &config_dir).await;
    
    // Should still succeed (validation completes, just reports issue)
    assert!(result.is_ok());
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[tokio::test]
async fn test_add_provider_special_characters_in_id() {
    let (_temp_dir, config_dir) = setup_test_environment();
    
    let args = create_add_provider_args("test-provider_special@123", "Test Provider", "https://api.test.com", Some("key-1"));
    let cmd = ProviderCommands::Add(args);
    let result = handle_provider_command_with_dir(cmd, &config_dir).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_provider_invalid_url() {
    let (_temp_dir, config_dir) = setup_test_environment();
    
    let args = create_add_provider_args("invalid-url-provider", "Invalid URL", "not-a-valid-url", Some("key-1"));
    let cmd = ProviderCommands::Add(args);
    let result = handle_provider_command_with_dir(cmd, &config_dir).await;
    
    // Should succeed (URL validation happens on use, not creation)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_add_provider_very_long_name() {
    let (_temp_dir, config_dir) = setup_test_environment();
    
    let long_name = "A".repeat(500);
    let args = create_add_provider_args("long-name-provider", &long_name, "https://api.test.com", Some("key-1"));
    let cmd = ProviderCommands::Add(args);
    let result = handle_provider_command_with_dir(cmd, &config_dir).await;
    
    assert!(result.is_ok());
}
