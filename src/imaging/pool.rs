//! Background decode pool. Images are decoded and downscaled once on worker
//! threads; the UI thread only ever reads finished thumbnails and small cached
//! half-block grids, so scrolling never blocks on disk or PNG decoding.

use super::HalfblockGrid;
use image::RgbaImage;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

/// Longest edge kept in memory; larger than any realistic preview box.
const THUMB_EDGE: u32 = 384;
const GRID_CACHE_LIMIT: usize = 256;

#[derive(Debug)]
pub struct DecodedImage {
    pub thumb: RgbaImage,
    /// Original pixel size, used for aspect-ratio fitting.
    pub dims: (u32, u32),
}

type DecodeResult = (String, Result<Arc<DecodedImage>, String>);

struct Queue {
    jobs: Mutex<VecDeque<String>>,
    ready: Condvar,
}

pub struct ImagePool {
    queue: Arc<Queue>,
    results: Receiver<DecodeResult>,
    decoded: HashMap<String, Arc<DecodedImage>>,
    failed: HashMap<String, String>,
    pending: HashSet<String>,
    grids: HashMap<(String, u16, u16), Arc<HalfblockGrid>>,
}

pub fn decode(path: &str) -> Result<DecodedImage, String> {
    let img = image::ImageReader::open(PathBuf::from(path))
        .map_err(|e| e.to_string())?
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;
    let dims = (img.width(), img.height());
    let thumb = if dims.0.max(dims.1) > THUMB_EDGE {
        img.thumbnail(THUMB_EDGE, THUMB_EDGE).to_rgba8()
    } else {
        img.to_rgba8()
    };
    Ok(DecodedImage { thumb, dims })
}

impl ImagePool {
    pub fn new(workers: usize) -> Self {
        let queue = Arc::new(Queue {
            jobs: Mutex::new(VecDeque::new()),
            ready: Condvar::new(),
        });
        let (tx, rx) = mpsc::channel::<DecodeResult>();
        for i in 0..workers.max(1) {
            let queue = Arc::clone(&queue);
            let tx: Sender<DecodeResult> = tx.clone();
            let _ = thread::Builder::new()
                .name(format!("tfs-decode-{i}"))
                .spawn(move || worker(queue, tx));
        }
        Self {
            queue,
            results: rx,
            decoded: HashMap::new(),
            failed: HashMap::new(),
            pending: HashSet::new(),
            grids: HashMap::new(),
        }
    }

    pub fn with_default_workers() -> Self {
        let n = thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(2);
        Self::new(n.clamp(2, 6))
    }

    fn known(&self, path: &str) -> bool {
        self.decoded.contains_key(path)
            || self.failed.contains_key(path)
            || self.pending.contains(path)
    }

    /// Queue `path`; `urgent` jumps the queue (the currently selected logo).
    pub fn request(&mut self, path: &str, urgent: bool) {
        let mut jobs = self.queue.jobs.lock().unwrap_or_else(|e| e.into_inner());
        if self.known(path) {
            if urgent && let Some(pos) = jobs.iter().position(|p| p == path) {
                let job = jobs.remove(pos).expect("position is valid");
                jobs.push_front(job);
            }
            return;
        }
        self.pending.insert(path.to_string());
        if urgent {
            jobs.push_front(path.to_string());
        } else {
            jobs.push_back(path.to_string());
        }
        self.queue.ready.notify_one();
    }

    /// Pre-decode a whole catalogue in the background.
    pub fn prefetch<'a>(&mut self, paths: impl IntoIterator<Item = &'a str>) {
        for p in paths {
            self.request(p, false);
        }
    }

    /// Collect finished decodes. Returns the paths that completed.
    pub fn poll(&mut self) -> Vec<String> {
        let mut done = Vec::new();
        while let Ok((path, res)) = self.results.try_recv() {
            self.pending.remove(&path);
            match res {
                Ok(img) => {
                    self.decoded.insert(path.clone(), img);
                }
                Err(e) => {
                    self.failed.insert(path.clone(), e);
                }
            }
            done.push(path);
        }
        done
    }

    /// Synchronous decode for one-shot CLI rendering.
    pub fn load_blocking(&mut self, path: &str) -> Option<Arc<DecodedImage>> {
        if let Some(img) = self.decoded.get(path) {
            return Some(Arc::clone(img));
        }
        match decode(path) {
            Ok(img) => {
                let img = Arc::new(img);
                self.decoded.insert(path.to_string(), Arc::clone(&img));
                Some(img)
            }
            Err(e) => {
                self.failed.insert(path.to_string(), e);
                None
            }
        }
    }

    pub fn get(&self, path: &str) -> Option<&Arc<DecodedImage>> {
        self.decoded.get(path)
    }

    pub fn error(&self, path: &str) -> Option<&str> {
        self.failed.get(path).map(String::as_str)
    }

    pub fn is_pending(&self, path: &str) -> bool {
        self.pending.contains(path)
    }

    /// Forget a path so it is decoded again (e.g. custom.png was replaced).
    pub fn invalidate(&mut self, path: &str) {
        self.decoded.remove(path);
        self.failed.remove(path);
        self.grids.retain(|(p, _, _), _| p != path);
    }

    /// Half-block raster at exactly `w` x `h` cells, cached per size.
    pub fn grid(&mut self, path: &str, w: u16, h: u16) -> Option<Arc<HalfblockGrid>> {
        let key = (path.to_string(), w, h);
        if let Some(g) = self.grids.get(&key) {
            return Some(Arc::clone(g));
        }
        let img = self.decoded.get(path)?;
        let grid = Arc::new(HalfblockGrid::from_image(&img.thumb, w, h));
        if self.grids.len() >= GRID_CACHE_LIMIT {
            self.grids.clear();
        }
        self.grids.insert(key, Arc::clone(&grid));
        Some(grid)
    }

    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    pub fn decoded_count(&self) -> usize {
        self.decoded.len()
    }
}

fn worker(queue: Arc<Queue>, tx: Sender<DecodeResult>) {
    loop {
        let job = {
            let mut jobs = queue.jobs.lock().unwrap_or_else(|e| e.into_inner());
            loop {
                if let Some(job) = jobs.pop_front() {
                    break job;
                }
                jobs = queue.ready.wait(jobs).unwrap_or_else(|e| e.into_inner());
            }
        };
        let res = decode(&job).map(Arc::new);
        if tx.send((job, res)).is_err() {
            return; // pool dropped
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn decodes_in_background_and_caches_grids() {
        let dir = std::env::temp_dir().join(format!("tfs-pool-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("img.png");
        RgbaImage::from_pixel(800, 400, image::Rgba([0, 255, 0, 255]))
            .save(&path)
            .unwrap();
        let p = path.to_string_lossy().into_owned();

        let mut pool = ImagePool::new(2);
        pool.request(&p, true);
        pool.request("/definitely/missing.png", false);
        let start = Instant::now();
        while (pool.get(&p).is_none() || pool.error("/definitely/missing.png").is_none())
            && start.elapsed() < Duration::from_secs(10)
        {
            pool.poll();
            thread::sleep(Duration::from_millis(5));
        }
        let img = pool.get(&p).expect("decoded");
        assert_eq!(img.dims, (800, 400));
        assert!(img.thumb.width() <= THUMB_EDGE);
        assert!(pool.error("/definitely/missing.png").is_some());

        let a = pool.grid(&p, 20, 5).unwrap();
        let b = pool.grid(&p, 20, 5).unwrap();
        assert!(Arc::ptr_eq(&a, &b));
        let _ = std::fs::remove_dir_all(dir);
    }
}
