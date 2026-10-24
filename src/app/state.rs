use crate::adapters::backup::{list_backups, BackupEntry};
use crate::adapters::doctor::Check;
use crate::adapters::fastfetch::runner::{self, RealInfo};
use crate::domain::logo::{builtin_logos, LogoItem};
use crate::domain::preset::{get_builtin_presets, ModuleKind, Preset};
use crate::domain::settings::Settings;
use crate::domain::theme::{get_builtin_themes, Theme};
use crate::imaging::kitty::KittyImages;
use crate::imaging::pool::ImagePool;
use crate::imaging::TermGeometry;
use crate::paths::Paths;
use ratatui::layout::Rect;
use ratatui::widgets::ListState;
use std::sync::mpsc::Receiver;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    #[default]
    Sidebar,
    Content,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Dashboard,
    Themes,
    Presets,
    Logos,
    Modules,
    Colors,
    Backup,
    Doctor,
}

impl Screen {
    pub const ALL: [Screen; 8] = [
        Screen::Dashboard,
        Screen::Themes,
        Screen::Presets,
        Screen::Logos,
        Screen::Modules,
        Screen::Colors,
        Screen::Backup,
        Screen::Doctor,
    ];

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&s| s == self).unwrap_or(0)
    }

    pub fn icon(self) -> &'static str {
        match self {
            Self::Dashboard => "\u{f056e}",
            Self::Themes => "\u{f03d8}",
            Self::Presets => "\u{f0645}",
            Self::Logos => "\u{f02e9}",
            Self::Modules => "\u{f0169}",
            Self::Colors => "\u{f020a}",
            Self::Backup => "\u{f006f}",
            Self::Doctor => "\u{f04d9}",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::Themes => "Themes",
            Self::Presets => "Presets",
            Self::Logos => "Logos",
            Self::Modules => "Module Builder",
            Self::Colors => "Colors",
            Self::Backup => "Backup & Restore",
            Self::Doctor => "Doctor",
        }
    }

    /// Context key hints for the status bar.
    pub fn hints(self) -> &'static str {
        match self {
            Self::Dashboard => "Enter apply · d preview · p fastfetch · 1-8 jump",
            Self::Themes | Self::Presets => "↑↓ browse · Enter apply · p fastfetch",
            Self::Logos => "↑↓ browse · s size · Enter apply · p fastfetch",
            Self::Modules => "Space toggle · J/K move · a apply",
            Self::Colors => "↑↓ field · ←→ colour · Enter reset/toggle · a apply",
            Self::Backup => "Enter create/restore · x delete",
            Self::Doctor => "r re-run checks",
        }
    }
}

/// Colour editor rows.
pub const COLOR_ROWS: usize = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusKind {
    Info,
    Ok,
    Warn,
    Error,
}

/// Side effects the event loop performs outside the pure reducer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Suspend the TUI and run fastfetch with the current (unsaved) choices.
    RunFastfetch,
    /// Raw escape sequence for the host terminal (OSC 10/11 theme colours).
    WriteTerminal(String),
}

/// Maps screen rows of the last rendered list back to logical items for mouse clicks.
#[derive(Debug, Clone, Default)]
pub struct ClickMap {
    pub area: Rect,
    pub offset: usize,
    pub rows: Vec<Option<usize>>,
}

impl ClickMap {
    pub fn hit(&self, col: u16, row: u16) -> Option<usize> {
        let a = self.area;
        if col < a.x || col >= a.x + a.width || row < a.y || row >= a.y + a.height {
            return None;
        }
        let idx = self.offset + usize::from(row - a.y);
        self.rows.get(idx).copied().flatten()
    }
}

pub struct AppState {
    pub paths: Paths,
    /// Working copy; the preview always renders this.
    pub settings: Settings,
    /// Last persisted settings; differs from `settings` while there are unsaved edits.
    pub saved: Settings,
    pub themes: Vec<Theme>,
    pub presets: Vec<Preset>,
    pub logos: Vec<LogoItem>,
    pub screen: Screen,
    pub focus: Focus,
    pub module_cursor: usize,
    pub color_cursor: usize,
    pub backup_cursor: usize,
    pub doctor_cursor: usize,
    pub list_states: [ListState; 8],
    pub click: ClickMap,
    pub sidebar_area: Rect,
    pub fastfetch_version: Option<String>,
    pub real_info: Option<RealInfo>,
    pub real_rx: Option<Receiver<RealInfo>>,
    pub pool: ImagePool,
    /// Kitty graphics uploader; `None` renders image logos with half blocks.
    pub kitty: Option<KittyImages>,
    pub geo: TermGeometry,
    pub backups: Vec<BackupEntry>,
    pub doctor: Vec<Check>,
    pub status: (StatusKind, String),
    pub running: bool,
    /// Printed to the shell after the TUI exits (auto-save result).
    pub exit_message: Option<(StatusKind, String)>,
    pub dirty: bool,
    pub effects: Vec<Effect>,
}

impl AppState {
    pub fn new(paths: Paths, settings: Settings, geo: TermGeometry) -> Self {
        let logos = builtin_logos(&paths);
        let mut app = Self {
            backups: list_backups(&paths),
            saved: settings.clone(),
            settings,
            themes: get_builtin_themes(),
            presets: get_builtin_presets(),
            logos,
            screen: Screen::Dashboard,
            focus: Focus::Sidebar,
            module_cursor: 0,
            color_cursor: 0,
            backup_cursor: 0,
            doctor_cursor: 0,
            list_states: Default::default(),
            click: ClickMap::default(),
            sidebar_area: Rect::default(),
            fastfetch_version: runner::version(),
            real_info: None,
            real_rx: None,
            pool: ImagePool::with_default_workers(),
            kitty: None,
            geo,
            doctor: Vec::new(),
            status: (
                StatusKind::Info,
                "Welcome to TermFetch Studio. Tab switches focus, Enter applies, p runs fastfetch.".into(),
            ),
            running: true,
            exit_message: None,
            dirty: true,
            effects: Vec::new(),
            paths,
        };
        app.sync_custom_preset();
        // Decode every image logo up front so browsing the gallery never waits.
        let images: Vec<String> = app.logos.iter().filter_map(|l| l.existing_image().map(str::to_string)).collect();
        if let Some(sel) = app.current_logo().existing_image().map(str::to_string) {
            app.pool.request(&sel, true);
        }
        app.pool.prefetch(images.iter().map(String::as_str));
        if !app.settings.demo_mode {
            app.start_real_probe();
        }
        app
    }

    /// Render image logos at full resolution through the Kitty graphics protocol.
    pub fn enable_kitty(&mut self) {
        if self.kitty.is_none() {
            self.kitty = Some(KittyImages::new(2));
            self.dirty = true;
        }
    }

    pub fn theme_idx(&self) -> usize {
        self.themes.iter().position(|t| t.id == self.settings.theme).unwrap_or(0)
    }

    pub fn preset_idx(&self) -> usize {
        self.presets.iter().position(|p| p.id == self.settings.preset).unwrap_or(0)
    }

    pub fn logo_idx(&self) -> usize {
        self.logos.iter().position(|l| l.id == self.settings.logo).unwrap_or(0)
    }

    pub fn current_theme(&self) -> &Theme {
        &self.themes[self.theme_idx()]
    }

    pub fn current_preset(&self) -> &Preset {
        &self.presets[self.preset_idx()]
    }

    pub fn current_logo(&self) -> &LogoItem {
        &self.logos[self.logo_idx()]
    }

    pub fn unsaved(&self) -> bool {
        self.settings != self.saved
    }

    /// Rebuild the "custom" preset entry from the Module Builder selection.
    pub fn sync_custom_preset(&mut self) {
        let custom = Preset::custom(&self.settings.custom_modules);
        match self.presets.iter_mut().find(|p| p.id == "custom") {
            Some(slot) => *slot = custom,
            None => self.presets.push(custom),
        }
    }

    /// Module Builder rows: enabled modules in their order, then the disabled ones.
    pub fn module_rows(&self) -> Vec<(ModuleKind, bool)> {
        let enabled = &self.settings.custom_modules;
        let mut rows: Vec<(ModuleKind, bool)> = enabled.iter().map(|&m| (m, true)).collect();
        rows.extend(ModuleKind::ALL.iter().filter(|m| !enabled.contains(m)).map(|&m| (m, false)));
        rows
    }

    pub fn set_status(&mut self, kind: StatusKind, msg: impl Into<String>) {
        self.status = (kind, msg.into());
    }

    /// Probe real system values in the background (only needed with demo mode off).
    pub fn start_real_probe(&mut self) {
        if self.real_info.is_some() || self.real_rx.is_some() || self.fastfetch_version.is_none() {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let scratch = self.paths.cache_dir.join("probe.jsonc");
        std::thread::spawn(move || {
            let info = runner::probe_real_info(&ModuleKind::ALL, &scratch).unwrap_or_default();
            let _ = tx.send(info);
        });
        self.real_rx = Some(rx);
    }

    pub fn refresh_backups(&mut self) {
        self.backups = list_backups(&self.paths);
        self.backup_cursor = self.backup_cursor.min(self.backups.len());
    }
}
