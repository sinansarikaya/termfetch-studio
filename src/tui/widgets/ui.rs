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

/// The small Arch-style logo of the website's theme art.
const ART: [&str; 4] = ["   /\\   ", "  /  \\  ", " / /\\ \\ ", "/_/  \\_\\"];
/// Bar widths of the five module-group lines, as on the website.
const BARS: [f64; 5] = [0.82, 0.62, 0.94, 0.72, 0.48];

/// Theme art, like the website's cards: a mini fastfetch in the theme's real
/// colours. The logo uses the primary colour, each module group gets an
/// `├ ▪ →` line with a bar in its colour, and the palette dots sit below.
/// Narrow areas (dashboard, sidebar) drop the logo and keep the lines.
pub fn draw_thumbnail(buf: &mut Buffer, area: Rect, theme: &Theme) {
    let area = area.intersection(buf.area);
    if area.is_empty() {
        return;
    }
    let bg = match theme.background_color() {
        Color::Reset => Color::Rgb(16, 16, 24),
        c => c,
    };
    fill(buf, area, Style::new().bg(bg));
    let groups = [
        &theme.groups.os,
        &theme.groups.desktop,
        &theme.groups.hardware,
        &theme.groups.network,
        &theme.groups.time,
    ]
    .map(|c| Theme::parse_color(c));

    let pad = u16::from(area.width >= 12);
    let x0 = area.x + pad;
    let w = area.width.saturating_sub(pad * 2);
    let show_logo = w >= 17 && area.height >= 4;
    let with_dots = area.height >= 6;
    let rows = (area.height - u16::from(with_dots)).min(BARS.len() as u16);
    let top = area.y + u16::from(area.height >= 7);

    if show_logo {
        let ly = top + rows.saturating_sub(ART.len() as u16) / 2;
        for (i, l) in ART.iter().enumerate() {
            let y = ly + i as u16;
            if y < area.bottom() {
                buf.set_string(x0, y, l, Style::new().fg(theme.primary_color()).bg(bg));
            }
        }
    }
    let lx = match (show_logo, w >= 24) {
        (false, _) => x0,
        (true, true) => x0 + 10,
        (true, false) => x0 + 9,
    };
    let lw = (x0 + w).saturating_sub(lx);
    let prefix = if lw >= 12 { "├ ▪ → " } else { "├→" };
    let bar_w = lw.saturating_sub(prefix.chars().count() as u16);
    for (i, &c) in groups.iter().enumerate().take(usize::from(rows)) {
        let y = top + i as u16;
        buf.set_stringn(lx, y, prefix, usize::from(lw), Style::new().fg(c).bg(bg));
        let n = ((f64::from(bar_w) * BARS[i]).round() as u16)
            .max(1)
            .min(bar_w);
        let bx = lx + prefix.chars().count() as u16;
        for dx in 0..n {
            if let Some(cell) = buf.cell_mut((bx + dx, y)) {
                cell.set_char('━').set_fg(c).set_bg(bg);
            }
        }
    }
    if with_dots {
        let y = top + rows;
        let dots = style::dot_colors(theme, &Settings::default());
        let fg = theme.text_color();
        for (i, c) in dots.iter().enumerate() {
            let x = lx + i as u16;
            if x >= area.right() || y >= area.bottom() {
                break;
            }
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char('●').set_fg(c.tui_color(fg)).set_bg(bg);
            }
        }
    }
}
