use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use super::theme;
use crate::app::{App, View};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let (title, content) = match app.view {
        View::Places => render_place_detail(app),
        View::Resources => render_resource_detail(app),
        View::Exporters => render_exporter_detail(app),
    };

    let block = Block::default()
        .title(format!(" {title} "))
        .borders(Borders::ALL)
        .border_style(theme::panel_border());

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll, 0));

    frame.render_widget(paragraph, area);
}

fn render_place_detail(app: &App) -> (String, Vec<Line<'static>>) {
    let names = app.filtered_place_names();
    let idx = app.selected_index.min(names.len().saturating_sub(1));

    let Some(name) = names.get(idx) else {
        return ("No selection".into(), vec![]);
    };
    let Some(place) = app.places.get(name) else {
        return ("Not found".into(), vec![]);
    };

    let mut lines = Vec::new();

    lines.push(field_line("Name", &place.name));
    lines.push(field_line("Comment", &place.comment));
    lines.push(field_line("Aliases", &place.aliases_display()));
    lines.push(field_line(
        "Acquired",
        place.acquired_display(),
    ));
    lines.push(field_line(
        "Reservation",
        place.reservation.as_deref().unwrap_or("none"),
    ));
    lines.push(Line::from(""));

    lines.push(section_header("Tags"));
    if place.tags.is_empty() {
        lines.push(dim_line("  (none)"));
    } else {
        for (k, v) in &place.tags {
            if v.is_empty() {
                lines.push(body_line(&format!("  {k}")));
            } else {
                lines.push(body_line(&format!("  {k} = {v}")));
            }
        }
    }
    lines.push(Line::from(""));

    lines.push(section_header("Matches"));
    if place.matches.is_empty() {
        lines.push(dim_line("  (none)"));
    } else {
        for m in &place.matches {
            lines.push(body_line(&format!("  {}", m.pattern_display())));
        }
    }
    lines.push(Line::from(""));

    lines.push(section_header("Acquired Resources"));
    if place.acquired_resources.is_empty() {
        lines.push(dim_line("  (none)"));
    } else {
        for r in &place.acquired_resources {
            lines.push(body_line(&format!("  {r}")));
        }
    }
    lines.push(Line::from(""));

    lines.push(section_header("Allowed"));
    if place.allowed.is_empty() {
        lines.push(dim_line("  (none)"));
    } else {
        for a in &place.allowed {
            lines.push(body_line(&format!("  {a}")));
        }
    }

    (format!("Place: {}", place.name), lines)
}

fn render_resource_detail(app: &App) -> (String, Vec<Line<'static>>) {
    let paths = app.filtered_resource_paths();
    let idx = app.selected_index.min(paths.len().saturating_sub(1));

    let Some(path) = paths.get(idx) else {
        return ("No selection".into(), vec![]);
    };
    let Some(res) = app.resources.get(path) else {
        return ("Not found".into(), vec![]);
    };

    let mut lines = Vec::new();

    lines.push(field_line("Exporter", &res.path.exporter));
    lines.push(field_line("Group", &res.path.group));
    lines.push(field_line("Name", &res.path.name));
    lines.push(field_line("Class", &res.cls));
    lines.push(field_line("Available", res.avail_display()));
    lines.push(field_line("Acquired", res.acquired_display()));
    lines.push(Line::from(""));

    lines.push(section_header("Parameters"));
    if res.params.is_empty() {
        lines.push(dim_line("  (none)"));
    } else {
        for (k, v) in &res.params {
            lines.push(body_line(&format!("  {k} = {v}")));
        }
    }
    lines.push(Line::from(""));

    lines.push(section_header("Extra"));
    if res.extra.is_empty() {
        lines.push(dim_line("  (none)"));
    } else {
        for (k, v) in &res.extra {
            lines.push(body_line(&format!("  {k} = {v}")));
        }
    }

    (format!("Resource: {path}"), lines)
}

fn render_exporter_detail(app: &App) -> (String, Vec<Line<'static>>) {
    let exporters = app.exporters();
    let idx = app.selected_index.min(exporters.len().saturating_sub(1));

    let Some(exp) = exporters.get(idx) else {
        return ("No selection".into(), vec![]);
    };

    let mut lines = Vec::new();

    lines.push(field_line("Name", &exp.name));
    lines.push(field_line("Total Resources", &exp.total_resources.to_string()));
    lines.push(field_line("Available", &exp.available.to_string()));
    lines.push(field_line("Unavailable", &exp.unavailable.to_string()));
    lines.push(field_line("Acquired", &exp.acquired.to_string()));
    lines.push(Line::from(""));

    lines.push(section_header("Resource Classes"));
    for cls in &exp.resource_classes {
        lines.push(body_line(&format!("  {cls}")));
    }
    lines.push(Line::from(""));

    lines.push(section_header("Resources"));
    let mut resources: Vec<_> = app
        .resources
        .values()
        .filter(|r| r.path.exporter == exp.name)
        .collect();
    resources.sort_by_key(|r| (&r.path.group, &r.path.name));

    for res in resources {
        let status = if res.available { "✓" } else { "✗" };
        let acquired = match &res.acquired {
            Some(a) if !a.is_empty() => format!(" [{a}]"),
            _ => String::new(),
        };
        lines.push(body_line(&format!(
            "  {status} {}/{} ({}){acquired}",
            res.path.group, res.path.name, res.cls
        )));
    }

    (format!("Exporter: {}", exp.name), lines)
}

fn field_line(label: &str, value: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label}: "), theme::field_label()),
        Span::styled(value.to_string(), theme::field_value()),
    ])
}

fn section_header(title: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("── {title} ──"),
        theme::section_header(),
    ))
}

fn body_line(text: &str) -> Line<'static> {
    Line::from(Span::styled(text.to_string(), theme::field_value()))
}

fn dim_line(text: &str) -> Line<'static> {
    Line::from(Span::styled(
        text.to_string(),
        Style::default().fg(theme::GREEN_FAINT),
    ))
}
