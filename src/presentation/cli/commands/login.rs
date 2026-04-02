use crate::app::services::auth::AuthService;
use crate::domain::traits::{AccountRepository, ProviderRepository};
use crate::error::Result;
use crate::infrastructure::gateway::llm_gateway::default_providers;
use crate::infrastructure::{JsonAccountRepository, JsonProviderRepository};
use crate::presentation::cli::output;
use std::sync::Arc;

/// Handle the login command to initiate authentication flow.
pub async fn handle_login_command(provider_id: String) -> Result<()> {
    // Validate provider exists in default_providers
    let available_providers = default_providers();
    if !available_providers.contains_key(&provider_id) {
        output::error(&format!("Unknown provider '{}'", provider_id));
        output::info("\nAvailable providers:");
        for (id, config) in &available_providers {
            output::dim(&format!("  - {} ({})", id, config.name));
        }
        return Ok(());
    }

    output::info(&format!("Starting authentication process for provider '{}'...", provider_id));

    // Initialize repositories as trait objects
    let account_repo: Arc<dyn AccountRepository + Send + Sync> =
        Arc::new(JsonAccountRepository::new()?);
    let provider_repo: Arc<dyn ProviderRepository + Send + Sync> =
        Arc::new(JsonProviderRepository::new()?);

    // Keep a clone for checking provider
    let provider_repo_check = Arc::clone(&provider_repo);

    // Initialize auth service (uses ownership of provider_repo)
    let auth_service = AuthService::new(account_repo, provider_repo);

    // Check if provider exists
    let provider_result = provider_repo_check.find_by_id(&provider_id).await;

    if provider_result.is_err() {
        output::error(&format!(
            "Provider '{}' not found. Please add a provider first using 'llm-router provider add'.",
            provider_id
        ));
        return Ok(());
    }

    let provider = provider_result.unwrap();

    if !provider.enabled {
        output::error(&format!(
            "Provider '{}' is disabled. Please enable it first using 'llm-router provider enable'.",
            provider_id
        ));
        return Ok(());
    }

    // Initiate authentication
    match auth_service.initiate_auth(&provider_id).await {
        Ok(verifier_or_instructions) => {
            if verifier_or_instructions.is_empty() {
                // API key authentication - ask for API key
                output::info(&format!("Please enter your API key for provider '{}':", provider_id));

                let mut api_key = String::new();
                std::io::stdin()
                    .read_line(&mut api_key)
                    .map_err(|e| crate::error::Error::Internal(e.to_string()))?;

                let api_key = api_key.trim();

                if api_key.is_empty() {
                    output::error("API key cannot be empty");
                    return Ok(());
                }

                // Complete authentication with the API key
                match auth_service
                    .complete_auth(&provider_id, api_key.to_string())
                    .await
                {
                    Ok(account) => {
                        output::success(&format!(
                            "Successfully authenticated with provider '{}'",
                            provider_id
                        ));
                        output::info(&format!("  Account ID: {}", account.id));
                        output::dim(&format!("  Auth type: {}", account.auth_type()));
                    },
                    Err(e) => {
                        output::error(&format!("Authentication failed: {}", e));
                    },
                }
            } else {
                // OAuth authentication - show instructions
                println!("{}", verifier_or_instructions);
                output::info("Please complete the authentication in your browser.");

                // In a real implementation, we would wait for the callback here
                // For this implementation, we'll simulate completion
                output::dim("Note: In this implementation, you would need to complete the OAuth flow manually.");
                output::dim("For testing purposes, you can simulate completion with a test code.");
            }
        },
        Err(e) => {
            output::error(&format!("Failed to initiate authentication: {}", e));
        },
    }

    Ok(())
}
