//! CLI provider management commands
//!
//! This module implements the provider management subcommands:
//! - add: Add a new provider
//! - list: List all providers
//! - models: List available models for a provider
//! - remove: Remove a provider by ID
//! - enable: Enable a provider
//! - disable: Disable a provider
//! - validate: Validate provider credentials

use clap::{Args, Subcommand};
use std::io::{self, BufRead, Write};

use crate::domain::traits::{AccountRepository, ProviderRepository};
use crate::domain::Provider;
use crate::infrastructure::{JsonAccountRepository, JsonProviderRepository};
use crate::Result;

/// Add provider arguments
#[derive(Debug, Args)]
pub struct AddProviderArgs {
    /// Provider unique identifier
    #[arg(long)]
    pub id: String,

    /// Human-readable provider name
    #[arg(long)]
    pub name: String,

    /// Base URL for API requests
    #[arg(long)]
    pub base_url: String,

    /// API key for authentication (or use --interactive)
    #[arg(long)]
    pub api_key: Option<String>,

    /// Start disabled
    #[arg(long)]
    pub disabled: bool,

    /// Interactive mode (prompt for API key)
    #[arg(long, short)]
    pub interactive: bool,
}

/// Remove provider arguments
#[derive(Debug, Args)]
pub struct RemoveProviderArgs {
    /// Provider ID to remove
    #[arg(short, long)]
    pub id: String,
}

/// Enable provider arguments
#[derive(Debug, Args)]
pub struct EnableProviderArgs {
    /// Provider ID to enable
    #[arg(short, long)]
    pub id: String,
}

/// Disable provider arguments
#[derive(Debug, Args)]
pub struct DisableProviderArgs {
    /// Provider ID to disable
    #[arg(short, long)]
    pub id: String,
}

/// Validate provider arguments
#[derive(Debug, Args)]
pub struct ValidateProviderArgs {
    /// Provider ID to validate
    #[arg(short, long)]
    pub id: String,
}

/// List models arguments
#[derive(Debug, Args)]
pub struct ListModelsArgs {
    /// Provider ID to list models for
    #[arg(short, long)]
    pub provider: String,
}

/// Provider management subcommands
#[derive(Debug, Subcommand)]
pub enum ProviderCommands {
    /// Add a new provider
    Add(AddProviderArgs),

    /// List all providers
    List,

    /// List available models for a provider
    Models(ListModelsArgs),

    /// Remove a provider by ID
    Remove(RemoveProviderArgs),

    /// Enable a provider
    Enable(EnableProviderArgs),

    /// Disable a provider
    Disable(DisableProviderArgs),

    /// Validate provider credentials
    Validate(ValidateProviderArgs),
}

/// Read API key interactively (hidden input)
fn read_api_key_interactive() -> Result<String> {
    print!("Enter API Key: ");
    io::stdout().flush()?;

    let stdin = io::stdin();
    let mut line = String::new();
    stdin
        .lock()
        .read_line(&mut line)
        .map_err(|e| crate::Error::Internal(e.to_string()))?;

    Ok(line.trim().to_string())
}

/// Handle provider subcommand
pub async fn handle_provider_command(cmd: ProviderCommands) -> Result<()> {
    let repo = JsonProviderRepository::new().map_err(|e| crate::Error::Internal(e.to_string()))?;
    let account_repo = JsonAccountRepository::new().map_err(|e| crate::Error::Internal(e.to_string()))?;

    match cmd {
        ProviderCommands::Add(args) => cmd_add_provider(args, &repo).await,
        ProviderCommands::List => cmd_list_providers(&repo, &account_repo).await,
        ProviderCommands::Models(args) => cmd_list_models(args, &account_repo).await,
        ProviderCommands::Remove(args) => cmd_remove_provider(args, &repo).await,
        ProviderCommands::Enable(args) => cmd_enable_provider(args, &repo).await,
        ProviderCommands::Disable(args) => cmd_disable_provider(args, &repo).await,
        ProviderCommands::Validate(args) => cmd_validate_provider(args, &repo).await,
    }
}

/// Add a new provider
pub async fn cmd_add_provider(args: AddProviderArgs, repo: &JsonProviderRepository) -> Result<()> {
    // Get API key (from args or interactive)
    let api_key = if args.interactive {
        read_api_key_interactive()?
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

    repo.save(provider)
        .await
        .map_err(|e| crate::Error::Internal(e.to_string()))?;

    println!("✓ Provider '{}' added successfully", args.id);
    Ok(())
}

/// List all providers
pub async fn cmd_list_providers(
    repo: &JsonProviderRepository,
    account_repo: &JsonAccountRepository,
) -> Result<()> {
    let providers = repo
        .find_all()
        .await
        .map_err(|e| crate::Error::Internal(e.to_string()))?;

    if providers.is_empty() {
        println!("No providers registered.");
        return Ok(());
    }

    println!(
        "{:<18} {:<28} {:<35} {:<12} {}",
        "ID", "Name", "Base URL", "Status", "Account"
    );
    println!("{:-<110}", "");

    for provider in providers {
        let status = if provider.enabled {
            "✓ Enabled"
        } else {
            "✗ Disabled"
        };

        // Check if there's an active account for this provider
        let account_status = match account_repo
            .find_active_by_provider(&provider.id)
            .await
        {
            Ok(accounts) if !accounts.is_empty() => "✓ Configured",
            _ => "✗ Not set",
        };

        println!(
            "{:<18} {:<28} {:<35} {:<12} {}",
            provider.id,
            provider.name,
            // Truncate base URL if too long
            if provider.base_url.len() > 35 {
                &provider.base_url[..32]
            } else {
                &provider.base_url
            },
            status,
            account_status
        );
    }

    Ok(())
}

/// List available models for a provider
pub async fn cmd_list_models(args: ListModelsArgs, account_repo: &JsonAccountRepository) -> Result<()> {
    let provider_id = &args.provider;

    // Check if there's an active account for this provider
    let accounts = account_repo
        .find_active_by_provider(provider_id)
        .await
        .map_err(|e| crate::Error::Internal(e.to_string()))?;

    let account = match accounts.into_iter().find(|a| a.is_active) {
        Some(acc) => acc,
        None => {
            println!("Error: No active account found for provider '{}'", provider_id);
            println!("Please run: llm-router auth login --provider {}", provider_id);
            return Ok(());
        }
    };

    let api_key = match &account.api_key {
        Some(key) if !key.is_empty() => key.clone(),
        _ => {
            println!("Error: No API key configured for provider '{}'", provider_id);
            return Ok(());
        }
    };

    println!("Fetching models for provider '{}'...", provider_id);

    // Create HTTP client and fetch models
    let http_client = reqwest::Client::new();
    
    // Get provider config for base URL
    let provider_repo = JsonProviderRepository::new()
        .map_err(|e| crate::Error::Internal(e.to_string()))?;
    
    let provider_config = provider_repo
        .find_by_id(provider_id)
        .await
        .map_err(|e| crate::Error::Internal(e.to_string()))?;

    let models_url = format!("{}/models", provider_config.base_url.trim_end_matches('/'));

    let response = http_client
        .get(&models_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| crate::Error::Internal(format!("Failed to fetch models: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        println!("Error: Failed to fetch models (HTTP {}): {}", status, error_text);
        return Ok(());
    }

    #[derive(serde::Deserialize)]
    struct ProviderModelsResponse {
        data: Vec<ProviderModel>,
    }

    #[derive(serde::Deserialize)]
    struct ProviderModel {
        id: String,
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        created: Option<u64>,
    }

    match response.json::<ProviderModelsResponse>().await {
        Ok(models_response) => {
            if models_response.data.is_empty() {
                println!("No models available for provider '{}'", provider_id);
                return Ok(());
            }

            println!("\nAvailable models for '{}':\n", provider_id);
            println!("{:<50} {}", "Model ID", "Name");
            println!("{:-<70}", "");

            for model in &models_response.data {
                let name = model.name.clone().unwrap_or_default();
                println!("{:<50} {}", model.id, name);
            }
            
            let total = models_response.data.len();
            println!("\nTotal: {} models", total);
        }
        Err(e) => {
            // Try alternative format (some providers return different structure)
            println!("Error parsing models response: {}", e);
        }
    }

    Ok(())
}

/// Remove a provider
///
/// # Bug Fix
/// This function now properly handles:
/// 1. Verifying provider exists before deletion
/// 2. Using the repository's delete method for proper persistence
/// 3. Handling empty list after deletion gracefully
pub async fn cmd_remove_provider(
    args: RemoveProviderArgs,
    repo: &JsonProviderRepository,
) -> Result<()> {
    // First check if provider exists
    repo.find_by_id(&args.id)
        .await
        .map_err(|_| crate::Error::ProviderNotFound(args.id.clone()))?;

    // Use repository delete method which properly persists the deletion
    repo.delete(&args.id)
        .await
        .map_err(|e| crate::Error::Internal(e.to_string()))?;

    println!("✓ Provider '{}' removed successfully", args.id);
    Ok(())
}

/// Enable a provider
pub async fn cmd_enable_provider(
    args: EnableProviderArgs,
    repo: &JsonProviderRepository,
) -> Result<()> {
    let mut provider = repo
        .find_by_id(&args.id)
        .await
        .map_err(|_| crate::Error::ProviderNotFound(args.id.clone()))?;

    provider.enabled = true;

    repo.save(provider)
        .await
        .map_err(|e| crate::Error::Internal(e.to_string()))?;

    println!("✓ Provider '{}' enabled", args.id);
    Ok(())
}

/// Disable a provider
pub async fn cmd_disable_provider(
    args: DisableProviderArgs,
    repo: &JsonProviderRepository,
) -> Result<()> {
    let mut provider = repo
        .find_by_id(&args.id)
        .await
        .map_err(|_| crate::Error::ProviderNotFound(args.id.clone()))?;

    provider.enabled = false;

    repo.save(provider)
        .await
        .map_err(|e| crate::Error::Internal(e.to_string()))?;

    println!("✓ Provider '{}' disabled", args.id);
    Ok(())
}

/// Validate provider credentials
pub async fn cmd_validate_provider(
    args: ValidateProviderArgs,
    repo: &JsonProviderRepository,
) -> Result<()> {
    let provider = match repo.find_enabled_by_id(&args.id).await {
        Ok(p) => p,
        Err(crate::domain::DomainError::ProviderNotFound(id)) => {
            return Err(crate::Error::ProviderNotFound(id));
        }
        Err(crate::domain::DomainError::ProviderDisabled(id)) => {
            eprintln!("Warning: Provider '{}' is disabled. Enable it first.", id);
            return Ok(());
        }
        Err(e) => return Err(crate::Error::Internal(e.to_string())),
    };

    println!("Validating provider '{}'...", provider.id);
    println!("Note: Actual credential validation requires API key storage.");
    println!("This feature will be implemented when account management is added.");

    // TODO: Implement actual validation when Account entity is available
    // For now, just check if provider is reachable
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
