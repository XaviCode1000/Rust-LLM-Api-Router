//! TUI Engine - Terminal User Interface render loop and widgets
//!
//! This module provides the TUI engine with real-time dashboard visualization
//! using Ratatui. It renders provider health, latency sparklines, and log streaming.

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
    widgets::{Block, Borders, Cell, Paragraph, Row, Sparkline, Table},
    Frame, Terminal,
};
use tokio::sync::watch;

use crate::presentation::tui::state::{LogLevel, TuiState};

/// Render throttle to prevent excessive CPU usage (~60fps max)
const RENDER_THROTTLE_MS: u64 = 16;

/// Idle sleep to prevent busy loop
const IDLE_SLEEP_MS: u64 = 10;

/// Run the TUI engine with the given state receiver
///
/// # Arguments
/// * `rx` - Watch receiver for TuiState updates
///
/// # Returns
/// * `Ok(())` on clean exit
/// * `Err(Box<dyn std::error::Error>)` on terminal error
pub fn run(rx: watch::Receiver<TuiState>) -> Result<(), Box<dyn std::error::Error>> {
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

    // Get initial state (ignore - we'll check has_changed)
    let _ = rx.borrow();

    while running {
        // Check for state updates (non-blocking)
        if rx.has_changed().unwrap_or(false) {
            dirty = true;
        }

        // Poll input (non-blocking with 0ms timeout)
        if poll(Duration::from_millis(0))? {
            if let Ok(Event::Key(key)) = event::read() {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => running = false,
                    _ => dirty = true,
                }
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
            })?;
            dirty = false;
            last_render = std::time::Instant::now();
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
        assert!(RENDER_THROTTLE_MS > 0);
        assert!(IDLE_SLEEP_MS > 0);
    }
}
