//! Type-safe fastfetch config generation (serde_json) with atomic writes.

use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::adapters::fsutil::{atomic_write, timestamp_utc};
use crate::domain::logo::{LogoItem, LogoKind};
use crate::domain::preset::{DEMO_DATA, ModuleKind, Preset};
use crate::domain::settings::Settings;
use crate::domain::style;
use crate::domain::theme::Theme;
use crate::imaging::{TermGeometry, config_logo_cells, image_dimensions};
use crate::paths::Paths;

pub const MARKER: &str = "TermFetch Studio";
pub const SCHEMA: &str = "https://github.com/fastfetch-cli/fastfetch/raw/dev/doc/json_schema.json";
pub const SEPARATOR: &str = " ➜ ";
pub const KEY_WIDTH: u16 = 10;
pub const LOGO_PADDING_TOP: u16 = 2;
pub const LOGO_PADDING_RIGHT: u16 = 4;
pub const DOT: &str = "●";

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FastfetchConfig {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub logo: LogoConfig,
    pub display: DisplayConfig,
    pub modules: Vec<ModuleEntry>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LogoConfig {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<BTreeMap<String, String>>,
    pub padding: Padding,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Padding {
    pub top: u16,
    pub right: u16,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DisplayConfig {
    pub separator: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(untagged)]
pub enum ModuleEntry {
    /// Bare string module such as `"break"`.
    Simple(String),
    Module(ModuleConfig),
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModuleConfig {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_width: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
}

/// How the logo is rendered by fastfetch.
#[derive(Debug, Clone, PartialEq)]
pub enum LogoSpec {
    Image {
        protocol: String,
        source: String,
        width: u16,
        height: u16,
    },
    Text {
        source: String,
    },
    Auto,
}

pub fn module_key(m: ModuleKind) -> String {
    format!(" ├ {} ", m.icon())
}

fn brk() -> ModuleEntry {
    ModuleEntry::Simple("break".into())
}

fn esc(sgr: &str, text: &str) -> String {
    format!("\u{1b}[1;{sgr}m{text}\u{1b}[0m")
}

pub fn dots_format(theme: &Theme, settings: &Settings) -> String {
    style::dot_colors(theme, settings)
        .iter()
        .map(|c| esc(&c.sgr(), DOT))
        .collect::<Vec<_>>()
        .join(" ")
}

fn title_format(theme: &Theme, settings: &Settings) -> String {
    let t = style::title_color(theme).sgr();
    let at = format!(
        "\u{1b}[{}m@\u{1b}[0m",
        style::text_color(theme, settings).sgr()
    );
    let (user, host) = if settings.demo_mode {
        DEMO_DATA
            .user_host
            .split_once('@')
            .unwrap_or((DEMO_DATA.user_host, ""))
    } else {
        ("{user-name}", "{host-name}")
    };
    format!("{}{at}{}", esc(&t, user), esc(&t, host))
}

/// Pure config construction; no I/O.
pub fn build_config(
    theme: &Theme,
    preset: &Preset,
    logo: &LogoSpec,
    settings: &Settings,
) -> FastfetchConfig {
    let padding = Padding {
        top: LOGO_PADDING_TOP,
        right: LOGO_PADDING_RIGHT,
    };
    let logo = match logo {
        LogoSpec::Image {
            protocol,
            source,
            width,
            height,
        } => LogoConfig {
            kind: protocol.clone(),
            source: Some(source.clone()),
            width: Some(*width),
            height: Some(*height),
            color: None,
            padding,
        },
        LogoSpec::Text { source } => LogoConfig {
            kind: "file".into(),
            source: Some(source.clone()),
            width: None,
            height: None,
            color: Some(BTreeMap::from([(
                "1".to_string(),
                style::logo_color(theme).fastfetch_value(),
            )])),
            padding,
        },
        LogoSpec::Auto => LogoConfig {
            kind: "auto".into(),
            source: None,
            width: None,
            height: None,
            color: None,
            padding,
        },
    };

    let mut modules = vec![brk(), brk()];
    // Title without a `key`: an empty key makes fastfetch print "Title ➜".
    modules.push(ModuleEntry::Module(ModuleConfig {
        kind: "title".into(),
        format: Some(title_format(theme, settings)),
        ..Default::default()
    }));
    modules.push(brk());
    let dots = || {
        ModuleEntry::Module(ModuleConfig {
            kind: "custom".into(),
            format: Some(dots_format(theme, settings)),
            ..Default::default()
        })
    };
    if preset.color_dots {
        modules.push(dots());
        modules.push(brk());
    }
    for (i, section) in preset.sections.iter().enumerate() {
        if i > 0 {
            modules.push(brk());
        }
        for &m in section {
            let colors = style::module_colors(theme, settings, m.group());
            modules.push(ModuleEntry::Module(ModuleConfig {
                kind: m.fastfetch_type().into(),
                key: Some(module_key(m)),
                format: settings.demo_mode.then(|| m.demo_value().to_string()),
                key_color: Some(colors.key.fastfetch_value()),
                output_color: Some(colors.output.fastfetch_value()),
                key_width: Some(KEY_WIDTH),
                timeout: m.timeout_ms(),
            }));
        }
    }
    modules.push(brk());
    if preset.color_dots {
        modules.push(dots());
    } else {
        modules.push(brk());
    }

    FastfetchConfig {
        schema: SCHEMA.into(),
        logo,
        display: DisplayConfig {
            separator: SEPARATOR.into(),
        },
        modules,
    }
}

/// Serialise as JSONC with a provenance header (4-space indent).
pub fn to_jsonc(
    cfg: &FastfetchConfig,
    theme: &Theme,
    preset: &Preset,
    settings: &Settings,
) -> Result<String> {
    let mut buf = Vec::new();
    let fmt = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, fmt);
    cfg.serialize(&mut ser)?;
    let body = String::from_utf8(buf)?;
    Ok(format!(
        "// {MARKER} v2 - generated fastfetch configuration\n\
         // Preset: {} | Theme: {} ({}) | Demo mode: {}\n\
         {body}\n",
        preset.name,
        theme.name,
        theme.category.as_str(),
        if settings.demo_mode { "on" } else { "off" },
    ))
}

/// Best fastfetch image protocol for the running terminal.
pub fn detect_image_protocol() -> &'static str {
    let env = |k: &str| std::env::var(k).unwrap_or_default();
    let term = env("TERM").to_ascii_lowercase();
    let program = env("TERM_PROGRAM").to_ascii_lowercase();
    if std::env::var_os("KITTY_WINDOW_ID").is_some()
        || term.contains("kitty")
        || term.contains("ghostty")
        || program == "ghostty"
        || program == "wezterm"
        || std::env::var_os("WEZTERM_EXECUTABLE").is_some()
    {
        "kitty-direct"
    } else if program == "iterm.app" || std::env::var_os("KONSOLE_VERSION").is_some() {
        "iterm"
    } else if term.starts_with("foot")
        || term.contains("mlterm")
        || term.contains("sixel")
        || std::env::var_os("WT_SESSION").is_some()
    {
        "sixel"
    } else {
        "chafa"
    }
}

fn copy_into(dir: &Path, src: &Path, name: &str) -> Result<PathBuf> {
    let dest = dir.join(name);
    if src == dest {
        return Ok(dest);
    }
    let bytes = fs::read(src).with_context(|| format!("reading {}", src.display()))?;
    if fs::read(&dest).ok().as_deref() != Some(bytes.as_slice()) {
        atomic_write(&dest, &bytes)?;
    }
    Ok(dest)
}

/// Prepare logo assets on disk and decide how fastfetch should draw them.
/// Images are copied into the config dir so the config keeps working even if
/// the asset directory moves.
pub fn resolve_logo_spec(
    logo: &LogoItem,
    settings: &Settings,
    paths: &Paths,
    geo: &TermGeometry,
) -> Result<LogoSpec> {
    match logo.kind {
        LogoKind::PresetImage | LogoKind::CustomImage => {
            let Some(src) = logo.existing_image().map(PathBuf::from) else {
                return Ok(LogoSpec::Auto);
            };
            let dims = image_dimensions(&src).unwrap_or((1, 1));
            let (width, height) = config_logo_cells(dims, settings.logo_size.max_cells(), geo);
            let source = if logo.kind == LogoKind::CustomImage {
                src
            } else {
                let name = src
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| format!("{}.png", logo.id));
                copy_into(&paths.images_dir(), &src, &name)?
            };
            Ok(LogoSpec::Image {
                protocol: detect_image_protocol().into(),
                source: source.to_string_lossy().into_owned(),
                width,
                height,
            })
        }
        LogoKind::Ascii => {
            if logo.art.trim().is_empty() {
                return Ok(LogoSpec::Auto);
            }
            let file = paths.logos_dir().join(format!("{}.txt", logo.id));
            atomic_write(&file, logo.art.as_bytes())?;
            Ok(LogoSpec::Text {
                source: file.to_string_lossy().into_owned(),
            })
        }
    }
}

/// The logo spec [`resolve_logo_spec`] would produce, without touching the disk
/// (used to show the generated config inside the TUI).
pub fn planned_logo_spec(
    logo: &LogoItem,
    settings: &Settings,
    paths: &Paths,
    geo: &TermGeometry,
) -> LogoSpec {
    match logo.kind {
        LogoKind::PresetImage | LogoKind::CustomImage => {
            let Some(src) = logo.existing_image().map(PathBuf::from) else {
                return LogoSpec::Auto;
            };
            let dims = image_dimensions(&src).unwrap_or((1, 1));
            let (width, height) = config_logo_cells(dims, settings.logo_size.max_cells(), geo);
            let source = if logo.kind == LogoKind::CustomImage {
                src
            } else {
                let name = src
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| format!("{}.png", logo.id));
                paths.images_dir().join(name)
            };
            LogoSpec::Image {
                protocol: detect_image_protocol().into(),
                source: source.to_string_lossy().into_owned(),
                width,
                height,
            }
        }
        LogoKind::Ascii if logo.art.trim().is_empty() => LogoSpec::Auto,
        LogoKind::Ascii => LogoSpec::Text {
            source: paths
                .logos_dir()
                .join(format!("{}.txt", logo.id))
                .to_string_lossy()
                .into_owned(),
        },
    }
}

/// Generated config text for display only; writes nothing.
pub fn render_dry(
    theme: &Theme,
    preset: &Preset,
    logo: &LogoItem,
    settings: &Settings,
    paths: &Paths,
    geo: &TermGeometry,
) -> Result<String> {
    let cfg = build_config(
        theme,
        preset,
        &planned_logo_spec(logo, settings, paths, geo),
        settings,
    );
    to_jsonc(&cfg, theme, preset, settings)
}

pub fn render(
    theme: &Theme,
    preset: &Preset,
    logo: &LogoItem,
    settings: &Settings,
    paths: &Paths,
    geo: &TermGeometry,
) -> Result<String> {
    let spec = resolve_logo_spec(logo, settings, paths, geo)?;
    let cfg = build_config(theme, preset, &spec, settings);
    to_jsonc(&cfg, theme, preset, settings)
}

/// Keep a copy of a hand-written fastfetch config before replacing it for the first time.
fn preserve_foreign_config(path: &Path, paths: &Paths) -> Result<Option<PathBuf>> {
    let Ok(existing) = fs::read_to_string(path) else {
        return Ok(None);
    };
    if existing.contains(MARKER) {
        return Ok(None);
    }
    let dest = paths.backups_dir().join(format!(
        "original-fastfetch-config_{}.jsonc",
        timestamp_utc()
    ));
    atomic_write(&dest, existing.as_bytes())?;
    Ok(Some(dest))
}

/// Write the generated config atomically to both targets.
pub fn write_targets(paths: &Paths, content: &str) -> Result<Vec<PathBuf>> {
    let targets = paths.config_targets();
    preserve_foreign_config(&paths.fastfetch_config, paths)?;
    for t in &targets {
        atomic_write(t, content.as_bytes()).with_context(|| format!("writing {}", t.display()))?;
    }
    Ok(targets.to_vec())
}

pub fn apply(
    theme: &Theme,
    preset: &Preset,
    logo: &LogoItem,
    settings: &Settings,
    paths: &Paths,
    geo: &TermGeometry,
) -> Result<Vec<PathBuf>> {
    let content = render(theme, preset, logo, settings, paths, geo)?;
    write_targets(paths, &content)
}

/// Scratch config for the live `p` preview.
pub fn write_preview(
    theme: &Theme,
    preset: &Preset,
    logo: &LogoItem,
    settings: &Settings,
    paths: &Paths,
    geo: &TermGeometry,
) -> Result<PathBuf> {
    let content = render(theme, preset, logo, settings, paths, geo)?;
    let p = paths.preview_config();
    atomic_write(&p, content.as_bytes())?;
    Ok(p)
}
