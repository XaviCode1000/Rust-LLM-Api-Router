//! Interactive provider list display and selection
//!
//! This module provides:
//! - [`display_known_providers()`]: Display all known providers in a formatted table
//! - [`select_provider_interactive()`]: Interactive provider selection
//! - [`auto_fill_provider()`]: Auto-fill provider details from known list
//!
//! # Design Decisions (from design document)
//!
//! - Follows err-no-unwrap-prod, async-no-lock-await patterns
//! - Uses SelectionState enum for selection tracking

use crate::domain::known_providers;
use crate::domain::SelectionState;
use crate::presentation::cli::{output, prompt};
use crate::Result;

/// Display all known providers in a formatted table.
///
/// Shows all 34 providers with their IDs, names, and base URLs.
/// # Example
///
/// ```no_run
/// use crate::presentation::cli::commands::provider_list::display_known_providers;
///
/// // display_known_providers()?;
/// ```
pub fn display_known_providers() -> Result<()> {
    let providers = known_providers::all();

    output::info(&format!("\n📋 Known Providers ({})\n", known_providers::count()));

    // Print header
    println!("{:<4} {:<18} {:<20} Base URL", "#", "ID", "Name");
    println!("{}", "-".repeat(70));

    // Print rows
    for (i, p) in providers.iter().enumerate() {
        println!("{:<4} {:<18} {:<20} {}", i + 1, p.id, p.name, p.base_url);
    }

    output::dim(&format!("\nTotal: {} providers", known_providers::count()));

    Ok(())
}

/// Interactive provider selection with numbered menu.
///
/// Displays numbered list, accepts input (number or ID),
/// returns selected provider or error.
/// # Example
///
/// ```no_run
/// use crate::presentation::cli::commands::provider_list::select_provider_interactive;
///
/// // let result = select_provider_interactive()?;
/// // match result.state {
/// //     SelectionState::Selected => println!("Selected: {}", result.provider_id.unwrap()),
/// //     SelectionState::Cancelled => println!("Cancelled"),
/// //     SelectionState::Invalid => println!("Invalid selection"),
/// // }
/// ```
pub fn select_provider_interactive() -> Result<crate::domain::ProviderSelection> {
    let providers = known_providers::all();

    // Display numbered list
    output::info("\n📋 Available providers:\n");
    for (i, p) in providers.iter().enumerate() {
        println!("  [{:>2}] {}", i + 1, p.name);
    }

    // Prompt for selection
    let input = prompt::prompt_text("Select provider (number or ID, Enter to cancel)")?;

    // Parse selection
    let result = if input.is_empty() {
        // Empty input = cancelled
        crate::domain::ProviderSelection::new(SelectionState::Cancelled, None)
    } else if let Ok(num) = input.parse::<usize>() {
        // Number selection
        if num > 0 && num <= providers.len() {
            let p = providers[num - 1];
            crate::domain::ProviderSelection::new(SelectionState::Selected, Some(p.id.to_string()))
        } else {
            crate::domain::ProviderSelection::new(SelectionState::Invalid, None)
        }
    } else {
        // ID selection (try case-insensitive)
        if let Some(p) = known_providers::find_case_insensitive(&input) {
            crate::domain::ProviderSelection::new(SelectionState::Selected, Some(p.id.to_string()))
        } else {
            crate::domain::ProviderSelection::new(SelectionState::Invalid, None)
        }
    };

    Ok(result)
}

/// Auto-fill provider details if ID matches known provider.
///
/// Returns (name, base_url) if found in known list,
/// or original values if not found.
/// # Example
///
/// ```no_run
/// use crate::presentation::cli::commands::provider_list::auto_fill_provider;
///
/// // let (name, base_url, was_filled) = auto_fill_provider("openai", "OpenAI", "https://custom.com");
/// // name = "OpenAI", base_url = "https://api.openai.com/v1", was_filled = true
/// ```
pub fn auto_fill_provider(
    id: impl Into<String>,
    name: impl Into<String>,
    base_url: impl Into<String>,
) -> (String, String, bool) {
    let id = id.into();
    let name = name.into();
    let base_url = base_url.into();

    if let Some(kp) = known_providers::find(&id) {
        // Use known provider values, mark as auto-filled
        (kp.name.to_string(), kp.base_url.to_string(), true)
    } else {
        // Use user-provided values
        (name, base_url, false)
    }
}

/// Display provider list and exit (--list flag).
pub fn cmd_list_known() -> Result<()> {
    display_known_providers()?;
    Ok(())
}
