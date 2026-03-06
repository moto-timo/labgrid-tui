use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Tabs};

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
                    // count unique exporters
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

    let block = Block::default()
        .title(connection_status)
        .title_alignment(Alignment::Right)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if app.connected {
            Color::Green
        } else {
            Color::Red
        }));

    let tabs = Tabs::new(titles)
        .block(block)
        .select(app.view.index())
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .divider("|");

    frame.render_widget(tabs, area);
}
