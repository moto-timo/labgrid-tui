use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};

use super::theme;
use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let names = app.filtered_place_names();

    let header = Row::new(vec!["NAME", "ALIASES", "COMMENT", "ACQUIRED", "MATCHES", "TAGS"])
        .style(theme::header_style())
        .bottom_margin(1);

    let rows: Vec<Row> = names
        .iter()
        .map(|name| {
            let place = &app.places[name];
            let acq_style = if place.acquired.is_some() {
                theme::acquired_style()
            } else {
                theme::row_style()
            };

            Row::new(vec![
                Cell::from(place.name.clone()),
                Cell::from(place.aliases_display()),
                Cell::from(place.comment.clone()),
                Cell::from(place.acquired_display().to_string()).style(acq_style),
                Cell::from(place.matches_display()),
                Cell::from(place.tags_display()),
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
            Constraint::Min(12),
            Constraint::Min(15),
            Constraint::Min(12),
            Constraint::Percentage(25),
            Constraint::Min(12),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(format!(" Places{filter_note} "))
            .borders(Borders::ALL)
            .border_style(theme::border_style()),
    )
    .row_highlight_style(theme::row_highlight_style())
    .highlight_symbol(theme::HIGHLIGHT_SYMBOL);

    let mut state = TableState::default();
    if !names.is_empty() {
        state.select(Some(app.selected_index.min(names.len().saturating_sub(1))));
    }

    frame.render_stateful_widget(table, area, &mut state);
}
