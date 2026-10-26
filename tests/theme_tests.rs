use termfetch_studio_v2::domain::logo::get_builtin_logos;
use termfetch_studio_v2::domain::preset::{DEMO_DATA, get_builtin_presets};
use termfetch_studio_v2::domain::theme::get_builtin_themes;

#[test]
fn test_builtin_themes_load_and_have_valid_colors() {
    let themes = get_builtin_themes();
    assert_eq!(themes.len(), 40, "Should contain all 40 themes");

    for theme in &themes {
        assert!(!theme.id.is_empty(), "Theme id must not be empty");
        assert!(!theme.name.is_empty(), "Theme name must not be empty");
        for hex in [
            &theme.primary,
            &theme.secondary,
            &theme.accent,
            &theme.background,
            &theme.text,
        ] {
            assert!(
                matches!(
                    termfetch_studio_v2::domain::theme::Theme::parse_color(hex),
                    ratatui::style::Color::Rgb(..)
                ),
                "{} has invalid colour {hex}",
                theme.id
            );
        }
    }
}

#[test]
fn test_theme_ids_unique() {
    let themes = get_builtin_themes();
    let mut ids: Vec<_> = themes.iter().map(|t| t.id.as_str()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), themes.len());
}

#[test]
fn test_all_categories_represented() {
    let themes = get_builtin_themes();
    assert!(
        themes
            .iter()
            .any(|t| t.id == "minecraft" || t.id == "matrix")
    );
    assert!(themes.iter().any(|t| t.id == "catppuccin-mocha"));
    assert!(
        themes
            .iter()
            .any(|t| t.id == "solarized-light" || t.id == "ayu-light")
    );
}

#[test]
fn test_all_logos_and_images_present() {
    let logos = get_builtin_logos();
    assert_eq!(
        logos.len(),
        31,
        "16 ASCII + 14 preset PNGs + 1 custom image"
    );
    for id in ["eren-1", "tux", "arch", "apple", "link-(zelda)"] {
        assert!(logos.iter().any(|l| l.id == id), "missing logo {id}");
    }
}

#[test]
fn test_preset_image_paths_are_discovered_not_hardcoded() {
    let logos = get_builtin_logos();
    for l in logos
        .iter()
        .filter(|l| l.kind == termfetch_studio_v2::domain::logo::LogoKind::PresetImage)
    {
        let p = l
            .file_path
            .as_deref()
            .expect("assets dir must be discovered in the repo");
        assert!(std::path::Path::new(p).is_file(), "{p} should exist");
    }
}

#[test]
fn test_demo_specs_not_empty_and_privacy_safe() {
    assert_eq!(DEMO_DATA.user_host, "cyberdeck@matrix-station");
    assert_eq!(DEMO_DATA.os, "Arch Linux x86_64");
    assert_eq!(DEMO_DATA.local_ip, "192.168.1.100 / 24");
}

#[test]
fn test_builtin_presets_match_original_installer() {
    let presets = get_builtin_presets();
    let ids: Vec<_> = presets.iter().map(|p| p.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "full",
            "minimal",
            "focused",
            "developer",
            "gaming",
            "custom"
        ]
    );
    assert_eq!(
        presets[0].module_count(),
        25,
        "full preset shows every module"
    );
    assert_eq!(presets[1].module_count(), 4);
    assert!(!presets[0].color_dots && presets[1].color_dots);
}
