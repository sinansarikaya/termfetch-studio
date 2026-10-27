use anyhow::{Context, Result, bail};
use clap::Parser;
use crossterm::{
    cursor::{Hide, Show},
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend, buffer::Buffer, layout::Rect};
use std::io::{self, BufRead, IsTerminal, Write, stdout};
use std::path::{Path, PathBuf};
use std::{panic, time::Duration};

use termfetch_studio_v2::adapters::backup;
use termfetch_studio_v2::adapters::cmd_autorun;
use termfetch_studio_v2::adapters::desktop;
use termfetch_studio_v2::adapters::doctor::{self, Level};
use termfetch_studio_v2::adapters::fastfetch::{adapt, config, runner};
use termfetch_studio_v2::adapters::install;
use termfetch_studio_v2::adapters::installer;
use termfetch_studio_v2::adapters::setup;
use termfetch_studio_v2::adapters::shell;
use termfetch_studio_v2::adapters::terminal_app;
use termfetch_studio_v2::adapters::wt_terminal;
use termfetch_studio_v2::app::actions::Action;
use termfetch_studio_v2::app::reducer::reduce;
use termfetch_studio_v2::app::state::{AppState, Effect, Screen, StatusKind};
use termfetch_studio_v2::cli::{BackupCmd, Cli, Command, DemoArg, ListArg, Selection};
use termfetch_studio_v2::domain::color::{ColorValue, PALETTE};
use termfetch_studio_v2::domain::logo::{LogoItem, builtin_logos};
use termfetch_studio_v2::domain::preset::{ModuleKind, Preset};
use termfetch_studio_v2::domain::settings::{LogoSize, Settings};
use termfetch_studio_v2::domain::theme::{Theme, get_builtin_themes};
use termfetch_studio_v2::imaging::TermGeometry;
use termfetch_studio_v2::imaging::kitty;
use termfetch_studio_v2::imaging::pool::ImagePool;
use termfetch_studio_v2::imaging::sixel;
use termfetch_studio_v2::paths::Paths;
use termfetch_studio_v2::tui::color_depth;
use termfetch_studio_v2::tui::render::render_ui;
use termfetch_studio_v2::tui::widgets::preview::{
    ImageState, LogoRaster, PreviewModel, buffer_to_ansi, draw_preview,
};

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
        let mut presets = termfetch_studio_v2::domain::preset::load_presets(&paths.config_dir);
        if let Some(slot) = presets.iter_mut().find(|p| p.id == "custom") {
            slot.sections = Preset::custom(&settings.custom_modules).sections;
        }
        Self {
            themes: get_builtin_themes(),
            presets,
            logos: builtin_logos(paths),
        }
    }

    fn resolve(&self, s: &Settings) -> (&Theme, &Preset, &LogoItem) {
        (
            self.themes
                .iter()
                .find(|t| t.id == s.theme)
                .unwrap_or(&self.themes[0]),
            self.presets
                .iter()
                .find(|p| p.id == s.preset)
                .unwrap_or(&self.presets[0]),
            self.logos
                .iter()
                .find(|l| l.id == s.logo)
                .unwrap_or(&self.logos[0]),
        )
    }
}

fn load_settings(paths: &Paths) -> Result<Settings> {
    Settings::load(&paths.settings_file())
}

fn apply_selection(cat: &Catalogue, s: &mut Settings, sel: &Selection) -> Result<()> {
    if let Some(t) = &sel.theme {
        if !cat.themes.iter().any(|x| &x.id == t) {
            bail!("unknown theme '{t}' (see `termfetch list themes`)");
        }
        s.theme = t.clone();
    }
    if let Some(p) = &sel.preset {
        if !cat.presets.iter().any(|x| &x.id == p) {
            bail!("unknown preset '{p}' (see `termfetch list presets`)");
        }
        s.preset = p.clone();
    }
    if let Some(l) = &sel.logo {
        if !cat.logos.iter().any(|x| &x.id == l) {
            bail!("unknown logo '{l}' (see `termfetch list logos`)");
        }
        s.logo = l.clone();
    }
    if let Some(sz) = &sel.logo_size {
        s.logo_size = LogoSize::parse(sz).with_context(|| format!("invalid logo size '{sz}'"))?;
    }
    if let Some(c) = &sel.icon_color {
        s.icon_color = ColorValue::parse(c)
            .filter(|v| *v != ColorValue::Inherit)
            .with_context(|| format!("invalid icon colour '{c}'"))?;
    }
    if let Some(c) = &sel.text_color {
        s.text_color = ColorValue::parse(c)
            .filter(|v| *v != ColorValue::Inherit)
            .with_context(|| format!("invalid text colour '{c}'"))?;
    }
    Ok(())
}

fn commit(paths: &Paths, s: &Settings) -> Result<Vec<PathBuf>> {
    let cat = Catalogue::load(paths, s);
    let (theme, preset, logo) = cat.resolve(s);
    s.save(&paths.settings_file())?;
    config::apply(theme, preset, logo, s, paths, &TermGeometry::detect())
}

/// Report a removed or kept launcher file.
fn report_launcher(path: &std::path::Path, result: anyhow::Result<bool>) {
    match result {
        Ok(true) => println!("✓ launcher removed: {}", path.display()),
        Ok(false) => {}
        Err(e) => println!("! {e:#}"),
    }
}

fn print_preview(paths: &Paths, s: &Settings) {
    let cat = Catalogue::load(paths, s);
    let (theme, preset, logo) = cat.resolve(s);
    let geo = TermGeometry::detect();
    let mut pool = ImagePool::new(1);
    let image = match logo.existing_image() {
        Some(p) => pool
            .load_blocking(p)
            .map(|i| ImageState::Ready(i.dims))
            .unwrap_or(ImageState::Missing),
        None if logo.is_image() => ImageState::Missing,
        None => ImageState::NotImage,
    };
    let info = (!s.demo_mode && runner::version().is_some())
        .then(|| {
            runner::probe_real_info(&ModuleKind::ALL, &paths.cache_dir.join("probe.jsonc")).ok()
        })
        .flatten();
    let model = PreviewModel {
        theme,
        preset,
        logo,
        settings: s,
        info: info.as_ref(),
        info_loading: false,
        image,
        geo,
        fill: false,
    };
    let width = geo.cols.clamp(60, 140);
    let lines = model.lines().len() as u16;
    let (_, lh) = model.logo_cells((width, 200));
    let height = lines.max(lh + config::LOGO_PADDING_TOP) + 1;
    let area = Rect::new(0, 0, width, height);
    let (cw, ch) = model.logo_cells((width, height));
    let grid = logo.existing_image().and_then(|p| pool.grid(p, cw, ch));
    let mut buf = Buffer::empty(area);
    let raster = grid.as_deref().map_or(LogoRaster::None, LogoRaster::Blocks);
    draw_preview(&mut buf, area, &model, raster, None);
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
        println!(
            "  \x1b[1;{color}m{}\x1b[0m {:<17} {}",
            c.level.symbol(),
            c.name,
            c.detail
        );
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
        apply_selection(
            &cat,
            &mut s,
            &Selection {
                theme: Some(theme.clone()),
                ..Default::default()
            },
        )?;
        if let Some(d) = cli.demo_override() {
            s.demo_mode = d;
        }
        print_preview(paths, &s);
        return Ok(Some(0));
    }
    let Some(cmd) = &cli.command else {
        return Ok(None);
    };
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
                println!(
                    "  theme={} preset={} logo={} demo={}",
                    s.theme,
                    s.preset,
                    s.logo,
                    if s.demo_mode { "on" } else { "off" }
                );
            }
        }
        Command::Settings => return Ok(None),
        Command::Install => {
            if !cfg!(windows) {
                bail!("`install` sets up Windows; use your package manager on this system");
            }
            let exe = std::env::current_exe().context("finding the running exe")?;
            let exe = exe.display().to_string();
            return Ok(Some(installer::run_script(&["-Source", &exe])?));
        }
        Command::Uninstall => {
            for p in shell::remove_all()? {
                println!("✓ shell startup removed from {}", p.display());
            }
            if let Some(path) = desktop::path().filter(|_| cfg!(target_os = "linux")) {
                report_launcher(&path, desktop::remove_if_ours(&path, "[Desktop Entry]"));
            }
            if let Some(path) = desktop::command_path().filter(|_| cfg!(target_os = "macos")) {
                report_launcher(&path, desktop::remove_if_ours(&path, desktop::COMMAND_MARK));
            }
            match terminal_app::remove(&paths.config_dir) {
                Ok(true) => println!("✓ Terminal.app default profile restored"),
                Ok(false) => {}
                Err(e) => println!("! Terminal.app profile not restored: {e:#}"),
            }
            if cfg!(windows) {
                println!("✓ {}", cmd_autorun::remove(cmd_autorun::KEY));
                for path in wt_terminal::settings_files() {
                    match wt_terminal::remove(&path) {
                        Ok(true) => println!(
                            "✓ Windows Terminal profile and colors removed from {}",
                            path.display()
                        ),
                        Ok(false) => {}
                        Err(e) => println!("! Windows Terminal not cleaned: {e:#}"),
                    }
                }
                return Ok(Some(installer::run_script(&["-Uninstall"])?));
            }
            if let Some(dir) = termfetch_studio_v2::paths::app_bundle_bin_dir() {
                let app = dir.join("../..").canonicalize().unwrap_or(dir);
                println!("Drag {} to the Trash to remove the app.", app.display());
            } else {
                println!(
                    "Remove the program itself with the package manager you installed it with."
                );
            }
        }
        Command::Run => {
            let cfg = paths.studio_config();
            if !cfg.exists() {
                bail!("no studio config yet - run `termfetch apply` first");
            }
            run_in_terminal(paths, &load_settings(paths).unwrap_or_default(), &cfg)?;
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
            println!(
                "Demo mode: {}",
                if s.demo_mode {
                    "ON (privacy-safe mock specs)"
                } else {
                    "OFF (real system specs)"
                }
            );
        }
        Command::List { what } => {
            let s = load_settings(paths).unwrap_or_default();
            let cat = Catalogue::load(paths, &s);
            match what {
                ListArg::Themes => cat
                    .themes
                    .iter()
                    .for_each(|t| println!("{:<22} {} [{}]", t.id, t.name, t.category.as_str())),
                ListArg::Presets => cat
                    .presets
                    .iter()
                    .for_each(|p| println!("{:<10} {:<16} {}", p.id, p.name, p.description)),
                ListArg::Logos => cat
                    .logos
                    .iter()
                    .for_each(|l| println!("{:<20} {} [{}]", l.id, l.name, l.group.as_str())),
                ListArg::Modules => ModuleKind::ALL.iter().for_each(|m| {
                    println!("{:<13} {} {}", m.fastfetch_type(), m.icon(), m.label())
                }),
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
                list.iter()
                    .for_each(|b| println!("{}  ({})", b.name, b.files.join(", ")));
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
        Command::Setup => {
            let s = load_settings(paths).unwrap_or_default();
            for p in commit(paths, &s)? {
                println!("✓ wrote {}", p.display());
            }
            let cat = Catalogue::load(paths, &s);
            let theme = cat
                .themes
                .iter()
                .find(|t| t.id == s.theme)
                .unwrap_or(&cat.themes[0]);
            for step in setup::integrate(&paths.studio_config(), theme, s.apply_terminal_colors) {
                println!("✓ {step}");
            }
        }
        Command::Doctor => return Ok(Some(if print_doctor(paths) { 0 } else { 1 })),
    }
    Ok(Some(0))
}

/// Run fastfetch in this terminal: the theme colours first where the terminal takes
/// them, then `config` adapted to what the terminal can show.
fn run_in_terminal(
    paths: &Paths,
    settings: &Settings,
    config: &Path,
) -> Result<std::process::ExitStatus> {
    let caps = adapt::Caps::detect();
    if settings.apply_terminal_colors
        && caps.osc_colors
        && io::stdout().is_terminal()
        && let Some(t) = get_builtin_themes()
            .into_iter()
            .find(|t| t.id == settings.theme)
    {
        print!("{}", t.osc_sequences());
        io::stdout().flush()?;
    }
    let cfg = adapt::prepare(config, &paths.cache_dir, &caps).unwrap_or(config.to_path_buf());
    runner::run_live(&cfg)
}

// ---------------------------------------------------------------------------
// TUI
// ---------------------------------------------------------------------------

fn setup_terminal() -> Result<Tui> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, EnableMouseCapture, Hide)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(out))?;
    terminal.hide_cursor()?;
    terminal.clear()?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Tui) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        Show
    )?;
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
        KeyCode::BackTab => Action::FocusPrev,
        KeyCode::Tab if shift => Action::FocusPrev,
        KeyCode::Tab => Action::ToggleFocus,
        KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Undo,
        KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Redo,
        KeyCode::Char('u') => Action::Undo,
        KeyCode::Char('U') => Action::Redo,
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
        KeyCode::Char('a' | 'A') => Action::Apply,
        KeyCode::Char('?') => Action::ToggleHelp,
        KeyCode::Char('f') => Action::FocusPreview,
        KeyCode::Char('c') => Action::FilterThemes,
        KeyCode::Char('/') => Action::SearchThemes,
        KeyCode::Char('o') => Action::CycleOutput,
        KeyCode::Char('d') => Action::Duplicate,
        KeyCode::Char('s') => Action::CycleLogoSize,
        KeyCode::Char('p') => Action::RunFastfetch,
        KeyCode::Char('i') => Action::Install,
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Redo,
        KeyCode::Char('r') => Action::Refresh,
        KeyCode::Char('x') | KeyCode::Delete => Action::Delete,
        KeyCode::Char(c @ '1'..='8') => Action::Number(c as u8 - b'0'),
        _ => return None,
    })
}

/// Enter the alternate screen again after the shell was handed over.
fn resume_tui(terminal: &mut Tui) -> Result<()> {
    enable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        EnterAlternateScreen,
        EnableMouseCapture,
        Hide
    )?;
    terminal.hide_cursor()?;
    terminal.clear()?;
    Ok(())
}

fn wait_for_enter() {
    let mut line = String::new();
    let _ = io::stdin().lock().read_line(&mut line);
}

/// Run the install command in the shell so sudo, UAC or installer prompts work, then come back.
fn install_fastfetch(terminal: &mut Tui, state: &mut AppState, plan: &install::Plan) -> Result<()> {
    restore_terminal(terminal)?;
    print!(
        "[2J[H[1mInstalling fastfetch:[0m {}

",
        plan.command_line()
    );
    io::stdout().flush()?;
    let status = std::process::Command::new(plan.program)
        .args(plan.args)
        .status();
    print!(
        "
[2m  Done - press Enter to return to TermFetch Studio…[0m"
    );
    io::stdout().flush()?;
    wait_for_enter();
    resume_tui(terminal)?;
    state.sixel.forget_shown();
    state.refresh_fastfetch();
    match (status, state.fastfetch_version.clone()) {
        (_, Some(v)) => state.set_status(StatusKind::Ok, format!("✓ fastfetch installed: {v}")),
        (Ok(s), None) => state.set_status(
            StatusKind::Error,
            format!("installer exited with {s}; fastfetch is still not found"),
        ),
        (Err(e), None) => state.set_status(
            StatusKind::Error,
            format!("could not start {}: {e}", plan.program),
        ),
    }
    state.dirty = true;
    Ok(())
}

/// Leave the TUI, run fastfetch with the current (possibly unsaved) choices, come back.
fn live_fastfetch(terminal: &mut Tui, state: &mut AppState) -> Result<()> {
    let (theme, preset, logo) = (
        state.current_theme().clone(),
        state.current_preset().clone(),
        state.current_logo().clone(),
    );
    let cfg = config::write_preview(
        &theme,
        &preset,
        &logo,
        &state.settings,
        &state.paths,
        &state.geo,
    )?;
    restore_terminal(terminal)?;
    print!("\x1b[2J\x1b[H");
    io::stdout().flush()?;
    let caps = adapt::Caps::detect();
    let status = adapt::prepare(&cfg, &state.paths.cache_dir, &caps)
        .and_then(|adapted| runner::run_live(&adapted));
    print!(
        "\n\x1b[2m  Live fastfetch preview ({}) · press Enter to return to TermFetch Studio…\x1b[0m",
        cfg.display()
    );
    io::stdout().flush()?;
    wait_for_enter();
    resume_tui(terminal)?;
    state.sixel.forget_shown();
    match status {
        Ok(s) if s.success() => state.set_status(
            StatusKind::Ok,
            "fastfetch ran with the current selection (nothing was saved)",
        ),
        Ok(s) => state.set_status(StatusKind::Error, format!("fastfetch exited with {s}")),
        Err(e) => state.set_status(StatusKind::Error, format!("{e:#}")),
    }
    state.dirty = true;
    Ok(())
}

fn write_raw(seq: &str) -> Result<()> {
    let mut out = stdout();
    out.write_all(seq.as_bytes())?;
    execute!(out, Hide)?;
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
            Effect::InstallFastfetch(plan) => {
                if let Err(e) = install_fastfetch(terminal, state, &plan) {
                    state.set_status(StatusKind::Error, format!("Install failed: {e:#}"));
                    state.dirty = true;
                }
            }
        }
    }
    Ok(())
}

/// Render the UI, reduced to 256 colours on terminals without truecolor.
fn render(f: &mut ratatui::Frame, state: &mut AppState) {
    render_ui(f, state);
    if color_depth::needs_256() {
        color_depth::downsample(f.buffer_mut());
    }
}

/// Draw one frame. A sixel image cannot be erased cell by cell, so when the image
/// on screen has to change the screen is cleared, repainted and the image placed on top.
fn draw_frame(terminal: &mut Tui, state: &mut AppState) -> Result<()> {
    state.sixel.begin_frame();
    terminal.draw(|f| render(f, state))?;
    if state.sixel.is_stale() {
        terminal.clear()?;
        state.sixel.begin_frame();
        terminal.draw(|f| render(f, state))?;
        if let Some(seq) = state.sixel.place() {
            write_raw(&seq)?;
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
            draw_frame(terminal, state)?;
            state.dirty = false;
            if state.editor.is_none() {
                let _ = terminal.hide_cursor();
            }
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
                    Event::Key(k) if k.kind != KeyEventKind::Release => {
                        if state.editor.is_some() {
                            match k.code {
                                KeyCode::Esc => Some(Action::Back),
                                KeyCode::Enter => Some(Action::Activate),
                                KeyCode::Backspace => Some(Action::Erase),
                                KeyCode::Char(c) => Some(Action::Text(c)),
                                _ => None,
                            }
                        } else {
                            key_action(k)
                        }
                    }
                    Event::Mouse(m) => match m.kind {
                        MouseEventKind::Down(_) => Some(Action::Click {
                            col: m.column,
                            row: m.row,
                        }),
                        MouseEventKind::ScrollUp => Some(Action::ScrollUp),
                        MouseEventKind::ScrollDown => Some(Action::ScrollDown),
                        _ => None,
                    },
                    Event::Resize(..) => {
                        let _ = execute!(terminal.backend_mut(), Hide);
                        let _ = terminal.hide_cursor();
                        let _ = terminal.clear();
                        state.sixel.forget_shown();
                        Some(Action::Resize)
                    }
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

/// Windows Terminal is the only Windows console that shows the logos and icons, so an
/// interactive run from the classic console reopens itself there. Returns whether it did.
fn relaunch_in_windows_terminal() -> bool {
    if !cfg!(windows) || std::env::var_os("WT_SESSION").is_some() {
        return false;
    }
    let Some(local) = std::env::var_os("LOCALAPPDATA") else {
        return false;
    };
    let wt = PathBuf::from(local)
        .join("Microsoft")
        .join("WindowsApps")
        .join("wt.exe");
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    if !wt.is_file() {
        return false;
    }
    // Forward the arguments too, so `termfetch settings` still opens Settings.
    std::process::Command::new(wt)
        .args(["-w", "new", "--"])
        .arg(exe)
        .args(std::env::args_os().skip(1))
        .spawn()
        .is_ok()
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let interactive = matches!(cli.command, None | Some(Command::Settings))
        && cli.preview.is_none()
        && !cli.doctor;
    if interactive && relaunch_in_windows_terminal() {
        return Ok(());
    }
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
    state.system_setup = true;
    if matches!(cli.command, Some(Command::Settings)) {
        reduce(&mut state, Action::Goto(Screen::Settings));
    }
    // First interactive start: apply once, so a plain copy of the exe is fully set up.
    let first_run = state.paths.config_dir.join("setup.done");
    if interactive && !first_run.exists() {
        reduce(&mut state, Action::Apply);
        if state.status.0 == StatusKind::Ok {
            let _ = std::fs::create_dir_all(&state.paths.config_dir);
            let _ = std::fs::write(&first_run, b"1");
        }
    }
    if kitty::detect() {
        state.enable_kitty();
    } else {
        state.sixel.enabled = sixel::detect();
    }
    state.block_logos = state.kitty.is_none() && !state.sixel.enabled;
    if let Some(e) = load_err {
        state.set_status(
            StatusKind::Warn,
            format!("settings.json unreadable ({e:#}); using defaults until you apply"),
        );
    }
    let res = run_app(&mut terminal, &mut state);
    if let Some(k) = state.kitty.as_mut() {
        let _ = write_raw(&k.cleanup());
    }
    restore_terminal(&mut terminal)?;
    if state.host_colors_applied {
        // Hand the shell its own colours back: OSC 110/111 reset foreground and background.
        print!("]110]111");
        let _ = io::stdout().flush();
    }
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
    match run_in_terminal(&state.paths, &state.saved, &cfg) {
        Ok(s) if !s.success() => eprintln!("\x1b[31mfastfetch exited with {s}\x1b[0m"),
        Err(e) => eprintln!("\x1b[31m{e:#}\x1b[0m"),
        Ok(_) => {}
    }
}
