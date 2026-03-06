use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::theme;
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
        theme::status_bar()
    } else {
        theme::status_bar_disconnected()
    };

    let paragraph = Paragraph::new(text).style(style);
    frame.render_widget(paragraph, area);
}

/// Render the filter input bar.
pub fn render_filter(frame: &mut Frame, app: &App, area: Rect) {
    let text = format!("/{}", app.input_buffer);
    let paragraph = Paragraph::new(text).style(theme::filter_bar());
    frame.render_widget(paragraph, area);

    frame.set_cursor_position((
        area.x + 1 + app.input_buffer.len() as u16,
        area.y,
    ));
}

/// Render the command input bar.
pub fn render_command(frame: &mut Frame, app: &App, area: Rect) {
    let text = format!(":{}", app.input_buffer);
    let paragraph = Paragraph::new(text).style(theme::command_bar());
    frame.render_widget(paragraph, area);

    frame.set_cursor_position((
        area.x + 1 + app.input_buffer.len() as u16,
        area.y,
    ));
}
