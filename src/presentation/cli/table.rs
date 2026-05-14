//! Table formatting utilities for CLI output.
//!
//! Provides helpers to render Provider and Account data as formatted tables
//! using comfy-table, with automatic truncation and API key masking.

use crate::domain::entities::{Account, Provider};
use comfy_table::{presets::UTF8_FULL, Cell, ContentArrangement, Table};

/// Render a table of providers.
///
/// Columns: ID, Name, Base URL, Status, Accounts (count).
pub fn provider_table(providers: &[Provider]) -> String {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("ID"),
            Cell::new("Name"),
            Cell::new("Base URL"),
            Cell::new("Status"),
        ]);

    for p in providers {
        let status = if p.enabled {
            "● Enabled"
        } else {
            "○ Disabled"
        };
        table.add_row(vec![
            Cell::new(&p.id),
            Cell::new(&p.name),
            Cell::new(truncate(&p.base_url, 30)),
            Cell::new(status),
        ]);
    }

    table.to_string()
}

/// Render a table of accounts.
///
/// Columns: ID, Provider, Priority, Status, API Key (masked).
pub fn account_table(accounts: &[Account]) -> String {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("ID"),
            Cell::new("Provider"),
            Cell::new("Priority"),
            Cell::new("Status"),
            Cell::new("API Key"),
        ]);

    for a in accounts {
        let status = if a.is_active {
            "✓ Active"
        } else {
            "✗ Inactive"
        };
        let api_key = mask_api_key(a.auth_method.api_key().unwrap_or(""));
        table.add_row(vec![
            Cell::new(&a.id),
            Cell::new(&a.provider_id),
            Cell::new(a.priority.to_string()),
            Cell::new(status),
            Cell::new(api_key),
        ]);
    }

    table.to_string()
}

/// Truncate a string to a maximum length, appending "..." if truncated.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        s.to_string()
    }
}

/// Mask an API key, showing only first 4 and last 4 characters.
fn mask_api_key(key: &str) -> String {
    if key.is_empty() {
        "(none)".to_string()
    } else if key.len() <= 8 {
        "****".to_string()
    } else {
        format!("{}...{}", &key[..4], &key[key.len() - 4..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_short_string() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_long_string() {
        assert_eq!(truncate("hello world foo bar", 10), "hello w...");
    }

    #[test]
    fn test_truncate_exact_length() {
        assert_eq!(truncate("abc", 3), "abc");
    }

    #[test]
    fn test_mask_api_key_short() {
        assert_eq!(mask_api_key(""), "(none)");
        assert_eq!(mask_api_key("short"), "****");
    }

    #[test]
    fn test_mask_api_key_long() {
        assert_eq!(mask_api_key("verylongapikey12345"), "very...2345");
    }

    #[test]
    fn test_provider_table_empty() {
        let table = provider_table(&[]);
        assert!(table.contains("ID"));
    }

    #[test]
    fn test_account_table_empty() {
        let table = account_table(&[]);
        assert!(table.contains("ID"));
    }
}
