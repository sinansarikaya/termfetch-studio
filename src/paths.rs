//! Runtime path discovery. Nothing here is hard-coded to a user or checkout:
//! config/data/cache roots come from XDG / `directories`, and the bundled
//! assets are located relative to the executable, the data dir or the
//! working directory.

use directories::BaseDirs;
use std::path::{Path, PathBuf};

pub const APP_DIR: &str = "termfetch-studio";
pub const ASSETS_ENV: &str = "TERMFETCH_STUDIO_ASSETS";

#[derive(Debug, Clone)]
pub struct Paths {
    /// `~/.config/termfetch-studio`
    pub config_dir: PathBuf,
    /// `~/.config/fastfetch/config.jsonc`
    pub fastfetch_config: PathBuf,
    /// `~/.local/share/termfetch-studio`
    pub data_dir: PathBuf,
    /// `~/.cache/termfetch-studio`
    pub cache_dir: PathBuf,
    /// Directory containing `preset-logos/`, if one could be found.
    pub assets_dir: Option<PathBuf>,
}

impl Paths {
    pub fn detect() -> Self {
        let base = BaseDirs::new();
        let home = base
            .as_ref()
            .map(|b| b.home_dir().to_path_buf())
            .unwrap_or_else(std::env::temp_dir);

        // Fastfetch reads `$XDG_CONFIG_HOME/fastfetch` or `~/.config/fastfetch` on Linux/macOS.
        // On Windows, Fastfetch checks `%APPDATA%/fastfetch/config.jsonc` as well as `~/.config/fastfetch/config.jsonc`.
        let config_root = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .unwrap_or_else(|| home.join(".config"));

        let fastfetch_config = if cfg!(windows) {
            if let Some(appdata) = std::env::var_os("APPDATA").map(PathBuf::from) {
                let appdata_cfg = appdata.join("fastfetch").join("config.jsonc");
                if appdata_cfg.exists() {
                    appdata_cfg
                } else {
                    config_root.join("fastfetch").join("config.jsonc")
                }
            } else {
                config_root.join("fastfetch").join("config.jsonc")
            }
        } else {
            config_root.join("fastfetch").join("config.jsonc")
        };

        let data_root = base
            .as_ref()
            .map(|b| b.data_dir().to_path_buf())
            .unwrap_or_else(|| home.join(".local").join("share"));
        let cache_root = base
            .as_ref()
            .map(|b| b.cache_dir().to_path_buf())
            .unwrap_or_else(|| home.join(".cache"));

        let data_dir = data_root.join(APP_DIR);
        Self {
            config_dir: config_root.join(APP_DIR),
            fastfetch_config,
            assets_dir: find_assets_dir(&data_dir),
            data_dir,
            cache_dir: cache_root.join(APP_DIR),
        }
    }

    /// Sandbox every writable location under `root` (used by tests and `--root`).
    pub fn with_root(root: &Path) -> Self {
        let data_dir = root.join("data").join(APP_DIR);
        Self {
            config_dir: root.join("config").join(APP_DIR),
            fastfetch_config: root.join("config").join("fastfetch").join("config.jsonc"),
            assets_dir: find_assets_dir(&data_dir),
            data_dir,
            cache_dir: root.join("cache").join(APP_DIR),
        }
    }

    pub fn settings_file(&self) -> PathBuf {
        self.config_dir.join("settings.json")
    }

    /// The config referenced by the shell integration of the original installer.
    pub fn studio_config(&self) -> PathBuf {
        self.config_dir.join("fastfetch.jsonc")
    }

    pub fn backups_dir(&self) -> PathBuf {
        self.config_dir.join("backups")
    }

    pub fn logos_dir(&self) -> PathBuf {
        self.config_dir.join("logos")
    }

    pub fn images_dir(&self) -> PathBuf {
        self.config_dir.join("images")
    }

    pub fn custom_image(&self) -> PathBuf {
        self.config_dir.join("custom.png")
    }

    /// Scratch config used by the live `p` preview; never touches user configs.
    pub fn preview_config(&self) -> PathBuf {
        self.cache_dir.join("preview.jsonc")
    }

    pub fn preset_logos_dir(&self) -> Option<PathBuf> {
        self.assets_dir.as_ref().map(|a| a.join("preset-logos"))
    }

    /// Both files written on apply.
    pub fn config_targets(&self) -> [PathBuf; 2] {
        [self.studio_config(), self.fastfetch_config.clone()]
    }
}

/// The directory of the running executable, with symlinks resolved.
pub fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|e| e.canonicalize().ok())
        .and_then(|e| e.parent().map(Path::to_path_buf))
}

/// `Contents/MacOS` when `dir` lies inside a macOS `.app` bundle.
pub fn bundle_bin_dir(dir: &Path) -> Option<&Path> {
    let contents = dir.parent()?;
    let app = contents.parent()?;
    (dir.file_name()? == "MacOS"
        && contents.file_name()? == "Contents"
        && app.extension()? == "app")
        .then_some(dir)
}

/// `Contents/MacOS` of the `.app` bundle we run from, if any (the macOS DMG install).
pub fn app_bundle_bin_dir() -> Option<PathBuf> {
    let dir = exe_dir()?;
    bundle_bin_dir(&dir).map(Path::to_path_buf)
}

fn is_assets_dir(p: &Path) -> bool {
    p.join("preset-logos").is_dir()
}

/// Candidate asset locations, most specific first.
pub fn asset_candidates(data_dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(env) = std::env::var_os(ASSETS_ENV) {
        out.push(PathBuf::from(env));
    }
    if let Some(exe_dir) = exe_dir() {
        out.push(exe_dir.join("assets"));
        // macOS app bundle: `Contents/MacOS/<exe>` -> `Contents/Resources/assets`.
        out.push(exe_dir.join("..").join("Resources").join("assets"));
        out.push(
            exe_dir
                .join("..")
                .join("share")
                .join(APP_DIR)
                .join("assets"),
        );
        // `target/{debug,release}/` -> crate root during development.
        out.push(exe_dir.join("..").join("..").join("assets"));
    }
    out.push(data_dir.join("assets"));
    if let Ok(cwd) = std::env::current_dir() {
        out.push(cwd.join("assets"));
    }
    out.push(Path::new(env!("CARGO_MANIFEST_DIR")).join("assets"));
    out
}

pub fn find_assets_dir(data_dir: &Path) -> Option<PathBuf> {
    asset_candidates(data_dir)
        .into_iter()
        .find(|p| is_assets_dir(p))
        .map(|p| p.canonicalize().unwrap_or(p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundle_bin_dir_only_matches_app_bundles() {
        let inside = Path::new("/Applications/TermFetch Studio.app/Contents/MacOS");
        assert_eq!(bundle_bin_dir(inside), Some(inside));
        assert_eq!(bundle_bin_dir(Path::new("/usr/local/bin")), None);
        assert_eq!(bundle_bin_dir(Path::new("/opt/Contents/MacOS")), None);
    }
}
