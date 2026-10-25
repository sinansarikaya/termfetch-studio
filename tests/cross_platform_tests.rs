//! Cross-platform compatibility and automated validation tests.
//!
//! Validates path handling, executable resolution, atomic file operations,
//! terminal emulation lifecycle, image protocol detection, and mock vs real Fastfetch behaviors
//! across Linux, macOS, and Windows environments.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use std::fs;
use termfetch_studio_v2::adapters::fastfetch::config::detect_image_protocol;
use termfetch_studio_v2::adapters::fastfetch::runner::{self, find_fastfetch, parse_probe_output};
use termfetch_studio_v2::adapters::fsutil::atomic_write;
use termfetch_studio_v2::adapters::sysinfo::SysInfo;
use termfetch_studio_v2::app::actions::Action;
use termfetch_studio_v2::app::reducer::reduce;
use termfetch_studio_v2::app::state::{AppState, Focus, Screen};
use termfetch_studio_v2::domain::preset::ModuleKind;
use termfetch_studio_v2::domain::settings::Settings;
use termfetch_studio_v2::imaging::TermGeometry;
use termfetch_studio_v2::paths::Paths;
use termfetch_studio_v2::tui::render::render_ui;

#[test]
fn test_paths_sandboxing_and_targets() {
    let temp_root = std::env::temp_dir().join(format!("tfs-test-paths-{}", std::process::id()));
    let paths = Paths::with_root(&temp_root);

    assert!(paths.config_dir.starts_with(&temp_root));
    assert!(paths.data_dir.starts_with(&temp_root));
    assert!(paths.cache_dir.starts_with(&temp_root));
    assert!(paths.fastfetch_config.starts_with(&temp_root));

    let targets = paths.config_targets();
    assert_eq!(targets.len(), 2);
    assert_eq!(targets[0], paths.studio_config());
    assert_eq!(targets[1], paths.fastfetch_config);

    assert_eq!(
        paths.settings_file(),
        paths.config_dir.join("settings.json")
    );
    assert_eq!(paths.backups_dir(), paths.config_dir.join("backups"));
    assert_eq!(
        paths.preview_config(),
        paths.cache_dir.join("preview.jsonc")
    );
}

#[test]
fn test_atomic_write_cross_platform_overwrites_and_cleans_up() {
    let dir = std::env::temp_dir().join(format!("tfs-atomic-test-{}", std::process::id()));
    let target_file = dir.join("nested_dir").join("config.jsonc");

    // 1. Initial write creates parent directories and file
    let initial_data = b"{\"first\": true}";
    atomic_write(&target_file, initial_data).expect("initial atomic write failed");
    assert_eq!(fs::read(&target_file).expect("read initial"), initial_data);

    // 2. Subsequent write safely overwrites destination
    let updated_data = b"{\"second\": true, \"version\": 2}";
    atomic_write(&target_file, updated_data).expect("second atomic write failed");
    assert_eq!(fs::read(&target_file).expect("read updated"), updated_data);

    // 3. Verify no dangling temporary files remain in the target directory
    let parent = target_file.parent().unwrap();
    let entries: Vec<_> = fs::read_dir(parent)
        .expect("read parent dir")
        .flatten()
        .collect();
    assert_eq!(
        entries.len(),
        1,
        "only target file should remain in directory"
    );

    // Clean up
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn test_fastfetch_binary_resolution_with_env_override() {
    let temp_dir = std::env::temp_dir().join(format!("tfs-fake-bin-{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();
    let fake_bin = temp_dir.join(if cfg!(windows) {
        "fake_fastfetch.exe"
    } else {
        "fake_fastfetch"
    });
    fs::write(&fake_bin, b"fake binary").unwrap();

    // Set FASTFETCH_BIN and test discovery
    unsafe {
        std::env::set_var("FASTFETCH_BIN", &fake_bin);
    }
    let found = find_fastfetch();
    assert_eq!(found, Some(fake_bin.clone()));

    // Clean up env
    unsafe {
        std::env::remove_var("FASTFETCH_BIN");
    }
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_probe_output_parser_with_crlf_and_lf() {
    let sep = '\u{1f}';
    let unix_output =
        format!("title{sep}alice@archlinux\nos{sep}Arch Linux x86_64\ncpu{sep}AMD Ryzen 9\n");
    let windows_output =
        format!("title{sep}bob@winhost\r\nos{sep}Windows 11 Pro\r\ncpu{sep}Intel Core i9\r\n");

    let unix_info = parse_probe_output(&unix_output);
    assert_eq!(unix_info.title.as_deref(), Some("alice@archlinux"));
    assert_eq!(unix_info.get(ModuleKind::Os), Some("Arch Linux x86_64"));
    assert_eq!(unix_info.get(ModuleKind::Cpu), Some("AMD Ryzen 9"));

    let win_info = parse_probe_output(&windows_output);
    assert_eq!(win_info.title.as_deref(), Some("bob@winhost"));
    assert_eq!(win_info.get(ModuleKind::Os), Some("Windows 11 Pro"));
    assert_eq!(win_info.get(ModuleKind::Cpu), Some("Intel Core i9"));
}

#[test]
fn test_sysinfo_icons_and_wallclock_offsets() {
    let check_icon = |ids: Vec<&str>| -> &'static str {
        let s = SysInfo {
            distro_ids: ids.into_iter().map(String::from).collect(),
            ..Default::default()
        };
        s.distro_icon()
    };

    assert_eq!(check_icon(vec!["arch"]), "\u{f08c7}");
    assert_eq!(check_icon(vec!["ubuntu"]), "\u{f0548}");
    assert_eq!(check_icon(vec!["debian"]), "\u{f08da}");
    assert_eq!(check_icon(vec!["fedora"]), "\u{f08db}");
    assert_eq!(check_icon(vec!["opensuse-tumbleweed", "suse"]), "\u{f08e8}");
    assert_eq!(check_icon(vec!["alpine"]), "\u{f08cd}");
    assert_eq!(check_icon(vec!["nixos"]), "\u{f08e4}");
    assert_eq!(check_icon(vec!["macos"]), "\u{f0035}");
    assert_eq!(check_icon(vec!["windows"]), "\u{f05be}");

    // Verify clock formatting with various offsets
    let clock_with_offset = |offset: i64| -> String {
        let s = SysInfo {
            utc_offset: offset,
            ..Default::default()
        };
        s.clock()
    };

    let clock_utc = clock_with_offset(0);
    assert_eq!(clock_utc.len(), 5);
    assert!(clock_utc.contains(':'));

    let clock_plus3 = clock_with_offset(3600 * 3);
    assert_eq!(clock_plus3.len(), 5);

    let clock_minus8 = clock_with_offset(-3600 * 8);
    assert_eq!(clock_minus8.len(), 5);
}

#[test]
fn test_image_protocol_detection() {
    // Test terminal protocol detection fallback logic
    let proto = detect_image_protocol();
    assert!(
        ["kitty-direct", "iterm", "sixel", "chafa"].contains(&proto),
        "Detected protocol '{proto}' must be one of the known supported protocols"
    );
}

#[test]
fn test_headless_pty_terminal_render_lifecycle() {
    let temp_root = std::env::temp_dir().join(format!("tfs-test-headless-{}", std::process::id()));
    let paths = Paths::with_root(&temp_root);
    let geo = TermGeometry::from_window(120, 35, 1200, 700);
    let mut state = AppState::new(paths, Settings::default(), geo);

    // 1. Initial terminal backend creation (standard 120x35)
    let backend = TestBackend::new(120, 35);
    let mut terminal = Terminal::new(backend).unwrap();

    // 2. Render all screens in sequence to verify headless rendering stability
    for screen in [
        Screen::Dashboard,
        Screen::Themes,
        Screen::Presets,
        Screen::Logos,
        Screen::Modules,
        Screen::Settings,
        Screen::Backup,
        Screen::Doctor,
    ] {
        state.screen = screen;
        terminal.draw(|f| render_ui(f, &mut state)).unwrap();
    }

    // 3. Test terminal resize event simulation (responsive scaling)
    terminal.backend_mut().resize(80, 24);
    terminal.draw(|f| render_ui(f, &mut state)).unwrap();

    terminal.backend_mut().resize(180, 60);
    terminal.draw(|f| render_ui(f, &mut state)).unwrap();

    // 4. Test state transitions via Reducer
    reduce(&mut state, Action::ToggleFocus);
    assert_eq!(state.focus, Focus::Content);

    reduce(&mut state, Action::ToggleHelp);
    assert!(state.help_open);

    reduce(&mut state, Action::ToggleHelp);
    assert!(!state.help_open);

    // Clean up
    let _ = fs::remove_dir_all(&temp_root);
}

#[test]
fn test_real_vs_mock_fastfetch_execution() {
    let version_result = runner::version();
    if let Some(ver) = version_result {
        // Real fastfetch is present in environment
        assert!(
            ver.to_lowercase().contains("fastfetch"),
            "Version output should identify fastfetch: {ver}"
        );

        let temp_root = std::env::temp_dir().join(format!("tfs-real-probe-{}", std::process::id()));
        let paths = Paths::with_root(&temp_root);
        let probe_file = paths.cache_dir.join("test_probe.jsonc");

        let modules = [ModuleKind::Os, ModuleKind::Cpu];
        let real_info = runner::probe_real_info(&modules, &probe_file);
        assert!(
            real_info.is_ok(),
            "Real fastfetch probing must succeed when binary is present"
        );
        let _ = fs::remove_dir_all(&temp_root);
    } else {
        // Fastfetch is not installed (mock fallback verification)
        let mock_output = "title\u{1f}user@host\nos\u{1f}Mock Linux\ncpu\u{1f}Mock CPU\n";
        let mock_info = parse_probe_output(mock_output);
        assert_eq!(mock_info.title.as_deref(), Some("user@host"));
        assert_eq!(mock_info.get(ModuleKind::Os), Some("Mock Linux"));
        assert_eq!(mock_info.get(ModuleKind::Cpu), Some("Mock CPU"));
    }
}
