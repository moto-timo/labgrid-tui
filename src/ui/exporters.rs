use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};

use super::theme;
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let exporters = app.exporters();

    let header = Row::new(vec![
        "EXPORTER", "RESOURCES", "AVAILABLE", "UNAVAILABLE", "ACQUIRED", "CLASSES",
    ])
    .style(theme::header_style())
    .bottom_margin(1);

    let rows: Vec<Row> = exporters
        .iter()
        .map(|exp| {
            Row::new(vec![
                Cell::from(exp.name.clone()),
                Cell::from(exp.total_resources.to_string()),
                Cell::from(exp.available.to_string())
                    .style(theme::avail_style()),
                Cell::from(exp.unavailable.to_string()).style(
                    if exp.unavailable > 0 {
                        theme::unavail_style()
                    } else {
                        theme::row_style()
                    },
                ),
                Cell::from(exp.acquired.to_string()).style(
                    if exp.acquired > 0 {
                        theme::acquired_style()
                    } else {
                        theme::row_style()
                    },
                ),
                Cell::from(exp.classes_display()),
            ])
            .style(theme::row_style())
        })
        .collect();

    let filter_note = match &app.filter {
        Some(f) => format!(" (filter: {f})"),
        None => String::new(),
    };

    let table = Table::new(
        rows,
        [
            Constraint::Min(15),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Percentage(25),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(format!(" Exporters{filter_note} "))
            .borders(Borders::ALL)
            .border_style(theme::border_style()),
    )
    .row_highlight_style(theme::row_highlight_style())
    .highlight_symbol(theme::HIGHLIGHT_SYMBOL);

    let mut state = TableState::default();
    if !exporters.is_empty() {
        state.select(Some(
            app.selected_index.min(exporters.len().saturating_sub(1)),
        ));
    }

    frame.render_stateful_widget(table, area, &mut state);
}
