use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let names = app.filtered_place_names();

    let header = Row::new(vec!["NAME", "ALIASES", "COMMENT", "ACQUIRED", "MATCHES", "TAGS"])
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows: Vec<Row> = names
        .iter()
        .map(|name| {
            let place = &app.places[name];
            let acquired_style = if place.acquired.is_some() {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(place.name.clone()),
                Cell::from(place.aliases_display()),
                Cell::from(place.comment.clone()),
                Cell::from(place.acquired_display().to_string()).style(acquired_style),
                Cell::from(place.matches_display()),
                Cell::from(place.tags_display()),
            ])
        })
        .collect();

    let filter_note = match &app.filter {
        Some(f) => format!(" (filter: {f})"),
        None => String::new(),
    };

    let table = Table::new(
        rows,
        [
            Constraint::Min(15),        // NAME
            Constraint::Min(12),        // ALIASES
            Constraint::Min(15),        // COMMENT
            Constraint::Min(12),        // ACQUIRED
            Constraint::Percentage(25), // MATCHES
            Constraint::Min(12),        // TAGS
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(format!(" Places{filter_note} "))
            .borders(Borders::ALL),
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▶ ");

    let mut state = TableState::default();
    if !names.is_empty() {
        state.select(Some(app.selected_index.min(names.len().saturating_sub(1))));
    }

    frame.render_stateful_widget(table, area, &mut state);
}
