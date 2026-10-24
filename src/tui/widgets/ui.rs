//! Small drawing primitives shared by the studio chrome: cards, key badges,
//! palette dots and the procedural landscape thumbnails of theme cards.

use crate::domain::contrast::{mix, to_rgb, Rgb};
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
        .border_style(Style::new().fg(if focused { pal.primary } else { pal.border }).bg(pal.bg))
        .style(Style::new().bg(pal.surface).fg(pal.fg))
}

/// Card with an `icon title` heading in its top border.
pub fn titled_card<'a>(pal: &Palette, icon: &str, icon_color: Color, title: &str, focused: bool) -> Block<'a> {
    card(pal, focused).title(Line::from(vec![
        Span::styled(" ", Style::new().bg(pal.bg)),
        Span::styled(format!("{icon} "), Style::new().fg(icon_color).bg(pal.bg)),
        Span::styled(format!("{title} "), Style::new().fg(pal.fg).bg(pal.bg).add_modifier(Modifier::BOLD)),
    ]))
}

/// ` key ` on a raised tile.
pub fn key_badge(pal: &Palette, key: &str) -> Span<'static> {
    Span::styled(format!(" {key} "), Style::new().fg(pal.fg).bg(pal.raised).add_modifier(Modifier::BOLD))
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
    let row = Rect { y: area.y + dy, height: 1, ..area }.intersection(buf.area);
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
    for (i, c) in style::dot_colors(theme, &Settings::default()).iter().enumerate() {
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
    let mut all: Vec<String> = [&theme.background, &theme.text, &theme.primary, &theme.secondary, &theme.accent, &g.os, &g.desktop, &g.hardware, &g.network, &g.time]
        .iter()
        .map(|s| s.to_ascii_lowercase())
        .collect();
    all.sort();
    all.dedup();
    all.len()
}

fn rgb(c: Color, fallback: Rgb) -> Rgb {
    to_rgb(c).unwrap_or(fallback)
}

/// Small deterministic hash so every theme gets its own skyline.
fn seed(id: &str) -> u32 {
    id.bytes().fold(2_166_136_261u32, |h, b| (h ^ u32::from(b)).wrapping_mul(16_777_619))
}

/// Ridge height (0 = top) of a mountain range at column `x` of `w`.
fn ridge(x: f64, w: f64, base: f64, peaks: &[(f64, f64)]) -> f64 {
    peaks.iter().fold(base, |acc, &(cx, height)| {
        let d = ((x - cx * w) / (w * 0.32)).abs();
        acc.min(base - height * (1.0 - d).max(0.0))
    })
}

/// Procedural "sunset over mountains" thumbnail in the theme's colours,
/// painted with `▀` half blocks (two pixels per cell).
pub fn draw_thumbnail(buf: &mut Buffer, area: Rect, theme: &Theme) {
    let area = area.intersection(buf.area);
    if area.is_empty() {
        return;
    }
    let bg = rgb(theme.background_color(), (0x10, 0x10, 0x18));
    let primary = rgb(theme.primary_color(), (0x89, 0xb4, 0xfa));
    let secondary = rgb(theme.secondary_color(), primary);
    let accent = rgb(theme.accent_color(), primary);
    let text = rgb(theme.text_color(), (0xee, 0xee, 0xee));
    let s = seed(&theme.id);
    let (w, h) = (f64::from(area.width), f64::from(area.height) * 2.0);
    let jitter = |k: u32| f64::from((s >> k) & 0xff) / 255.0;

    let sky_top = mix(bg, primary, 0.18);
    let horizon = mix(mix(secondary, accent, 0.35), bg, 0.15);
    let sun = mix(secondary, (0xff, 0xf4, 0xe0), 0.35);
    let (sun_x, sun_y, sun_r) = ((0.55 + 0.3 * jitter(0)) * w, h * 0.38, (h * 0.2).max(1.2));
    let far = [(0.25 + 0.3 * jitter(8), h * 0.5), (0.75, h * 0.3)];
    let near = [(0.1 + 0.2 * jitter(16), h * 0.22), (0.6 + 0.3 * jitter(24), h * 0.28)];
    let far_c = mix(primary, bg, 0.35);
    let snow = mix(text, primary, 0.25);
    let near_c = mix(bg, primary, 0.22);

    let pixel = |px: f64, py: f64| -> Rgb {
        let far_ridge = ridge(px, w, h * 0.72, &far);
        let near_ridge = ridge(px, w, h * 0.92, &near);
        if py >= near_ridge {
            return mix(near_c, bg, ((py - near_ridge) / h).min(1.0) * 0.8);
        }
        if py >= far_ridge {
            return if py - far_ridge < h * 0.07 && far_ridge < h * 0.42 { snow } else { mix(far_c, bg, ((py - far_ridge) / h) * 0.9) };
        }
        let (dx, dy) = (px - sun_x, (py - sun_y) * 1.0);
        if (dx * dx + dy * dy).sqrt() <= sun_r {
            return sun;
        }
        mix(sky_top, horizon, (py / (h * 0.75)).min(1.0))
    };

    for cy in 0..area.height {
        for cx in 0..area.width {
            let (px, py) = (f64::from(cx) + 0.5, f64::from(cy) * 2.0 + 0.5);
            let (t, b) = (pixel(px, py), pixel(px, py + 1.0));
            if let Some(cell) = buf.cell_mut((area.x + cx, area.y + cy)) {
                cell.reset();
                cell.set_char('▀').set_fg(Color::Rgb(t.0, t.1, t.2)).set_bg(Color::Rgb(b.0, b.1, b.2));
            }
        }
    }
}
