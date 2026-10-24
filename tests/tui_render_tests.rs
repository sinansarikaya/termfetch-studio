use ratatui::{backend::TestBackend, Terminal};
use std::time::{Duration, Instant};
use termfetch_studio_v2::app::actions::Action;
use termfetch_studio_v2::app::reducer::reduce;
use termfetch_studio_v2::app::state::{AppState, Focus, Screen};
use termfetch_studio_v2::domain::settings::Settings;
use termfetch_studio_v2::imaging::TermGeometry;
use termfetch_studio_v2::paths::Paths;
use termfetch_studio_v2::tui::render::render_ui;

fn app(name: &str) -> (AppState, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!("tfs-tui-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let geo = TermGeometry::from_window(180, 45, 1800, 900);
    (AppState::new(Paths::with_root(&root), Settings::default(), geo), root)
}

/// Text of the emulated terminal inside the live preview box.
fn preview_text(t: &Terminal<TestBackend>, body: ratatui::layout::Rect) -> String {
    let buf = t.backend().buffer();
    let mut s = String::new();
    for y in body.top()..body.bottom() {
        for x in body.left()..body.right() {
            s.push_str(buf[(x, y)].symbol());
        }
        s.push('\n');
    }
    s
}

fn dump(t: &Terminal<TestBackend>) -> String {
    let buf = t.backend().buffer();
    let mut s = String::new();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            s.push_str(buf[(x, y)].symbol());
        }
        s.push('\n');
    }
    s
}

#[test]
fn every_screen_renders_at_every_size() {
    let (mut state, root) = app("sizes");
    for (w, h) in [(180u16, 45u16), (120, 32), (90, 24), (60, 16), (20, 6)] {
        let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
        for screen in Screen::ALL {
            reduce(&mut state, Action::Goto(screen));
            state.focus = Focus::Content;
            for _ in 0..3 {
                reduce(&mut state, Action::Down);
            }
            t.draw(|f| render_ui(f, &mut state)).unwrap();
        }
    }
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn dashboard_matches_the_studio_layout() {
    let (mut state, root) = app("dashboard");
    let mut t = Terminal::new(TestBackend::new(180, 45)).unwrap();
    t.draw(|f| render_ui(f, &mut state)).unwrap();
    let out = dump(&t);
    if std::env::var_os("TFS_DUMP").is_some() {
        println!("{out}");
    }
    for needle in [
        "TermFetch Studio", "v2.0", "Customize your terminal, your way.", "Fastfetch:",
        "Dashboard", "Themes", "Presets", "Logos", "Modules", "Settings",
        "CURRENT THEME", "CURRENT PRESET", "CONFIG PATH",
        "Welcome to", "Quick Actions", "Edit Theme", "Manage Presets", "Change Logo", "Apply & Preview",
        "Recent Themes", "View All", "System Overview", "Tools & Utilities", "System Diagnostics", "Backup & Restore",
        "Live Preview", "Preview Options", "Terminal Preview", "Apply Theme to System",
        "Navigate", "Next Section", "Demo Mode", "Run Fastfetch", "Quit",
    ] {
        assert!(out.contains(needle), "missing {needle:?}\n{out}");
    }
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn dashboard_keyboard_flow_and_options_dropdown() {
    use termfetch_studio_v2::app::state::{DashRow, PreviewOutput};
    let (mut state, root) = app("flow");
    let mut t = Terminal::new(TestBackend::new(180, 45)).unwrap();
    t.draw(|f| render_ui(f, &mut state)).unwrap();
    // Tab: menu -> stage -> options.
    reduce(&mut state, Action::ToggleFocus);
    assert_eq!(state.focus, Focus::Content);
    reduce(&mut state, Action::Down);
    assert_eq!(state.dash.row, DashRow::Recent);
    reduce(&mut state, Action::Down);
    assert_eq!(state.dash.row, DashRow::Tools);
    reduce(&mut state, Action::Up);
    reduce(&mut state, Action::Up);
    reduce(&mut state, Action::Number(1));
    assert_eq!(state.screen, Screen::Themes, "quick action 1 opens the theme editor");
    reduce(&mut state, Action::Goto(Screen::Dashboard));
    reduce(&mut state, Action::ToggleFocus);
    assert_eq!(state.focus, Focus::Options);
    // Dropdown: browse with arrows, Esc restores the original theme.
    let before = state.settings.theme.clone();
    reduce(&mut state, Action::Activate);
    assert!(state.dropdown.is_some());
    t.draw(|f| render_ui(f, &mut state)).unwrap();
    reduce(&mut state, Action::Down);
    assert_ne!(state.settings.theme, before);
    reduce(&mut state, Action::Back);
    assert!(state.dropdown.is_none());
    assert_eq!(state.settings.theme, before);
    // Output field cycles to the generated config view.
    state.options_cursor = 3;
    reduce(&mut state, Action::Right);
    assert_eq!(state.preview_output, PreviewOutput::Config);
    t.draw(|f| render_ui(f, &mut state)).unwrap();
    assert!(dump(&t).contains("\"modules\""), "config output shows the JSONC");
    // Apply button writes both configs and records the recent theme.
    state.options_cursor = 4;
    reduce(&mut state, Action::Activate);
    assert!(state.paths.fastfetch_config.exists(), "{:?}", state.status);
    assert_eq!(state.saved.recent_themes.first(), Some(&state.saved.theme));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn image_logos_render_from_decode_pool_without_blocking() {
    let (mut state, root) = app("pool");
    reduce(&mut state, Action::Goto(Screen::Logos));
    state.focus = Focus::Content;
    let pika = state.logos.iter().position(|l| l.id == "pikachu").unwrap();
    while state.logo_idx() != pika {
        reduce(&mut state, Action::Down);
    }
    // Wait for the background decode of the whole catalogue.
    let start = Instant::now();
    while state.pool.has_pending() && start.elapsed() < Duration::from_secs(20) {
        reduce(&mut state, Action::Tick);
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(state.pool.decoded_count() >= 14, "decoded {}", state.pool.decoded_count());

    // Scrolling through every logo, frames must be cheap once decoded.
    let mut t = Terminal::new(TestBackend::new(180, 45)).unwrap();
    t.draw(|f| render_ui(f, &mut state)).unwrap();
    let frames = state.logos.len() * 2;
    let start = Instant::now();
    for _ in 0..frames {
        reduce(&mut state, Action::Down);
        t.draw(|f| render_ui(f, &mut state)).unwrap();
    }
    let per_frame = start.elapsed() / frames as u32;
    eprintln!("avg frame while scrolling logos: {per_frame:?}");
    // Generous bound for unoptimised debug builds on CI; release is far faster.
    assert!(per_frame < Duration::from_millis(50), "{per_frame:?}");

    while state.logo_idx() != pika {
        reduce(&mut state, Action::Down);
    }
    t.draw(|f| render_ui(f, &mut state)).unwrap();
    let out = dump(&t);
    if std::env::var_os("TFS_DUMP").is_some() {
        println!("{out}");
    }
    let preview = preview_text(&t, state.preview_body);
    assert!(preview.contains('▀'), "half-block image expected in preview");
    assert!(preview.contains("cyberdeck"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn kitty_mode_uploads_full_resolution_and_draws_placeholders() {
    use termfetch_studio_v2::app::state::Effect;
    use termfetch_studio_v2::imaging::kitty::PLACEHOLDER;

    let (mut state, root) = app("kitty");
    state.enable_kitty();
    reduce(&mut state, Action::Goto(Screen::Logos));
    state.focus = Focus::Content;
    let pika = state.logos.iter().position(|l| l.id == "pikachu").unwrap();
    while state.logo_idx() != pika {
        reduce(&mut state, Action::Down);
    }
    let mut t = Terminal::new(TestBackend::new(180, 45)).unwrap();
    let mut uploads = Vec::new();
    let start = Instant::now();
    loop {
        t.draw(|f| render_ui(f, &mut state)).unwrap();
        if dump(&t).contains(PLACEHOLDER) || start.elapsed() > Duration::from_secs(30) {
            break;
        }
        reduce(&mut state, Action::Tick);
        for e in std::mem::take(&mut state.effects) {
            if let Effect::WriteTerminal(seq) = e {
                uploads.push(seq);
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let preview = preview_text(&t, state.preview_body);
    assert!(preview.contains(PLACEHOLDER), "placeholder cells expected once uploaded");
    assert!(!preview.contains('▀'), "half blocks must be replaced by the real image");
    assert!(uploads.iter().any(|s| s.starts_with("\x1b_Ga=T,U=1,f=100")));
    let _ = std::fs::remove_dir_all(root);
}
