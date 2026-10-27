//! Small drawing primitives shared by the studio chrome: cards, key badges,
//! palette dots and the swatch thumbnails of theme cards.

use crate::domain::settings::Settings;
use crate::domain::style;
use crate::domain::theme::Theme;
use crate::tui::palette::Palette;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Widget},
};
use unicode_width::UnicodeWidthStr;

/// Rounded card on the surface colour; the border lights up when focused.
pub fn card<'a>(pal: &Palette, focused: bool) -> Block<'a> {
    Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(
            Style::new()
                .fg(if focused { pal.primary } else { pal.border })
                .bg(pal.bg),
        )
        .style(Style::new().bg(pal.surface).fg(pal.fg))
}

/// Card with an `icon title` heading in its top border.
pub fn titled_card<'a>(
    pal: &Palette,
    icon: &str,
    icon_color: Color,
    title: &str,
    focused: bool,
) -> Block<'a> {
    card(pal, focused).title(Line::from(vec![
        Span::styled(" ", Style::new().bg(pal.bg)),
        Span::styled(format!("{icon} "), Style::new().fg(icon_color).bg(pal.bg)),
        Span::styled(
            format!("{title} "),
            Style::new()
                .fg(pal.fg)
                .bg(pal.bg)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
}

/// ` key ` on a raised tile.
pub fn key_badge(pal: &Palette, key: &str) -> Span<'static> {
    Span::styled(
        format!(" {key} "),
        Style::new()
            .fg(pal.fg)
            .bg(pal.raised)
            .add_modifier(Modifier::BOLD),
    )
}

/// Paint `area` with `style` and clear its symbols.
pub fn fill(buf: &mut Buffer, area: Rect, style: Style) {
    let area = area.intersection(buf.area);
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(c) = buf.cell_mut((x, y)) {
                c.reset();
                c.set_char(' ').set_style(style);
            }
        }
    }
}

/// Render `line` on row `dy` of `area` (clipped; rows past the bottom are skipped).
pub fn line_at(buf: &mut Buffer, area: Rect, dy: u16, line: Line) {
    if dy >= area.height || area.width == 0 {
        return;
    }
    let row = Rect {
        y: area.y + dy,
        height: 1,
        ..area
    }
    .intersection(buf.area);
    if !row.is_empty() {
        line.render(row, buf);
    }
}

/// Display width of a string in cells.
pub fn width(s: &str) -> u16 {
    s.width().min(u16::MAX as usize) as u16
}

/// Truncate to `max` cells, ending with `…` when shortened.
pub fn ellipsize(s: &str, max: usize) -> String {
    if s.width() <= max {
        return s.to_string();
    }
    let mut out = String::new();
    let mut w = 0;
    for ch in s.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if w + cw + 1 > max {
            break;
        }
        out.push(ch);
        w += cw;
    }
    out.push('…');
    out
}

/// The theme's eight signature colours as `●` dots (primary, secondary,
/// accent, then the five module groups).
pub fn theme_dots(theme: &Theme, gap: bool) -> Vec<Span<'static>> {
    let fg = theme.text_color();
    let mut spans = Vec::new();
    for (i, c) in style::dot_colors(theme, &Settings::default())
        .iter()
        .enumerate()
    {
        if gap && i > 0 {
            spans.push(Span::raw(" "));
        }
        spans.push(Span::styled("●", Style::new().fg(c.tui_color(fg))));
    }
    spans
}

/// Distinct colours a theme defines (background, text, accents and groups).
pub fn theme_color_count(theme: &Theme) -> usize {
    let g = &theme.groups;
    let mut all: Vec<String> = [
        &theme.background,
        &theme.text,
        &theme.primary,
        &theme.secondary,
        &theme.accent,
        &g.os,
        &g.desktop,
        &g.hardware,
        &g.network,
        &g.time,
    ]
    .iter()
    .map(|s| s.to_ascii_lowercase())
    .collect();
    all.sort();
    all.dedup();
    all.len()
}

/// Bar widths of the four "info lines", as a fraction of the drawable width.
const BARS: [f64; 4] = [0.55, 0.85, 0.6, 0.95];

/// Theme swatch, like the website's cards: the theme's background with four
/// primary-coloured info lines and the secondary, accent and hardware colours
/// as dots in the corner. Readable at any size, from the 9x3 sidebar chip to
/// the gallery cards.
pub fn draw_thumbnail(buf: &mut Buffer, area: Rect, theme: &Theme) {
    let area = area.intersection(buf.area);
    if area.is_empty() {
        return;
    }
    let bg = theme.background_color();
    let bg = if bg == Color::Reset {
        Color::Rgb(16, 16, 24)
    } else {
        bg
    };
    let primary = theme.primary_color();
    fill(buf, area, Style::new().bg(bg));

    let pad = if area.width >= 16 { 2 } else { 1 };
    let inner_w = area.width.saturating_sub(pad * 2);
    let (first, last) = if area.height >= 5 {
        (1, area.height - 1)
    } else {
        (0, area.height)
    };
    let dots = [
        theme.secondary_color(),
        theme.accent_color(),
        Theme::parse_color(&theme.groups.hardware),
    ];
    let gap = area.width >= 16;
    let dots_w: u16 = if gap { 5 } else { 3 };
    let show_dots = inner_w >= dots_w + 3;
    for (i, row) in (first..last).take(BARS.len()).enumerate() {
        let mut w = (f64::from(inner_w) * BARS[i]).round() as u16;
        if i == 0 && show_dots {
            w = w.min(inner_w.saturating_sub(dots_w + 1));
        }
        for dx in 0..w.max(1).min(inner_w) {
            if let Some(cell) = buf.cell_mut((area.x + pad + dx, area.y + row)) {
                cell.set_char('▄').set_fg(primary).set_bg(bg);
            }
        }
    }
    if show_dots {
        let mut x = area.x + pad + inner_w - dots_w;
        for (i, c) in dots.into_iter().enumerate() {
            if gap && i > 0 {
                x += 1;
            }
            if let Some(cell) = buf.cell_mut((x, area.y + first)) {
                cell.set_char('●').set_fg(c).set_bg(bg);
            }
            x += 1;
        }
    }
}
