//! Aspect-ratio-correct image sizing and block-character rasterisation.

pub mod kitty;
mod kitty_diacritics;
pub mod pool;
pub mod sixel;

use image::RgbaImage;
use std::path::Path;

/// Pixel size of one terminal cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellSize {
    pub w: u32,
    pub h: u32,
}

impl CellSize {
    /// Conventional 1:2 cell used when the terminal does not report pixels.
    pub const FALLBACK: CellSize = CellSize { w: 8, h: 16 };
}

/// Terminal geometry: size in cells plus cell pixel size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TermGeometry {
    pub cols: u16,
    pub rows: u16,
    pub cell: CellSize,
    /// Whether `cell` came from the terminal (TIOCGWINSZ) rather than the fallback.
    pub cell_reported: bool,
}

impl TermGeometry {
    pub fn detect() -> Self {
        if let Ok(ws) = crossterm::terminal::window_size() {
            return Self::from_window(ws.columns, ws.rows, ws.width, ws.height);
        }
        let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
        Self::from_window(cols, rows, 0, 0)
    }

    pub fn from_window(cols: u16, rows: u16, px_w: u16, px_h: u16) -> Self {
        let cols = cols.max(1);
        let rows = rows.max(1);
        let (cw, ch) = (
            u32::from(px_w) / u32::from(cols),
            u32::from(px_h) / u32::from(rows),
        );
        let reported = cw > 0 && ch > 0;
        Self {
            cols,
            rows,
            cell: if reported {
                CellSize { w: cw, h: ch }
            } else {
                CellSize::FALLBACK
            },
            cell_reported: reported,
        }
    }
}

/// Image size from the file header only (no full decode).
pub fn image_dimensions(path: &Path) -> Option<(u32, u32)> {
    image::ImageReader::open(path)
        .ok()?
        .with_guessed_format()
        .ok()?
        .into_dimensions()
        .ok()
        .filter(|&(w, h)| w > 0 && h > 0)
}

fn div_round(n: u64, d: u64) -> u64 {
    (n + d / 2) / d.max(1)
}

/// Largest cell rectangle within `max_w` x `max_h` that shows a `pw` x `ph`
/// image without distortion, given the cell pixel size.
///
/// Port of the installer's `compute_logo_dimensions`: fit to the bounds, then
/// prefer the largest integer rectangle whose ratio is within 5% of the image.
pub fn fit_image_cells(pw: u32, ph: u32, cell: CellSize, max_w: u16, max_h: u16) -> (u16, u16) {
    let bound_w = u64::from(max_w.max(1));
    let bound_h = u64::from(max_h.max(1));
    let (mut pw, mut ph) = (u64::from(pw), u64::from(ph));
    if pw == 0 || ph == 0 || pw > 1_000_000 || ph > 1_000_000 {
        pw = 1;
        ph = 1;
    }
    let (cw, ch) = (u64::from(cell.w.max(1)), u64::from(cell.h.max(1)));

    let mut w = bound_w;
    let mut h = div_round(w * ph * cw, pw * ch).max(1);
    if h > bound_h {
        h = bound_h;
        w = div_round(h * pw * ch, ph * cw).clamp(1, bound_w);
    }

    let (mut best_area, mut best) = (0u64, (w, h));
    for cand_h in 1..=bound_h {
        let cand_w = div_round(cand_h * pw * ch, ph * cw);
        if cand_w < 1 || cand_w > bound_w {
            continue;
        }
        let target = cand_h * pw * ch;
        let delta = (cand_w * ph * cw).abs_diff(target);
        let area = cand_w * cand_h;
        if delta * 100 <= target * 5 && area > best_area {
            best_area = area;
            best = (cand_w, cand_h);
        }
    }
    (best.0 as u16, best.1 as u16)
}

/// Logo box used in the generated fastfetch config: the size preset, bounded by
/// a third of the terminal width and its height minus 4 rows (as the installer did).
pub fn config_logo_cells(dims: (u32, u32), max: (u16, u16), geo: &TermGeometry) -> (u16, u16) {
    let bound_w = (geo.cols / 3).clamp(1, max.0.max(1));
    let bound_h = geo.rows.saturating_sub(4).clamp(1, max.1.max(1));
    fit_image_cells(dims.0, dims.1, geo.cell, bound_w, bound_h)
}

/// One drawn cell: `ch` painted in `fg` over `bg`; `bg: None` keeps the terminal's own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Block {
    pub ch: char,
    pub fg: (u8, u8, u8),
    pub bg: Option<(u8, u8, u8)>,
}

/// Quadrant glyph per foreground mask: bit 0 top-left, 1 top-right, 2 bottom-left,
/// 3 bottom-right.
const QUADRANTS: [char; 16] = [
    ' ', '▘', '▝', '▀', '▖', '▌', '▞', '▛', '▗', '▚', '▐', '▜', '▄', '▙', '▟', '█',
];

/// First code point of the TermFetch Blocks font (`assets/fonts/TermFetchBlocks.ttf`):
/// octant pattern `m` is drawn by U+100000 + m.
pub const OCTANT_BASE: u32 = 0x10_0000;

/// How finely a cell is divided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Shape {
    /// 2 x 2 per cell with the standard quadrant characters; works with any font.
    Quadrant,
    /// 2 x 4 per cell with the TermFetch Blocks font.
    Octant,
}

impl Shape {
    /// Sub-pixels per cell (columns, rows).
    pub fn grid(self) -> (u16, u16) {
        match self {
            Self::Quadrant => (2, 2),
            Self::Octant => (2, 4),
        }
    }

    /// Glyph whose foreground covers `mask` (bit `row * 2 + col`).
    pub fn glyph(self, mask: u8) -> char {
        match self {
            Self::Quadrant => QUADRANTS[usize::from(mask & 15)],
            Self::Octant => char::from_u32(OCTANT_BASE + u32::from(mask)).unwrap_or(' '),
        }
    }

    /// Octants where the TermFetch Blocks font is installed and the terminal has no
    /// image protocol of its own (Terminal.app); quadrants elsewhere.
    pub fn detect() -> Self {
        static SHAPE: std::sync::OnceLock<Shape> = std::sync::OnceLock::new();
        *SHAPE.get_or_init(|| {
            let apple = std::env::var("TERM_PROGRAM").is_ok_and(|p| p == "Apple_Terminal");
            let font = std::env::var_os("HOME").is_some_and(|h| {
                Path::new(&h)
                    .join("Library/Fonts/TermFetchBlocks.ttf")
                    .is_file()
            });
            if apple && font {
                Self::Octant
            } else {
                Self::Quadrant
            }
        })
    }
}

/// An image rasterised for block-character rendering: `w` x `h` cells, each sampled as
/// a small grid of RGBA pixels (see [`Shape`]) and drawn with the glyph and two colours
/// that fit it best.
#[derive(Debug, Clone)]
pub struct BlockGrid {
    pub w: u16,
    pub h: u16,
    pub shape: Shape,
    /// `w * cols` x `h * rows` pixels, row-major.
    pub px: Vec<[u8; 4]>,
}

/// Resize with premultiplied alpha, so transparent pixels do not darken soft edges.
fn resize_premultiplied(img: &RgbaImage, w: u32, h: u32) -> RgbaImage {
    let mut pre = img.clone();
    for p in pre.pixels_mut() {
        let a = u16::from(p.0[3]);
        for c in &mut p.0[..3] {
            *c = ((u16::from(*c) * a + 127) / 255) as u8;
        }
    }
    let mut out = image::imageops::resize(&pre, w, h, image::imageops::FilterType::CatmullRom);
    for p in out.pixels_mut() {
        let a = u16::from(p.0[3]);
        for c in &mut p.0[..3] {
            if let Some(v) = (u16::from(*c) * 255 + a / 2).checked_div(a) {
                *c = v.min(255) as u8;
            }
        }
    }
    out
}

/// Per-mask colour statistics of up to eight pixels, so every pattern is scored in one
/// pass: the squared error of a group is `sum(c^2) - sum(c)^2 / n` per channel.
struct Stats {
    px: [(u8, u8, u8); 8],
    len: usize,
}

impl Stats {
    fn new(px: impl Iterator<Item = (u8, u8, u8)>) -> Self {
        let mut s = Self {
            px: [(0, 0, 0); 8],
            len: 0,
        };
        for p in px.take(8) {
            s.px[s.len] = p;
            s.len += 1;
        }
        s
    }

    /// Split into two colour groups (2-means from the two most different pixels); the
    /// mask of the first group, or `all` for a flat cell.
    fn two_means(&self, all: u8) -> u8 {
        let px = &self.px[..self.len];
        let d = |a: (u8, u8, u8), b: (u8, u8, u8)| {
            let c = |x: u8, y: u8| (i32::from(x) - i32::from(y)).pow(2) as u32;
            c(a.0, b.0) + c(a.1, b.1) + c(a.2, b.2)
        };
        let (mut fg, mut bg) = (0..px.len())
            .flat_map(|i| (i + 1..px.len()).map(move |j| (i, j)))
            .max_by_key(|&(i, j)| d(px[i], px[j]))
            .map_or((px[0], px[0]), |(i, j)| (px[i], px[j]));
        let mut mask = all;
        for _ in 0..2 {
            mask = px
                .iter()
                .enumerate()
                .filter(|(_, p)| d(**p, fg) <= d(**p, bg))
                .fold(0, |m, (i, _)| m | 1 << i);
            if mask == all || mask == 0 {
                return all;
            }
            (fg, bg) = (self.group(mask, true).0, self.group(mask, false).0);
        }
        mask
    }

    /// Mean colour and squared error of the pixels in (`on`) or out of `mask`.
    fn group(&self, mask: u8, on: bool) -> ((u8, u8, u8), u64) {
        let (mut n, mut sum, mut sq) = (0u64, [0u64; 3], 0u64);
        for (i, &(r, g, b)) in self.px[..self.len].iter().enumerate() {
            if (mask >> i & 1 == 1) == on {
                n += 1;
                for (k, c) in [r, g, b].into_iter().enumerate() {
                    let c = u64::from(c);
                    sum[k] += c;
                    sq += c * c;
                }
            }
        }
        if n == 0 {
            return ((0, 0, 0), 0);
        }
        let err = sq - sum.iter().map(|s| s * s / n).sum::<u64>();
        let mean = sum.map(|s| (s / n) as u8);
        ((mean[0], mean[1], mean[2]), err)
    }
}

impl BlockGrid {
    pub fn from_image(img: &RgbaImage, w: u16, h: u16, shape: Shape) -> Self {
        let (w, h) = (w.max(1), h.max(1));
        let (sx, sy) = shape.grid();
        let scaled = resize_premultiplied(img, u32::from(w * sx), u32::from(h * sy));
        let px = scaled.pixels().map(|p| p.0).collect();
        Self { w, h, shape, px }
    }

    /// The pixels of a cell, row by row.
    pub fn pixels(&self, x: u16, y: u16) -> Vec<[u8; 4]> {
        let (sx, sy) = self.shape.grid();
        let w = usize::from(self.w * sx);
        let (x, y) = (usize::from(x * sx), usize::from(y * sy));
        (0..usize::from(sy))
            .flat_map(|dy| (0..usize::from(sx)).map(move |dx| (dx, dy)))
            .map(|(dx, dy)| self.px[(y + dy) * w + x + dx])
            .collect()
    }

    /// The best block for a cell: the glyph and two colours closest to its pixels. With
    /// `bg` the pixels are blended onto it; without, transparent pixels keep the
    /// terminal's background (one colour on the opaque part) and a fully transparent cell
    /// is `None`.
    pub fn cell(&self, x: u16, y: u16, bg: Option<(u8, u8, u8)>) -> Option<Block> {
        let px = self.pixels(x, y);
        let all = ((1u16 << px.len()) - 1) as u8;
        let opaque = px.iter().all(|p| p[3] >= 128);
        let stats = match bg {
            Some(bg) => Some(Stats::new(px.iter().map(|&p| blend(p, bg)))),
            // Without a known background only fully opaque cells get two colours.
            None if opaque => Some(Stats::new(px.iter().map(|p| (p[0], p[1], p[2])))),
            None => None,
        };
        if let Some(stats) = stats {
            let mask = if stats.len <= 4 {
                // Full block first, so a flat cell stays one solid glyph on ties.
                (1..=all)
                    .rev()
                    .map(|mask| (stats.group(mask, true).1 + stats.group(mask, false).1, mask))
                    .min_by_key(|t| t.0)?
                    .1
            } else {
                stats.two_means(all)
            };
            let fg = stats.group(mask, true).0;
            let back = if mask == all {
                fg
            } else {
                stats.group(mask, false).0
            };
            return Some(Block {
                ch: self.shape.glyph(mask),
                fg,
                bg: Some(back),
            });
        }
        let mask = px
            .iter()
            .enumerate()
            .filter(|(_, p)| p[3] >= 128)
            .fold(0u8, |m, (i, _)| m | 1 << i);
        if mask == 0 {
            return None;
        }
        let stats = Stats::new(px.iter().map(|p| (p[0], p[1], p[2])));
        Some(Block {
            ch: self.shape.glyph(mask),
            fg: stats.group(mask, true).0,
            bg: None,
        })
    }
}

/// Alpha-blend `px` over an opaque background.
pub fn blend(px: [u8; 4], bg: (u8, u8, u8)) -> (u8, u8, u8) {
    let a = u16::from(px[3]);
    let mix = |c: u8, b: u8| ((u16::from(c) * a + u16::from(b) * (255 - a) + 127) / 255) as u8;
    (mix(px[0], bg.0), mix(px[1], bg.1), mix(px[2], bg.2))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CELL: CellSize = CellSize { w: 10, h: 20 };

    fn ratio_error(pw: u32, ph: u32, cell: CellSize, (w, h): (u16, u16)) -> f64 {
        let img = f64::from(pw) / f64::from(ph);
        let shown = (f64::from(w) * f64::from(cell.w)) / (f64::from(h) * f64::from(cell.h));
        (shown / img - 1.0).abs()
    }

    #[test]
    fn square_image_on_1x2_cells_is_twice_as_wide() {
        let (w, h) = fit_image_cells(512, 512, CELL, 36, 22);
        assert_eq!(w, 2 * h);
        assert!(w <= 36 && h <= 22);
    }

    #[test]
    fn respects_bounds_and_ratio_for_many_shapes() {
        for &(pw, ph) in &[
            (512, 512),
            (1920, 1080),
            (300, 900),
            (1000, 333),
            (17, 61),
            (4000, 30),
        ] {
            for &(mw, mh) in &[(20u16, 14u16), (36, 22), (52, 32), (8, 30)] {
                let fit = fit_image_cells(pw, ph, CELL, mw, mh);
                assert!(
                    fit.0 >= 1 && fit.0 <= mw && fit.1 >= 1 && fit.1 <= mh,
                    "{pw}x{ph} {fit:?}"
                );
                // Extreme ratios that can't fit in whole cells are allowed more slack.
                if fit.0 > 2 && fit.1 > 2 {
                    assert!(
                        ratio_error(pw, ph, CELL, fit) < 0.12,
                        "{pw}x{ph} in {mw}x{mh} -> {fit:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn uses_real_cell_size() {
        // Square cells: a square image becomes a square cell box.
        let (w, h) = fit_image_cells(400, 400, CellSize { w: 16, h: 16 }, 30, 30);
        assert_eq!(w, h);
    }

    #[test]
    fn config_bounds_follow_terminal() {
        let geo = TermGeometry::from_window(60, 20, 600, 400);
        let (w, h) = config_logo_cells((512, 512), (36, 22), &geo);
        assert!(w <= 20 && h <= 16);
        assert_eq!(geo.cell, CELL);
    }

    #[test]
    fn block_grid_dimensions() {
        let img = RgbaImage::from_pixel(64, 64, image::Rgba([255, 0, 0, 255]));
        let g = BlockGrid::from_image(&img, 12, 6, Shape::Quadrant);
        assert_eq!(g.px.len(), 24 * 12);
        let o = BlockGrid::from_image(&img, 12, 6, Shape::Octant);
        assert_eq!(o.px.len(), 24 * 24);
        let cell = g.cell(11, 5, Some((0, 0, 0))).unwrap();
        assert_eq!((cell.fg, cell.bg), ((255, 0, 0), Some((255, 0, 0))));
        assert_eq!(blend([0, 0, 0, 0], (1, 2, 3)), (1, 2, 3));
    }

    fn grid(px: [[u8; 4]; 4]) -> BlockGrid {
        BlockGrid {
            w: 1,
            h: 1,
            shape: Shape::Quadrant,
            px: px.to_vec(),
        }
    }

    #[test]
    fn octants_resolve_quarter_height_detail() {
        let (red, blue) = ([255, 0, 0, 255], [0, 0, 255, 255]);
        // Top quarter red, the rest blue: only octants can draw this exactly.
        let g = BlockGrid {
            w: 1,
            h: 1,
            shape: Shape::Octant,
            px: vec![red, red, blue, blue, blue, blue, blue, blue],
        };
        let cell = g.cell(0, 0, Some((0, 0, 0))).unwrap();
        let fits =
            |m: u32, f, b| cell.ch as u32 == OCTANT_BASE + m && cell.fg == f && cell.bg == Some(b);
        assert!(
            fits(0b11, (255, 0, 0), (0, 0, 255)) || fits(0b1111_1100, (0, 0, 255), (255, 0, 0))
        );
        assert_eq!(Shape::Quadrant.glyph(0b1001), '▚');
    }

    #[test]
    fn quadrants_follow_the_shape_inside_a_cell() {
        let (red, blue) = ([255, 0, 0, 255], [0, 0, 255, 255]);
        // Left column red, right column blue: a vertical split, not a top/bottom one.
        let cell = grid([red, blue, red, blue])
            .cell(0, 0, Some((0, 0, 0)))
            .unwrap();
        assert!(
            (cell.ch, cell.fg, cell.bg) == ('▌', (255, 0, 0), Some((0, 0, 255)))
                || (cell.ch, cell.fg, cell.bg) == ('▐', (0, 0, 255), Some((255, 0, 0)))
        );
        let diagonal = grid([red, blue, blue, red])
            .cell(0, 0, Some((0, 0, 0)))
            .unwrap();
        assert!(matches!(diagonal.ch, '▚' | '▞'));
    }

    #[test]
    fn transparent_pixels_keep_the_terminal_background() {
        let (red, clear) = ([255, 0, 0, 255], [0, 0, 0, 0]);
        let cell = grid([red, clear, clear, clear]).cell(0, 0, None).unwrap();
        assert_eq!((cell.ch, cell.fg, cell.bg), ('▘', (255, 0, 0), None));
        assert_eq!(grid([clear; 4]).cell(0, 0, None), None);
        // Opaque cells keep their detail: two colours even without a background.
        let blue = [0, 0, 255, 255];
        let split = grid([red, red, blue, blue]).cell(0, 0, None).unwrap();
        assert!(matches!(split.ch, '▀' | '▄') && split.bg.is_some());
    }

    #[test]
    fn resizing_does_not_darken_soft_edges() {
        let mut img = RgbaImage::from_pixel(8, 1, image::Rgba([0, 0, 0, 0]));
        for x in 0..4 {
            img.put_pixel(x, 0, image::Rgba([255, 255, 0, 255]));
        }
        let out = resize_premultiplied(&img, 3, 1);
        let edge = out.get_pixel(1, 0).0;
        assert!(edge[3] > 0 && edge[0] > 200 && edge[1] > 200, "{edge:?}");
    }
}
