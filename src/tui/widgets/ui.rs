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
/// Each terminal cell carries two crisp pixels (upper & lower half-block); no interpolated edges.
/// The 64x32 canvas is partitioned into three distinct depth planes:
/// - Background / Sky & Celestial: y = 0..9 (Sampled in top row 0 and 1 of card)
/// - Midground Silhouette: y = 9..19 (Sampled in mid rows 2 and 3 of card)
/// - Foreground Base: y = 19..32 (Sampled in bottom rows 4 and 5 of card)
pub fn draw_thumbnail(buf: &mut Buffer, area: Rect, theme: &Theme) {
    let area = area.intersection(buf.area);
    if area.is_empty() {
        return;
    }
    let bg = rgb(theme.background_color(), (16, 16, 24));
    let p = rgb(theme.primary_color(), (137, 180, 250));
    let _a = rgb(theme.accent_color(), p);
    let q = rgb(theme.secondary_color(), p);
    let light = rgb(theme.text_color(), (238, 238, 238));
    let _dark = mix(bg, (5, 8, 20), 0.65);
    let id = theme.id.as_str();
    let hash = seed(id);
    let pixel = |x: i32, y: i32| -> Rgb {
        let star = (x * 37 + y * 71 + (hash % 97) as i32) % 89 == 0;
        let moon = (x - 47).pow(2) + (y - 5).pow(2) <= 16 && (x - 50).pow(2) + (y - 4).pow(2) >= 14;
        let mut c = if id == "monochrome" {
            (0, 0, 0)
        } else {
            let mut base = mix(bg, q, (y / 4) as f64 * 0.07);
            if star && y < 10 {
                base = light;
            }
            base
        };
        match id {
            "monochrome" => {
                // Background: Pitch-black void with crisp star pixels & bright crescent moon
                if (x == 12 && y == 3) || (x == 26 && y == 5) || (x == 56 && y == 2) {
                    c = (225, 225, 225);
                }
                let moon_dist = (x - 47).pow(2) + (y - 5).pow(2);
                let moon_cut = (x - 50).pow(2) + (y - 4).pow(2);
                if moon_dist <= 18 && moon_cut >= 14 {
                    c = (255, 255, 255);
                }
                // Midground: Stark minimalist mountain ridge with sharp peaks
                let p1 = 18 - (9 - (x - 20).abs()).max(0);
                let p2 = 19 - (8 - (x - 36).abs()).max(0);
                let ridge = p1.min(p2);
                if y >= ridge {
                    if y - ridge <= 2 && ridge <= 13 {
                        c = (250, 250, 250); // Pure white snowcaps
                    } else {
                        c = (95, 95, 95); // Mid-gray mountain slopes
                    }
                }
                // Foreground: Dark charcoal minimalist foothills and pine tree silhouette
                let fore = 23 - (6 - (x - 50).abs()).max(0);
                if y >= fore {
                    c = (35, 35, 35);
                }
                // Minimalist sharp lone pine tree at x=50, y=14..23
                if (x - 50).abs() <= (23 - y) / 3 && y >= 14 && y < 23 {
                    c = (12, 12, 12);
                }
                if y >= 25 {
                    c = (18, 18, 18);
                }
            }
            "one-light" => {
                // Background: Clean pale ivory sky
                c = (242, 244, 248);
                // Warm golden morning sun at x=14, y=5
                if (x - 14).pow(2) + (y - 5).pow(2) <= 14 {
                    c = (245, 185, 75);
                }
                // Glowing horizontal lighthouse beam across pale sky to the left
                if y >= 5 && y <= 7 && x >= 12 && x < 41 && (x + y) % 3 != 0 {
                    c = (255, 235, 140);
                }
                // Lighthouse tower at x=41..47, y=4..19
                if (41..=47).contains(&x) {
                    if y >= 4 && y <= 7 {
                        if (42..=46).contains(&x) {
                            c = if y >= 5 && y <= 6 { (255, 235, 110) } else { (45, 50, 60) };
                        }
                    } else if y >= 8 && y <= 18 {
                        let band = (y - 8) / 3;
                        c = if band % 2 == 0 { (228, 86, 73) } else { (255, 255, 255) };
                        if x == 41 || x == 47 {
                            c = (55, 60, 70);
                        }
                    }
                }
                // Coastal cliff rock foundation under lighthouse at x >= 36, y >= 17
                let cliff = 19 - (6 - (x - 44).abs()).max(0);
                if x >= 36 && y >= cliff {
                    c = (50, 56, 68);
                }
                // Foreground: Vibrant deep ocean blue with white foam crest
                if y >= 19 {
                    if y == 19 || (y == 20 && (x % 3 == 0)) {
                        c = (255, 255, 255);
                    } else if y >= 24 {
                        c = (35, 70, 150);
                    } else {
                        c = (64, 120, 242);
                    }
                }
            }
            "doom" => {
                // Fiery crimson/amber sky gradient
                c = mix((90, 15, 12), (235, 65, 20), f64::from(y) / 10.0);
                if y < 8 && (x * 7 + y * 11) % 19 < 4 {
                    c = (140, 35, 15);
                }
                // Midground: Jagged black obsidian demon spires
                let s1 = 18 - (10 - (x - 16).abs() * 2).max(0);
                let s2 = 18 - (12 - (x - 32).abs() * 2).max(0);
                let s3 = 18 - (9 - (x - 48).abs() * 2).max(0);
                let spires = s1.min(s2).min(s3);
                if y >= spires {
                    c = (18, 6, 8); // Obsidian black
                }
                // Foreground: Molten lava river winding through dark basalt crags
                if y >= 18 {
                    c = (28, 10, 12);
                    let river_center = 26 + (y - 18) * 2;
                    if (x - river_center).abs() <= 6 {
                        c = if (x - river_center).abs() <= 2 { (255, 225, 60) } else { (255, 110, 25) };
                    }
                    if y >= 25 && (x + y) % 3 != 0 {
                        c = (245, 95, 20);
                    }
                }
            }
            "spiderman" => {
                // Deep midnight navy sky
                c = (12, 16, 28);
                if (x - 14).pow(2) + (y - 5).pow(2) <= 12 {
                    c = (240, 245, 255);
                }
                // Web strand cutting from top right to center
                if ((x - 28) - (y - 4) * 2).abs() <= 1 && y >= 4 && y <= 12 {
                    c = (220, 230, 255);
                }
                // City skyline silhouettes in background (y >= 9)
                let bldg = (x / 7 * 19 + 7) % 8;
                if y >= 10 + bldg {
                    c = (20, 26, 42);
                    if x % 7 >= 2 && x % 2 == 0 && (y - bldg) % 3 == 0 && y >= 12 {
                        c = if x % 4 == 0 { (255, 210, 80) } else { (80, 180, 255) };
                    }
                }
                // Spider-Man in action silhouette leaping at x=28..34, y=10..15
                if (29..=33).contains(&x) && y >= 10 && y <= 15 {
                    c = if y <= 12 { (215, 30, 45) } else { (0, 75, 195) };
                    if y == 11 && (x == 30 || x == 32) {
                        c = (255, 255, 255);
                    }
                }
                // Foreground rooftops at y >= 19
                if y >= 19 {
                    c = (10, 12, 20);
                    if y == 19 && (x % 6 == 0 || x % 6 == 1) {
                        c = if x % 12 < 6 { (215, 30, 45) } else { (0, 80, 210) };
                    }
                }
            }
            "naruto" => {
                // Vibrant Hidden Leaf sunset sky in warm orange & amber
                c = mix((255, 110, 20), (255, 185, 45), f64::from(y) / 10.0);
                // Hokage monument mountain cliff in midground (y >= 9)
                let cliff = 16 - (10 - (x - 30).abs() / 2).max(0);
                if y >= cliff {
                    c = (140, 75, 35);
                    if (y + x) % 4 == 0 && y >= cliff + 2 {
                        c = (105, 55, 25);
                    }
                    if y == cliff + 3 && x >= 18 && x <= 42 {
                        c = (190, 195, 205);
                    }
                }
                // Foreground: Village roofs and pine forest in deep leaf green
                if y >= 18 {
                    c = (35, 75, 40);
                    if (x % 8 <= 3 && y == 19) || (y >= 24) {
                        c = if y == 19 { (235, 95, 25) } else { (25, 55, 30) };
                    }
                }
            }
            "valorant" => {
                // Dark radianite navy sky
                c = (15, 25, 35);
                if (x * 3 + y * 5) % 17 == 0 && y < 10 {
                    c = (60, 215, 195);
                }
                // Angular radianite facility spires in midground (y >= 9)
                let spire = 18 - (12 - (x - 48).abs()).max(0);
                if y >= spire {
                    c = (25, 38, 50);
                }
                // Bold Radiant 'V' slash emblem in center (x=24..40, y=7..19)
                let left_slash = (x - 26) == (y - 7);
                let left_thick = (x - 27) == (y - 7) || (x - 25) == (y - 7);
                if (left_slash || left_thick) && y >= 7 && y <= 18 {
                    c = (245, 245, 250);
                }
                let right_slash = (38 - x) == (y - 7);
                let right_thick = (39 - x) == (y - 7) || (37 - x) == (y - 7);
                if (right_slash || right_thick) && y >= 7 && y <= 18 {
                    c = (255, 70, 85);
                }
                // Foreground: High-tech angled site platform at y >= 19
                if y >= 19 {
                    c = (20, 30, 42);
                    if y == 19 && (x % 4 == 0) {
                        c = (60, 215, 195);
                    }
                }
            }
            "palenight" => {
                // Deep starry purple night sky
                c = mix((35, 32, 54), (55, 45, 80), f64::from(y) / 10.0);
                if star && y < 10 {
                    c = (200, 210, 255);
                }
                if moon {
                    c = (240, 240, 255);
                }
                // Floating celestial fantasy island at x=20..44, y=10..18
                let isle_w = 12 - (y - 12).abs() * 2;
                if (x - 32).abs() <= isle_w && y >= 10 && y <= 17 {
                    c = (55, 50, 75);
                    if (31..=33).contains(&x) && y >= 6 && y <= 10 {
                        c = (199, 146, 234);
                    }
                    if y == 10 && (x % 2 == 0) {
                        c = (195, 232, 141);
                    }
                }
                // Foreground: Floating clouds of lilac evening mist at y >= 19
                if y >= 19 {
                    c = (42, 38, 62);
                    if (y == 20 || y == 21) && ((x + 2) % 6 < 4) {
                        c = (85, 70, 115);
                    }
                    if y >= 25 {
                        c = (30, 28, 46);
                    }
                }
            }
            "synthwave" => {
                // Twilight purple to deep magenta sky
                c = mix((25, 12, 45), (120, 20, 90), f64::from(y) / 11.0);
                let sun_dist = (x - 32).pow(2) + (y - 7).pow(2);
                if sun_dist <= 26 {
                    c = if y % 2 == 0 { (255, 80, 140) } else { (255, 180, 40) };
                }
                // Distant wireframe mountain horizon at y=12..16
                let mtn = 16 - (8 - (x - 32).abs() / 2).max(0);
                if y >= mtn && y < 17 {
                    c = (40, 15, 60);
                    if y == mtn {
                        c = (0, 240, 255);
                    }
                }
                // Foreground: Perspective neon grid at y >= 17
                if y >= 17 {
                    c = (18, 8, 32);
                    if [17, 21, 26].contains(&y) {
                        c = (255, 45, 140);
                    }
                    let depth = (y - 15).max(1);
                    if ((x - 32) * 8 / depth).abs() % 6 == 0 {
                        c = (0, 230, 255);
                    }
                }
            }
            "monokai-pro" => {
                // Sleek Monokai dark charcoal backdrop
                c = (45, 42, 46);
                // Retro digital horizon with Monokai rainbow spectrum bars at y=9..17
                if (x - 32).abs() <= 18 && y >= 9 && y <= 17 {
                    if y == 9 || y == 10 {
                        c = (255, 97, 136);
                    } else if y == 11 || y == 12 {
                        c = (252, 152, 103);
                    } else if y == 13 || y == 14 {
                        c = (255, 216, 102);
                    } else if y == 15 || y == 16 {
                        c = (169, 220, 118);
                    } else if y == 17 {
                        c = (120, 220, 232);
                    }
                }
                // Foreground: Clean dark charcoal digital grid floor at y >= 18
                if y >= 18 {
                    c = (34, 31, 34);
                    if [19, 24].contains(&y) || (x % 8 == 0) {
                        c = (171, 157, 242);
                    }
                }
            }
            "kanagawa" => {
                // Warm vintage parchment sky
                c = (220, 215, 186);
                let fuji = 12 - (6 - (x - 46).abs()).max(0);
                if y >= fuji && y < 14 {
                    c = if y - fuji <= 2 { (255, 255, 255) } else { (45, 79, 103) };
                }
                let wave = 17 + ((x as f64 / 6.0).sin() * 5.0) as i32;
                if y >= wave {
                    if y - wave <= 2 {
                        c = (255, 255, 255);
                    } else if y - wave <= 5 {
                        c = (45, 79, 103);
                    } else {
                        c = (34, 50, 73);
                    }
                }
                if y >= 20 {
                    c = (30, 44, 62);
                    if y == 21 && (x % 3 == 0) {
                        c = (255, 255, 255);
                    }
                }
            }
            "oceanic-next" => {
                c = mix((27, 43, 52), (102, 153, 204), f64::from(y) / 10.0);
                if (x + y * 2) % 7 == 0 && y < 10 {
                    c = (92, 207, 230);
                }
                let reef = 18 - (8 - (x - 24).abs()).max(0);
                if y >= reef {
                    c = (40, 70, 85);
                    if (x + y) % 3 == 0 {
                        c = (236, 95, 103);
                    }
                }
                if y >= 20 {
                    c = (18, 30, 38);
                }
            }
            "nord" => {
                // Deep arctic midnight blue sky
                c = (46, 52, 64);
                if star && y < 10 {
                    c = (236, 239, 244);
                }
                let aurora = 4 + ((x as f64 / 7.0).sin() * 2.5) as i32;
                if y >= aurora && y <= aurora + 3 {
                    c = mix((136, 192, 208), (163, 190, 140), f64::from(y - aurora) / 3.0);
                }
                let mtn = 17 - (11 - (x - 28).abs()).max(0);
                if y >= mtn {
                    c = if y - mtn <= 2 { (236, 239, 244) } else { (76, 86, 106) };
                }
                if y >= 19 {
                    c = (36, 41, 51);
                    if y == 19 && (x % 3 == 0) {
                        c = (94, 129, 172);
                    }
                    if y >= 24 {
                        c = (24, 28, 36);
                    }
                }
            }
            "gruvbox-dark" | "gruvbox-light" => {
                let is_light = id == "gruvbox-light";
                c = if is_light { (245, 235, 215) } else { (60, 56, 54) };
                if (x - 44).pow(2) + (y - 5).pow(2) <= 14 {
                    c = (250, 189, 47);
                }
                let mesa = if (18..=38).contains(&x) { 11 } else { 16 };
                if y >= mesa {
                    c = (214, 93, 14);
                    if y >= mesa + 3 {
                        c = (175, 58, 3);
                    }
                }
                if y >= 19 {
                    c = if is_light { (120, 100, 75) } else { (40, 36, 34) };
                    if (x == 14 && y >= 14 && y <= 22) || (x == 12 && y >= 16 && y <= 19) || (x == 16 && y >= 17 && y <= 20) {
                        c = (152, 151, 26);
                    }
                    if y >= 25 {
                        c = if is_light { (90, 75, 55) } else { (29, 32, 33) };
                    }
                }
            }
            "tokyo-night" => {
                c = (26, 27, 38);
                if star && y < 8 {
                    c = (192, 202, 245);
                }
                if (x == 46 && y >= 2 && y <= 12) || ((x - 46).abs() == 1 && y >= 8 && y <= 12) {
                    c = (247, 118, 142);
                }
                let roof = 10 + ((x / 6 * 17 + 5) % 8);
                if y >= roof {
                    c = (36, 40, 59);
                    if x % 6 != 0 && x % 2 == 0 && (y - roof) % 2 == 0 && y >= 11 {
                        c = if x % 4 == 0 { (125, 207, 255) } else { (187, 154, 247) };
                    }
                }
                if y >= 19 {
                    c = (18, 20, 28);
                    if y == 20 && (x % 4 == 0) {
                        c = (125, 207, 255);
                    }
                }
            }
            "cyberpunk" => {
                c = (30, 24, 12);
                let mega = 9 + ((x / 8 * 13 + 3) % 7);
                if y >= mega {
                    c = (20, 18, 24);
                    if x % 8 >= 2 && x % 8 <= 6 && y >= 10 && y <= 13 {
                        c = if x % 2 == 0 { (252, 238, 10) } else { (0, 240, 255) };
                    }
                }
                if y >= 19 {
                    c = (12, 10, 14);
                    if [19, 23].contains(&y) {
                        c = (255, 0, 85);
                    }
                }
            }
            "dracula" => {
                c = (40, 42, 54);
                if (x - 16).pow(2) + (y - 5).pow(2) <= 14 {
                    c = (241, 250, 140);
                }
                if ((x == 26 || x == 28) && y == 4) || (x == 27 && y == 5) {
                    c = (18, 19, 25);
                }
                let in_tower = (35..=39).contains(&x) || (45..=49).contains(&x);
                let in_wall = (40..=44).contains(&x) && y >= 13;
                if (in_tower && y >= 8) || in_wall {
                    c = (25, 27, 36);
                    if in_tower && y < 11 && (x - 37).abs().min((x - 47).abs()) > (y - 8) {
                        c = (40, 42, 54);
                    } else if y >= 12 && y <= 14 && (x == 37 || x == 47) {
                        c = (255, 184, 108);
                    }
                }
                if y >= 19 {
                    c = (18, 20, 28);
                    if y >= 25 {
                        c = (12, 14, 20);
                    }
                }
            }
            "solarized-light" | "ayu-light" => {
                let is_solar = id == "solarized-light";
                c = if is_solar { (253, 246, 227) } else { (250, 250, 250) };
                if (x - 46).pow(2) + (y - 5).pow(2) <= 14 {
                    c = (245, 185, 60);
                }
                let dune1 = 15 + ((x as f64 / 9.0).sin() * 4.0) as i32;
                if y >= dune1 {
                    c = if is_solar { (181, 137, 0) } else { (235, 160, 65) };
                }
                let dune2 = 21 + ((x as f64 / 12.0).cos() * 3.0) as i32;
                if y >= dune2 {
                    c = if is_solar { (203, 75, 22) } else { (215, 110, 45) };
                    if y >= 25 {
                        c = if is_solar { (133, 85, 30) } else { (160, 80, 35) };
                    }
                }
            }
            "matrix" => {
                c = (2, 8, 4);
                let col = x / 4;
                let speed = (col * 17 + 5) % 11;
                let lead_y = 6 + speed;
                if x % 4 == 0 {
                    if y == lead_y {
                        c = (195, 255, 205);
                    } else if y > lead_y - 8 && y < lead_y {
                        let fade = f64::from(y - (lead_y - 8)) / 8.0;
                        c = mix((0, 80, 25), (0, 255, 100), fade);
                    }
                }
                if y >= 20 {
                    c = (0, 25, 8);
                    if [21, 25].contains(&y) && x % 2 == 0 {
                        c = (0, 180, 70);
                    }
                }
            }
            "minecraft" => {
                c = (115, 180, 235);
                if (34..=52).contains(&x) && y >= 3 && y <= 6 {
                    c = (245, 250, 255);
                }
                if (10..=24).contains(&x) && y >= 6 && y <= 14 {
                    c = if (x + y) % 2 == 0 { (80, 145, 45) } else { (60, 115, 35) };
                }
                if (15..=18).contains(&x) && y >= 14 && y <= 18 {
                    c = (110, 75, 45);
                }
                if y >= 18 {
                    if y == 18 || y == 19 {
                        c = (95, 165, 55);
                    } else {
                        c = if (x + y) % 3 == 0 { (125, 85, 50) } else { (95, 65, 38) };
                    }
                }
            }
            "pokemon" => {
                c = (120, 190, 245);
                if ((x - 12) == (y - 2) * 2 && y >= 2 && y <= 4) || (y == 4 && x >= 13 && x <= 16) || ((x - 14) == (y - 4) * 2 && y >= 4 && y <= 7) {
                    c = (255, 225, 40);
                }
                let ball_dist = (x - 32).pow(2) + (y - 14).pow(2);
                if ball_dist <= 26 {
                    if y == 14 {
                        c = (20, 20, 25);
                    } else if y < 14 {
                        c = (235, 50, 45);
                    } else {
                        c = (250, 250, 255);
                    }
                    let btn_dist = (x - 32).pow(2) + (y - 14).pow(2);
                    if btn_dist <= 4 {
                        c = (250, 250, 255);
                    } else if btn_dist <= 8 {
                        c = (20, 20, 25);
                    }
                }
                if y >= 20 {
                    c = (85, 160, 60);
                    if y >= 24 {
                        c = (60, 120, 45);
                    }
                }
            }
            "sakura" | "lavender" | "candy" => {
                c = mix(bg, (245, 195, 210), f64::from(y) / 10.0);
                if moon {
                    c = (255, 250, 240);
                }
                if (x >= 12 && x <= 38 && y == 13) || ((x == 22 || x == 32) && y >= 11 && y <= 15) {
                    c = (70, 45, 55);
                }
                let near_blossom = ((x - 22).pow(2) + (y - 11).pow(2) <= 18) || ((x - 34).pow(2) + (y - 12).pow(2) <= 16);
                if near_blossom && (x + y) % 4 != 0 {
                    c = if (x * 3 + y * 7) % 3 == 0 { (255, 220, 235) } else { (255, 150, 185) };
                }
                if (x == 18 && y == 16) || (x == 28 && y == 18) || (x == 42 && y == 15) {
                    c = (255, 160, 190);
                }
                if y >= 19 {
                    c = mix(q, bg, 0.4);
                    if y >= 24 {
                        c = mix(bg, p, 0.35);
                    }
                }
            }
            "everforest" => {
                c = (55, 65, 58);
                let m1 = 15 - (8 - (x - 24).abs() / 2).max(0);
                if y >= m1 {
                    c = (75, 95, 80);
                }
                let m2 = 18 - (6 - (x - 42).abs() / 2).max(0);
                if y >= m2 {
                    c = (90, 120, 95);
                }
                if y >= 17 && y <= 18 && (x + y) % 3 != 0 {
                    c = (140, 160, 145);
                }
                if y >= 19 {
                    c = (42, 54, 46);
                    if y == 19 && (x % 4 == 0) {
                        c = (110, 140, 115);
                    }
                    if y >= 25 {
                        c = (30, 40, 34);
                    }
                }
            }
            "rose-pine" => {
                c = mix((42, 38, 56), (235, 188, 186), f64::from(y) / 10.0);
                if moon {
                    c = (246, 193, 119);
                }
                let mtn = 16 - (9 - (x - 30).abs()).max(0);
                if y >= mtn {
                    c = (55, 50, 72);
                }
                if y >= 18 {
                    c = (35, 33, 50);
                    let local = x % 8 - 4;
                    if local.abs() <= (23 - y) && y >= 16 && y < 24 {
                        c = (25, 23, 36);
                    }
                    if y >= 25 {
                        c = (20, 18, 28);
                    }
                }
            }
            "catppuccin-latte" => {
                c = (239, 241, 245);
                if (x - 14).pow(2) + (y - 5).pow(2) <= 14 {
                    c = (234, 160, 50);
                }
                let m1 = 16 - (9 - (x - 32).abs()).max(0);
                if y >= m1 {
                    c = (222, 149, 132);
                }
                if y >= 19 {
                    c = (136, 57, 239);
                    if y >= 24 {
                        c = (76, 79, 105);
                    }
                }
            }
            _ => {
                // catppuccin-mocha, one-dark, etc.
                c = mix(bg, (40, 42, 65), f64::from(y) / 10.0);
                if star && y < 9 {
                    c = light;
                }
                if moon {
                    c = (245, 224, 220);
                }
                let m1 = 15 - (8 - (x - 22).abs()).max(0);
                if y >= m1 {
                    c = mix(bg, p, 0.45);
                }
                let m2 = 21 + ((x as f64 / 8.0).sin() * 3.0) as i32;
                if y >= m2 {
                    c = mix(bg, q, 0.35);
                    if y >= 25 {
                        c = mix(bg, (17, 17, 27), 0.7);
                    }
                }
            }
        }
        c
    };
    let total_rows = i32::from(area.height) * 2;
    for cy in 0..area.height {
        for cx in 0..area.width {
            let x = i32::from(cx) * 64 / i32::from(area.width);
            let top_row = i32::from(cy) * 2;
            let bot_row = top_row + 1;
            let yt = (top_row * 32 / total_rows).min(31);
            let yb = (bot_row * 32 / total_rows).min(31);
            let t = pixel(x, yt);
            let b = pixel(x, yb);
            if let Some(cell) = buf.cell_mut((area.x + cx, area.y + cy)) {
                cell.reset();
                cell.set_char('▀').set_fg(Color::Rgb(t.0, t.1, t.2)).set_bg(Color::Rgb(b.0, b.1, b.2));
            }
        }
    }
}
