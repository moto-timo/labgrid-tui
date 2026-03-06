use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Tabs};

use super::theme;
use crate::app::{App, View};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = View::all()
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let num = i + 1;
            let label = v.label();
            let count = match v {
                View::Places => app.places.len(),
                View::Resources => app.resources.len(),
                View::Exporters => {
                    app.resources
                        .values()
                        .map(|r| &r.path.exporter)
                        .collect::<std::collections::HashSet<_>>()
                        .len()
                }
            };
            Line::from(format!(" {num}:{label} ({count}) "))
        })
        .collect();

    let connection_status = if app.connected {
        format!(" ● {}", app.coordinator_url)
    } else {
        format!(" ○ {}", app.coordinator_url)
    };

    let border = if app.connected {
        theme::border_connected()
    } else {
        theme::border_disconnected()
    };

    let block = Block::default()
        .title(connection_status)
        .title_alignment(Alignment::Right)
        .borders(Borders::ALL)
        .border_style(border);

    let tabs = Tabs::new(titles)
        .block(block)
        .select(app.view.index())
        .style(theme::tab_inactive())
        .highlight_style(theme::tab_active())
        .divider("|");

    frame.render_widget(tabs, area);
}
