//! Left sidebar: navigation pills on top, context cards (current theme,
//! current preset, config path) anchored to the bottom.

use crate::app::state::{AppState, Focus, Hit, Screen};
use crate::tui::palette::Palette;
use crate::tui::widgets::ui::{
    draw_thumbnail, ellipsize, fill, line_at, theme_color_count, theme_dots, width,
};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};

/// Rows the context cards need (theme 5, preset 4, config path 3).
const CARDS_H: u16 = 13;

fn home_relative(p: &std::path::Path) -> String {
    let s = p.display().to_string();
    match directories::BaseDirs::new().map(|b| b.home_dir().display().to_string()) {
        Some(home) if s.starts_with(&home) => format!("~{}", &s[home.len()..]),
        _ => s,
    }
}

pub fn render_sidebar(f: &mut Frame, area: Rect, state: &mut AppState) {
    let pal = state.palette();
    let buf = f.buffer_mut();
    fill(buf, area, Style::new().bg(pal.bg).fg(pal.fg));
    Block::new()
        .borders(Borders::RIGHT)
        .border_style(Style::new().fg(pal.border).bg(pal.bg))
        .render(area, buf);
    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(3),
        height: area.height.saturating_sub(1),
    };
    if inner.width < 6 || inner.height == 0 {
        return;
    }

    // Backup / Doctor appear as a sub-entry under Settings while active.
    let mut entries: Vec<(Screen, bool)> = Screen::NAV.iter().map(|&s| (s, false)).collect();
    if matches!(state.screen, Screen::Backup | Screen::Doctor) {
        entries.push((state.screen, true));
    }
    let cards = inner.height > entries.len() as u16 * 2 + CARDS_H;
    let spacing = if inner.height >= entries.len() as u16 * 2 + if cards { CARDS_H } else { 0 } {
        2
    } else {
        1
    };
    let focused = state.focus == Focus::Sidebar;
    let w = usize::from(inner.width);

    for (i, &(screen, sub)) in entries.iter().enumerate() {
        let dy = i as u16 * spacing;
        if dy >= inner.height {
            break;
        }
        let active = screen == state.screen;
        let num = format!(" {} ", screen.index() + 1);
        let label = if sub {
            format!("   └ {}  {}", screen.icon(), screen.title())
        } else {
            format!(" {}  {}", screen.icon(), screen.title())
        };
        let label = ellipsize(&label, w.saturating_sub(num.len() + 1));
        let pad = w.saturating_sub(width(&label) as usize + num.len());
        let line = if active {
            let (base, badge) = if focused {
                (
                    Palette::pill(pal.accent_fill).add_modifier(Modifier::BOLD),
                    Style::new()
                        .fg(pal.accent_fill)
                        .bg(Palette::on(pal.accent_fill))
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                (
                    Style::new()
                        .fg(pal.primary)
                        .bg(pal.sel_bg)
                        .add_modifier(Modifier::BOLD),
                    Style::new()
                        .fg(pal.fg)
                        .bg(pal.raised)
                        .add_modifier(Modifier::BOLD),
                )
            };
            Line::from(vec![
                Span::styled(label, base),
                Span::styled(" ".repeat(pad.saturating_sub(1)), base),
                Span::styled(num, badge),
                Span::styled(" ", base),
            ])
        } else {
            Line::from(vec![
                Span::styled(
                    label,
                    Style::new()
                        .fg(if sub { pal.muted } else { pal.fg })
                        .bg(pal.bg),
                ),
                Span::styled(" ".repeat(pad.saturating_sub(1)), Style::new().bg(pal.bg)),
                Span::styled(num, Style::new().fg(pal.dim).bg(pal.raised)),
                Span::styled(" ", Style::new().bg(pal.bg)),
            ])
        };
        line_at(buf, inner, dy, line);
        state.hits.push((
            Rect {
                y: inner.y + dy,
                height: 1,
                ..inner
            },
            Hit::Nav(screen),
        ));
    }

    if !cards {
        return;
    }
    let top = inner.y + inner.height - CARDS_H;
    let c = Rect {
        y: top,
        height: CARDS_H,
        ..inner
    };
    let header = |t: &str| {
        Line::from(Span::styled(
            t.to_string(),
            Style::new()
                .fg(pal.dim)
                .bg(pal.bg)
                .add_modifier(Modifier::BOLD),
        ))
    };
    let sep = Line::from(Span::styled(
        "─".repeat(w),
        Style::new().fg(pal.border).bg(pal.bg),
    ));

    // CURRENT THEME
    let theme = state.current_theme();
    line_at(buf, c, 0, sep.clone());
    line_at(buf, c, 1, header("CURRENT THEME"));
    let thumb_w = 9.min(inner.width / 3);
    draw_thumbnail(
        buf,
        Rect {
            x: c.x,
            y: c.y + 2,
            width: thumb_w,
            height: 3,
        },
        theme,
    );
    let text = Rect {
        x: c.x + thumb_w + 1,
        width: c.width.saturating_sub(thumb_w + 1),
        ..c
    };
    let tw = usize::from(text.width);
    line_at(
        buf,
        text,
        2,
        Line::from(Span::styled(
            ellipsize(&theme.name, tw),
            Style::new()
                .fg(pal.fg)
                .bg(pal.bg)
                .add_modifier(Modifier::BOLD),
        )),
    );
    line_at(
        buf,
        text,
        3,
        Line::from(Span::styled(
            format!("{} colors", theme_color_count(theme)),
            Style::new().fg(pal.muted).bg(pal.bg),
        )),
    );
    // Under the art, across the full width, so the dots keep their spacing.
    line_at(buf, c, 5, Line::from(theme_dots(theme, c.width >= 15)));

    // CURRENT PRESET
    let preset = state.current_preset();
    line_at(buf, c, 6, sep.clone());
    line_at(buf, c, 7, header("CURRENT PRESET"));
    line_at(
        buf,
        c,
        8,
        Line::from(vec![
            Span::styled(
                format!(" {} ", Screen::Presets.icon()),
                Style::new().fg(pal.ok).bg(pal.raised),
            ),
            Span::styled(" ", Style::new().bg(pal.bg)),
            Span::styled(
                ellipsize(&preset.name, w.saturating_sub(5)),
                Style::new()
                    .fg(pal.fg)
                    .bg(pal.bg)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    );
    let groups: Vec<&str> = {
        let mut g: Vec<&str> = preset.modules().map(|m| m.group().label()).collect();
        g.dedup();
        g
    };
    line_at(
        buf,
        c,
        9,
        Line::from(Span::styled(
            format!(
                "     {}",
                ellipsize(&groups.join(" + "), w.saturating_sub(5))
            ),
            Style::new().fg(pal.muted).bg(pal.bg),
        )),
    );

    // CONFIG PATH
    line_at(buf, c, 10, sep);
    line_at(buf, c, 11, header("CONFIG PATH"));
    let dir = state
        .paths
        .fastfetch_config
        .parent()
        .map(home_relative)
        .unwrap_or_default();
    line_at(
        buf,
        c,
        12,
        Line::from(vec![
            Span::styled("\u{f024b} ", Style::new().fg(pal.warn).bg(pal.bg)),
            Span::styled(
                ellipsize(&dir, w.saturating_sub(5)),
                Style::new().fg(pal.fg).bg(pal.bg),
            ),
            Span::styled(" \u{f03cb}", Style::new().fg(pal.dim).bg(pal.bg)),
        ]),
    );
}
