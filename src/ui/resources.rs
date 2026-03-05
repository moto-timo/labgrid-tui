use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Row, Table, TableState};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let paths = app.filtered_resource_paths();

    let header = Row::new(vec![
        "EXPORTER", "GROUP", "NAME", "CLASS", "AVAIL", "ACQUIRED",
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )
    .bottom_margin(1);

    let rows: Vec<Row> = paths
        .iter()
        .filter_map(|path| {
            let res = app.resources.get(path)?;
            let avail_style = if res.available {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };
            let acquired_style = if res.acquired.is_some() {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            };

            Some(Row::new(vec![
                Cell::from(res.path.exporter.clone()),
                Cell::from(res.path.group.clone()),
                Cell::from(res.path.name.clone()),
                Cell::from(res.cls.clone()),
                Cell::from(res.avail_display().to_string()).style(avail_style),
                Cell::from(res.acquired_display().to_string()).style(acquired_style),
            ]))
        })
        .collect();

    let filter_note = match &app.filter {
        Some(f) => format!(" (filter: {f})"),
        None => String::new(),
    };

    let table = Table::new(
        rows,
        [
            Constraint::Min(12),        // EXPORTER
            Constraint::Min(10),        // GROUP
            Constraint::Min(15),        // NAME
            Constraint::Min(15),        // CLASS
            Constraint::Length(7),      // AVAIL
            Constraint::Percentage(15), // ACQUIRED
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(format!(" Resources{filter_note} "))
            .borders(Borders::ALL),
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▶ ");

    let mut state = TableState::default();
    if !paths.is_empty() {
        state.select(Some(app.selected_index.min(paths.len().saturating_sub(1))));
    }

    frame.render_stateful_widget(table, area, &mut state);
}
