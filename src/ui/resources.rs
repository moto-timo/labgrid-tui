use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};

use super::theme;
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let paths = app.filtered_resource_paths();

    let header = Row::new(vec![
        "EXPORTER", "GROUP", "NAME", "CLASS", "AVAIL", "ACQUIRED",
    ])
    .style(theme::header_style())
    .bottom_margin(1);

    let rows: Vec<Row> = paths
        .iter()
        .filter_map(|path| {
            let res = app.resources.get(path)?;
            let avail_style = if res.available {
                theme::avail_style()
            } else {
                theme::unavail_style()
            };
            let acq_style = if res.acquired.is_some() {
                theme::acquired_style()
            } else {
                theme::row_style()
            };

            Some(
                Row::new(vec![
                    Cell::from(res.path.exporter.clone()),
                    Cell::from(res.path.group.clone()),
                    Cell::from(res.path.name.clone()),
                    Cell::from(res.cls.clone()),
                    Cell::from(res.avail_display().to_string()).style(avail_style),
                    Cell::from(res.acquired_display().to_string()).style(acq_style),
                ])
                .style(theme::row_style()),
            )
        })
        .collect();

    let filter_note = match &app.filter {
        Some(f) => format!(" (filter: {f})"),
        None => String::new(),
    };

    let table = Table::new(
        rows,
        [
            Constraint::Min(12),
            Constraint::Min(10),
            Constraint::Min(15),
            Constraint::Min(15),
            Constraint::Length(7),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(format!(" Resources{filter_note} "))
            .borders(Borders::ALL)
            .border_style(theme::border_style()),
    )
    .row_highlight_style(theme::row_highlight_style())
    .highlight_symbol(theme::HIGHLIGHT_SYMBOL);

    let mut state = TableState::default();
    if !paths.is_empty() {
        state.select(Some(app.selected_index.min(paths.len().saturating_sub(1))));
    }

    frame.render_stateful_widget(table, area, &mut state);
}
