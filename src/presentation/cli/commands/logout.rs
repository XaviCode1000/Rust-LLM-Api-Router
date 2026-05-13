use crate::app::services::auth::AuthService;
use crate::domain::traits::AccountRepository;
use crate::error::Result;
use crate::infrastructure::{JsonAccountRepository, JsonProviderRepository};
use crate::presentation::cli::output;
use std::sync::Arc;

/// Handle the logout command to revoke tokens and clear credentials.
pub async fn handle_logout_command() -> Result<()> {
    output::info("Starting logout process...");

    // Initialize repositories
    let account_repo = Arc::new(JsonAccountRepository::new()?);
    let provider_repo = Arc::new(JsonProviderRepository::new()?);

    // Initialize auth service
    let auth_service = AuthService::new(account_repo.clone(), provider_repo);

    // Get all accounts
    let accounts_result = account_repo.find_all().await;

    if accounts_result.is_err() {
        output::error(&format!(
            "Failed to retrieve accounts: {}",
            accounts_result.err().unwrap()
        ));
        return Ok(());
    }

    let accounts = accounts_result.unwrap();

    if accounts.is_empty() {
        output::info("No accounts found to log out from.");
        return Ok(());
    }

    output::info(&format!(
        "Found {} account(s). Logging out from all...",
        accounts.len()
    ));

    // Log out from each account
    for account in accounts {
        output::dim(&format!(
            "Logging out from account '{}' (provider: {})...",
            account.id, account.provider_id
        ));

        match auth_service.revoke_token(&account.id).await {
            Ok(()) => {
                output::success(&format!(
                    "Successfully logged out from account '{}'",
                    account.id
                ));
            }
            Err(e) => {
                output::error(&format!(
                    "Failed to log out from account '{}': {}",
                    account.id, e
                ));
                // Continue with other accounts even if one fails
            }
        }
    }

    output::success("Logout process completed.");
    Ok(())
}
