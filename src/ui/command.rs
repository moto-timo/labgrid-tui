use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::app::App;

/// Render the status bar (normal mode).
pub fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let filter_info = match &app.filter {
        Some(f) => format!(" | filter: {f}"),
        None => String::new(),
    };

    let text = format!(
        " {} | {}{} | ? for help",
        app.view.label(),
        app.status_message,
        filter_info,
    );

    let style = if app.connected {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White).bg(Color::Red)
    };

    let paragraph = Paragraph::new(text).style(style);
    frame.render_widget(paragraph, area);
}

/// Render the filter input bar.
pub fn render_filter(frame: &mut Frame, app: &App, area: Rect) {
    let text = format!("/{}", app.input_buffer);
    let paragraph = Paragraph::new(text).style(
        Style::default()
            .fg(Color::White)
            .bg(Color::Blue),
    );
    frame.render_widget(paragraph, area);

    // Position cursor
    frame.set_cursor_position((
        area.x + 1 + app.input_buffer.len() as u16,
        area.y,
    ));
}

/// Render the command input bar.
pub fn render_command(frame: &mut Frame, app: &App, area: Rect) {
    let text = format!(":{}", app.input_buffer);
    let paragraph = Paragraph::new(text).style(
        Style::default()
            .fg(Color::White)
            .bg(Color::Magenta),
    );
    frame.render_widget(paragraph, area);

    // Position cursor
    frame.set_cursor_position((
        area.x + 1 + app.input_buffer.len() as u16,
        area.y,
    ));
}
