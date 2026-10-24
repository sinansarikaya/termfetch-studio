//! Kitty graphics protocol via Unicode placeholders.
//!
//! Each logo is decoded from the original file, resized to the exact pixel box
//! of its preview cells and PNG-encoded on a background thread, then uploaded
//! once with a virtual placement (`U=1`). The preview only has to paint
//! `U+10EEEE` placeholder cells whose foreground colour carries the image id,
//! so ratatui's normal cell diffing positions, clips and clears the image and
//! browsing a previously shown logo costs no upload at all.

use super::kitty_diacritics::DIACRITICS;
use base64::Engine;
use image::ImageEncoder;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use ratatui::style::Color;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

pub const GRAPHICS_ENV: &str = "TERMFETCH_STUDIO_GRAPHICS";
pub const PLACEHOLDER: char = '\u{10EEEE}';
const CHUNK: usize = 4096;
/// Uploaded images kept in the terminal before the oldest ones are freed.
const UPLOAD_LIMIT: usize = 48;

/// Whether the host terminal renders Kitty Unicode placeholders.
///
/// `TERMFETCH_STUDIO_GRAPHICS=kitty|halfblock` overrides detection. Multiplexers
/// are excluded because they need explicit passthrough.
pub fn detect() -> bool {
    let var = |k: &str| std::env::var(k).unwrap_or_default();
    match var(GRAPHICS_ENV).to_ascii_lowercase().as_str() {
        "kitty" => return true,
        "halfblock" | "blocks" | "off" | "none" => return false,
        _ => {}
    }
    if std::env::var_os("TMUX").is_some()
        || std::env::var_os("STY").is_some()
        || var("TERM").starts_with("screen")
    {
        return false;
    }
    let term = var("TERM");
    std::env::var_os("KITTY_WINDOW_ID").is_some()
        || term.contains("kitty")
        || term == "xterm-ghostty"
        || var("TERM_PROGRAM").eq_ignore_ascii_case("ghostty")
}

/// Placeholder grapheme for the image cell at (`row`, `col`).
pub fn placeholder(row: u16, col: u16) -> String {
    let d = |i: u16| DIACRITICS[usize::from(i).min(DIACRITICS.len() - 1)];
    [PLACEHOLDER, d(row), d(col)].iter().collect()
}

/// Foreground colour that encodes a 24-bit image id.
pub fn id_color(id: u32) -> Color {
    Color::Rgb((id >> 16) as u8, (id >> 8) as u8, id as u8)
}

/// Transmit PNG data and create a virtual placement of `cols` x `rows` cells.
pub fn transmit_sequence(id: u32, png: &[u8], cols: u16, rows: u16) -> String {
    let data = base64::engine::general_purpose::STANDARD.encode(png);
    let chunks: Vec<&str> = data
        .as_bytes()
        .chunks(CHUNK)
        .map(|c| std::str::from_utf8(c).expect("base64 is ASCII"))
        .collect();
    let mut out = String::with_capacity(data.len() + chunks.len() * 16 + 64);
    for (i, chunk) in chunks.iter().enumerate() {
        let more = u8::from(i + 1 < chunks.len());
        if i == 0 {
            out.push_str(&format!(
                "\x1b_Ga=T,U=1,f=100,t=d,i={id},c={cols},r={rows},q=2,m={more};{chunk}\x1b\\"
            ));
        } else {
            out.push_str(&format!("\x1b_Gm={more};{chunk}\x1b\\"));
        }
    }
    out
}

/// Delete an image and free its data.
pub fn delete_sequence(id: u32) -> String {
    format!("\x1b_Ga=d,d=I,i={id},q=2\x1b\\")
}

/// Decode `path` at full resolution and encode it as a PNG of exactly `px` pixels.
pub fn encode_png(path: &str, px: (u32, u32)) -> Result<Vec<u8>, String> {
    let img = image::ImageReader::open(PathBuf::from(path))
        .map_err(|e| e.to_string())?
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;
    let (w, h) = (px.0.max(1), px.1.max(1));
    let scaled = img
        .resize_exact(w, h, image::imageops::FilterType::Lanczos3)
        .to_rgba8();
    let mut png = Vec::new();
    PngEncoder::new_with_quality(&mut png, CompressionType::Fast, FilterType::Adaptive)
        .write_image(scaled.as_raw(), w, h, image::ExtendedColorType::Rgba8)
        .map_err(|e| e.to_string())?;
    Ok(png)
}

/// One uploaded rendition: a file at a cell box and pixel size.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImageKey {
    pub path: String,
    pub cells: (u16, u16),
    pub px: (u32, u32),
}

type EncodeResult = (ImageKey, Result<Vec<u8>, String>);

struct Queue {
    jobs: Mutex<VecDeque<ImageKey>>,
    ready: Condvar,
}

/// Background encoder plus the bookkeeping of what the terminal already holds.
pub struct KittyImages {
    queue: Arc<Queue>,
    results: Receiver<EncodeResult>,
    pending: HashSet<ImageKey>,
    failed: HashSet<ImageKey>,
    uploaded: HashMap<ImageKey, u32>,
    order: VecDeque<ImageKey>,
    next_id: u32,
}

impl KittyImages {
    pub fn new(workers: usize) -> Self {
        let queue = Arc::new(Queue {
            jobs: Mutex::new(VecDeque::new()),
            ready: Condvar::new(),
        });
        let (tx, rx) = mpsc::channel::<EncodeResult>();
        for i in 0..workers.max(1) {
            let queue = Arc::clone(&queue);
            let tx: Sender<EncodeResult> = tx.clone();
            let _ = thread::Builder::new()
                .name(format!("tfs-kitty-{i}"))
                .spawn(move || worker(queue, tx));
        }
        // Ids are 24-bit (they travel as an RGB colour); seed from the pid so
        // two instances in one kitty window do not collide.
        let next_id = ((std::process::id() & 0x7FF) << 12) | 1;
        Self {
            queue,
            results: rx,
            pending: HashSet::new(),
            failed: HashSet::new(),
            uploaded: HashMap::new(),
            order: VecDeque::new(),
            next_id,
        }
    }

    /// Image id if `key` is already in the terminal.
    pub fn id(&self, key: &ImageKey) -> Option<u32> {
        self.uploaded.get(key).copied()
    }

    /// Queue an encode; `urgent` (the visible logo) jumps ahead of pre-uploads.
    pub fn request(&mut self, key: &ImageKey, urgent: bool) {
        let mut jobs = self.queue.jobs.lock().unwrap_or_else(|e| e.into_inner());
        if self.uploaded.contains_key(key) || self.failed.contains(key) {
            return;
        }
        if self.pending.contains(key) {
            if urgent && let Some(pos) = jobs.iter().position(|k| k == key) {
                let job = jobs.remove(pos).expect("position is valid");
                jobs.push_front(job);
            }
            return;
        }
        self.pending.insert(key.clone());
        if urgent {
            jobs.push_front(key.clone());
        } else {
            jobs.push_back(key.clone());
        }
        self.queue.ready.notify_one();
    }

    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Collect finished encodes as escape sequences to write to the terminal.
    /// Keys become usable as soon as they are returned here.
    pub fn poll(&mut self) -> Vec<String> {
        let mut out = Vec::new();
        while let Ok((key, res)) = self.results.try_recv() {
            self.pending.remove(&key);
            match res {
                Ok(png) => {
                    while self.order.len() >= UPLOAD_LIMIT {
                        if let Some(old) = self.order.pop_front()
                            && let Some(id) = self.uploaded.remove(&old)
                        {
                            out.push(delete_sequence(id));
                        }
                    }
                    let id = self.alloc_id();
                    out.push(transmit_sequence(id, &png, key.cells.0, key.cells.1));
                    self.order.push_back(key.clone());
                    self.uploaded.insert(key, id);
                }
                Err(_) => {
                    self.failed.insert(key);
                }
            }
        }
        out
    }

    fn alloc_id(&mut self) -> u32 {
        let id = self.next_id & 0xFF_FFFF;
        self.next_id = self.next_id.wrapping_add(1);
        if id == 0 { self.alloc_id() } else { id }
    }

    /// Forget every rendition of `path` (the file was replaced on disk).
    pub fn invalidate(&mut self, path: &str) -> Vec<String> {
        self.failed.retain(|k| k.path != path);
        let stale: Vec<ImageKey> = self
            .uploaded
            .keys()
            .filter(|k| k.path == path)
            .cloned()
            .collect();
        self.order.retain(|k| k.path != path);
        stale
            .iter()
            .filter_map(|k| self.uploaded.remove(k))
            .map(delete_sequence)
            .collect()
    }

    /// Sequences that free everything this instance uploaded.
    pub fn cleanup(&mut self) -> String {
        self.order.clear();
        self.uploaded
            .drain()
            .map(|(_, id)| delete_sequence(id))
            .collect()
    }
}

fn worker(queue: Arc<Queue>, tx: Sender<EncodeResult>) {
    loop {
        let key = {
            let mut jobs = queue.jobs.lock().unwrap_or_else(|e| e.into_inner());
            loop {
                if let Some(job) = jobs.pop_front() {
                    break job;
                }
                jobs = queue.ready.wait(jobs).unwrap_or_else(|e| e.into_inner());
            }
        };
        let res = encode_png(&key.path, key.px);
        if tx.send((key, res)).is_err() {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    use unicode_width::UnicodeWidthStr;

    #[test]
    fn placeholder_is_one_cell_wide() {
        let p = placeholder(3, 40);
        assert_eq!(p.chars().count(), 3);
        assert!(p.starts_with(PLACEHOLDER));
        assert_eq!(p.width(), 1);
        assert_eq!(placeholder(0, 0).chars().nth(1), Some('\u{0305}'));
    }

    #[test]
    fn id_roundtrips_through_rgb() {
        assert_eq!(id_color(0x12_34_56), Color::Rgb(0x12, 0x34, 0x56));
    }

    #[test]
    fn transmit_is_chunked() {
        let seq = transmit_sequence(7, &vec![0u8; 10_000], 10, 5);
        assert!(seq.starts_with("\x1b_Ga=T,U=1,f=100,t=d,i=7,c=10,r=5,q=2,m=1;"));
        assert!(seq.ends_with("\x1b\\"));
        assert_eq!(
            seq.matches("\x1b_G").count(),
            (10_000usize.div_ceil(3) * 4).div_ceil(CHUNK)
        );
        assert!(seq.contains("\x1b_Gm=0;"));
    }

    #[test]
    fn encodes_in_background_and_uploads_once() {
        let dir = std::env::temp_dir().join(format!("tfs-kitty-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("img.png");
        image::RgbaImage::from_pixel(300, 200, image::Rgba([0, 0, 255, 255]))
            .save(&path)
            .unwrap();
        let key = ImageKey {
            path: path.to_string_lossy().into_owned(),
            cells: (12, 4),
            px: (120, 80),
        };

        let mut k = KittyImages::new(1);
        k.request(&key, true);
        k.request(&key, true);
        let start = Instant::now();
        let mut seqs = Vec::new();
        while k.id(&key).is_none() && start.elapsed() < Duration::from_secs(10) {
            seqs.extend(k.poll());
            thread::sleep(Duration::from_millis(5));
        }
        let id = k.id(&key).expect("uploaded");
        assert_eq!(seqs.len(), 1);
        assert!(seqs[0].contains(&format!("i={id},c=12,r=4")));
        k.request(&key, true);
        assert!(!k.has_pending());
        assert_eq!(k.cleanup(), delete_sequence(id));
        let _ = std::fs::remove_dir_all(dir);
    }
}
