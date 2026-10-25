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

fn rgb(c: Color, fallback: Rgb) -> Rgb {
    to_rgb(c).unwrap_or(fallback)
}

/// Small deterministic hash so every theme gets its own skyline.
fn seed(id: &str) -> u32 {
    id.bytes().fold(2_166_136_261u32, |h, b| {
        (h ^ u32::from(b)).wrapping_mul(16_777_619)
    })
}

/// Fixed-resolution pixel scenes, enlarged with nearest-neighbor sampling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThumbnailMode {
    Gallery,
    Compact,
}

/// Renders theme landscape in full gallery fidelity.
pub fn draw_thumbnail(buf: &mut Buffer, area: Rect, theme: &Theme) {
    draw_thumbnail_mode(buf, area, theme, ThumbnailMode::Gallery);
}

/// Renders theme landscape in high-contrast compact profile for dashboard & sidebar.
pub fn draw_thumbnail_compact(buf: &mut Buffer, area: Rect, theme: &Theme) {
    draw_thumbnail_mode(buf, area, theme, ThumbnailMode::Compact);
}

/// Each terminal cell carries two crisp pixels (upper & lower half-block); no interpolated edges.
/// The 64x32 canvas is partitioned into three distinct depth planes:
/// - Background / Sky & Celestial: y = 0..9 (Sampled in top row 0 and 1 of card)
/// - Midground Silhouette: y = 9..19 (Sampled in mid rows 2 and 3 of card)
/// - Foreground Base: y = 19..32 (Sampled in bottom rows 4 and 5 of card)
pub fn draw_thumbnail_mode(buf: &mut Buffer, area: Rect, theme: &Theme, mode: ThumbnailMode) {
    let area = area.intersection(buf.area);
    if area.is_empty() {
        return;
    }
    let compact = mode == ThumbnailMode::Compact;
    let bg = rgb(theme.background_color(), (16, 16, 24));
    let p = rgb(theme.primary_color(), (137, 180, 250));
    let _a = rgb(theme.accent_color(), p);
    let q = rgb(theme.secondary_color(), p);
    let light = rgb(theme.text_color(), (238, 238, 238));
    let _dark = mix(bg, (5, 8, 20), 0.65);
    let id = theme.id.as_str();
    let hash = seed(id);
    let pixel = |x: i32, y: i32| -> Rgb {
        let star = if compact {
            (x * 43 + y * 67 + (hash % 83) as i32) % 113 == 0
        } else {
            (x * 37 + y * 71 + (hash % 97) as i32) % 89 == 0
        };
        let moon_r = if compact { 20 } else { 16 };
        let moon_c = if compact { 16 } else { 14 };
        let moon = (x - 47).pow(2) + (y - 5).pow(2) <= moon_r
            && (x - 50).pow(2) + (y - 4).pow(2) >= moon_c;
        let mut c: Rgb;
        match id {
            "catppuccin-mocha" => {
                // Background: Deep mocha starry night sky (#1e1e2e -> #313244)
                c = mix((30, 30, 46), (49, 50, 68), f64::from(y) / 10.0);
                if compact {
                    // Compact Mode: Centered peach crescent moon and dreamy lilac cloud over pastel peak
                    let moon_dist = (x - 32).pow(2) + (y - 5).pow(2);
                    let moon_cut = (x - 35).pow(2) + (y - 4).pow(2);
                    if moon_dist <= 20 && moon_cut >= 13 {
                        c = (250, 179, 135); // fab387 peach
                    }
                    if (20..=44).contains(&x)
                        && (8..=11).contains(&y)
                        && ((x - 32).abs() + (y - 9).abs() <= 10)
                    {
                        c = (203, 166, 247); // cba6f7 lilac
                    }
                    let mtn = 16 - (9 - (x - 32).abs() / 2).max(0);
                    if y >= mtn {
                        c = if y == mtn {
                            (180, 190, 254)
                        } else {
                            (137, 180, 250)
                        };
                    }
                    if y >= 20 {
                        c = (24, 24, 37); // #181825 deep base
                    }
                } else {
                    // Gallery Mode: Rich starry pastel cosmos with peach crescent moon and layered lavender mountains
                    if star && y < 10 {
                        c = (205, 214, 244);
                    }
                    let moon_dist = (x - 47).pow(2) + (y - 5).pow(2);
                    let moon_cut = (x - 50).pow(2) + (y - 4).pow(2);
                    if moon_dist <= 18 && moon_cut >= 13 {
                        c = (250, 179, 135);
                    }
                    if (12..=34).contains(&x) && (7..=9).contains(&y) && (x + y) % 3 != 0 {
                        c = (203, 166, 247);
                    }
                    let m1 = 16 - (10 - (x - 22).abs()).max(0);
                    let m2 = 18 - (12 - (x - 44).abs()).max(0);
                    if y >= m1 {
                        c = (137, 180, 250);
                    }
                    if y >= m2 {
                        c = (180, 190, 254);
                    }
                    if y >= 21 {
                        c = (49, 50, 68);
                    }
                    if y >= 26 {
                        c = (24, 24, 37);
                    }
                }
            }
            "catppuccin-latte" => {
                c = (239, 241, 245);
                if compact {
                    if (x - 32).pow(2) + (y - 5).pow(2) <= 16 {
                        c = (234, 160, 50);
                    }
                    let m1 = 15 - (8 - (x - 32).abs()).max(0);
                    if y >= m1 {
                        c = (222, 149, 132);
                    }
                    if y >= 19 {
                        c = (136, 57, 239);
                    }
                } else {
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
            }
            "rose-pine" => {
                c = mix((42, 38, 56), (235, 188, 186), f64::from(y) / 10.0);
                if compact {
                    if (x - 32).pow(2) + (y - 5).pow(2) <= 16 {
                        c = (246, 193, 119);
                    }
                    let mtn = 15 - (9 - (x - 32).abs()).max(0);
                    if y >= mtn {
                        c = (235, 188, 186);
                    }
                    if (x == 20 || x == 44) && (12..=19).contains(&y) {
                        c = (49, 116, 143);
                    }
                    if y >= 20 {
                        c = (25, 23, 36);
                    }
                } else {
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
                        if local.abs() <= (23 - y) && (16..24).contains(&y) {
                            c = (25, 23, 36);
                        }
                        if y >= 25 {
                            c = (20, 18, 28);
                        }
                    }
                }
            }
            "kanagawa" => {
                c = (220, 215, 186);
                if compact {
                    let fuji = 12 - (6 - (x - 32).abs()).max(0);
                    if y >= fuji && y < 15 {
                        c = if y - fuji <= 2 {
                            (255, 255, 255)
                        } else {
                            (45, 79, 103)
                        };
                    }
                    let wave = 16 + ((x as f64 / 5.0).sin() * 4.0) as i32;
                    if y >= wave {
                        c = if y - wave <= 2 {
                            (255, 255, 255)
                        } else {
                            (31, 31, 40)
                        };
                    }
                } else {
                    let fuji = 12 - (6 - (x - 46).abs()).max(0);
                    if y >= fuji && y < 14 {
                        c = if y - fuji <= 2 {
                            (255, 255, 255)
                        } else {
                            (45, 79, 103)
                        };
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
            }
            "everforest" => {
                c = (55, 65, 58);
                if compact {
                    // Compact Mode: Centered majestic pine tree with emerald foliage & warm spirit lantern
                    let tree_top = 7;
                    let dx = (x - 32).abs();
                    if (tree_top..=21).contains(&y) && dx <= (y - tree_top) / 2 + 1 {
                        c = if (y - tree_top) % 3 == 0 {
                            (167, 192, 128)
                        } else {
                            (133, 153, 0)
                        };
                    }
                    if (x - 32).pow(2) + (y - 12).pow(2) <= 4 {
                        c = (219, 188, 127); // warm lantern
                    }
                    if y >= 20 {
                        c = (39, 46, 51);
                    }
                } else {
                    let m1 = 15 - (8 - (x - 24).abs() / 2).max(0);
                    if y >= m1 {
                        c = (75, 95, 80);
                    }
                    let m2 = 18 - (6 - (x - 42).abs() / 2).max(0);
                    if y >= m2 {
                        c = (90, 120, 95);
                    }
                    if (17..=18).contains(&y) && (x + y) % 3 != 0 {
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
            }
            "dracula" => {
                c = (40, 42, 54);
                if compact {
                    let moon_dist = (x - 32).pow(2) + (y - 5).pow(2);
                    if moon_dist <= 18 {
                        c = (241, 250, 140);
                    }
                    let in_spire = (x == 26 || x == 38) && (6..=16).contains(&y);
                    let in_wall = (27..=37).contains(&x) && y >= 11;
                    if in_spire || in_wall {
                        c = (25, 27, 36);
                        if (y == 12 || y == 13) && (x == 29 || x == 35) {
                            c = (255, 121, 198);
                        }
                    }
                    if y >= 18 {
                        c = (18, 20, 28);
                    }
                } else {
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
                        } else if (12..=14).contains(&y) && (x == 37 || x == 47) {
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
            }
            "nord" => {
                c = (46, 52, 64);
                if compact {
                    let aurora = 5 + ((x as f64 / 6.0).sin() * 2.5) as i32;
                    if (aurora..=aurora + 3).contains(&y) {
                        c = mix(
                            (136, 192, 208),
                            (163, 190, 140),
                            f64::from(y - aurora) / 3.0,
                        );
                    }
                    let mtn = 15 - (9 - (x - 32).abs()).max(0);
                    if y >= mtn {
                        c = if y - mtn <= 2 {
                            (236, 239, 244)
                        } else {
                            (76, 86, 106)
                        };
                    }
                    if y >= 20 {
                        c = (36, 41, 51);
                    }
                } else {
                    if star && y < 10 {
                        c = (236, 239, 244);
                    }
                    let aurora = 4 + ((x as f64 / 7.0).sin() * 2.5) as i32;
                    if (aurora..=aurora + 3).contains(&y) {
                        c = mix(
                            (136, 192, 208),
                            (163, 190, 140),
                            f64::from(y - aurora) / 3.0,
                        );
                    }
                    let mtn = 17 - (11 - (x - 28).abs()).max(0);
                    if y >= mtn {
                        c = if y - mtn <= 2 {
                            (236, 239, 244)
                        } else {
                            (76, 86, 106)
                        };
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
            }
            "tokyo-night" => {
                c = (26, 27, 38);
                if compact {
                    if (x == 32 && (3..=16).contains(&y))
                        || ((x - 32).abs() == 1 && (10..=16).contains(&y))
                    {
                        c = (247, 118, 142);
                    }
                    let bldg = if (20..=28).contains(&x) {
                        11
                    } else if (36..=44).contains(&x) {
                        9
                    } else {
                        16
                    };
                    if y >= bldg {
                        c = (36, 40, 59);
                        if x % 2 == 0 && (y - bldg) % 2 == 1 && y <= 18 {
                            c = if x % 4 == 0 {
                                (122, 162, 247)
                            } else {
                                (187, 154, 247)
                            };
                        }
                    }
                    if y >= 20 {
                        c = (18, 20, 28);
                    }
                } else {
                    if star && y < 8 {
                        c = (192, 202, 245);
                    }
                    if (x == 46 && (2..=12).contains(&y))
                        || ((x - 46).abs() == 1 && (8..=12).contains(&y))
                    {
                        c = (247, 118, 142);
                    }
                    let roof = 10 + ((x / 6 * 17 + 5) % 8);
                    if y >= roof {
                        c = (36, 40, 59);
                        if x % 6 != 0 && x % 2 == 0 && (y - roof) % 2 == 0 && y >= 11 {
                            c = if x % 4 == 0 {
                                (125, 207, 255)
                            } else {
                                (187, 154, 247)
                            };
                        }
                    }
                    if y >= 19 {
                        c = (18, 20, 28);
                        if y == 20 && (x % 4 == 0) {
                            c = (125, 207, 255);
                        }
                    }
                }
            }
            "gruvbox-dark" => {
                c = mix((40, 40, 40), (80, 73, 69), f64::from(y) / 10.0);
                if compact {
                    if (x - 32).pow(2) + (y - 5).pow(2) <= 16 {
                        c = (250, 189, 47);
                    }
                    let mesa = if (20..=44).contains(&x) { 12 } else { 16 };
                    if y >= mesa {
                        c = (214, 93, 14);
                    }
                    if (x == 20 && (10..=19).contains(&y))
                        || (x == 18 && (12..=16).contains(&y))
                        || (x == 22 && (13..=17).contains(&y))
                    {
                        c = (152, 151, 26);
                    }
                    if y >= 20 {
                        c = (29, 32, 33);
                    }
                } else {
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
                        c = (40, 36, 34);
                        if (x == 14 && (14..=22).contains(&y))
                            || (x == 12 && (16..=19).contains(&y))
                            || (x == 16 && (17..=20).contains(&y))
                        {
                            c = (152, 151, 26);
                        }
                        if y >= 25 {
                            c = (29, 32, 33);
                        }
                    }
                }
            }
            "gruvbox-light" => {
                c = (251, 241, 199);
                if compact {
                    if (x - 32).pow(2) + (y - 5).pow(2) <= 16 {
                        c = (214, 93, 14);
                    }
                    let mesa = if (20..=44).contains(&x) { 12 } else { 16 };
                    if y >= mesa {
                        c = (175, 58, 3);
                    }
                    if y >= 20 {
                        c = (120, 100, 75);
                    }
                } else {
                    if (x - 44).pow(2) + (y - 5).pow(2) <= 14 {
                        c = (214, 93, 14);
                    }
                    let mesa = if (18..=38).contains(&x) { 11 } else { 16 };
                    if y >= mesa {
                        c = (175, 58, 3);
                    }
                    if y >= 19 {
                        c = (120, 100, 75);
                        if y >= 25 {
                            c = (90, 75, 55);
                        }
                    }
                }
            }
            "one-dark" => {
                c = mix((40, 44, 52), (30, 34, 42), f64::from(y) / 10.0);
                if compact {
                    if (x - 32).abs() <= 1 && (3..=5).contains(&y)
                        || (y == 4 && (31..=33).contains(&x))
                    {
                        c = (97, 175, 239);
                    }
                    let p1 = 15 - (8 - (x - 24).abs()).max(0);
                    let p2 = 14 - (8 - (x - 40).abs()).max(0);
                    let peak = p1.min(p2);
                    if y >= peak {
                        c = if y == peak {
                            (97, 175, 239)
                        } else {
                            (198, 120, 221)
                        };
                    }
                    if y >= 19 {
                        c = (33, 37, 43);
                    }
                } else {
                    if star && y < 10 {
                        c = (171, 178, 191);
                    }
                    if (x - 18).abs() <= 1 && (4..=6).contains(&y)
                        || (y == 5 && (17..=19).contains(&x))
                    {
                        c = (97, 175, 239);
                    }
                    let m1 = 16 - (10 - (x - 30).abs()).max(0);
                    if y >= m1 {
                        c = (198, 120, 221);
                    }
                    let m2 = 19 - (9 - (x - 48).abs()).max(0);
                    let m3 = 20 - (8 - (x - 16).abs()).max(0);
                    let fore = m2.min(m3);
                    if y >= fore {
                        c = if y - fore <= 1 {
                            (97, 175, 239)
                        } else {
                            (64, 70, 84)
                        };
                    }
                    if y >= 25 {
                        c = (30, 34, 42);
                    }
                }
            }
            "synthwave" => {
                c = mix((25, 12, 45), (120, 20, 90), f64::from(y) / 11.0);
                if compact {
                    let sun_dist = (x - 32).pow(2) + (y - 6).pow(2);
                    if sun_dist <= 30 {
                        c = if y % 2 == 0 {
                            (255, 45, 140)
                        } else {
                            (255, 215, 0)
                        };
                    }
                    if y >= 15 {
                        c = (18, 8, 32);
                        if [15, 19, 24].contains(&y) {
                            c = (255, 45, 140);
                        }
                        let depth = (y - 13).max(1);
                        if ((x - 32) * 6 / depth).abs() % 4 == 0 {
                            c = (0, 240, 255);
                        }
                    }
                } else {
                    let sun_dist = (x - 32).pow(2) + (y - 7).pow(2);
                    if sun_dist <= 26 {
                        c = if y % 2 == 0 {
                            (255, 80, 140)
                        } else {
                            (255, 180, 40)
                        };
                    }
                    let mtn = 16 - (8 - (x - 32).abs() / 2).max(0);
                    if y >= mtn && y < 17 {
                        c = (40, 15, 60);
                        if y == mtn {
                            c = (0, 240, 255);
                        }
                    }
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
            }
            "oceanic-next" => {
                c = mix((27, 43, 52), (102, 153, 204), f64::from(y) / 10.0);
                if compact {
                    let reef = 15 - (9 - (x - 32).abs()).max(0);
                    if y >= reef {
                        c = (40, 70, 85);
                        if y == reef || (x + y) % 2 == 0 {
                            c = (95, 179, 179);
                        }
                    }
                    if y >= 20 {
                        c = (18, 30, 38);
                    }
                } else {
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
            }
            "monokai-pro" => {
                c = (45, 42, 46);
                if compact {
                    if (18..=46).contains(&x) && (10..=15).contains(&y) {
                        c = match y {
                            10 => (255, 97, 136),
                            11 => (252, 152, 103),
                            12 => (255, 216, 102),
                            13 => (169, 220, 118),
                            14 => (120, 220, 232),
                            _ => (171, 157, 242),
                        };
                    }
                    if y >= 17 {
                        c = (34, 31, 34);
                        if [18, 22].contains(&y) || (x % 6 == 0) {
                            c = (171, 157, 242);
                        }
                    }
                } else {
                    if (x - 32).abs() <= 18 && (9..=17).contains(&y) {
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
                    if y >= 18 {
                        c = (34, 31, 34);
                        if [19, 24].contains(&y) || (x % 8 == 0) {
                            c = (171, 157, 242);
                        }
                    }
                }
            }
            "palenight" => {
                c = mix((35, 32, 54), (55, 45, 80), f64::from(y) / 10.0);
                if compact {
                    let isle_w = 12 - (y - 12).abs() * 2;
                    if (x - 32).abs() <= isle_w && (10..=17).contains(&y) {
                        c = (55, 50, 75);
                        if (31..=33).contains(&x) && (6..=10).contains(&y) {
                            c = (199, 146, 234);
                        }
                    }
                    if y >= 19 {
                        c = (30, 28, 46);
                    }
                } else {
                    if star && y < 10 {
                        c = (200, 210, 255);
                    }
                    if moon {
                        c = (240, 240, 255);
                    }
                    let isle_w = 12 - (y - 12).abs() * 2;
                    if (x - 32).abs() <= isle_w && (10..=17).contains(&y) {
                        c = (55, 50, 75);
                        if (31..=33).contains(&x) && (6..=10).contains(&y) {
                            c = (199, 146, 234);
                        }
                        if y == 10 && (x % 2 == 0) {
                            c = (195, 232, 141);
                        }
                    }
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
            }
            "monochrome" => {
                c = (0, 0, 0);
                if compact {
                    let moon_dist = (x - 32).pow(2) + (y - 5).pow(2);
                    let moon_cut = (x - 35).pow(2) + (y - 4).pow(2);
                    if moon_dist <= 20 && moon_cut >= 14 {
                        c = (255, 255, 255);
                    }
                    let p = 15 - (9 - (x - 32).abs()).max(0);
                    if y >= p {
                        c = if y - p <= 2 {
                            (255, 255, 255)
                        } else {
                            (95, 95, 95)
                        };
                    }
                    if y >= 20 {
                        c = (20, 20, 20);
                    }
                } else {
                    if (x == 12 && y == 3) || (x == 26 && y == 5) || (x == 56 && y == 2) {
                        c = (225, 225, 225);
                    }
                    let moon_dist = (x - 47).pow(2) + (y - 5).pow(2);
                    let moon_cut = (x - 50).pow(2) + (y - 4).pow(2);
                    if moon_dist <= 18 && moon_cut >= 14 {
                        c = (255, 255, 255);
                    }
                    let p1 = 18 - (9 - (x - 20).abs()).max(0);
                    let p2 = 19 - (8 - (x - 36).abs()).max(0);
                    let ridge = p1.min(p2);
                    if y >= ridge {
                        if y - ridge <= 2 && ridge <= 13 {
                            c = (250, 250, 250);
                        } else {
                            c = (95, 95, 95);
                        }
                    }
                    let fore = 23 - (6 - (x - 50).abs()).max(0);
                    if y >= fore {
                        c = (35, 35, 35);
                    }
                    if (x - 50).abs() <= (23 - y) / 3 && (14..23).contains(&y) {
                        c = (12, 12, 12);
                    }
                    if y >= 25 {
                        c = (18, 18, 18);
                    }
                }
            }
            "solarized-light" => {
                c = (253, 246, 227);
                if compact {
                    if (x - 32).pow(2) + (y - 5).pow(2) <= 16 {
                        c = (181, 137, 0);
                    }
                    let dune1 = 14 + ((x as f64 / 8.0).sin() * 3.5) as i32;
                    if y >= dune1 {
                        c = (203, 75, 22);
                    }
                    let dune2 = 19 + ((x as f64 / 10.0).cos() * 2.5) as i32;
                    if y >= dune2 {
                        c = (133, 153, 0);
                    }
                    if y >= 22 {
                        c = (101, 123, 131);
                    }
                } else {
                    if (x - 46).pow(2) + (y - 5).pow(2) <= 14 {
                        c = (181, 137, 0);
                    }
                    let dune1 = 15 + ((x as f64 / 9.0).sin() * 4.0) as i32;
                    if y >= dune1 {
                        c = (203, 75, 22);
                    }
                    let dune2 = 21 + ((x as f64 / 12.0).cos() * 3.0) as i32;
                    if y >= dune2 {
                        c = (133, 153, 0);
                    }
                    if y >= 25 {
                        c = (101, 123, 131);
                    }
                }
            }
            "ayu-light" => {
                c = (250, 250, 250);
                if compact {
                    if (x - 32).pow(2) + (y - 5).pow(2) <= 18 {
                        c = (255, 153, 64);
                    }
                    let mtn = 14 - (8 - (x - 32).abs() / 2).max(0);
                    if y >= mtn {
                        c = (242, 151, 24);
                    }
                    if y >= 19 {
                        c = (54, 163, 217);
                    }
                    if y >= 23 {
                        c = (92, 97, 102);
                    }
                } else {
                    if (x - 46).pow(2) + (y - 5).pow(2) <= 14 {
                        c = (255, 153, 64);
                    }
                    let m1 = 16 - (9 - (x - 28).abs()).max(0);
                    if y >= m1 {
                        c = (242, 151, 24);
                    }
                    let m2 = 20 - (10 - (x - 46).abs()).max(0);
                    if y >= m2 {
                        c = (54, 163, 217);
                    }
                    if y >= 24 {
                        c = (92, 97, 102);
                    }
                }
            }
            "one-light" => {
                c = (242, 244, 248);
                if compact {
                    if (x - 20).pow(2) + (y - 5).pow(2) <= 14 {
                        c = (245, 185, 75);
                    }
                    if (30..=36).contains(&x) && (4..=18).contains(&y) {
                        let band = (y - 4) / 3;
                        c = if band % 2 == 0 {
                            (228, 86, 73)
                        } else {
                            (255, 255, 255)
                        };
                    }
                    if y >= 18 {
                        c = (64, 120, 242);
                    }
                } else {
                    if (x - 14).pow(2) + (y - 5).pow(2) <= 14 {
                        c = (245, 185, 75);
                    }
                    if (5..=7).contains(&y) && (12..41).contains(&x) && (x + y) % 3 != 0 {
                        c = (255, 235, 140);
                    }
                    if (41..=47).contains(&x) {
                        if (4..=7).contains(&y) {
                            if (42..=46).contains(&x) {
                                c = if (5..=6).contains(&y) {
                                    (255, 235, 110)
                                } else {
                                    (45, 50, 60)
                                };
                            }
                        } else if (8..=18).contains(&y) {
                            let band = (y - 8) / 3;
                            c = if band % 2 == 0 {
                                (228, 86, 73)
                            } else {
                                (255, 255, 255)
                            };
                            if x == 41 || x == 47 {
                                c = (55, 60, 70);
                            }
                        }
                    }
                    let cliff = 19 - (6 - (x - 44).abs()).max(0);
                    if x >= 36 && y >= cliff {
                        c = (50, 56, 68);
                    }
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
            }
            "sakura" => {
                c = mix((30, 24, 34), (74, 45, 60), f64::from(y) / 10.0);
                if compact {
                    let in_post = (x == 28 || x == 36) && (10..=18).contains(&y);
                    let in_beam = (25..=39).contains(&x) && (y == 11 || y == 13);
                    if in_post || in_beam {
                        c = (210, 15, 57);
                    }
                    if y <= 9 && (x + y) % 3 == 0 {
                        c = (243, 139, 168);
                    }
                    if y >= 19 {
                        c = (25, 18, 26);
                    }
                } else {
                    let moon_dist = (x - 47).pow(2) + (y - 5).pow(2);
                    if moon_dist <= 14 {
                        c = (255, 240, 245);
                    }
                    let in_post = (x == 26 || x == 38) && (11..=18).contains(&y);
                    let in_beam = (22..=42).contains(&x) && (y == 12 || y == 14);
                    if in_post || in_beam {
                        c = (210, 15, 57);
                    }
                    let near_blossom = ((x - 22).pow(2) + (y - 11).pow(2) <= 24)
                        || ((x - 42).pow(2) + (y - 11).pow(2) <= 24)
                        || (y < 9 && (x * 7 + y * 13) % 17 < 3);
                    if near_blossom {
                        c = (243, 139, 168);
                    }
                    if y >= 19 {
                        c = (28, 20, 30);
                    }
                }
            }
            "lavender" => {
                c = mix((35, 29, 52), (88, 75, 114), f64::from(y) / 10.0);
                if compact {
                    if (x - 32).pow(2) + (y - 5).pow(2) <= 16 {
                        c = (245, 224, 220);
                    }
                    let hill1 = 14 + ((x as f64 / 6.0).sin() * 3.0) as i32;
                    if y >= hill1 {
                        c = (203, 166, 247);
                    }
                    let hill2 = 18 + ((x as f64 / 8.0).cos() * 2.5) as i32;
                    if y >= hill2 {
                        c = (136, 57, 239);
                    }
                    if y >= 21 {
                        c = (24, 20, 36);
                    }
                } else {
                    if (x - 47).pow(2) + (y - 5).pow(2) <= 14 {
                        c = (245, 224, 220);
                    }
                    let h1 = 15 + ((x as f64 / 8.0).sin() * 3.0) as i32;
                    if y >= h1 {
                        c = (203, 166, 247);
                    }
                    let h2 = 19 + ((x as f64 / 11.0).cos() * 3.0) as i32;
                    if y >= h2 {
                        c = (136, 57, 239);
                    }
                    if y >= 23 {
                        c = (30, 24, 45);
                    }
                }
            }
            "candy" => {
                c = mix((38, 30, 48), (110, 75, 120), f64::from(y) / 10.0);
                if compact {
                    let cloud = (x - 32).pow(2) + (y - 7).pow(2);
                    if cloud <= 24 {
                        c = (245, 194, 231);
                    }
                    let hill = 14 + ((x as f64 / 7.0).sin() * 3.0) as i32;
                    if y >= hill {
                        c = (166, 227, 161);
                    }
                    let fore = 18 + ((x as f64 / 9.0).cos() * 2.5) as i32;
                    if y >= fore {
                        c = (137, 220, 235);
                    }
                    if y >= 21 {
                        c = (28, 22, 36);
                    }
                } else {
                    let c1 = (x - 22).pow(2) + (y - 7).pow(2);
                    let c2 = (x - 42).pow(2) + (y - 6).pow(2);
                    if c1 <= 18 || c2 <= 20 {
                        c = (245, 194, 231);
                    }
                    let h1 = 15 + ((x as f64 / 8.0).sin() * 3.0) as i32;
                    if y >= h1 {
                        c = (166, 227, 161);
                    }
                    let h2 = 20 + ((x as f64 / 10.0).cos() * 3.0) as i32;
                    if y >= h2 {
                        c = (137, 220, 235);
                    }
                    if y >= 24 {
                        c = (32, 24, 42);
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
            "cyberpunk" => {
                c = (10, 8, 20);
                let bldg = (x / 8 * 13 + 5) % 9;
                if y >= 9 + bldg {
                    c = (25, 15, 45);
                    if (y - bldg) % 2 == 0 && x % 4 == 0 && y >= 11 {
                        c = if x % 2 == 0 {
                            (252, 238, 10)
                        } else {
                            (0, 240, 255)
                        };
                    }
                }
                if y >= 19 {
                    c = (12, 10, 14);
                    if [19, 23].contains(&y) {
                        c = (255, 0, 85);
                    }
                }
            }
            "naruto" => {
                c = mix((255, 110, 20), (255, 185, 45), f64::from(y) / 10.0);
                let cliff = 16 - (10 - (x - 30).abs() / 2).max(0);
                if y >= cliff {
                    c = (140, 75, 35);
                    if (y + x) % 4 == 0 && y >= cliff + 2 {
                        c = (105, 55, 25);
                    }
                    if y == cliff + 3 && (18..=42).contains(&x) {
                        c = (190, 195, 205);
                    }
                }
                if y >= 18 {
                    c = (35, 75, 40);
                    if (x % 8 <= 3 && y == 19) || (y >= 24) {
                        c = if y == 19 { (235, 95, 25) } else { (25, 55, 30) };
                    }
                }
            }
            "pokemon" => {
                c = (120, 190, 245);
                if ((x - 12) == (y - 2) * 2 && (2..=4).contains(&y))
                    || (y == 4 && (13..=16).contains(&x))
                    || ((x - 14) == (y - 4) * 2 && (4..=7).contains(&y))
                {
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
                    c = (85, 170, 50);
                    if y >= 25 {
                        c = (45, 95, 25);
                    }
                }
            }
            "spiderman" => {
                c = (12, 16, 28);
                if (x - 14).pow(2) + (y - 5).pow(2) <= 12 {
                    c = (240, 245, 255);
                }
                if ((x - 28) - (y - 4) * 2).abs() <= 1 && (4..=12).contains(&y) {
                    c = (220, 230, 255);
                }
                let bldg = (x / 7 * 19 + 7) % 8;
                if y >= 10 + bldg {
                    c = (20, 26, 42);
                    if x % 7 >= 2 && x % 2 == 0 && (y - bldg) % 3 == 0 && y >= 12 {
                        c = if x % 4 == 0 {
                            (255, 210, 80)
                        } else {
                            (80, 180, 255)
                        };
                    }
                }
                if (29..=33).contains(&x) && (10..=15).contains(&y) {
                    c = if y <= 12 { (215, 30, 45) } else { (0, 75, 195) };
                    if y == 11 && (x == 30 || x == 32) {
                        c = (255, 255, 255);
                    }
                }
                if y >= 19 {
                    c = (10, 12, 20);
                    if y == 19 && (x % 6 == 0 || x % 6 == 1) {
                        c = if x % 12 < 6 {
                            (215, 30, 45)
                        } else {
                            (0, 80, 210)
                        };
                    }
                }
            }
            "doom" => {
                c = mix((90, 15, 12), (235, 65, 20), f64::from(y) / 10.0);
                if y < 8 && (x * 7 + y * 11) % 19 < 4 {
                    c = (140, 35, 15);
                }
                let s1 = 18 - (10 - (x - 16).abs() * 2).max(0);
                let s2 = 18 - (12 - (x - 32).abs() * 2).max(0);
                let s3 = 18 - (9 - (x - 48).abs() * 2).max(0);
                let spires = s1.min(s2).min(s3);
                if y >= spires {
                    c = (18, 6, 8);
                }
                if y >= 18 {
                    c = (28, 10, 12);
                    let river_center = 26 + (y - 18) * 2;
                    if (x - river_center).abs() <= 6 {
                        c = if (x - river_center).abs() <= 2 {
                            (255, 225, 60)
                        } else {
                            (255, 110, 25)
                        };
                    }
                    if y >= 25 && (x + y) % 3 != 0 {
                        c = (245, 95, 20);
                    }
                }
            }
            "valorant" => {
                c = (15, 25, 35);
                if (x * 3 + y * 5) % 17 == 0 && y < 10 {
                    c = (60, 215, 195);
                }
                let spire = 18 - (12 - (x - 48).abs()).max(0);
                if y >= spire {
                    c = (25, 38, 50);
                }
                let left_slash = (x - 26) == (y - 7);
                let left_thick = (x - 27) == (y - 7) || (x - 25) == (y - 7);
                if (left_slash || left_thick) && (7..=18).contains(&y) {
                    c = (245, 245, 250);
                }
                let right_slash = (38 - x) == (y - 7);
                let right_thick = (39 - x) == (y - 7) || (37 - x) == (y - 7);
                if (right_slash || right_thick) && (7..=18).contains(&y) {
                    c = (255, 70, 85);
                }
                if y >= 19 {
                    c = (20, 30, 42);
                    if y == 19 && (x % 4 == 0) {
                        c = (60, 215, 195);
                    }
                }
            }
            "minecraft" => {
                c = (115, 180, 235);
                if (34..=52).contains(&x) && (3..=6).contains(&y) {
                    c = (245, 250, 255);
                }
                if (10..=24).contains(&x) && (6..=14).contains(&y) {
                    c = if (x + y) % 2 == 0 {
                        (80, 145, 45)
                    } else {
                        (60, 115, 35)
                    };
                }
                if (15..=18).contains(&x) && (14..=18).contains(&y) {
                    c = (110, 75, 45);
                }
                if y >= 18 {
                    if y == 18 || y == 19 {
                        c = (95, 165, 55);
                    } else {
                        c = if (x + y) % 3 == 0 {
                            (125, 85, 50)
                        } else {
                            (95, 65, 38)
                        };
                    }
                }
            }
            _ => {
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
                cell.set_char('▀')
                    .set_fg(Color::Rgb(t.0, t.1, t.2))
                    .set_bg(Color::Rgb(b.0, b.1, b.2));
            }
        }
    }
}
