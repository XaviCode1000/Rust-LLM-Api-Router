//! TUI Engine - Terminal User Interface render loop and widgets
//!
//! This module provides the TUI engine with real-time dashboard visualization
//! using Ratatui. It renders provider health, latency sparklines, log streaming,
//! and interactive forms for account management.

use std::time::Duration;

use crossterm::{
    event::{self, poll, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Sparkline, Table},
    Frame, Terminal,
};
use tokio::sync::mpsc;
use tokio::sync::watch;

use crate::presentation::tui::state::{FormState, InputMode, LogLevel, TuiState};
use crate::presentation::tui::TuiAction;

/// Render throttle to prevent excessive CPU usage (~60fps max)
const RENDER_THROTTLE_MS: u64 = 16;

/// Idle sleep to prevent busy loop
const IDLE_SLEEP_MS: u64 = 10;

/// Timeout for processing mode (30 seconds)
const PROCESSING_TIMEOUT_SECS: u64 = 30;

/// Run the TUI engine with the given state receiver and action sender
///
/// # Arguments
/// * `rx` - Watch receiver for TuiState updates
/// * `action_tx` - Sender for TuiAction commands to the core system
///
/// # Returns
/// * `Ok(())` on clean exit
/// * `Err(Box<dyn std::error::Error>)` on terminal error
pub fn run(
    rx: watch::Receiver<TuiState>,
    action_tx: mpsc::Sender<TuiAction>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    enable_raw_mode()?;
    execute!(std::io::stdout(), EnterAlternateScreen)?;
    terminal.hide_cursor()?;

    let mut dirty = true;
    let mut last_render = std::time::Instant::now();
    let mut running = true;

    // Track input mode and form state locally
    let mut input_mode = InputMode::Normal;
    let mut form_state = FormState::default();
    let mut processing_timeout: Option<std::time::Instant> = None;
    let mut tick: u64 = 0;

    // Get initial state (ignore - we'll check has_changed)
    let _ = rx.borrow();

    while running {
        // Check for state updates (non-blocking)
        if rx.has_changed().unwrap_or(false) {
            dirty = true;
            // Sync input mode from state
            let state = rx.borrow();
            input_mode = state.input_mode.clone();
            form_state = state.form_state.clone();
            processing_timeout = state.processing_timeout;
        }

        // Check processing timeout
        if input_mode == InputMode::Processing {
            if let Some(timeout) = processing_timeout {
                if std::time::Instant::now() > timeout {
                    // Timeout expired - revert to Normal mode
                    input_mode = InputMode::Normal;
                    form_state.clear();
                    processing_timeout = None;
                    dirty = true;
                }
            }
        }

        // Poll input (non-blocking with 0ms timeout)
        if poll(Duration::from_millis(0))? {
            if let Ok(Event::Key(key)) = event::read() {
                // Handle input based on current mode
                let (new_mode, action) = handle_input(key, &input_mode, &mut form_state);
                input_mode = new_mode;

                // Send action if generated
                if let Some(act) = action {
                    match &act {
                        TuiAction::Quit => running = false,
                        TuiAction::AddAccount {
                            provider_id: _,
                            api_key: _,
                        } => {
                            // Set processing mode with timeout
                            processing_timeout = Some(
                                std::time::Instant::now()
                                    + Duration::from_secs(PROCESSING_TIMEOUT_SECS),
                            );
                            // Send action to core system
                            let _ = action_tx.try_send(act);
                        }
                        _ => {
                            let _ = action_tx.try_send(act);
                        }
                    }
                }

                dirty = true;
            }
            if let Ok(Event::Resize(_, _)) = event::read() {
                dirty = true;
            }
        }

        // Render if dirty and throttle elapsed
        if dirty && last_render.elapsed() > Duration::from_millis(RENDER_THROTTLE_MS) {
            let state = rx.borrow();
            terminal.draw(|f| {
                draw_dashboard(f, f.area(), &state);

                // Render popup overlays based on input mode
                match input_mode {
                    InputMode::Editing => {
                        render_popup(f, f.area(), &form_state, &input_mode);
                    }
                    InputMode::Processing => {
                        render_spinner(f, f.area(), tick);
                    }
                    InputMode::Normal => {}
                }
            })?;
            dirty = false;
            last_render = std::time::Instant::now();
            tick = tick.wrapping_add(1);
        }

        // Sleep to prevent busy loop
        std::thread::sleep(Duration::from_millis(IDLE_SLEEP_MS));
    }

    // Cleanup terminal
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    terminal.show_cursor()?;

    Ok(())
}

/// Handle keyboard input based on current input mode
fn handle_input(
    key: crossterm::event::KeyEvent,
    input_mode: &InputMode,
    form_state: &mut FormState,
) -> (InputMode, Option<TuiAction>) {
    match input_mode {
        InputMode::Normal => match key.code {
            KeyCode::Char('i') => (InputMode::Editing, None),
            KeyCode::Char('q') | KeyCode::Esc => (InputMode::Normal, Some(TuiAction::Quit)),
            _ => (InputMode::Normal, None),
        },
        InputMode::Editing => match key.code {
            KeyCode::Enter => {
                // Submit form - create AddAccount action
                let action = TuiAction::AddAccount {
                    provider_id: form_state.provider_id.clone(),
                    api_key: form_state.api_key_buffer.clone(),
                };
                form_state.clear();
                (InputMode::Processing, Some(action))
            }
            KeyCode::Esc => {
                form_state.clear();
                (InputMode::Normal, None)
            }
            KeyCode::Char(c) => {
                form_state.api_key_buffer.push(c);
                (InputMode::Editing, None)
            }
            KeyCode::Backspace => {
                form_state.api_key_buffer.pop();
                (InputMode::Editing, None)
            }
            KeyCode::Tab => {
                // Toggle between provider_id and api_key field
                // (simplified - toggle focus)
                (InputMode::Editing, None)
            }
            _ => (InputMode::Editing, None),
        },
        InputMode::Processing => {
            // Input blocked during processing
            (InputMode::Processing, None)
        }
    }
}

/// Calculate centered rectangle for popup overlay
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Render the popup form for adding a new account
fn render_popup<'a>(
    f: &mut ratatui::Frame<'a>,
    area: Rect,
    form_state: &FormState,
    _input_mode: &InputMode,
) {
    // Clear overlay area with semi-transparent background
    f.render_widget(Clear, area);

    let popup_area = centered_rect(60, 40, area);

    // Title
    let title = " Add New Account ";

    // Provider ID field display
    let provider_text = if form_state.provider_id.is_empty() {
        "Enter provider_id..."
    } else {
        &form_state.provider_id
    };

    // SECURITY: Mask API key - never show actual value
    let masked_key = "*".repeat(form_state.api_key_buffer.len().max(1));
    let key_label = if form_state.api_key_buffer.is_empty() {
        "API Key: "
    } else {
        &masked_key
    };

    let content = format!(
        "{}\n\nProvider ID: {}\n\n{}\n\n[Enter] Submit  [Esc] Cancel",
        title, provider_text, key_label
    );

    f.render_widget(
        Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title(title))
            .style(Style::default().fg(Color::White)),
        popup_area,
    );
}

/// Render the processing spinner during async validation
fn render_spinner<'a>(f: &mut ratatui::Frame<'a>, area: Rect, tick: u64) {
    // Calculate centered area for spinner (smaller than popup)
    let spinner_area = centered_rect(40, 20, area);

    // Animated spinner: - \ | /
    let spinner_chars = ['-', '\\', '|', '/'];
    let spin = spinner_chars[(tick % 4) as usize];

    f.render_widget(Clear, area);

    f.render_widget(
        Paragraph::new(format!(" Validating... {} ", spin))
            .block(Block::default().borders(Borders::ALL).title("Processing"))
            .style(Style::default().fg(Color::Yellow)),
        spinner_area,
    );
}

/// Draw the complete dashboard layout
///
/// Layout: Header (10%), Providers (60%), Logs (30%)
fn draw_dashboard(f: &mut Frame, area: Rect, state: &TuiState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(60),
            Constraint::Percentage(30),
        ])
        .split(area);

    draw_header(f, chunks[0], state);
    draw_provider_table(f, chunks[1], state);
    draw_logs(f, chunks[2], state);
}

/// Draw the header with global stats and latency sparkline
fn draw_header(f: &mut Frame, area: Rect, state: &TuiState) {
    let stats = &state.global_stats;
    let content = format!(
        "Requests: {} | Success: {} | Failed: {} | Avg Latency: {:.1}ms | Cost: ${:.4}",
        stats.requests_total,
        stats.requests_success,
        stats.requests_failed,
        stats.avg_latency_ms,
        stats.cost_micro_dollars as f64 / 1_000_000.0
    );

    // Sparkline for latency pulse
    let latency_data: Vec<u64> = state.latency_history.iter().cloned().collect();
    let sparkline = Sparkline::default()
        .data(&latency_data)
        .block(
            Block::default()
                .title("Latency Pulse (50 samples)")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::Cyan));

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    f.render_widget(
        Paragraph::new(content).block(Block::default().title("Global Stats").borders(Borders::ALL)),
        chunks[0],
    );
    f.render_widget(sparkline, chunks[1]);
}

/// Draw the provider health table
///
/// BTreeMap ensures sorted order by provider_id for consistent display
fn draw_provider_table(f: &mut Frame, area: Rect, state: &TuiState) {
    let rows: Vec<Row> = state
        .provider_status
        .iter()
        .map(|(id, metrics)| {
            let status_color = if metrics.circuit_breaker_open {
                Color::Red
            } else if metrics.requests_failed > metrics.requests_success {
                Color::Yellow
            } else {
                Color::Cyan
            };

            let status_text = if metrics.circuit_breaker_open {
                "CIRCUIT OPEN"
            } else {
                "HEALTHY"
            };

            let success_rate = if metrics.requests_success + metrics.requests_failed > 0 {
                (metrics.requests_success as f64
                    / (metrics.requests_success + metrics.requests_failed) as f64
                    * 100.0) as u64
            } else {
                100
            };

            // Use Cell for owned data to avoid lifetime issues
            Row::new(vec![
                Cell::from(id.as_str()),
                Cell::from(status_text),
                Cell::from(format!("{}ms", metrics.latency_ms.unwrap_or(0))),
                Cell::from(format!("{}%", success_rate)),
            ])
            .style(Style::default().fg(status_color))
        })
        .collect();

    let widths = [
        Constraint::Percentage(20),
        Constraint::Percentage(25),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ];

    let table = Table::new(rows, widths)
        .block(
            Block::default()
                .title("Provider Health")
                .borders(Borders::ALL),
        )
        .column_spacing(1);

    f.render_widget(table, area);
}

/// Draw the recent logs panel
fn draw_logs(f: &mut Frame, area: Rect, state: &TuiState) {
    let log_text: String = state
        .log_buffer
        .iter()
        .rev()
        .take(area.height as usize)
        .map(|e| {
            let level_str = match e.level {
                LogLevel::Error => "[ERR]",
                LogLevel::Warn => "[WRN]",
                LogLevel::Info => "[INF]",
                LogLevel::Debug => "[DBG]",
            };
            format!("{} {}", level_str, e.message)
        })
        .collect::<Vec<_>>()
        .join("\n");

    f.render_widget(
        Paragraph::new(log_text)
            .block(Block::default().title("Recent Logs").borders(Borders::ALL))
            .wrap(ratatui::widgets::Wrap { trim: true }),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_constants_defined() {
        const RENDER_OK: () = assert!(RENDER_THROTTLE_MS > 0);
        const IDLE_OK: () = assert!(IDLE_SLEEP_MS > 0);
        let _ = (RENDER_OK, IDLE_OK);
    }
}
