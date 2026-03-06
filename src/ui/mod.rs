pub mod command;
pub mod detail;
pub mod exporters;
pub mod header;
pub mod help;
pub mod places;
pub mod resources;

use ratatui::prelude::*;
use ratatui::widgets::Clear;

use crate::app::{App, InputMode, View};

/// Main rendering entrypoint — dispatches to sub-views.
pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // header
            Constraint::Min(5),    // main content
            Constraint::Length(1), // status / input bar
        ])
        .split(frame.area());

    // Header bar
    header::render(frame, app, chunks[0]);

    // Main content area — optionally split for detail panel
    if app.detail_open {
        let hsplit = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(chunks[1]);

        render_table(frame, app, hsplit[0]);
        detail::render(frame, app, hsplit[1]);
    } else {
        render_table(frame, app, chunks[1]);
    }

    // Bottom bar — status or input
    match app.input_mode {
        InputMode::Filter => command::render_filter(frame, app, chunks[2]),
        InputMode::Command => command::render_command(frame, app, chunks[2]),
        InputMode::Normal => command::render_status(frame, app, chunks[2]),
    }

    // Help overlay (modal)
    if app.help_open {
        let area = centered_rect(60, 70, frame.area());
        frame.render_widget(Clear, area);
        help::render(frame, app, area);
    }
}

fn render_table(frame: &mut Frame, app: &App, area: Rect) {
    match app.view {
        View::Places => places::render(frame, app, area),
        View::Resources => resources::render(frame, app, area),
        View::Exporters => exporters::render(frame, app, area),
    }
}

/// Create a centered rectangle for modal overlays.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
