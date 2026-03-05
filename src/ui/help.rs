use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::app::App;

pub fn render(frame: &mut Frame, _app: &App, area: Rect) {
    let help_text = vec![
        Line::from(Span::styled(
            "labgrid-tui Keybindings",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        section("Navigation"),
        binding("j / ↓", "Move down"),
        binding("k / ↑", "Move up"),
        binding("g / Home", "Go to first item"),
        binding("G / End", "Go to last item"),
        Line::from(""),
        section("Views"),
        binding("1", "Places view"),
        binding("2", "Resources view"),
        binding("3", "Exporters view"),
        Line::from(""),
        section("Actions"),
        binding("Enter", "Toggle detail panel"),
        binding("/", "Filter mode"),
        binding(":", "Command mode"),
        binding("Esc", "Close filter / detail / back"),
        binding("?", "Toggle this help"),
        binding("q / Ctrl-C", "Quit"),
        Line::from(""),
        section("Detail Panel"),
        binding("d", "Scroll detail down"),
        binding("u", "Scroll detail up"),
        Line::from(""),
        section("Commands"),
        binding(":places / :p", "Switch to places view"),
        binding(":resources / :r", "Switch to resources view"),
        binding(":exporters / :e", "Switch to exporters view"),
        binding(":quit / :q", "Quit"),
    ];

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

fn section(title: &str) -> Line<'static> {
    Line::from(Span::styled(
        title.to_string(),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ))
}

fn binding(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("  {key:<16}"),
            Style::default().fg(Color::Green),
        ),
        Span::raw(desc.to_string()),
    ])
}
