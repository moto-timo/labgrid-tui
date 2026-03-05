use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

use crate::app::{App, InputMode, View};
use crate::grpc::client::CoordinatorEvent;

/// All events the main loop processes.
pub enum AppEvent {
    /// Terminal key/resize event.
    Terminal(CrosstermEvent),
    /// Tick for periodic UI refresh.
    Tick,
    /// Event from the gRPC coordinator stream.
    Coordinator(CoordinatorEvent),
}

/// Spawn a task that reads terminal events and sends ticks.
pub fn spawn_terminal_event_reader(
    tx: mpsc::UnboundedSender<AppEvent>,
    tick_rate: Duration,
) {
    tokio::task::spawn_blocking(move || {
        loop {
            if event::poll(tick_rate).unwrap_or(false) {
                if let Ok(ev) = event::read() {
                    if tx.send(AppEvent::Terminal(ev)).is_err() {
                        break;
                    }
                }
            } else if tx.send(AppEvent::Tick).is_err() {
                break;
            }
        }
    });
}

/// Process a terminal event against the app state.
/// Returns true if the app should quit.
pub fn handle_terminal_event(app: &mut App, event: CrosstermEvent) -> bool {
    match event {
        CrosstermEvent::Key(key) => handle_key(app, key),
        CrosstermEvent::Resize(_, _) => false, // ratatui handles resize automatically
        _ => false,
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    // Input mode handling
    match app.input_mode {
        InputMode::Filter | InputMode::Command => {
            return handle_input_key(app, key);
        }
        InputMode::Normal => {}
    }

    // Help overlay handling
    if app.help_open {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => {
                app.help_open = false;
            }
            _ => {}
        }
        return false;
    }

    // Normal mode
    match key.code {
        // Quit
        KeyCode::Char('q') => return true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return true,

        // View switching
        KeyCode::Char('1') => app.switch_view(View::Places),
        KeyCode::Char('2') => app.switch_view(View::Resources),
        KeyCode::Char('3') => app.switch_view(View::Exporters),

        // Navigation
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Up | KeyCode::Char('k') => app.select_prev(),
        KeyCode::Home | KeyCode::Char('g') => app.select_first(),
        KeyCode::End | KeyCode::Char('G') => app.select_last(),

        // Detail panel
        KeyCode::Enter => app.toggle_detail(),
        KeyCode::Char('d') if app.detail_open => app.scroll_detail_down(),
        KeyCode::Char('u') if app.detail_open => app.scroll_detail_up(),

        // Filter / command
        KeyCode::Char('/') => app.enter_filter_mode(),
        KeyCode::Char(':') => app.enter_command_mode(),
        KeyCode::Esc => {
            if app.filter.is_some() {
                app.clear_filter();
            } else if app.detail_open {
                app.detail_open = false;
            }
        }

        // Help
        KeyCode::Char('?') => app.toggle_help(),

        _ => {}
    }

    false
}

fn handle_input_key(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Enter => {
            app.submit_input();
        }
        KeyCode::Esc => {
            app.exit_input_mode();
        }
        KeyCode::Char(c) => {
            app.input_buffer.push(c);
        }
        KeyCode::Backspace => {
            app.input_buffer.pop();
        }
        _ => {}
    }
    false
}
