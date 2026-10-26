//! The saved config serves every terminal; at run time it is adapted to the one that
//! shows the output: the image protocol it speaks, its colour depth and, where no image
//! protocol exists (Terminal.app), the image logo pre-drawn as coloured block characters.

use super::config::detect_image_protocol;
use crate::adapters::fsutil::atomic_write;
use crate::domain::color::nearest_256;
use crate::imaging::{BlockGrid, CellSize, Shape, TermGeometry, fit_image_cells};
use crate::tui::color_depth::lacks_truecolor;
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// fastfetch logo types that draw an image file.
const IMAGE_TYPES: [&str; 5] = ["kitty-direct", "kitty", "iterm", "sixel", "chafa"];

/// What the terminal printing the output can show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Caps {
    /// fastfetch logo type for images; `chafa` means no image protocol.
    pub image: &'static str,
    pub truecolor: bool,
    /// Whether OSC 10/11/12 change the terminal's colours (Terminal.app ignores them).
    pub osc_colors: bool,
    /// Terminal size in columns and rows, when known.
    pub cols: Option<u16>,
    pub rows: Option<u16>,
    /// Pixel size of a cell, for the logo's aspect ratio.
    pub cell: CellSize,
    /// Cell division for block-drawn logos.
    pub shape: Shape,
}

impl Caps {
    pub fn detect() -> Self {
        let env = |k: &str| std::env::var(k).unwrap_or_default();
        let program = env("TERM_PROGRAM");
        let size = crossterm::terminal::size().ok();
        Self {
            image: detect_image_protocol(),
            truecolor: !lacks_truecolor(&program, &env("COLORTERM")),
            osc_colors: program != "Apple_Terminal",
            cols: size.map(|(c, _)| c),
            rows: size.map(|(_, r)| r),
            cell: TermGeometry::detect().cell,
            shape: Shape::detect(),
        }
    }
}

/// Columns the module text needs beside the logo to stay readable (icon, key and the
/// start of the value); longer lines are cut at the window edge by fastfetch.
const TEXT_COLS: u16 = 40;

/// Columns left for the module text when a block-drawn logo grows: the longest common
/// lines (disks, Wi-Fi) with their icon and key.
const FULL_TEXT_COLS: u16 = 72;

/// Width of the logo in columns: the image box, or the widest line of an ASCII file.
fn logo_cols(logo: &serde_json::Map<String, Value>) -> Option<u16> {
    if let Some(w) = logo.get("width").and_then(Value::as_u64) {
        return Some(w as u16);
    }
    let source = logo.get("source").and_then(Value::as_str)?;
    let text = std::fs::read_to_string(source).ok()?;
    text.lines()
        .map(|l| unicode_width::UnicodeWidthStr::width(l) as u16)
        .max()
}

/// `38;2;r;g;b` / `48;2;r;g;b` sequences and bare `#rrggbb` values as 256-colour codes.
pub fn to_256(s: &str) -> String {
    if let Some(hex) = s.strip_prefix('#').filter(|h| h.len() == 6)
        && let Ok(v) = u32::from_str_radix(hex, 16)
    {
        return format!(
            "38;5;{}",
            nearest_256((v >> 16) as u8, (v >> 8) as u8, v as u8)
        );
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find(";2;") {
        let (head, tail) = rest.split_at(i);
        let layer = &head[head.len().saturating_sub(2)..];
        let rgb: Vec<&str> = tail[3..]
            .splitn(4, |c: char| !c.is_ascii_digit())
            .take(3)
            .collect();
        let nums: Option<Vec<u8>> = rgb.iter().map(|n| n.parse().ok()).collect();
        match nums {
            Some(n) if (layer == "38" || layer == "48") && n.len() == 3 => {
                let len = 3 + rgb.iter().map(|n| n.len()).sum::<usize>() + 2;
                out.push_str(head);
                out.push_str(&format!(";5;{}", nearest_256(n[0], n[1], n[2])));
                rest = &tail[len..];
            }
            _ => {
                out.push_str(head);
                out.push_str(";2;");
                rest = &tail[3..];
            }
        }
    }
    out.push_str(rest);
    out
}

fn strings_to_256(v: &mut Value) {
    match v {
        Value::String(s) => *s = to_256(s),
        Value::Array(a) => a.iter_mut().for_each(strings_to_256),
        Value::Object(o) => o.values_mut().for_each(strings_to_256),
        _ => {}
    }
}

/// Adapt a parsed config to `caps`. Where the terminal has no image protocol,
/// `blocks(source, max_w, max_h)` draws an image logo as block characters within that
/// box and returns the file and its width in columns.
pub fn adapt(
    cfg: &mut Value,
    caps: &Caps,
    blocks: impl FnOnce(&str, u16, u16) -> Option<(PathBuf, u16)>,
) {
    let lines = cfg
        .get("modules")
        .and_then(Value::as_array)
        .map_or(0, Vec::len) as u16
        + 2;
    if let Some(logo) = cfg.get_mut("logo").and_then(Value::as_object_mut) {
        let pad = logo
            .get("padding")
            .and_then(|p| p.get("right"))
            .and_then(Value::as_u64)
            .unwrap_or(0) as u16;
        let mut width = logo_cols(logo);
        let is_image = logo
            .get("type")
            .and_then(Value::as_str)
            .is_some_and(|t| IMAGE_TYPES.contains(&t));
        if is_image {
            let size = |k: &str| logo.get(k).and_then(Value::as_u64).map(|n| n as u16);
            let source = logo.get("source").and_then(Value::as_str).map(String::from);
            let drawn = match (caps.image, source, size("width"), size("height")) {
                ("chafa", Some(src), Some(w), Some(h)) => {
                    // Blocks show more detail the more cells they get: as tall as the
                    // info text and as wide as the window leaves room for.
                    let max_h = h
                        .max(lines)
                        .min(caps.rows.map_or(u16::MAX, |r| r.saturating_sub(2)));
                    let max_w = caps
                        .cols
                        .map_or(w, |c| c.saturating_sub(pad + FULL_TEXT_COLS))
                        .max(w);
                    blocks(&src, max_w, max_h)
                }
                _ => None,
            };
            if let Some((file, w)) = drawn {
                logo.insert("type".into(), "file-raw".into());
                logo.insert("source".into(), file.to_string_lossy().into_owned().into());
                logo.remove("width");
                logo.remove("height");
                width = Some(w);
            } else {
                logo.insert("type".into(), caps.image.into());
            }
        }
        // In a window too narrow for logo and text side by side, the logo goes on top.
        if let (Some(cols), Some(w)) = (caps.cols, width)
            && cols < w + pad + TEXT_COLS
        {
            logo.insert("position".into(), "top".into());
        }
    }
    if !caps.truecolor {
        strings_to_256(cfg);
    }
}

fn sgr((r, g, b): (u8, u8, u8), fg: bool, truecolor: bool) -> String {
    let layer = if fg { 38 } else { 48 };
    if truecolor {
        format!("{layer};2;{r};{g};{b}")
    } else {
        format!("{layer};5;{}", nearest_256(r, g, b))
    }
}

/// `grid` as ANSI text in quadrant blocks; transparent pixels keep the terminal's own
/// background.
pub fn block_text(grid: &BlockGrid, truecolor: bool) -> String {
    let mut out = String::new();
    for y in 0..grid.h {
        for x in 0..grid.w {
            match grid.cell(x, y, None) {
                None => out.push_str("\x1b[0m "),
                Some(b) => {
                    out.push_str("\x1b[0;");
                    out.push_str(&sgr(b.fg, true, truecolor));
                    if let Some(bg) = b.bg {
                        out.push(';');
                        out.push_str(&sgr(bg, false, truecolor));
                    }
                    out.push('m');
                    out.push(b.ch);
                }
            }
        }
        out.push_str("\x1b[0m\n");
    }
    out
}

/// Draw `source` into `cache` as block characters fitted into `max_w` x `max_h` cells,
/// reusing a drawing newer than the image. Returns the file and its width.
fn draw_blocks(
    cache: &Path,
    source: &str,
    max_w: u16,
    max_h: u16,
    caps: &Caps,
) -> Option<(PathBuf, u16)> {
    let (pw, ph) = crate::imaging::image_dimensions(Path::new(source))?;
    let (w, h) = fit_image_cells(pw, ph, caps.cell, max_w, max_h);
    let stem = Path::new(source)
        .file_stem()?
        .to_string_lossy()
        .into_owned();
    let depth = if caps.truecolor { "rgb" } else { "256" };
    // The drawing method is in the name, so a file drawn another way is never reused.
    let shape = match caps.shape {
        Shape::Quadrant => "q",
        Shape::Octant => "o",
    };
    let file = cache.join(format!("logo-{stem}-{w}x{h}-{depth}-{shape}.ans"));
    let modified = |p: &Path| std::fs::metadata(p).and_then(|m| m.modified()).ok();
    if modified(&file).is_some_and(|t| modified(Path::new(source)).is_some_and(|s| t >= s)) {
        return Some((file, w));
    }
    let img = crate::imaging::pool::decode(source).ok()?;
    let grid = BlockGrid::from_image(&img.thumb, w, h, caps.shape);
    let text = block_text(&grid, caps.truecolor);
    atomic_write(&file, text.as_bytes()).ok()?;
    Some((file, w))
}

/// Write `config` adapted to `caps` into `cache` and return its path. A config this
/// cannot read is used as it is.
pub fn prepare(config: &Path, cache: &Path, caps: &Caps) -> Result<PathBuf> {
    let text =
        std::fs::read_to_string(config).with_context(|| format!("reading {}", config.display()))?;
    let Ok(mut cfg) = jsonc_parser::parse_to_serde_value::<Value>(&text, &Default::default())
    else {
        return Ok(config.to_path_buf());
    };
    adapt(&mut cfg, caps, |src, w, h| {
        draw_blocks(cache, src, w, h, caps)
    });
    let out = cache.join("terminal.jsonc");
    let json = serde_json::to_string_pretty(&cfg)?;
    if std::fs::read_to_string(&out).ok().as_deref() != Some(json.as_str()) {
        atomic_write(&out, json.as_bytes())?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const KITTY: Caps = Caps {
        image: "kitty-direct",
        truecolor: true,
        osc_colors: true,
        cols: None,
        rows: None,
        cell: CellSize { w: 10, h: 20 },
        shape: Shape::Quadrant,
    };
    const APPLE: Caps = Caps {
        image: "chafa",
        truecolor: false,
        osc_colors: false,
        cols: None,
        rows: None,
        cell: CellSize { w: 10, h: 20 },
        shape: Shape::Quadrant,
    };

    fn image_cfg() -> Value {
        json!({
            "logo": {"type": "chafa", "source": "/x/eren.png", "width": 20, "height": 10},
            "modules": [{"type": "os", "keyColor": "#ff0000", "format": "\u{1b}[38;2;0;0;0mme"}]
        })
    }

    #[test]
    fn image_protocol_follows_the_terminal() {
        let mut cfg = image_cfg();
        adapt(&mut cfg, &KITTY, |_, _, _| panic!("no blocks needed"));
        assert_eq!(cfg["logo"]["type"], "kitty-direct");
        assert_eq!(cfg["logo"]["width"], 20);
        assert_eq!(cfg["modules"][0]["keyColor"], "#ff0000");
    }

    #[test]
    fn terminal_app_gets_drawn_logo_and_256_colours() {
        let mut cfg = image_cfg();
        adapt(&mut cfg, &APPLE, |src, w, h| {
            assert_eq!((src, w, h), ("/x/eren.png", 20, 10));
            Some((PathBuf::from("/cache/eren.ans"), 20))
        });
        assert_eq!(cfg["logo"]["type"], "file-raw");
        assert_eq!(cfg["logo"]["source"], "/cache/eren.ans");
        assert!(cfg["logo"].get("width").is_none());
        assert_eq!(cfg["modules"][0]["keyColor"], "38;5;196");
        assert_eq!(cfg["modules"][0]["format"], "\u{1b}[38;5;16mme");
    }

    #[test]
    fn logo_moves_above_the_text_when_both_do_not_fit_side_by_side() {
        let narrow = Caps {
            cols: Some(50),
            ..KITTY
        };
        let mut cfg = image_cfg();
        adapt(&mut cfg, &narrow, |_, _, _| None);
        assert_eq!(cfg["logo"]["position"], "top");
        let wide = Caps {
            cols: Some(80),
            ..KITTY
        };
        let mut cfg = image_cfg();
        adapt(&mut cfg, &wide, |_, _, _| None);
        assert!(cfg["logo"].get("position").is_none());
    }

    #[test]
    fn block_logos_grow_to_the_info_text_within_the_window() {
        let mut cfg = image_cfg();
        cfg["modules"] = json!(vec!["break"; 30]);
        let window = Caps {
            cols: Some(200),
            rows: Some(25),
            ..APPLE
        };
        adapt(&mut cfg, &window, |_, w, h| {
            // 32 info lines, capped by the 25-row window; width leaves room for the text.
            assert_eq!((w, h), (200 - FULL_TEXT_COLS, 23));
            Some((PathBuf::from("/cache/eren.ans"), 30))
        });
        assert!(cfg["logo"].get("position").is_none());
    }

    #[test]
    fn ascii_logos_are_left_alone() {
        let mut cfg = json!({"logo": {"type": "file", "source": "/x/apple.txt"}});
        adapt(&mut cfg, &APPLE, |_, _, _| panic!("not an image"));
        assert_eq!(cfg["logo"]["type"], "file");
    }

    #[test]
    fn to_256_rewrites_rgb_sequences_only() {
        assert_eq!(to_256("1;38;2;255;0;0;48;2;0;0;0"), "1;38;5;196;48;5;16");
        assert_eq!(to_256("39"), "39");
        assert_eq!(to_256("x;2;y"), "x;2;y");
        assert_eq!(to_256("#zzzzzz"), "#zzzzzz");
    }

    #[test]
    fn block_text_keeps_transparent_pixels_clear() {
        let (red, clear) = ([255, 0, 0, 255], [0, 0, 0, 0]);
        let grid = BlockGrid {
            w: 2,
            h: 1,
            shape: Shape::Quadrant,
            px: vec![red, red, clear, clear, red, red, clear, clear],
        };
        let text = block_text(&grid, true);
        assert_eq!(text, "\x1b[0;38;2;255;0;0;48;2;255;0;0m█\x1b[0m \x1b[0m\n");
        assert!(block_text(&grid, false).contains("38;5;196;48;5;196m█"));
    }
}
