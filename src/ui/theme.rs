//! Green phosphor (Matrix) color theme.
//!
//! All colors used across the TUI are defined here so the palette
//! can be adjusted in a single place.

use ratatui::prelude::*;

// ── Base palette ──────────────────────────────────────────────────

/// Bright green — primary text, active elements.
pub const GREEN_BRIGHT: Color = Color::Rgb(0, 255, 65);

/// Medium green — secondary text, borders.
pub const GREEN_MEDIUM: Color = Color::Rgb(0, 190, 50);

/// Dim green — tertiary text, inactive elements.
pub const GREEN_DIM: Color = Color::Rgb(0, 120, 30);

/// Very dim green — subtle accents, disabled text.
pub const GREEN_FAINT: Color = Color::Rgb(0, 70, 20);

/// Background black.
pub const BG: Color = Color::Rgb(0, 10, 0);

/// Slightly lighter background for selection highlight.
pub const BG_HIGHLIGHT: Color = Color::Rgb(0, 40, 10);

/// Amber accent — warnings, acquired status.
pub const AMBER: Color = Color::Rgb(255, 176, 0);

/// Red accent — errors, unavailable.
pub const RED: Color = Color::Rgb(200, 40, 40);

// ── Semantic styles ───────────────────────────────────────────────

/// Column headers in table views.
pub fn header_style() -> Style {
    Style::default()
        .fg(GREEN_BRIGHT)
        .add_modifier(Modifier::BOLD)
}

/// Normal text in table rows.
pub fn row_style() -> Style {
    Style::default().fg(GREEN_MEDIUM)
}

/// Highlighted / selected row.
pub fn row_highlight_style() -> Style {
    Style::default()
        .fg(GREEN_BRIGHT)
        .bg(BG_HIGHLIGHT)
        .add_modifier(Modifier::BOLD)
}

/// Available / healthy status.
pub fn avail_style() -> Style {
    Style::default().fg(GREEN_BRIGHT)
}

/// Unavailable / unhealthy status.
pub fn unavail_style() -> Style {
    Style::default().fg(RED)
}

/// Acquired / in-use status.
pub fn acquired_style() -> Style {
    Style::default().fg(AMBER)
}

/// Default border style.
pub fn border_style() -> Style {
    Style::default().fg(GREEN_DIM)
}

/// Border when connected.
pub fn border_connected() -> Style {
    Style::default().fg(GREEN_BRIGHT)
}

/// Border when disconnected.
pub fn border_disconnected() -> Style {
    Style::default().fg(RED)
}

/// Status bar (normal mode).
pub fn status_bar() -> Style {
    Style::default().fg(GREEN_BRIGHT).bg(BG_HIGHLIGHT)
}

/// Status bar when disconnected.
pub fn status_bar_disconnected() -> Style {
    Style::default().fg(GREEN_BRIGHT).bg(RED)
}

/// Filter input bar.
pub fn filter_bar() -> Style {
    Style::default().fg(GREEN_BRIGHT).bg(Color::Rgb(0, 60, 30))
}

/// Command input bar.
pub fn command_bar() -> Style {
    Style::default().fg(GREEN_BRIGHT).bg(Color::Rgb(30, 60, 0))
}

/// Active tab in header.
pub fn tab_active() -> Style {
    Style::default()
        .fg(GREEN_BRIGHT)
        .bg(BG_HIGHLIGHT)
        .add_modifier(Modifier::BOLD)
}

/// Inactive tab in header.
pub fn tab_inactive() -> Style {
    Style::default().fg(GREEN_DIM)
}

/// Section headers in detail/help views.
pub fn section_header() -> Style {
    Style::default()
        .fg(GREEN_BRIGHT)
        .add_modifier(Modifier::BOLD)
}

/// Field labels in detail view.
pub fn field_label() -> Style {
    Style::default()
        .fg(GREEN_BRIGHT)
        .add_modifier(Modifier::BOLD)
}

/// Field values in detail view.
pub fn field_value() -> Style {
    Style::default().fg(GREEN_MEDIUM)
}

/// Keybinding keys in help view.
pub fn help_key() -> Style {
    Style::default().fg(GREEN_BRIGHT)
}

/// Keybinding descriptions in help view.
pub fn help_desc() -> Style {
    Style::default().fg(GREEN_MEDIUM)
}

/// Help/detail panel border.
pub fn panel_border() -> Style {
    Style::default().fg(GREEN_MEDIUM)
}

/// The highlight symbol for selected rows.
pub const HIGHLIGHT_SYMBOL: &str = "▶ ";
