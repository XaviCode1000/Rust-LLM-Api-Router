//! TTY detection and color policy.
//!
//! Provides runtime detection of whether stdout is attached to a terminal,
//! and whether colors should be used (respects `NO_COLOR` env var).

use std::sync::atomic::{AtomicBool, Ordering};

static NO_COLOR: AtomicBool = AtomicBool::new(false);
static IS_TTY: AtomicBool = AtomicBool::new(false);

/// Initialise TTY detection. Call once at program start.
pub fn init() {
    IS_TTY.store(
        is_terminal::is_terminal(std::io::stdout()),
        Ordering::Relaxed,
    );
    NO_COLOR.store(std::env::var_os("NO_COLOR").is_some(), Ordering::Relaxed);
}

/// Returns `true` if stdout is attached to a terminal.
#[must_use]
pub fn is_tty() -> bool {
    IS_TTY.load(Ordering::Relaxed)
}

/// Returns `true` if coloured output should be used.
///
/// This is `false` when either:
/// - stdout is **not** a TTY (piped / redirected), or
/// - the `NO_COLOR` environment variable is set.
#[must_use]
pub fn should_use_color() -> bool {
    is_tty() && !NO_COLOR.load(Ordering::Relaxed)
}
