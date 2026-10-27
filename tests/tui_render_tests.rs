use ratatui::{Terminal, backend::TestBackend};
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
    (
        AppState::new(
            Paths::with_root(&root),
            // Mock specs keep the rendered text independent of this machine.
            Settings {
                demo_mode: true,
                ..Settings::default()
            },
            geo,
        ),
        root,
    )
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
        "TermFetch Studio",
        concat!("v", env!("CARGO_PKG_VERSION")),
        "Customize your terminal, your way.",
        "Fastfetch:",
        "Dashboard",
        "Themes",
        "Presets",
        "Logos",
        "Modules",
        "Settings",
        "CURRENT THEME",
        "CURRENT PRESET",
        "CONFIG PATH",
        "Welcome to",
        "Quick Actions",
        "Edit Theme",
        "Manage Presets",
        "Change Logo",
        "Apply & Preview",
        "Recent Themes",
        "View All",
        "System Overview",
        "Tools & Utilities",
        "System Diagnostics",
        "Backup & Restore",
        "Live Preview",
        "Preview Options",
        "Terminal Preview",
        "Apply Theme to System",
        "Navigate",
        "Next Section",
        "Run Fastfetch",
        "Quit",
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
    assert_eq!(
        state.screen,
        Screen::Themes,
        "quick action 1 opens the theme editor"
    );
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
    assert!(
        dump(&t).contains("\"modules\""),
        "config output shows the JSONC"
    );
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
    while state.pool.has_pending() && start.elapsed() < Duration::from_secs(120) {
        reduce(&mut state, Action::Tick);
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        state.pool.decoded_count() >= 14,
        "decoded {}",
        state.pool.decoded_count()
    );

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
    // Quadrant blocks, or TermFetch Blocks octants when run inside Terminal.app.
    assert!(
        preview
            .chars()
            .any(|c| "▀▄▌▐▘▝▖▗▚▞▛▜▙▟█".contains(c) || ('\u{100000}'..='\u{1000ff}').contains(&c)),
        "block-drawn image expected in preview"
    );
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
        if dump(&t).contains(PLACEHOLDER) || start.elapsed() > Duration::from_secs(120) {
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
    assert!(
        preview.contains(PLACEHOLDER),
        "placeholder cells expected once uploaded"
    );
    assert!(
        !preview.contains('▀'),
        "half blocks must be replaced by the real image"
    );
    assert!(uploads.iter().any(|s| s.starts_with("\x1b_Ga=T,U=1,f=100")));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn sixel_mode_places_the_logo_over_blank_cells() {
    let (mut state, root) = app("sixel");
    state.sixel.enabled = true;
    reduce(&mut state, Action::Goto(Screen::Logos));
    state.focus = Focus::Content;
    let pika = state.logos.iter().position(|l| l.id == "pikachu").unwrap();
    while state.logo_idx() != pika {
        reduce(&mut state, Action::Down);
    }
    let mut t = Terminal::new(TestBackend::new(180, 45)).unwrap();
    let start = Instant::now();
    loop {
        state.sixel.begin_frame();
        t.draw(|f| render_ui(f, &mut state)).unwrap();
        if state.sixel.is_stale() || start.elapsed() > Duration::from_secs(120) {
            break;
        }
        reduce(&mut state, Action::Tick);
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        state.sixel.is_stale(),
        "the decoded logo must be wanted on screen"
    );
    let seq = state.sixel.place().expect("a placement sequence");
    assert!(seq.starts_with("7[") && seq.contains("Pq") && seq.ends_with("8"));
    assert!(!state.sixel.is_stale(), "placed image counts as shown");
    let preview = preview_text(&t, state.preview_body);
    assert!(
        !preview.contains('▀'),
        "half blocks must not be drawn when sixel is available"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn focus_preview_expands_and_restores_with_keyboard_and_mouse() {
    use termfetch_studio_v2::app::state::Hit;
    let (mut state, root) = app("focus-upgrade");
    let mut t = Terminal::new(TestBackend::new(180, 45)).unwrap();
    t.draw(|f| render_ui(f, &mut state)).unwrap();
    let normal = state.preview_body;
    assert!((73..=85).contains(&normal.width));
    let (button, _) = state
        .hits
        .iter()
        .find(|(_, h)| *h == Hit::FocusPreview)
        .unwrap();
    let (col, row) = (button.x, button.y);
    reduce(&mut state, Action::Click { col, row });
    t.draw(|f| render_ui(f, &mut state)).unwrap();
    assert!(state.preview_body.width > normal.width * 2);
    reduce(&mut state, Action::Back);
    assert!(!state.preview_focus);
    t.draw(|f| render_ui(f, &mut state)).unwrap();
    assert_eq!(state.preview_body, normal);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn theme_search_filters_navigation_and_supports_empty_results() {
    let (mut state, root) = app("search-upgrade");
    state.screen = Screen::Themes;
    state.focus = Focus::Content;
    reduce(&mut state, Action::SearchThemes);
    for c in "gruvbox".chars() {
        reduce(&mut state, Action::Text(c));
    }
    reduce(&mut state, Action::Activate);
    assert_eq!(state.filtered_themes().len(), 2);
    reduce(&mut state, Action::Right);
    assert!(state.current_theme().id.contains("gruvbox"));
    reduce(&mut state, Action::FilterThemes);
    assert_eq!(state.current_theme().id, "gruvbox-dark");
    state.theme_search = "no such theme".into();
    reduce(&mut state, Action::Down);
    let mut t = Terminal::new(TestBackend::new(180, 45)).unwrap();
    t.draw(|f| render_ui(f, &mut state)).unwrap();
    assert!(state.filtered_themes().is_empty());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn preset_copy_rename_and_confirmed_delete_persist() {
    let (mut state, root) = app("presets-upgrade");
    state.screen = Screen::Presets;
    state.focus = Focus::Content;
    let count = state.presets.len();
    let sections = state.current_preset().sections.clone();
    reduce(&mut state, Action::Duplicate);
    assert_eq!(state.presets.len(), count + 1);
    assert_eq!(state.current_preset().sections, sections);
    reduce(&mut state, Action::Refresh);
    state.input = "My workstation".into();
    reduce(&mut state, Action::Activate);
    let loaded = termfetch_studio_v2::domain::preset::load_presets(&state.paths.config_dir);
    assert!(loaded.iter().any(|p| p.name == "My workstation"));
    reduce(&mut state, Action::Delete);
    reduce(&mut state, Action::Back);
    assert_eq!(state.presets.len(), count + 1);
    reduce(&mut state, Action::Delete);
    for c in "yes".chars() {
        reduce(&mut state, Action::Text(c));
    }
    reduce(&mut state, Action::Activate);
    assert_eq!(state.presets.len(), count);
    assert_eq!(
        termfetch_studio_v2::domain::preset::load_presets(&state.paths.config_dir).len(),
        count
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn thumbnails_are_deterministic_and_theme_specific() {
    use ratatui::{buffer::Buffer, layout::Rect};
    use termfetch_studio_v2::{
        domain::theme::get_builtin_themes, tui::widgets::ui::draw_thumbnail,
    };
    let area = Rect::new(0, 0, 64, 16);
    let mut scenes = Vec::new();
    for theme in get_builtin_themes() {
        let mut a = Buffer::empty(area);
        let mut b = Buffer::empty(area);
        draw_thumbnail(&mut a, area, &theme);
        draw_thumbnail(&mut b, area, &theme);
        assert_eq!(a, b);
        assert!(!scenes.contains(&a), "duplicate scene {}", theme.id);
        scenes.push(a);
        draw_thumbnail(&mut b, Rect::new(63, 15, 8, 8), &theme);
    }
}

#[test]
fn custom_ascii_gallery_does_not_import_generated_logo_cache() {
    let (state, root) = app("ascii-upgrade");
    let dir = state.paths.config_dir.join("ascii");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("robot.txt"), " [o_o]\n /|_|\\").unwrap();
    let logos = termfetch_studio_v2::domain::logo::builtin_logos(&state.paths);
    let logo = logos.iter().find(|l| l.id == "custom-ascii-robot").unwrap();
    assert_eq!(logo.art, " [o_o]\n /|_|\\");
    assert!(!logo.is_image());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn all_quick_actions_have_click_targets_at_standard_and_compact_sizes() {
    use termfetch_studio_v2::app::state::Hit;
    let (mut state, root) = app("quick-upgrade");
    for (w, h) in [(180, 45), (120, 32)] {
        let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
        for i in 0..4 {
            state.screen = Screen::Dashboard;
            state.focus = Focus::Content;
            t.draw(|f| render_ui(f, &mut state)).unwrap();
            let rect = state
                .hits
                .iter()
                .find(|(_, hit)| *hit == Hit::Quick(i))
                .unwrap()
                .0;
            assert!(rect.width > 8 && rect.height >= 3);
            reduce(
                &mut state,
                Action::Click {
                    col: rect.x,
                    row: rect.y,
                },
            );
            let clicked = state.screen;
            state.screen = Screen::Dashboard;
            state.focus = Focus::Content;
            reduce(&mut state, Action::Number((i + 1) as u8));
            assert_eq!(state.screen, clicked);
        }
    }
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn builtin_presets_can_be_renamed_and_deleted_with_confirmation() {
    let (mut state, root) = app("builtin-preset-upgrade");
    state.screen = Screen::Presets;
    state.focus = Focus::Content;
    let id = state.current_preset().id.clone();
    reduce(&mut state, Action::Refresh);
    state.input = "Renamed built-in".into();
    reduce(&mut state, Action::Activate);
    let loaded = termfetch_studio_v2::domain::preset::load_presets(&state.paths.config_dir);
    assert_eq!(
        loaded.iter().find(|p| p.id == id).unwrap().name,
        "Renamed built-in"
    );
    reduce(&mut state, Action::Delete);
    state.input = "yes".into();
    reduce(&mut state, Action::Activate);
    let loaded = termfetch_studio_v2::domain::preset::load_presets(&state.paths.config_dir);
    assert!(!loaded.iter().any(|p| p.id == id));
    let _ = std::fs::remove_dir_all(root);
}
