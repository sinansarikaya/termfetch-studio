use crate::adapters::backup::{BackupEntry, list_backups};
use crate::adapters::doctor::Check;
use crate::adapters::fastfetch::runner::{self, RealInfo};
use crate::domain::logo::{LogoItem, builtin_logos};
use crate::domain::preset::{ModuleKind, Preset};
use crate::domain::settings::Settings;
use crate::domain::theme::{Theme, get_builtin_themes};
use crate::imaging::TermGeometry;
use crate::imaging::kitty::KittyImages;
use crate::imaging::pool::ImagePool;
use crate::paths::Paths;
use ratatui::layout::Rect;
use ratatui::widgets::ListState;
use std::sync::mpsc::Receiver;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    #[default]
    Sidebar,
    Content,
    /// Preview Options card under the live preview.
    Options,
}

impl Focus {
    /// `Tab` order: menu → center stage → preview options.
    pub fn next(self) -> Self {
        match self {
            Self::Sidebar => Self::Content,
            Self::Content => Self::Options,
            Self::Options => Self::Sidebar,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Sidebar => Self::Options,
            Self::Content => Self::Sidebar,
            Self::Options => Self::Content,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Dashboard,
    Themes,
    Presets,
    Logos,
    Modules,
    Settings,
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
        Screen::Settings,
        Screen::Backup,
        Screen::Doctor,
    ];

    /// Entries of the main navigation; Backup and Doctor live under Settings
    /// (Dashboard → Tools & Utilities, or keys 7 / 8).
    pub const NAV: [Screen; 6] = [
        Screen::Dashboard,
        Screen::Themes,
        Screen::Presets,
        Screen::Logos,
        Screen::Modules,
        Screen::Settings,
    ];

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&s| s == self).unwrap_or(0)
    }

    pub fn icon(self) -> &'static str {
        match self {
            Self::Dashboard => "\u{f056e}",
            Self::Themes => "\u{f03d8}",
            Self::Presets => "\u{f0249}",
            Self::Logos => "\u{f02d4}",
            Self::Modules => "\u{f0169}",
            Self::Settings => "\u{f0493}",
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
            Self::Modules => "Modules",
            Self::Settings => "Settings",
            Self::Backup => "Backup & Restore",
            Self::Doctor => "Doctor",
        }
    }

    /// One-line description under the center-stage title.
    pub fn subtitle(self) -> &'static str {
        match self {
            Self::Dashboard => "Customize, preview and manage your Fastfetch themes with ease.",
            Self::Themes => "Browse the palettes - the live preview follows the highlighted card.",
            Self::Presets => "Choose which modules fastfetch prints and how they are grouped.",
            Self::Logos => "ASCII art or hi-res images, aspect-locked to the configured size.",
            Self::Modules => "Build your own preset: toggle modules and set their order.",
            Self::Settings => "Icon & text colours, preview mode and studio preferences.",
            Self::Backup => "Snapshot and restore settings.json and both fastfetch configs.",
            Self::Doctor => "Check dependencies, terminal capabilities and config health.",
        }
    }

    /// Screen-specific key badges shown in the bottom bar.
    pub fn extra_keys(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::Dashboard => &[("1-4", "Quick Action")],
            Self::Themes => &[("c", "Category"), ("/", "Search"), ("a", "Apply")],
            Self::Presets => &[
                ("Enter", "Use"),
                ("d", "Copy"),
                ("r", "Rename"),
                ("x", "Delete"),
            ],
            Self::Logos => &[("s", "Size"), ("a", "Apply")],
            Self::Modules => &[("Space", "Toggle"), ("J/K", "Reorder")],
            Self::Settings => &[("←→", "Change")],
            Self::Backup => &[("x", "Delete")],
            Self::Doctor => &[("r", "Re-run")],
        }
    }
}

/// Settings screen rows: 0-6 colours, 7 reset, 8 terminal colours,
/// 9 preview mode, 10 logo size, 11 fastfetch on exit, 12 studio chrome.
pub const SETTINGS_ROWS: usize = 15;

/// Dashboard rows that take keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DashRow {
    #[default]
    Quick,
    Recent,
    Tools,
}

/// Dashboard focus: a row plus the index of the card / entry inside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DashCursor {
    pub row: DashRow,
    pub col: usize,
}

pub const QUICK_ACTIONS: usize = 4;
pub const TOOLS: usize = 4;
/// Preview Options rows: theme, preset, logo, output, then the Apply button.
pub const OPTION_ROWS: usize = 5;
pub const OPTION_APPLY: usize = 4;

/// What the live preview box shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PreviewOutput {
    #[default]
    Terminal,
    Config,
}

impl PreviewOutput {
    pub const ALL: [PreviewOutput; 2] = [Self::Terminal, Self::Config];

    pub fn label(self) -> &'static str {
        match self {
            Self::Terminal => "Terminal Preview",
            Self::Config => "Generated Config (JSONC)",
        }
    }
}

/// Open Preview Options dropdown: the field it belongs to and the value to
/// restore on `Esc`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dropdown {
    pub field: usize,
    pub original: usize,
}

/// Clickable regions registered while rendering (topmost last).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hit {
    Nav(Screen),
    Quick(usize),
    Recent(usize),
    RecentScroll,
    ViewAll,
    Tool(usize),
    Option(usize),
    DropItem(usize),
    ThemeCard(usize),
    FocusPreview,
    ThemeFilter,
}

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
    pub undo_stack: Vec<Settings>,
    pub redo_stack: Vec<Settings>,
    pub themes: Vec<Theme>,
    pub presets: Vec<Preset>,
    pub logos: Vec<LogoItem>,
    pub screen: Screen,
    pub focus: Focus,
    pub module_cursor: usize,
    pub color_cursor: usize,
    pub dash: DashCursor,
    /// First visible Recent Themes card and how many fit (set while rendering).
    pub recent_offset: usize,
    pub recent_visible: usize,
    /// Columns of the Themes card grid and its first visible row (set while rendering).
    pub grid_cols: usize,
    pub grid_scroll: usize,
    pub options_cursor: usize,
    pub dropdown: Option<Dropdown>,
    pub preview_output: PreviewOutput,
    /// Generated config shown by the JSONC output, cached per selection.
    pub config_cache: Option<(Settings, String)>,
    pub help_open: bool,
    pub preview_focus: bool,
    pub preview_scroll: u16,
    pub theme_filter: usize,
    pub theme_search: String,
    /// Active text dialog: search, rename, or delete confirmation.
    pub editor: Option<String>,
    pub input: String,
    pub hits: Vec<(Rect, Hit)>,
    /// Inner rect of the live preview box (the emulated terminal) from the last frame.
    pub preview_body: Rect,
    pub sysinfo: crate::adapters::sysinfo::SysInfo,
    pub clock: String,
    pub backup_cursor: usize,
    pub doctor_cursor: usize,
    pub list_states: [ListState; 8],
    pub click: ClickMap,
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
    pub fn filtered_themes(&self) -> Vec<usize> {
        self.themes
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                let category = match self.theme_filter {
                    1 => "Dark",
                    2 => "Light",
                    3 => "Top Community",
                    4 => "Pastel",
                    5 => "Pop Culture & Gaming",
                    _ => "",
                };
                (category.is_empty() || t.category.as_str() == category)
                    && t.name
                        .to_lowercase()
                        .contains(&self.theme_search.to_lowercase())
            })
            .map(|(i, _)| i)
            .collect()
    }

    pub fn new(paths: Paths, settings: Settings, geo: TermGeometry) -> Self {
        let logos = builtin_logos(&paths);
        let mut app = Self {
            backups: list_backups(&paths),
            saved: settings.clone(),
            settings,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            themes: get_builtin_themes(),
            presets: crate::domain::preset::load_presets(&paths.config_dir),
            logos,
            screen: Screen::Dashboard,
            focus: Focus::Sidebar,
            module_cursor: 0,
            color_cursor: 0,
            dash: DashCursor::default(),
            recent_offset: 0,
            recent_visible: 4,
            grid_cols: 1,
            grid_scroll: 0,
            options_cursor: 0,
            dropdown: None,
            preview_output: PreviewOutput::default(),
            config_cache: None,
            help_open: false,
            preview_focus: false,
            preview_scroll: 0,
            theme_filter: 0,
            theme_search: String::new(),
            editor: None,
            input: String::new(),
            hits: Vec::new(),
            preview_body: Rect::default(),
            sysinfo: crate::adapters::sysinfo::SysInfo::detect(),
            clock: String::new(),
            backup_cursor: 0,
            doctor_cursor: 0,
            list_states: Default::default(),
            click: ClickMap::default(),
            fastfetch_version: runner::version(),
            real_info: None,
            real_rx: None,
            pool: ImagePool::with_default_workers(),
            kitty: None,
            geo,
            doctor: Vec::new(),
            status: (
                StatusKind::Info,
                "Welcome to TermFetch Studio. Tab switches focus, Enter applies, p runs fastfetch."
                    .into(),
            ),
            running: true,
            exit_message: None,
            dirty: true,
            effects: Vec::new(),
            paths,
        };
        if app.presets.iter().any(|p| p.id == "custom") {
            app.sync_custom_preset();
        }
        app.clock = app.sysinfo.clock();
        // Decode every image logo up front so browsing the gallery never waits.
        let images: Vec<String> = app
            .logos
            .iter()
            .filter_map(|l| l.existing_image().map(str::to_string))
            .collect();
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

    /// Chrome palette for the current settings.
    pub fn palette(&self) -> crate::tui::palette::Palette {
        crate::tui::palette::Palette::for_settings(
            self.settings.ui_follow_theme,
            self.current_theme(),
        )
    }

    /// Theme indices for the Dashboard's Recent Themes row: the applied theme,
    /// the recently applied ones, then a few favourites to fill the row.
    pub fn recent_theme_indices(&self) -> Vec<usize> {
        const FAVOURITES: [&str; 6] = [
            "monokai-pro",
            "tokyo-night",
            "catppuccin-mocha",
            "nord",
            "dracula",
            "rose-pine",
        ];
        let ids = std::iter::once(self.saved.theme.as_str())
            .chain(self.saved.recent_themes.iter().map(String::as_str))
            .chain(FAVOURITES);
        let mut out: Vec<usize> = Vec::new();
        for id in ids {
            if let Some(i) = self.themes.iter().position(|t| t.id == id)
                && !out.contains(&i)
            {
                out.push(i);
            }
            if out.len() == crate::domain::settings::RECENT_THEMES_MAX {
                break;
            }
        }
        out
    }

    /// Number of choices of a Preview Options field.
    pub fn option_len(&self, field: usize) -> usize {
        match field {
            0 => self.themes.len(),
            1 => self.presets.len(),
            2 => self.logos.len(),
            3 => PreviewOutput::ALL.len(),
            _ => 0,
        }
    }

    /// Currently selected choice of a Preview Options field.
    pub fn option_index(&self, field: usize) -> usize {
        match field {
            0 => self.theme_idx(),
            1 => self.preset_idx(),
            2 => self.logo_idx(),
            3 => PreviewOutput::ALL
                .iter()
                .position(|&o| o == self.preview_output)
                .unwrap_or(0),
            _ => 0,
        }
    }

    pub fn option_label(&self, field: usize, i: usize) -> String {
        match field {
            0 => self.themes.get(i).map(|t| t.name.clone()),
            1 => self.presets.get(i).map(|p| p.name.clone()),
            2 => self.logos.get(i).map(|l| l.name.clone()),
            3 => PreviewOutput::ALL.get(i).map(|o| o.label().to_string()),
            _ => None,
        }
        .unwrap_or_default()
    }

    pub fn theme_idx(&self) -> usize {
        self.themes
            .iter()
            .position(|t| t.id == self.settings.theme)
            .unwrap_or(0)
    }

    pub fn preset_idx(&self) -> usize {
        self.presets
            .iter()
            .position(|p| p.id == self.settings.preset)
            .unwrap_or(0)
    }

    pub fn logo_idx(&self) -> usize {
        self.logos
            .iter()
            .position(|l| l.id == self.settings.logo)
            .unwrap_or(0)
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

    pub fn is_modified(&self) -> bool {
        self.has_explicit_modifications()
    }

    pub fn has_explicit_modifications(&self) -> bool {
        self.settings.icon_color != self.saved.icon_color
            || self.settings.text_color != self.saved.text_color
            || self.settings.group_colors != self.saved.group_colors
            || self.settings.custom_modules != self.saved.custom_modules
            || self.settings.apply_terminal_colors != self.saved.apply_terminal_colors
            || self.settings.demo_mode != self.saved.demo_mode
            || self.settings.logo_size != self.saved.logo_size
            || self.settings.run_fastfetch_on_exit != self.saved.run_fastfetch_on_exit
            || self.settings.ui_follow_theme != self.saved.ui_follow_theme
    }

    pub fn is_previewing_theme(&self) -> bool {
        self.settings.theme != self.saved.theme
    }

    pub fn push_undo(&mut self) {
        if self.undo_stack.last() != Some(&self.settings) {
            self.undo_stack.push(self.settings.clone());
            if self.undo_stack.len() > 50 {
                self.undo_stack.remove(0);
            }
            self.redo_stack.clear();
        }
    }

    /// Rebuild the "custom" preset entry from the Module Builder selection.
    pub fn sync_custom_preset(&mut self) {
        let custom = Preset::custom(&self.settings.custom_modules);
        match self.presets.iter_mut().find(|p| p.id == "custom") {
            Some(slot) => slot.sections = custom.sections,
            None => self.presets.push(custom),
        }
    }

    /// Module Builder rows: enabled modules in their order, then the disabled ones.
    pub fn module_rows(&self) -> Vec<(ModuleKind, bool)> {
        let enabled = &self.settings.custom_modules;
        let mut rows: Vec<(ModuleKind, bool)> = enabled.iter().map(|&m| (m, true)).collect();
        rows.extend(
            ModuleKind::ALL
                .iter()
                .filter(|m| !enabled.contains(m))
                .map(|&m| (m, false)),
        );
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
