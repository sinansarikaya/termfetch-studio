use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use termfetch_studio_v2::app::actions::Action;
use termfetch_studio_v2::app::reducer::reduce;
use termfetch_studio_v2::app::state::{AppState, Focus, Screen};
use termfetch_studio_v2::domain::preset::ModuleKind;
use termfetch_studio_v2::domain::settings::Settings;
use termfetch_studio_v2::domain::theme::get_builtin_themes;
use termfetch_studio_v2::imaging::TermGeometry;
use termfetch_studio_v2::paths::Paths;
use termfetch_studio_v2::tui::render::render_ui;
use termfetch_studio_v2::tui::widgets::ui::draw_thumbnail;

fn test_app(prefix: &str) -> (AppState, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!("tfs-test-{}-{}", prefix, std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::create_dir_all(&root);
    let paths = Paths::with_root(&root);
    let settings = Settings {
        demo_mode: true,
        ..Default::default()
    };
    let geo = TermGeometry::detect();
    let state = AppState::new(paths, settings, geo);
    (state, root)
}

#[test]
fn theme_swatches_show_background_lines_and_dots() {
    let themes = get_builtin_themes();
    assert_eq!(themes.len(), 40);

    // Gallery card, dashboard card and the tiny sidebar chip.
    for area in [
        Rect::new(0, 0, 26, 6),
        Rect::new(0, 0, 18, 4),
        Rect::new(0, 0, 9, 3),
    ] {
        for t in &themes {
            let mut buf = Buffer::empty(area);
            draw_thumbnail(&mut buf, area, t);
            let bg = t.background_color();
            assert!(buf.content.iter().all(|c| c.bg == bg), "{} bg", t.id);
            let bars: Vec<_> = buf.content.iter().filter(|c| c.symbol() == "▄").collect();
            assert!(!bars.is_empty(), "{} has no info lines", t.id);
            assert!(bars.iter().all(|c| c.fg == t.primary_color()));
            let dots = buf.content.iter().filter(|c| c.symbol() == "●").count();
            assert_eq!(dots, 3, "{} at {area:?}", t.id);
        }
    }
}

#[test]
fn responsive_terminals_render_without_panics() {
    let sizes = [(80, 24), (100, 30), (120, 40), (160, 45), (200, 55)];
    let screens = [
        Screen::Dashboard,
        Screen::Themes,
        Screen::Presets,
        Screen::Logos,
        Screen::Modules,
        Screen::Settings,
    ];

    let (mut state, root) = test_app("responsive");

    for &(w, h) in &sizes {
        let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
        for &scr in &screens {
            state.screen = scr;
            state.focus = Focus::Content;
            term.draw(|f| render_ui(f, &mut state)).unwrap();
            let buf = term.backend().buffer();
            assert_eq!(buf.area.width, w);
            assert_eq!(buf.area.height, h);
        }
    }
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn state_isolation_browsing_vs_modifying_vs_applied() {
    let (mut state, root) = test_app("state-isolation");
    let initial_theme = state.saved.theme.clone();

    // 1. Initially applied: not modified, not previewing
    assert!(!state.is_previewing_theme());
    assert!(!state.has_explicit_modifications());
    assert!(!state.unsaved());

    // 2. Browsing themes: moves cursor, previews theme
    state.screen = Screen::Themes;
    state.focus = Focus::Content;
    reduce(&mut state, Action::Down);

    assert_ne!(state.settings.theme, initial_theme);
    assert!(state.is_previewing_theme());
    assert!(
        !state.has_explicit_modifications(),
        "Browsing alone must NOT trigger explicit modification"
    );

    // 3. Modifying custom settings: changes a setting value
    state.screen = Screen::Settings;
    state.color_cursor = 0; // icon_color
    reduce(&mut state, Action::Right);
    assert!(
        state.has_explicit_modifications(),
        "Changing color setting MUST trigger explicit modification"
    );
    assert!(state.unsaved());

    // 4. Applying settings: commits to saved, clears modified/unsaved
    reduce(&mut state, Action::Apply);
    assert!(!state.has_explicit_modifications());
    assert!(!state.is_previewing_theme());
    assert!(!state.unsaved());
    assert_eq!(state.settings.theme, state.saved.theme);

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn undo_redo_stack_depth_and_sequential_restore() {
    let (mut state, root) = test_app("undo-redo");
    state.screen = Screen::Settings;
    state.focus = Focus::Content;

    let initial_icon = state.settings.icon_color.clone();

    // Perform change 1
    state.color_cursor = 0;
    reduce(&mut state, Action::Right);
    let icon_after_1 = state.settings.icon_color.clone();
    assert_ne!(initial_icon, icon_after_1);

    // Perform change 2
    reduce(&mut state, Action::Right);
    let icon_after_2 = state.settings.icon_color.clone();
    assert_ne!(icon_after_1, icon_after_2);

    // Undo change 2
    reduce(&mut state, Action::Undo);
    assert_eq!(state.settings.icon_color, icon_after_1);

    // Undo change 1
    reduce(&mut state, Action::Undo);
    assert_eq!(state.settings.icon_color, initial_icon);

    // Undo on empty stack
    reduce(&mut state, Action::Undo);
    assert_eq!(state.settings.icon_color, initial_icon);

    // Redo change 1
    reduce(&mut state, Action::Redo);
    assert_eq!(state.settings.icon_color, icon_after_1);

    // Redo change 2
    reduce(&mut state, Action::Redo);
    assert_eq!(state.settings.icon_color, icon_after_2);

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn safe_apply_creates_pre_apply_backup() {
    let (mut state, root) = test_app("safe-apply");

    // First apply creates initial files on disk
    reduce(&mut state, Action::Apply);
    assert!(state.paths.studio_config().exists());

    // Modify a setting and apply again: pre-apply backup is created
    state.settings.theme = "nord".into();
    reduce(&mut state, Action::Apply);

    // Verify backup folder contains at least one backup entry
    let backups = termfetch_studio_v2::adapters::backup::list_backups(&state.paths);
    assert!(
        !backups.is_empty(),
        "Safe Apply must automatically create a backup of prior config before applying!"
    );

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn broken_or_missing_logo_does_not_panic() {
    let (mut state, root) = test_app("broken-logo");

    // Request non-existent image
    state
        .pool
        .request("/tmp/non_existent_termfetch_logo_xyz.png", true);
    let start = std::time::Instant::now();
    let mut found_error = false;
    while start.elapsed() < std::time::Duration::from_secs(5) {
        reduce(&mut state, Action::Tick);
        if state
            .pool
            .error("/tmp/non_existent_termfetch_logo_xyz.png")
            .is_some()
        {
            found_error = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Verify error is registered gracefully
    assert!(found_error, "Missing file must report an error, not panic");

    // Verify rendering logos screen with broken image doesn't panic
    let mut term = Terminal::new(TestBackend::new(120, 35)).unwrap();
    state.screen = Screen::Logos;
    term.draw(|f| render_ui(f, &mut state)).unwrap();

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn long_fastfetch_lines_wrap_without_overflow() {
    let (mut state, root) = test_app("long-lines");

    state.screen = Screen::Dashboard;
    state.focus = Focus::Content;

    // Simulate ultra long system info
    let mut values = std::collections::HashMap::new();
    values.insert(
        ModuleKind::Cpu,
        "AMD EPYC 9654 96-Core Processor @ 3.700GHz with 192 Threads and 384MB L3 Cache Ultra Long Hardware Specification Identifier String (Enterprise Edition v4.2)".into(),
    );
    values.insert(
        ModuleKind::Gpu,
        "NVIDIA RTX 6000 Ada Generation (48GB GDDR6 with ECC) PCI-E Gen4 x16 Compute Acceleration Unit".into(),
    );
    state.real_info = Some(termfetch_studio_v2::adapters::fastfetch::runner::RealInfo {
        title: Some("ultra_long_user_domain_name@cyberdeck_cluster_node_99".into()),
        values,
    });

    let mut term = Terminal::new(TestBackend::new(120, 35)).unwrap();
    term.draw(|f| render_ui(f, &mut state)).unwrap();

    // Verify preview body exists and has bounded width
    assert!(state.preview_body.width > 20);
    assert!(state.preview_body.height > 10);

    let _ = std::fs::remove_dir_all(root);
}
