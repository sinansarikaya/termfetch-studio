//! Aspect-ratio-correct image sizing and half-block rasterisation.

pub mod pool;

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
        let (cw, ch) = (u32::from(px_w) / u32::from(cols), u32::from(px_h) / u32::from(rows));
        let reported = cw > 0 && ch > 0;
        Self {
            cols,
            rows,
            cell: if reported { CellSize { w: cw, h: ch } } else { CellSize::FALLBACK },
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

/// An image rasterised for `▀` half-block rendering: `w` cells wide and `h`
/// cells tall, i.e. `w` x `2h` RGBA pixels.
#[derive(Debug, Clone)]
pub struct HalfblockGrid {
    pub w: u16,
    pub h: u16,
    pub px: Vec<[u8; 4]>,
}

impl HalfblockGrid {
    pub fn from_image(img: &RgbaImage, w: u16, h: u16) -> Self {
        let (w, h) = (w.max(1), h.max(1));
        let scaled = image::imageops::resize(
            img,
            u32::from(w),
            u32::from(h) * 2,
            image::imageops::FilterType::CatmullRom,
        );
        let px = scaled.pixels().map(|p| p.0).collect();
        Self { w, h, px }
    }

    /// (top, bottom) pixel for a cell.
    pub fn cell(&self, x: u16, y: u16) -> ([u8; 4], [u8; 4]) {
        let w = usize::from(self.w);
        let (x, y) = (usize::from(x), usize::from(y));
        (self.px[2 * y * w + x], self.px[(2 * y + 1) * w + x])
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
        for &(pw, ph) in &[(512, 512), (1920, 1080), (300, 900), (1000, 333), (17, 61), (4000, 30)] {
            for &(mw, mh) in &[(20u16, 14u16), (36, 22), (52, 32), (8, 30)] {
                let fit = fit_image_cells(pw, ph, CELL, mw, mh);
                assert!(fit.0 >= 1 && fit.0 <= mw && fit.1 >= 1 && fit.1 <= mh, "{pw}x{ph} {fit:?}");
                // Extreme ratios that can't fit in whole cells are allowed more slack.
                if fit.0 > 2 && fit.1 > 2 {
                    assert!(ratio_error(pw, ph, CELL, fit) < 0.12, "{pw}x{ph} in {mw}x{mh} -> {fit:?}");
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
    fn halfblock_grid_dimensions() {
        let img = RgbaImage::from_pixel(64, 64, image::Rgba([255, 0, 0, 255]));
        let g = HalfblockGrid::from_image(&img, 12, 6);
        assert_eq!(g.px.len(), 12 * 12);
        assert_eq!(g.cell(11, 5).0, [255, 0, 0, 255]);
        assert_eq!(blend([0, 0, 0, 0], (1, 2, 3)), (1, 2, 3));
    }
}
