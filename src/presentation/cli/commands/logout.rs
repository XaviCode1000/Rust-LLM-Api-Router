use crate::app::services::auth::AuthService;
use crate::domain::traits::AccountRepository;
use crate::error::Result;
use crate::infrastructure::{JsonAccountRepository, JsonProviderRepository};
use std::sync::Arc;

/// Handle the logout command to revoke tokens and clear credentials.
pub async fn handle_logout_command() -> Result<()> {
    println!("Starting logout process...");
    
    // Initialize repositories
    let account_repo = Arc::new(JsonAccountRepository::new()?);
    let provider_repo = Arc::new(JsonProviderRepository::new()?);
    
    // Initialize auth service
    let auth_service = AuthService::new(account_repo.clone(), provider_repo);
    
    // Get all accounts
    let accounts_result = account_repo.find_all().await;
    
    if accounts_result.is_err() {
        println!("Failed to retrieve accounts: {}", accounts_result.err().unwrap());
        return Ok(());
    }
    
    let accounts = accounts_result.unwrap();
    
    if accounts.is_empty() {
        println!("No accounts found to log out from.");
        return Ok(());
    }
    
    println!("Found {} account(s). Logging out from all...", accounts.len());
    
    // Log out from each account
    for account in accounts {
        println!("Logging out from account '{}' (provider: {})...", account.id, account.provider_id);
        
        match auth_service.revoke_token(&account.id).await {
            Ok(()) => {
                println!("✓ Successfully logged out from account '{}'", account.id);
            }
            Err(e) => {
                println!("✗ Failed to log out from account '{}': {}", account.id, e);
                // Continue with other accounts even if one fails
            }
        }
    }
    
    println!("Logout process completed.");
    Ok(())
}