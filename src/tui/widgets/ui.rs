//! Small drawing primitives shared by the studio chrome: cards, key badges,
//! palette dots and the procedural landscape thumbnails of theme cards.

use crate::domain::contrast::{Rgb, mix, to_rgb};
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

/// Fixed-resolution pixel scenes, enlarged with nearest-neighbor sampling.
/// Each terminal cell carries two crisp pixels; no interpolated edges.
pub fn draw_thumbnail(buf: &mut Buffer, area: Rect, theme: &Theme) {
    let area = area.intersection(buf.area);
    if area.is_empty() {
        return;
    }
    let bg = rgb(theme.background_color(), (16, 16, 24));
    let p = rgb(theme.primary_color(), (137, 180, 250));
    let a = rgb(theme.accent_color(), p);
    let q = rgb(theme.secondary_color(), p);
    let light = rgb(theme.text_color(), (238, 238, 238));
    let dark = mix(bg, (5, 8, 20), 0.65);
    let id = theme.id.as_str();
    let hash = seed(id);
    let pixel = |x: i32, y: i32| -> Rgb {
        let star = (x * 37 + y * 71 + (hash % 97) as i32) % 89 == 0;
        let moon = (x - 47).pow(2) + (y - 8).pow(2) < 25 && (x - 49).pow(2) + (y - 6).pow(2) >= 24;
        let mut c = mix(bg, q, (y / 4) as f64 * 0.07);
        if star && y < 17 {
            c = light;
        }
        match id {
            "catppuccin-mocha" | "catppuccin-latte" => {
                c = mix(bg, q, f64::from(y / 4) * 0.06);
                if star && y < 18 {
                    c = light;
                }
                if moon {
                    c = mix(light, q, 0.2);
                }
                if y > 21 + ((x as f64 / 13.0).sin() * 3.0) as i32 {
                    c = mix(bg, q, 0.45);
                }
                if y > 27 + ((x as f64 / 11.0).cos() * 2.0) as i32 {
                    c = mix(bg, a, 0.3);
                }
            }
            "monochrome" => {
                if (x - 32).pow(2) + (y - 15).pow(2) < 100 {
                    c = mix(light, bg, 0.15);
                }
                if [(28, 12, 7), (36, 17, 11), (27, 20, 4)].iter().any(|&(cx, cy, r)| (x - cx).pow(2) + (y - cy).pow(2) < r) {
                    c = mix(light, bg, 0.45);
                }
                if y > 28 {
                    c = dark;
                }
            }
            "one-light" => {
                c = mix(bg, p, 0.12);
                if y > 22 {
                    c = if y % 3 == 0 { q } else { mix(bg, p, 0.45) };
                }
                if (43..48).contains(&x) && (10..27).contains(&y) {
                    c = if y % 5 < 2 { a } else { light };
                }
                if (41..50).contains(&x) && (8..11).contains(&y) {
                    c = dark;
                }
                if y == 9 && x > 20 && x < 43 {
                    c = mix(light, a, 0.2);
                }
            }
            "palenight" => {
                if moon {
                    c = light;
                }
                for (cx, cy) in [(16, 16), (44, 23)] {
                    if (x - cx).abs() < 10 && y >= cy && y < cy + 6 - (x - cx).abs() / 2 {
                        c = if y == cy { a } else { mix(bg, p, 0.4) };
                    }
                    if x == cx && y > cy - 6 && y < cy {
                        c = q;
                    }
                }
            }
            "matrix" => {
                c = (3, 12, 8);
                if x % 5 == 0 && (y + x * 3) % 13 < 9 {
                    c = if (y + x * 3) % 13 == 8 { (185, 255, 196) } else { mix((0, 40, 12), p, f64::from((y + x) % 7) / 7.0) };
                }
            }
            "tokyo-night" | "cyberpunk" | "spiderman" => {
                let roof = 10 + ((x / 6 * 17 + 9) % 12);
                if y >= roof {
                    c = dark;
                    if x % 6 != 0 && x % 2 == 0 && y % 3 == 0 {
                        c = if x % 4 == 0 { p } else { a };
                    }
                    if y == roof && x % 6 != 0 {
                        c = q;
                    }
                }
            }
            "synthwave" | "monokai-pro" => {
                if (x - 32).pow(2) + (y - 11).pow(2) < 64 && y % 3 != 0 {
                    c = mix(a, q, f64::from(y) / 24.0);
                }
                if y >= 22 {
                    c = dark;
                    let depth = y - 20;
                    if [23, 26, 30].contains(&y) || ((x - 32) * 12 / depth.max(1)) % 12 == 0 {
                        c = p;
                    }
                }
            }
            "gruvbox-dark" | "gruvbox-light" | "solarized-light" | "ayu-light" => {
                c = mix(bg, (238, 167, 72), f64::from(y / 5) * 0.14);
                if (x - 46).pow(2) + (y - 9).pow(2) < 30 {
                    c = (255, 211, 112);
                }
                let dune = 19 + ((x as f64 / 12.0).sin() * 3.0) as i32;
                if y > dune {
                    c = mix(q, (177, 99, 47), 0.45);
                }
                if y > 26 + ((x as f64 / 16.0).cos() * 2.0) as i32 {
                    c = mix(bg, a, 0.35);
                }
                if id.starts_with("gruvbox") && y > 14 + (x / 9 % 3) * 3 && x < 28 {
                    c = mix(q, (133, 71, 35), 0.6);
                }
            }
            "everforest" | "rose-pine" => {
                if id == "rose-pine" {
                    c = mix(bg, (239, 172, 153), f64::from(y / 4) * 0.12);
                }
                for layer in 0..3 {
                    let local = (x + layer * 5) % 16 - 8;
                    let top = 8 + layer * 5;
                    let crown = y >= top && y < top + 13 && local.abs() <= (y - top) / 3 + 1;
                    let trunk = local == 0 && y >= top + 10;
                    if crown || trunk || y >= 27 + layer {
                        c = mix(p, dark, 0.35 + f64::from(layer) * 0.27);
                    }
                }
                if id == "everforest" && (y == 21 || y == 25) {
                    c = mix(c, light, 0.25);
                }
            }
            "dracula" => {
                if moon {
                    c = light;
                }
                let tower = (18..24).contains(&x) || (39..45).contains(&x);
                if tower && y >= 9 + (x % 3) || (24..39).contains(&x) && y >= 18 {
                    c = dark;
                    if y > 16 && y < 21 && (x == 21 || x == 42 || x == 31) {
                        c = a;
                    }
                }
                if tower && y >= 6 && y < 10 && (x - 21).abs().min((x - 42).abs()) <= y - 6 {
                    c = dark;
                }
            }
            "kanagawa" | "oceanic-next" => {
                if y >= 19 - (12 - (x - 42).abs()).max(0) {
                    c = mix(p, bg, 0.45);
                }
                let wave = 22 + ((x as f64 / 8.0).sin() * 5.0) as i32;
                if y >= wave {
                    c = if y - wave < 2 {
                        light
                    } else if (y + x / 3) % 5 == 0 {
                        p
                    } else {
                        mix(bg, q, 0.4)
                    };
                }
            }
            "minecraft" => {
                c = (112, 178, 224);
                if y < 10 && (x / 10 + y / 3) % 7 == 0 {
                    c = (230, 240, 244);
                }
                let ground = 18 + (x / 8 % 3) * 3;
                if y >= ground {
                    c = if y == ground {
                        (91, 158, 57)
                    } else if (x + y) % 4 == 0 {
                        (115, 81, 49)
                    } else {
                        (91, 66, 42)
                    };
                }
                if (12..16).contains(&x) && y >= 9 && y < 22 {
                    c = (95, 66, 39);
                }
                if (6..22).contains(&x) && (5..13).contains(&y) {
                    c = if (x + y) % 3 == 0 { (73, 132, 44) } else { (49, 106, 36) };
                }
            }
            "doom" => {
                c = mix((25, 7, 8), (180, 48, 18), f64::from(y / 4) * 0.12);
                if y > 17 + (x * 7 % 9) {
                    c = dark;
                }
                if y > 26 && (x + y) % 7 < 3 {
                    c = if y % 2 == 0 { (255, 137, 25) } else { (218, 53, 17) };
                }
            }
            "sakura" | "lavender" | "candy" => {
                c = mix(bg, q, 0.18);
                if (x - 16).abs() <= 1 && y > 10 || y == 13 && (8..30).contains(&x) {
                    c = mix(bg, (95, 53, 78), 0.65);
                }
                if ((x - 16).pow(2) / 3 + (y - 9).pow(2) < 35 || (x - 28).pow(2) + (y - 12).pow(2) < 20) && (x + y) % 5 != 0 {
                    c = if (x * 7 + y * 13) % 7 < 2 { mix(light, p, 0.25) } else { p };
                }
                if y > 26 {
                    c = mix(q, bg, 0.4);
                }
                if x > 32 && (x * 7 + y * 13) % 59 == 0 {
                    c = p;
                }
            }
            "pokemon" => {
                let ball = (x - 32).pow(2) + (y - 16).pow(2);
                if ball < 100 {
                    c = if y < 15 { (234, 77, 70) } else { light };
                    if y == 15 || y == 16 {
                        c = dark;
                    }
                }
                if ball < 9 {
                    c = dark;
                }
                if ball < 3 {
                    c = light;
                }
            }
            "naruto" | "valorant" => {
                if (x - 32).pow(2) + (y - 14).pow(2) < 75 {
                    c = a;
                }
                if id == "naruto" && (x - 32).pow(2) + (y - 14).pow(2) < 35 && (x + y) % 5 < 2 {
                    c = dark;
                }
                if id == "valorant" && y > 9 && y < 23 && ((x - 32).abs() - (y - 9) / 2).abs() < 3 {
                    c = light;
                }
                if y > 27 {
                    c = dark;
                }
            }
            _ => {
                if moon && id != "nord" {
                    c = light;
                }
                if id == "nord" {
                    let ribbon = 6 + ((x as f64 / 9.0).sin() * 3.0) as i32;
                    if y >= ribbon && y < ribbon + 4 {
                        c = mix((88, 224, 179), p, f64::from(y - ribbon) / 4.0);
                    }
                }
                let ridge = 24 - (13 - (x - 23).abs() / 2).max(0);
                if y >= ridge {
                    c = if y - ridge < 2 && ridge < 16 { mix(light, p, 0.2) } else { mix(bg, p, 0.4) };
                }
                let near = 29 - (8 - (x - 51).abs() / 3).max(0);
                if y >= near {
                    c = mix(bg, q, 0.18);
                }
                if id.starts_with("catppuccin") && y > 22 + ((x as f64 / 12.0).sin() * 3.0) as i32 {
                    c = mix(bg, a, 0.35);
                }
            }
        }
        c
    };
    for cy in 0..area.height {
        for cx in 0..area.width {
            let x = i32::from(cx) * 64 / i32::from(area.width);
            let y = i32::from(cy) * 64 / i32::from(area.height * 2);
            let t = pixel(x, y);
            let b = pixel(x, ((i32::from(cy) * 2 + 1) * 32 / i32::from(area.height * 2)).min(31));
            if let Some(cell) = buf.cell_mut((area.x + cx, area.y + cy)) {
                cell.reset();
                cell.set_char('▀').set_fg(Color::Rgb(t.0, t.1, t.2)).set_bg(Color::Rgb(b.0, b.1, b.2));
            }
        }
    }
}
