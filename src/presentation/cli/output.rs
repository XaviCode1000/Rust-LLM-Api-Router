//! Coloured output utilities.
//!
//! Provides semantic-colour helpers for consistent CLI messaging:
//! - `success()` — green checkmark
//! - `error()`   — red cross
//! - `warning()` — yellow warning sign
//! - `info()`    — blue info sign
//! - `dim()`     — dimmed secondary text

use crate::presentation::cli::tty::should_use_color;
use owo_colors::OwoColorize;

/// Print a success message (green ✓).
pub fn success(msg: &str) {
    if should_use_color() {
        println!("{} {}", "✓".green().bold(), msg.green());
    } else {
        println!("✓ {msg}");
    }
}

/// Print an error message (red ✗) to stderr.
pub fn error(msg: &str) {
    if should_use_color() {
        eprintln!("{} {}", "✗".red().bold(), msg.red());
    } else {
        eprintln!("✗ {msg}");
    }
}

/// Print a warning message (yellow ⚠) to stderr.
pub fn warning(msg: &str) {
    if should_use_color() {
        eprintln!("{} {}", "⚠".yellow().bold(), msg.yellow());
    } else {
        eprintln!("⚠ {msg}");
    }
}

/// Print an informational message (blue ℹ).
pub fn info(msg: &str) {
    if should_use_color() {
        println!("{} {}", "ℹ".blue().bold(), msg.blue());
    } else {
        println!("ℹ {msg}");
    }
}

/// Print dimmed / secondary text.
pub fn dim(msg: &str) {
    if should_use_color() {
        println!("{}", msg.dimmed());
    } else {
        println!("{msg}");
    }
}

/// Return a bold version of the string (honours colour policy).
#[must_use]
pub fn bold(msg: &str) -> String {
    if should_use_color() {
        msg.bold().to_string()
    } else {
        msg.to_string()
    }
}
