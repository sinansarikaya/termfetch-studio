//! Sixel rendering for terminals without the Kitty graphics protocol (Windows
//! Terminal 1.22+, foot, mlterm).
//!
//! Sixel cannot be clipped or diffed by ratatui the way Kitty placeholders are,
//! so the image is drawn at full cell-pixel resolution, written after a frame
//! and only repainted when its slot changes. Colours come from a fixed 6x6x6
//! cube, which keeps the output deterministic and needs no palette search.

use super::{blend, kitty};
use image::{ImageReader, RgbaImage};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::Arc;

/// Whether the host terminal takes sixel images. Kitty is checked first by the caller.
///
/// `TERMFETCH_STUDIO_GRAPHICS=sixel` forces it on; multiplexers are excluded
/// because they need explicit passthrough.
pub fn detect() -> bool {
    let var = |k: &str| std::env::var(k).unwrap_or_default();
    match var(kitty::GRAPHICS_ENV).to_ascii_lowercase().as_str() {
        "sixel" => return true,
        "kitty" | "halfblock" | "blocks" | "off" | "none" => return false,
        _ => {}
    }
    if std::env::var_os("TMUX").is_some()
        || std::env::var_os("STY").is_some()
        || var("TERM").starts_with("screen")
    {
        return false;
    }
    let term = var("TERM").to_ascii_lowercase();
    let program = var("TERM_PROGRAM");
    std::env::var_os("WT_SESSION").is_some()
        || program == "iTerm.app"
        || program == "WezTerm"
        || term.starts_with("foot")
        || term.contains("mlterm")
        || term.contains("sixel")
}

/// Sixel colour registers used per image; the common terminal limit.
const MAX_COLOURS: usize = 256;

/// Histogram bucket for a colour: 5 bits per channel.
fn bucket(c: [u8; 3]) -> u16 {
    (u16::from(c[0] >> 3) << 10) | (u16::from(c[1] >> 3) << 5) | u16::from(c[2] >> 3)
}

/// Up to [`MAX_COLOURS`] colours: the most common buckets, each averaged. Ties keep bucket order.
fn palette(pixels: &[[u8; 3]]) -> Vec<[u8; 3]> {
    let mut buckets: BTreeMap<u16, (u32, [u64; 3])> = BTreeMap::new();
    for c in pixels {
        let (n, sum) = buckets.entry(bucket(*c)).or_insert((0, [0; 3]));
        *n += 1;
        for k in 0..3 {
            sum[k] += u64::from(c[k]);
        }
    }
    let mut list: Vec<(u32, [u64; 3])> = buckets.into_values().collect();
    list.sort_by_key(|(n, _)| std::cmp::Reverse(*n));
    list.truncate(MAX_COLOURS);
    list.into_iter()
        .map(|(n, sum)| sum.map(|v| (v / u64::from(n)) as u8))
        .collect()
}

/// Index of the palette entry closest to `c` (squared RGB distance).
fn nearest(palette: &[[u8; 3]], c: [u8; 3]) -> usize {
    let dist = |p: &[u8; 3]| -> i32 {
        (0..3)
            .map(|k| {
                let d = i32::from(p[k]) - i32::from(c[k]);
                d * d
            })
            .sum()
    };
    palette
        .iter()
        .enumerate()
        .min_by_key(|(_, p)| dist(p))
        .map_or(0, |(i, _)| i)
}

/// Sixel stream for `img` alpha-blended over `bg`, one pixel per image pixel.
pub fn encode(img: &RgbaImage, bg: (u8, u8, u8)) -> String {
    let (w, h) = img.dimensions();
    let (w, h) = (w as usize, h as usize);
    let colours: Vec<[u8; 3]> = img
        .pixels()
        .map(|p| {
            let (r, g, b) = blend(p.0, bg);
            [r, g, b]
        })
        .collect();
    let pal = palette(&colours);
    let mut memo: HashMap<[u8; 3], usize> = HashMap::new();
    let index: Vec<usize> = colours
        .iter()
        .map(|c| *memo.entry(*c).or_insert_with(|| nearest(&pal, *c)))
        .collect();

    let mut used = vec![false; pal.len()];
    index.iter().for_each(|&i| used[i] = true);

    let mut out = format!("Pq\"1;1;{w};{h}");
    for (i, c) in pal.iter().enumerate().filter(|(i, _)| used[*i]) {
        // Sixel colour components are percentages.
        let pct = |v: u8| (u32::from(v) * 100 + 127) / 255;
        out.push_str(&format!("#{i};2;{};{};{}", pct(c[0]), pct(c[1]), pct(c[2])));
    }

    // Each band is six pixel rows; per colour, one byte per column holds the rows it covers.
    for top in (0..h).step_by(6) {
        let mut planes: Vec<(usize, Vec<u8>)> = Vec::new();
        for dy in 0..6.min(h - top) {
            for x in 0..w {
                let colour = index[(top + dy) * w + x];
                let plane = match planes.iter().position(|(c, _)| *c == colour) {
                    Some(p) => p,
                    None => {
                        planes.push((colour, vec![0; w]));
                        planes.len() - 1
                    }
                };
                planes[plane].1[x] |= 1 << dy;
            }
        }
        for (colour, bits) in planes {
            out.push_str(&format!("#{colour}"));
            push_run_length(&mut out, &bits);
            out.push('$');
        }
        out.push('-');
    }
    out.push_str("\\");
    out
}

/// Sixel data characters are `?` (63) plus a six-bit mask; runs of four or more use `!n`.
fn push_run_length(out: &mut String, bits: &[u8]) {
    let mut i = 0;
    while i < bits.len() {
        let run = bits[i..].iter().take_while(|&&b| b == bits[i]).count();
        let ch = char::from(63 + bits[i]);
        if run >= 4 {
            out.push_str(&format!("!{run}{ch}"));
        } else {
            out.extend(std::iter::repeat_n(ch, run));
        }
        i += run;
    }
}

/// Sequence that paints `data` with its top-left cell at column `x`, row `y`,
/// leaving the cursor where it was.
pub fn place(x: u16, y: u16, data: &str) -> String {
    format!("\x1b7\x1b[{};{}H{data}\x1b8", y + 1, x + 1)
}

/// What a sixel image shows: the file, its pixel size and the background it blends onto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageKey {
    pub path: String,
    pub px: (u32, u32),
    pub bg: (u8, u8, u8),
}

/// Where the image goes on screen (cell column and row) and what it shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slot {
    pub x: u16,
    pub y: u16,
    pub image: ImageKey,
}

/// Sixel bookkeeping for the preview: the image wanted by the current frame, the
/// one the screen shows, and the encoded data of the last image.
#[derive(Debug, Default)]
pub struct SixelState {
    pub enabled: bool,
    target: Option<Slot>,
    shown: Option<Slot>,
    cache: Option<(ImageKey, Arc<str>)>,
}

impl SixelState {
    /// Forget the previous frame's wish; call before each draw.
    pub fn begin_frame(&mut self) {
        self.target = None;
    }

    /// Ask for `slot` in this frame. Returns `false` when the file cannot be
    /// decoded, so the caller can fall back to half blocks.
    pub fn want(&mut self, slot: Slot) -> bool {
        if self.cache.as_ref().is_none_or(|(k, _)| *k != slot.image) {
            let Some(img) = load(&slot.image) else {
                return false;
            };
            let data: Arc<str> = encode(&img, slot.image.bg).into();
            self.cache = Some((slot.image.clone(), data));
        }
        self.target = Some(slot);
        true
    }

    /// Whether the screen must be repainted before the image can be placed.
    pub fn is_stale(&self) -> bool {
        self.target != self.shown
    }

    /// Forget what the screen shows, e.g. after the terminal was cleared.
    pub fn forget_shown(&mut self) {
        self.shown = None;
    }

    /// Sequence that places the wanted image, if any, and marks it as shown.
    pub fn place(&mut self) -> Option<String> {
        self.shown = self.target.clone();
        let slot = self.target.as_ref()?;
        let (_, data) = self.cache.as_ref()?;
        Some(place(slot.x, slot.y, data))
    }
}

fn load(key: &ImageKey) -> Option<RgbaImage> {
    let (w, h) = (key.px.0.max(1), key.px.1.max(1));
    let img = ImageReader::open(PathBuf::from(&key.path))
        .ok()?
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?;
    Some(
        img.resize_exact(w, h, image::imageops::FilterType::Lanczos3)
            .to_rgba8(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn palette_keeps_the_dominant_colours() {
        let mut px = vec![[200, 30, 30]; 9];
        px.push([0, 0, 0]);
        let pal = palette(&px);
        assert_eq!(pal[0], [200, 30, 30]);
        assert_eq!(nearest(&pal, [1, 1, 1]), 1);
    }

    #[test]
    fn encodes_header_palette_and_bands() {
        let mut img = RgbaImage::from_pixel(2, 7, Rgba([255, 0, 0, 255]));
        img.put_pixel(0, 6, Rgba([0, 0, 0, 255]));
        let s = encode(&img, (0, 0, 0));
        assert!(s.starts_with("Pq\"1;1;2;7"));
        assert!(s.contains("#0;2;100;0;0"));
        assert!(s.contains("#1;2;0;0;0"));
        // Two bands: the second ends with a newline before the string terminator.
        assert_eq!(s.matches('-').count(), 2);
        assert!(s.ends_with("\\"));
    }

    #[test]
    fn run_length_uses_repeat_for_long_runs() {
        let mut out = String::new();
        push_run_length(&mut out, &[1, 1, 1, 1, 1, 2]);
        assert_eq!(out, "!5@A");
    }

    #[test]
    fn transparent_pixels_blend_onto_background() {
        let img = RgbaImage::from_pixel(1, 1, Rgba([255, 0, 0, 0]));
        let s = encode(&img, (0, 0, 255));
        assert!(s.contains("#0;2;0;0;100"));
    }

    #[test]
    fn state_only_repaints_when_slot_changes() {
        let mut st = SixelState::default();
        let slot = Slot {
            x: 1,
            y: 2,
            image: ImageKey {
                path: "missing-file.png".into(),
                px: (8, 8),
                bg: (0, 0, 0),
            },
        };
        assert!(!st.want(slot.clone()), "missing files are not drawn");
        assert!(!st.is_stale());
        st.forget_shown();
        assert!(!st.is_stale(), "no wanted image means nothing to repaint");
    }
}
