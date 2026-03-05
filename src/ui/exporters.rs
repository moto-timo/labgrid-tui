use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let exporters = app.exporters();

    let header = Row::new(vec![
        "EXPORTER", "RESOURCES", "AVAILABLE", "UNAVAILABLE", "ACQUIRED", "CLASSES",
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )
    .bottom_margin(1);

    let rows: Vec<Row> = exporters
        .iter()
        .map(|exp| {
            Row::new(vec![
                Cell::from(exp.name.clone()),
                Cell::from(exp.total_resources.to_string()),
                Cell::from(exp.available.to_string())
                    .style(Style::default().fg(Color::Green)),
                Cell::from(exp.unavailable.to_string()).style(
                    if exp.unavailable > 0 {
                        Style::default().fg(Color::Red)
                    } else {
                        Style::default()
                    },
                ),
                Cell::from(exp.acquired.to_string()).style(
                    if exp.acquired > 0 {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default()
                    },
                ),
                Cell::from(exp.classes_display()),
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
            Constraint::Min(15),        // EXPORTER
            Constraint::Length(10),      // RESOURCES
            Constraint::Length(10),      // AVAILABLE
            Constraint::Length(12),      // UNAVAILABLE
            Constraint::Length(10),      // ACQUIRED
            Constraint::Percentage(25), // CLASSES
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(format!(" Exporters{filter_note} "))
            .borders(Borders::ALL),
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▶ ");

    let mut state = TableState::default();
    if !exporters.is_empty() {
        state.select(Some(
            app.selected_index.min(exporters.len().saturating_sub(1)),
        ));
    }

    frame.render_stateful_widget(table, area, &mut state);
}
