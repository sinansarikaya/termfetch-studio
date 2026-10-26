//! Top header: brand, version badge and tagline on the left; fastfetch status,
//! distribution and local clock on the right.

use crate::app::state::AppState;
use crate::domain::preset::DEMO_DATA;
use crate::tui::widgets::ui::{fill, line_at, width};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};

/// `fastfetch 2.70.0 (x86_64)` → `2.70.0`.
fn short_version(v: &str) -> &str {
    v.split_whitespace()
        .find(|t| t.chars().next().is_some_and(|c| c.is_ascii_digit()))
        .unwrap_or(v)
}

pub fn render_header(f: &mut Frame, area: Rect, state: &AppState) {
    let pal = state.palette();
    let buf = f.buffer_mut();
    fill(buf, area, Style::new().bg(pal.bg).fg(pal.fg));
    // Roomy layout: one content row with a separator line below.
    let row = if area.height >= 3 {
        Block::new()
            .borders(Borders::BOTTOM)
            .border_style(Style::new().fg(pal.border).bg(pal.bg))
            .render(area, buf);
        Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: 1,
        }
    } else {
        area
    };

    let bg = Style::new().bg(pal.bg);
    let mut right: Vec<Span> = Vec::new();
    if state.unsaved() {
        right.push(Span::styled(" ● unsaved ", ratatui_pill(pal.warn)));
        right.push(Span::styled("  ", bg));
    }
    if state.settings.demo_mode {
        right.push(Span::styled(" ◉ PREVIEW ", ratatui_pill(pal.ok)));
        right.push(Span::styled("  ", bg));
    }
    match &state.fastfetch_version {
        Some(v) => {
            right.push(Span::styled("● ", Style::new().fg(pal.ok).bg(pal.bg)));
            right.push(Span::styled(
                format!("Fastfetch: OK ({})", short_version(v)),
                Style::new().fg(pal.ok).bg(pal.bg),
            ));
        }
        None => {
            right.push(Span::styled("● ", Style::new().fg(pal.error).bg(pal.bg)));
            right.push(Span::styled(
                "Fastfetch: missing",
                Style::new().fg(pal.error).bg(pal.bg),
            ));
        }
    }
    let distro = if state.settings.demo_mode {
        DEMO_DATA.os.trim_end_matches(" x86_64").to_string()
    } else {
        state.sysinfo.distro.clone()
    };
    right.push(Span::styled(
        "  │  ",
        Style::new().fg(pal.border).bg(pal.bg),
    ));
    right.push(Span::styled(
        format!("{} ", state.sysinfo.distro_icon()),
        Style::new().fg(pal.info).bg(pal.bg),
    ));
    right.push(Span::styled(distro, Style::new().fg(pal.fg).bg(pal.bg)));
    right.push(Span::styled(
        "  │  ",
        Style::new().fg(pal.border).bg(pal.bg),
    ));
    right.push(Span::styled(
        "\u{f0954} ",
        Style::new().fg(pal.muted).bg(pal.bg),
    ));
    right.push(Span::styled(
        state.clock.clone(),
        Style::new()
            .fg(pal.fg)
            .bg(pal.bg)
            .add_modifier(Modifier::BOLD),
    ));
    right.push(Span::styled(" ", bg));
    let right_w: u16 = right.iter().map(|s| width(&s.content)).sum();

    let mut left = vec![
        Span::styled(
            "TF· ",
            Style::new()
                .fg(pal.primary)
                .bg(pal.raised)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " TermFetch Studio ",
            Style::new()
                .fg(pal.fg)
                .bg(pal.bg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" v2.0 ", Style::new().fg(pal.muted).bg(pal.raised)),
    ];
    let left_w: u16 = left.iter().map(|s| width(&s.content)).sum();
    let tagline = "   Customize your terminal, your way.";
    if left_w + width(tagline) + right_w + 2 <= row.width {
        left.push(Span::styled(tagline, Style::new().fg(pal.link).bg(pal.bg)));
    }
    line_at(buf, row, 0, Line::from(left));
    if left_w + right_w < row.width {
        line_at(buf, row, 0, Line::from(right).right_aligned());
    }
}

fn ratatui_pill(fill: ratatui::style::Color) -> Style {
    crate::tui::palette::Palette::pill(fill).add_modifier(Modifier::BOLD)
}

#[cfg(test)]
mod tests {
    #[test]
    fn version_is_shortened() {
        assert_eq!(super::short_version("fastfetch 2.70.0 (x86_64)"), "2.70.0");
        assert_eq!(super::short_version("weird"), "weird");
    }
}
