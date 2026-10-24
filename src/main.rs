use anyhow::{bail, Context, Result};
use clap::Parser;
use crossterm::{
    cursor::Show,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, buffer::Buffer, layout::Rect, Terminal};
use std::io::{self, stdout, BufRead, Write};
use std::path::PathBuf;
use std::{panic, time::Duration};

use termfetch_studio_v2::adapters::backup;
use termfetch_studio_v2::adapters::doctor::{self, Level};
use termfetch_studio_v2::adapters::fastfetch::{config, runner};
use termfetch_studio_v2::app::actions::Action;
use termfetch_studio_v2::app::reducer::reduce;
use termfetch_studio_v2::app::state::{AppState, Effect, Screen, StatusKind};
use termfetch_studio_v2::cli::{BackupCmd, Cli, Command, DemoArg, ListArg, Selection};
use termfetch_studio_v2::domain::color::{ColorValue, PALETTE};
use termfetch_studio_v2::domain::logo::{builtin_logos, LogoItem};
use termfetch_studio_v2::domain::preset::{get_builtin_presets, ModuleKind, Preset};
use termfetch_studio_v2::domain::settings::{LogoSize, Settings};
use termfetch_studio_v2::domain::theme::{get_builtin_themes, Theme};
use termfetch_studio_v2::imaging::kitty;
use termfetch_studio_v2::imaging::pool::ImagePool;
use termfetch_studio_v2::imaging::TermGeometry;
use termfetch_studio_v2::paths::Paths;
use termfetch_studio_v2::tui::render::render_ui;
use termfetch_studio_v2::tui::widgets::preview::{buffer_to_ansi, draw_preview, ImageState, LogoRaster, PreviewModel};

type Tui = Terminal<CrosstermBackend<io::Stdout>>;

// ---------------------------------------------------------------------------
// Catalogue helpers shared by CLI commands
// ---------------------------------------------------------------------------

struct Catalogue {
    themes: Vec<Theme>,
    presets: Vec<Preset>,
    logos: Vec<LogoItem>,
}

impl Catalogue {
    fn load(paths: &Paths, settings: &Settings) -> Self {
        let mut presets = get_builtin_presets();
        if let Some(slot) = presets.iter_mut().find(|p| p.id == "custom") {
            *slot = Preset::custom(&settings.custom_modules);
        }
        Self { themes: get_builtin_themes(), presets, logos: builtin_logos(paths) }
    }

    fn resolve(&self, s: &Settings) -> (&Theme, &Preset, &LogoItem) {
        (
            self.themes.iter().find(|t| t.id == s.theme).unwrap_or(&self.themes[0]),
            self.presets.iter().find(|p| p.id == s.preset).unwrap_or(&self.presets[0]),
            self.logos.iter().find(|l| l.id == s.logo).unwrap_or(&self.logos[0]),
        )
    }
}

fn load_settings(paths: &Paths) -> Result<Settings> {
    Settings::load(&paths.settings_file())
}

fn apply_selection(cat: &Catalogue, s: &mut Settings, sel: &Selection) -> Result<()> {
    if let Some(t) = &sel.theme {
        if !cat.themes.iter().any(|x| &x.id == t) {
            bail!("unknown theme '{t}' (see `termfetch-studio list themes`)");
        }
        s.theme = t.clone();
    }
    if let Some(p) = &sel.preset {
        if !cat.presets.iter().any(|x| &x.id == p) {
            bail!("unknown preset '{p}' (see `termfetch-studio list presets`)");
        }
        s.preset = p.clone();
    }
    if let Some(l) = &sel.logo {
        if !cat.logos.iter().any(|x| &x.id == l) {
            bail!("unknown logo '{l}' (see `termfetch-studio list logos`)");
        }
        s.logo = l.clone();
    }
    if let Some(sz) = &sel.logo_size {
        s.logo_size = LogoSize::parse(sz).with_context(|| format!("invalid logo size '{sz}'"))?;
    }
    if let Some(c) = &sel.icon_color {
        s.icon_color = ColorValue::parse(c).filter(|v| *v != ColorValue::Inherit).with_context(|| format!("invalid icon colour '{c}'"))?;
    }
    if let Some(c) = &sel.text_color {
        s.text_color = ColorValue::parse(c).filter(|v| *v != ColorValue::Inherit).with_context(|| format!("invalid text colour '{c}'"))?;
    }
    Ok(())
}

fn commit(paths: &Paths, s: &Settings) -> Result<Vec<PathBuf>> {
    let cat = Catalogue::load(paths, s);
    let (theme, preset, logo) = cat.resolve(s);
    s.save(&paths.settings_file())?;
    config::apply(theme, preset, logo, s, paths, &TermGeometry::detect())
}

fn print_preview(paths: &Paths, s: &Settings) {
    let cat = Catalogue::load(paths, s);
    let (theme, preset, logo) = cat.resolve(s);
    let geo = TermGeometry::detect();
    let mut pool = ImagePool::new(1);
    let image = match logo.existing_image() {
        Some(p) => pool.load_blocking(p).map(|i| ImageState::Ready(i.dims)).unwrap_or(ImageState::Missing),
        None if logo.is_image() => ImageState::Missing,
        None => ImageState::NotImage,
    };
    let info = (!s.demo_mode && runner::version().is_some())
        .then(|| runner::probe_real_info(&ModuleKind::ALL, &paths.cache_dir.join("probe.jsonc")).ok())
        .flatten();
    let model = PreviewModel { theme, preset, logo, settings: s, info: info.as_ref(), info_loading: false, image, geo };
    let width = geo.cols.clamp(60, 140);
    let lines = model.lines().len() as u16;
    let (_, lh) = model.logo_cells((width, 200));
    let height = lines.max(lh + config::LOGO_PADDING_TOP) + 1;
    let area = Rect::new(0, 0, width, height);
    let (cw, ch) = model.logo_cells((width, height));
    let grid = logo.existing_image().and_then(|p| pool.grid(p, cw, ch));
    let mut buf = Buffer::empty(area);
    let raster = grid.as_deref().map_or(LogoRaster::None, LogoRaster::Halfblock);
    draw_preview(&mut buf, area, &model, raster, false);
    print!("{}", buffer_to_ansi(&buf));
}

fn print_doctor(paths: &Paths) -> bool {
    println!("TermFetch Studio v2 - Doctor\n");
    let checks = doctor::run_checks(paths);
    for c in &checks {
        let color = match c.level {
            Level::Ok => 32,
            Level::Info => 36,
            Level::Warn => 33,
            Level::Fail => 31,
        };
        println!("  \x1b[1;{color}m{}\x1b[0m {:<17} {}", c.level.symbol(), c.name, c.detail);
    }
    !checks.iter().any(|c| c.level == Level::Fail)
}

fn run_cli(cli: &Cli, paths: &Paths) -> Result<Option<i32>> {
    if cli.doctor {
        return Ok(Some(if print_doctor(paths) { 0 } else { 1 }));
    }
    if let Some(theme) = &cli.preview {
        let mut s = load_settings(paths)?;
        let cat = Catalogue::load(paths, &s);
        apply_selection(&cat, &mut s, &Selection { theme: Some(theme.clone()), ..Default::default() })?;
        if let Some(d) = cli.demo_override() {
            s.demo_mode = d;
        }
        print_preview(paths, &s);
        return Ok(Some(0));
    }
    let Some(cmd) = &cli.command else { return Ok(None) };
    match cmd {
        Command::Apply(sel) | Command::Preview(sel) => {
            let mut s = load_settings(paths)?;
            let cat = Catalogue::load(paths, &s);
            apply_selection(&cat, &mut s, sel)?;
            if let Some(d) = cli.demo_override() {
                s.demo_mode = d;
            }
            if matches!(cmd, Command::Preview(_)) {
                print_preview(paths, &s);
            } else {
                for p in commit(paths, &s)? {
                    println!("✓ wrote {}", p.display());
                }
                println!("  theme={} preset={} logo={} demo={}", s.theme, s.preset, s.logo, if s.demo_mode { "on" } else { "off" });
            }
        }
        Command::Run => {
            let cfg = paths.studio_config();
            if !cfg.exists() {
                bail!("no studio config yet - run `termfetch-studio apply` first");
            }
            runner::run_live(&cfg)?;
        }
        Command::Demo { state } => {
            let mut s = load_settings(paths)?;
            let new = match state {
                DemoArg::On => Some(true),
                DemoArg::Off => Some(false),
                DemoArg::Toggle => Some(!s.demo_mode),
                DemoArg::Status => None,
            };
            if let Some(on) = new {
                s.demo_mode = on;
                if paths.studio_config().exists() {
                    commit(paths, &s)?;
                    println!("✓ fastfetch config regenerated");
                } else {
                    s.save(&paths.settings_file())?;
                }
            }
            println!("Demo mode: {}", if s.demo_mode { "ON (privacy-safe mock specs)" } else { "OFF (real system specs)" });
        }
        Command::List { what } => {
            let s = load_settings(paths).unwrap_or_default();
            let cat = Catalogue::load(paths, &s);
            match what {
                ListArg::Themes => cat.themes.iter().for_each(|t| println!("{:<22} {} [{}]", t.id, t.name, t.category.as_str())),
                ListArg::Presets => cat.presets.iter().for_each(|p| println!("{:<10} {:<16} {}", p.id, p.name, p.description)),
                ListArg::Logos => cat.logos.iter().for_each(|l| println!("{:<20} {} [{}]", l.id, l.name, l.group.as_str())),
                ListArg::Modules => ModuleKind::ALL.iter().for_each(|m| println!("{:<13} {} {}", m.fastfetch_type(), m.icon(), m.label())),
                ListArg::Colors => PALETTE.iter().for_each(|(v, n)| println!("{v:<8} {n}")),
            }
        }
        Command::Backup { action } => match action {
            BackupCmd::Create => {
                let b = backup::create_backup(paths)?;
                println!("✓ created {} ({})", b.name, b.files.join(", "));
            }
            BackupCmd::List => {
                let list = backup::list_backups(paths);
                if list.is_empty() {
                    println!("No backups in {}", paths.backups_dir().display());
                }
                list.iter().for_each(|b| println!("{}  ({})", b.name, b.files.join(", ")));
            }
            BackupCmd::Restore { name } => {
                let b = backup::find_backup(paths, name)?;
                let n = backup::restore_backup(paths, &b)?;
                println!("✓ restored {n} files from {}", b.name);
            }
            BackupCmd::Delete { name } => {
                let b = backup::find_backup(paths, name)?;
                backup::delete_backup(paths, &b)?;
                println!("✓ deleted {}", b.name);
            }
        },
        Command::Doctor => return Ok(Some(if print_doctor(paths) { 0 } else { 1 })),
    }
    Ok(Some(0))
}

// ---------------------------------------------------------------------------
// TUI
// ---------------------------------------------------------------------------

fn setup_terminal() -> Result<Tui> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, EnableMouseCapture)?;
    Ok(Terminal::new(CrosstermBackend::new(out))?)
}

fn restore_terminal(terminal: &mut Tui) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    Ok(())
}

fn set_panic_hook() {
    let original = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture, Show);
        original(info);
    }));
}

fn key_action(key: KeyEvent) -> Option<Action> {
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    Some(match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
        KeyCode::Char('q') => Action::Quit,
        KeyCode::Esc => Action::Back,
        KeyCode::Tab | KeyCode::BackTab => Action::ToggleFocus,
        KeyCode::Up if shift => Action::MoveItem { up: true },
        KeyCode::Down if shift => Action::MoveItem { up: false },
        KeyCode::Char('K') => Action::MoveItem { up: true },
        KeyCode::Char('J') => Action::MoveItem { up: false },
        KeyCode::Up | KeyCode::Char('k') => Action::Up,
        KeyCode::Down | KeyCode::Char('j') => Action::Down,
        KeyCode::PageUp => Action::PageUp,
        KeyCode::PageDown => Action::PageDown,
        KeyCode::Home | KeyCode::Char('g') => Action::Home,
        KeyCode::End | KeyCode::Char('G') => Action::End,
        KeyCode::Left | KeyCode::Char('h') => Action::Left,
        KeyCode::Right | KeyCode::Char('l') => Action::Right,
        KeyCode::Enter => Action::Activate,
        KeyCode::Char(' ') => Action::Toggle,
        KeyCode::Char('a') => Action::Apply,
        KeyCode::Char('d') => Action::ToggleDemo,
        KeyCode::Char('s') => Action::CycleLogoSize,
        KeyCode::Char('p') => Action::RunFastfetch,
        KeyCode::Char('r') => Action::Refresh,
        KeyCode::Char('x') | KeyCode::Delete => Action::Delete,
        KeyCode::Char(c @ '1'..='8') => Action::Goto(Screen::ALL[(c as u8 - b'1') as usize]),
        _ => return None,
    })
}

/// Leave the TUI, run fastfetch with the current (possibly unsaved) choices, come back.
fn live_fastfetch(terminal: &mut Tui, state: &mut AppState) -> Result<()> {
    let (theme, preset, logo) = (state.current_theme().clone(), state.current_preset().clone(), state.current_logo().clone());
    let cfg = config::write_preview(&theme, &preset, &logo, &state.settings, &state.paths, &state.geo)?;
    restore_terminal(terminal)?;
    print!("\x1b[2J\x1b[H");
    io::stdout().flush()?;
    let status = runner::run_live(&cfg);
    print!("\n\x1b[2m  Live fastfetch preview ({}) · press Enter to return to TermFetch Studio…\x1b[0m", cfg.display());
    io::stdout().flush()?;
    let mut line = String::new();
    let _ = io::stdin().lock().read_line(&mut line);
    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
    terminal.clear()?;
    match status {
        Ok(s) if s.success() => state.set_status(StatusKind::Ok, "fastfetch ran with the current selection (nothing was saved)"),
        Ok(s) => state.set_status(StatusKind::Error, format!("fastfetch exited with {s}")),
        Err(e) => state.set_status(StatusKind::Error, format!("{e:#}")),
    }
    state.dirty = true;
    Ok(())
}

fn write_raw(seq: &str) -> Result<()> {
    let mut out = stdout();
    out.write_all(seq.as_bytes())?;
    out.flush()?;
    Ok(())
}

fn run_effects(terminal: &mut Tui, state: &mut AppState) -> Result<()> {
    for effect in std::mem::take(&mut state.effects) {
        match effect {
            Effect::WriteTerminal(seq) => write_raw(&seq)?,
            Effect::RunFastfetch => {
                if let Err(e) = live_fastfetch(terminal, state) {
                    state.set_status(StatusKind::Error, format!("Live run failed: {e:#}"));
                    state.dirty = true;
                }
            }
        }
    }
    Ok(())
}

fn run_app(terminal: &mut Tui, state: &mut AppState) -> Result<()> {
    while state.running {
        // Effects first: Kitty uploads must reach the terminal before the
        // frame that references them through placeholder cells.
        run_effects(terminal, state)?;
        if state.dirty {
            terminal.draw(|f| render_ui(f, state))?;
            state.dirty = false;
        }

        // Fast ticks only while background work is in flight; otherwise idle cheaply.
        let busy = state.pool.has_pending()
            || state.real_rx.is_some()
            || state.kitty.as_ref().is_some_and(|k| k.has_pending());
        let timeout = Duration::from_millis(if busy { 16 } else { 250 });
        if event::poll(timeout)? {
            // Drain the whole queue before redrawing so key repeat never lags behind.
            loop {
                let action = match event::read()? {
                    Event::Key(k) if k.kind != KeyEventKind::Release => key_action(k),
                    Event::Mouse(m) => match m.kind {
                        MouseEventKind::Down(_) => Some(Action::Click { col: m.column, row: m.row }),
                        MouseEventKind::ScrollUp => Some(Action::ScrollUp),
                        MouseEventKind::ScrollDown => Some(Action::ScrollDown),
                        _ => None,
                    },
                    Event::Resize(..) => Some(Action::Resize),
                    _ => None,
                };
                if let Some(a) = action {
                    reduce(state, a);
                }
                if !state.running || !event::poll(Duration::ZERO)? {
                    break;
                }
            }
        }
        reduce(state, Action::Tick);
    }
    // Terminal colours applied by the auto-save on quit.
    for effect in std::mem::take(&mut state.effects) {
        if let Effect::WriteTerminal(seq) = effect {
            write_raw(&seq)?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let paths = Paths::detect();

    match run_cli(&cli, &paths) {
        Ok(Some(code)) => std::process::exit(code),
        Ok(None) => {}
        Err(e) => {
            eprintln!("error: {e:#}");
            std::process::exit(1);
        }
    }

    let (mut settings, load_err) = match load_settings(&paths) {
        Ok(s) => (s, None),
        Err(e) => (Settings::default(), Some(e)),
    };
    if let Some(d) = cli.demo_override() {
        settings.demo_mode = d;
    }

    set_panic_hook();
    let mut terminal = setup_terminal()?;
    let mut state = AppState::new(paths, settings, TermGeometry::detect());
    if kitty::detect() {
        state.enable_kitty();
    }
    if let Some(e) = load_err {
        state.set_status(StatusKind::Warn, format!("settings.json unreadable ({e:#}); using defaults until you apply"));
    }
    let res = run_app(&mut terminal, &mut state);
    if let Some(k) = state.kitty.as_mut() {
        let _ = write_raw(&k.cleanup());
    }
    restore_terminal(&mut terminal)?;
    if let Some((kind, msg)) = &state.exit_message {
        let color = if *kind == StatusKind::Ok { 32 } else { 31 };
        eprintln!("\x1b[{color}m{msg}\x1b[0m");
    }
    if res.is_ok() {
        fastfetch_on_exit(&state);
    }
    res
}

/// Show the freshly saved theme/logo straight away in the shell the user returns to.
fn fastfetch_on_exit(state: &AppState) {
    let cfg = state.paths.studio_config();
    if !state.saved.run_fastfetch_on_exit || !cfg.exists() || state.fastfetch_version.is_none() {
        return;
    }
    let _ = io::stdout().flush();
    match runner::run_live(&cfg) {
        Ok(s) if !s.success() => eprintln!("\x1b[31mfastfetch exited with {s}\x1b[0m"),
        Err(e) => eprintln!("\x1b[31m{e:#}\x1b[0m"),
        Ok(_) => {}
    }
}
