//! Tests for CLI module

use clap::Parser;
use rust_llm_api_router::cli::{Cli, CliCommands};

#[test]
fn test_cli_parse_default_args() {
    let cli = Cli::parse_from(["llm-router"]);
    assert_eq!(cli.host, "0.0.0.0");
    assert_eq!(cli.port, 8080);
    assert_eq!(cli.log_level, "info");
    assert!(cli.commands.is_none());
}

#[test]
fn test_cli_parse_host_port() {
    let cli = Cli::parse_from(["llm-router", "--host", "127.0.0.1", "-p", "3000"]);
    assert_eq!(cli.host, "127.0.0.1");
    assert_eq!(cli.port, 3000);
}

#[test]
fn test_cli_parse_log_level() {
    let cli = Cli::parse_from(["llm-router", "--log-level", "debug"]);
    assert_eq!(cli.log_level, "debug");
}

#[test]
fn test_cli_parse_provider_subcommand() {
    let cli = Cli::parse_from(["llm-router", "provider", "list"]);
    assert!(cli.commands.is_some());
    match cli.commands {
        Some(CliCommands::Provider(_)) => {},
        _ => panic!("Expected Provider command"),
    }
}

#[test]
fn test_cli_parse_account_subcommand() {
    let cli = Cli::parse_from(["llm-router", "account", "list"]);
    assert!(cli.commands.is_some());
    match cli.commands {
        Some(CliCommands::Account(_)) => {},
        _ => panic!("Expected Account command"),
    }
}

#[test]
fn test_cli_debug_format() {
    let cli = Cli::parse_from(["llm-router"]);
    let debug = format!("{:?}", cli);
    assert!(debug.contains("Cli"));
}
