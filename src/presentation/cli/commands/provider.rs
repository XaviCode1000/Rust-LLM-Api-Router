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

use crate::domain::known_providers;
use crate::domain::traits::{AccountRepository, ProviderRepository};
use crate::domain::{Provider, SelectionState};
use crate::presentation::cli::input::read_api_key_interactive;
use crate::presentation::cli::{output, prompt, spinner, table};
use crate::Result;

// Re-export provider list functions from sibling module
pub use crate::presentation::cli::commands::provider_list::{
    auto_fill_provider, cmd_list_known, display_known_providers, select_provider_interactive,
};

/// Add provider arguments
///
/// Modified to support:
/// - --list flag to show known providers
/// - --interactive mode for interactive selection
/// - Partial args when using interactive mode
#[derive(Debug, Args)]
pub struct AddProviderArgs {
    /// Provider unique identifier (required without --interactive, optional with --interactive)
    #[arg(long)]
    pub id: Option<String>,

    /// Human-readable provider name (auto-filled with --interactive from known providers)
    #[arg(long)]
    pub name: Option<String>,

    /// Base URL for API requests (auto-filled with --interactive from known providers)
    #[arg(long)]
    pub base_url: Option<String>,

    /// API key for authentication (or use --interactive)
    #[arg(long)]
    pub api_key: Option<String>,

    /// Start disabled
    #[arg(long)]
    pub disabled: bool,

    /// Interactive mode: select provider from list or enter custom
    #[arg(long, short)]
    pub interactive: bool,

    /// List all known providers and exit
    #[arg(long)]
    pub list: bool,
}

/// Remove provider arguments
#[derive(Debug, Args)]
pub struct RemoveProviderArgs {
    /// Provider ID to remove
    #[arg(short, long)]
    pub id: String,

    /// Skip confirmation prompt (for non-interactive use)
    #[arg(long, short = 'y')]
    pub force: bool,
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

/// Handle provider subcommand
pub async fn handle_provider_command(
    cmd: ProviderCommands,
    repo: &impl ProviderRepository,
    account_repo: &impl AccountRepository,
) -> Result<()> {
    match cmd {
        ProviderCommands::Add(args) => cmd_add_provider(args, repo).await,
        ProviderCommands::List => cmd_list_providers(repo, account_repo).await,
        ProviderCommands::Models(args) => cmd_list_models(args, repo, account_repo).await,
        ProviderCommands::Remove(args) => cmd_remove_provider(args, repo).await,
        ProviderCommands::Enable(args) => cmd_enable_provider(args, repo).await,
        ProviderCommands::Disable(args) => cmd_disable_provider(args, repo).await,
        ProviderCommands::Validate(args) => cmd_validate_provider(args, repo).await,
    }
}

/// Add a new provider
///
/// Handles:
/// - `--list` flag: Display known providers and exit
/// - `--interactive` mode: Interactive provider selection
/// - Auto-fill from known providers if ID matches
/// - Backward compatibility with existing flags
pub async fn cmd_add_provider(args: AddProviderArgs, repo: &impl ProviderRepository) -> Result<()> {
    // Check --list flag first
    if args.list {
        display_known_providers()?;
        return Ok(());
    }

    // Determine provider details
    let (provider_id, provider_name, provider_base_url) = if args.interactive {
        // Interactive mode: check if we need selection or can auto-fill
        if let (Some(id), (None, None)) = (&args.id, (&args.name, &args.base_url)) {
            // If ID provided but name/base_url missing, try auto-fill
            let (name, base_url, _) = auto_fill_provider(id, "", "");
            (id.clone(), name, base_url)
        } else if args.id.is_none() && args.name.is_none() && args.base_url.is_none() {
            // No args: show interactive selection
            let selection = select_provider_interactive()?;

            match selection.state {
                SelectionState::Selected => {
                    let id = selection.provider_id.unwrap();
                    let kp = known_providers::find(&id).unwrap();
                    (id, kp.name.to_string(), kp.base_url.to_string())
                }
                SelectionState::Cancelled => {
                    output::info("Provider selection cancelled.");
                    return Ok(());
                }
                SelectionState::Invalid => {
                    output::error("Invalid selection. Enter a number (1-34) or provider ID.");
                    return Ok(());
                }
                _ => {
                    output::error("Selection error.");
                    return Ok(());
                }
            }
        } else {
            // Has args, use them
            (
                args.id.unwrap_or_default(),
                args.name.unwrap_or_default(),
                args.base_url.unwrap_or_default(),
            )
        }
    } else {
        // Non-interactive: validate required args
        let id = args.id.clone().unwrap_or_default();
        let name = args.name.clone().unwrap_or_default();
        let base_url = args.base_url.clone().unwrap_or_default();

        if id.is_empty() || name.is_empty() || base_url.is_empty() {
            return Err(crate::Error::Internal(
                "Missing required arguments: --id, --name, and --base_url are required without --interactive".to_string()
            ));
        }

        // Auto-fill if ID matches known provider
        let (name, base_url, was_filled) = auto_fill_provider(&id, &name, &base_url);
        if was_filled {
            output::dim(&format!(
                "Auto-filled '{}' details from known providers",
                name
            ));
        }

        (id, name, base_url)
    };

    // Get API key (from args or interactive)
    let api_key = if args.interactive {
        read_api_key_interactive()?
    } else {
        args.api_key.unwrap_or_default()
    };

    if api_key.is_empty() && !args.interactive {
        output::warning("No API key provided. Use --api-key or --interactive.");
    }

    // Create provider
    let provider = if args.disabled {
        Provider::disabled(&provider_id, &provider_name, &provider_base_url)
    } else {
        Provider::new(&provider_id, &provider_name, &provider_base_url)
    };

    repo.save(provider)
        .await
        .map_err(|e| crate::Error::Internal(e.to_string()))?;

    output::success(&format!("Provider '{}' added successfully", provider_id));
    Ok(())
}

/// List all providers
pub async fn cmd_list_providers(
    repo: &impl ProviderRepository,
    account_repo: &impl AccountRepository,
) -> Result<()> {
    let providers = repo
        .find_all()
        .await
        .map_err(|e| crate::Error::Internal(e.to_string()))?;

    if providers.is_empty() {
        output::info("No providers registered.");
        return Ok(());
    }

    // Get provider IDs with active accounts
    let mut provider_ids_with_accounts: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    for provider in &providers {
        if let Ok(accounts) = account_repo.find_active_by_provider(&provider.id).await {
            if !accounts.is_empty() {
                provider_ids_with_accounts.insert(provider.id.clone());
            }
        }
    }

    println!("{}", table::provider_table(&providers));

    // Print additional info about accounts
    for provider in &providers {
        if provider_ids_with_accounts.contains(&provider.id) {
            output::dim(&format!("  {}: has active account(s)", provider.id));
        } else {
            output::dim(&format!("  {}: no account configured", provider.id));
        }
    }

    Ok(())
}

/// List available models for a provider
pub async fn cmd_list_models(
    args: ListModelsArgs,
    provider_repo: &impl ProviderRepository,
    account_repo: &impl AccountRepository,
) -> Result<()> {
    let provider_id = &args.provider;

    // Check if there's an active account for this provider
    let accounts = account_repo
        .find_active_by_provider(provider_id)
        .await
        .map_err(|e| crate::Error::Internal(e.to_string()))?;

    let account = match accounts.into_iter().find(|a| a.is_active) {
        Some(acc) => acc,
        None => {
            output::error(&format!(
                "No active account found for provider '{}'",
                provider_id
            ));
            output::info(&format!(
                "Please run: llm-router auth login --provider {}",
                provider_id
            ));
            return Ok(());
        }
    };

    let api_key = match &account.api_key {
        Some(key) if !key.is_empty() => key.clone(),
        _ => {
            output::error(&format!(
                "No API key configured for provider '{}'",
                provider_id
            ));
            return Ok(());
        }
    };

    output::info(&format!(
        "Fetching models for provider '{}'...",
        provider_id
    ));

    // Create HTTP client and fetch models
    let http_client = reqwest::Client::new();

    // Get provider config for base URL
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
        output::error(&format!(
            "Failed to fetch models (HTTP {}): {}",
            status, error_text
        ));
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
        #[allow(dead_code)]
        created: Option<u64>,
    }

    match response.json::<ProviderModelsResponse>().await {
        Ok(models_response) => {
            if models_response.data.is_empty() {
                output::info(&format!(
                    "No models available for provider '{}'",
                    provider_id
                ));
                return Ok(());
            }

            output::info(&format!("\nAvailable models for '{}':\n", provider_id));
            println!("{:<50} Name", "Model ID");
            println!("{:-<70}", "");

            for model in &models_response.data {
                let name = model.name.clone().unwrap_or_default();
                println!("{:<50} {}", model.id, name);
            }

            let total = models_response.data.len();
            output::info(&format!("\nTotal: {} models", total));
        }
        Err(e) => {
            // Try alternative format (some providers return different structure)
            output::error(&format!("Error parsing models response: {}", e));
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
    repo: &impl ProviderRepository,
) -> Result<()> {
    // First check if provider exists
    repo.find_by_id(&args.id)
        .await
        .map_err(|_| crate::Error::ProviderNotFound(args.id.clone()))?;

    // Confirmation prompt (skip if --force is used)
    if !args.force && !prompt::confirm(&format!(
        "Are you sure you want to remove provider '{}'? This will also remove all associated accounts.",
        args.id
    ))? {
        output::info(&format!("Cancelled. Provider '{}' was not removed.", args.id));
        return Ok(());
    }

    // Use repository delete method which properly persists the deletion
    repo.delete(&args.id)
        .await
        .map_err(|e| crate::Error::Internal(e.to_string()))?;

    output::success(&format!("Provider '{}' removed successfully", args.id));
    Ok(())
}

/// Enable a provider
pub async fn cmd_enable_provider(
    args: EnableProviderArgs,
    repo: &impl ProviderRepository,
) -> Result<()> {
    let mut provider = repo
        .find_by_id(&args.id)
        .await
        .map_err(|_| crate::Error::ProviderNotFound(args.id.clone()))?;

    provider.enabled = true;

    repo.save(provider)
        .await
        .map_err(|e| crate::Error::Internal(e.to_string()))?;

    output::success(&format!("Provider '{}' enabled", args.id));
    Ok(())
}

/// Disable a provider
pub async fn cmd_disable_provider(
    args: DisableProviderArgs,
    repo: &impl ProviderRepository,
) -> Result<()> {
    let mut provider = repo
        .find_by_id(&args.id)
        .await
        .map_err(|_| crate::Error::ProviderNotFound(args.id.clone()))?;

    provider.enabled = false;

    repo.save(provider)
        .await
        .map_err(|e| crate::Error::Internal(e.to_string()))?;

    output::success(&format!("Provider '{}' disabled", args.id));
    Ok(())
}

/// Validate provider credentials
pub async fn cmd_validate_provider(
    args: ValidateProviderArgs,
    repo: &impl ProviderRepository,
) -> Result<()> {
    let provider = match repo.find_enabled_by_id(&args.id).await {
        Ok(p) => p,
        Err(crate::domain::DomainError::ProviderNotFound(id)) => {
            return Err(crate::Error::ProviderNotFound(id));
        }
        Err(crate::domain::DomainError::ProviderDisabled(id)) => {
            output::warning(&format!("Provider '{}' is disabled. Enable it first.", id));
            return Ok(());
        }
        Err(e) => return Err(crate::Error::Internal(e.to_string())),
    };

    let spinner = spinner::CliSpinner::new(&format!("Validating provider '{}'...", args.id));
    output::dim("Note: Actual credential validation requires API key storage.");
    output::dim("This feature will be implemented when account management is added.");

    // TODO: Implement actual validation when Account entity is available
    // For now, just check if provider is reachable
    let client = reqwest::Client::new();
    match client.get(&provider.base_url).send().await {
        Ok(response) => {
            if response.status().is_success() || response.status().is_client_error() {
                spinner.finish_with_message(&format!(
                    "✓ Provider '{}' is reachable at {}",
                    provider.id, provider.base_url
                ));
            } else {
                spinner.finish_with_message(&format!(
                    "⚠ Provider '{}' returned status: {}",
                    provider.id,
                    response.status()
                ));
                output::warning(&format!(
                    "Provider '{}' returned status: {}",
                    provider.id,
                    response.status()
                ));
            }
        }
        Err(e) => {
            spinner.finish_with_message(&format!(
                "✗ Provider '{}' is not reachable: {}",
                provider.id, e
            ));
            output::error(&format!(
                "Provider '{}' is not reachable: {}",
                provider.id, e
            ));
        }
    }

    Ok(())
}
