use std::path::Path;
use termfetch_studio_v2::adapters::{desktop, wt_terminal};

fn temp(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("tfs-brand-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn desktop_installs_embedded_icon_and_removes_only_ours() {
    let dir = temp("desktop");
    let launcher = dir.join("termfetch-studio.desktop");
    let icon = launcher.with_extension("svg");
    let _ = std::fs::remove_file(&launcher);
    let _ = std::fs::remove_file(&icon);
    assert!(desktop::write(&launcher, Path::new("/opt/TermFetch Studio/app")).unwrap());
    let text = std::fs::read_to_string(&launcher).unwrap();
    assert!(
        text.lines()
            .any(|line| line == format!("Icon={}", icon.display()))
    );
    assert!(text.contains("Exec=\"/opt/TermFetch Studio/app\""));
    assert_eq!(std::fs::read_to_string(&icon).unwrap(), desktop::ICON);
    assert!(!desktop::write(&launcher, Path::new("/opt/TermFetch Studio/app")).unwrap());
    assert!(desktop::remove_if_ours(&launcher, "Name=TermFetch").unwrap());
    assert!(!icon.exists());
    std::fs::write(&icon, "personal icon").unwrap();
    assert!(desktop::write(&launcher, Path::new("/app")).is_err());
    assert!(!launcher.exists());
    assert_eq!(std::fs::read_to_string(&icon).unwrap(), "personal icon");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn desktop_uninstall_keeps_modified_icon() {
    let dir = temp("modified");
    let launcher = dir.join("termfetch-studio.desktop");
    desktop::write(&launcher, Path::new("/app")).unwrap();
    let icon = launcher.with_extension("svg");
    std::fs::write(&icon, "customized").unwrap();
    desktop::remove_if_ours(&launcher, "Name=TermFetch").unwrap();
    assert_eq!(std::fs::read_to_string(&icon).unwrap(), "customized");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn terminal_profile_uses_embedded_png_and_cleanup_preserves_other_profiles() {
    let dir = temp("wt");
    let settings = dir.join("settings.json");
    std::fs::write(
        &settings,
        r#"{"profiles":{"list":[{"guid":"other","name":"Keep"}]}}"#,
    )
    .unwrap();
    assert!(wt_terminal::ensure_profile(&settings, Path::new("C:/TermFetch/app.exe")).unwrap());
    assert!(!wt_terminal::ensure_profile(&settings, Path::new("C:/TermFetch/app.exe")).unwrap());
    let value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
    let icon = dir.join("termfetch-studio-icon.png");
    assert_eq!(value["profiles"]["list"][1]["icon"], icon.to_str().unwrap());
    assert_eq!(
        std::fs::read(&icon).unwrap(),
        include_bytes!("../assets/brand/termfetch-64.png")
    );
    wt_terminal::remove(&settings).unwrap();
    assert!(!icon.exists());
    let value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
    assert_eq!(value["profiles"]["list"].as_array().unwrap().len(), 1);
    assert_eq!(value["profiles"]["list"][0]["name"], "Keep");
    assert!(
        settings
            .with_file_name("settings.json.termfetch-backup")
            .exists()
    );
    std::fs::write(&icon, b"personal").unwrap();
    assert!(wt_terminal::ensure_profile(&settings, Path::new("app.exe")).is_err());
    wt_terminal::remove(&settings).unwrap();
    assert_eq!(std::fs::read(&icon).unwrap(), b"personal");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn installer_has_embedded_icon_and_shortcut_assignment() {
    let script = include_str!("../scripts/install-windows.ps1");
    assert!(script.contains("$shortcut.IconLocation = \"$iconPath,0\""));
    assert!(script.contains("__TERMFETCH_ICON_BASE64__"));
    assert_eq!(
        &include_bytes!("../assets/brand/termfetch.ico")[..4],
        &[0, 0, 1, 0]
    );
}

#[test]
fn invalid_terminal_settings_do_not_create_an_icon() {
    let dir = temp("invalid");
    let settings = dir.join("settings.json");
    std::fs::write(&settings, "{ invalid").unwrap();
    assert!(wt_terminal::ensure_profile(&settings, Path::new("app.exe")).is_err());
    assert!(!dir.join("termfetch-studio-icon.png").exists());
    assert_eq!(std::fs::read_to_string(&settings).unwrap(), "{ invalid");
    std::fs::remove_dir_all(dir).unwrap();
}
