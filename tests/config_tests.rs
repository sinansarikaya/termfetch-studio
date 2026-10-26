use std::path::PathBuf;
use termfetch_studio_v2::adapters::fastfetch::config::{self, LogoSpec, build_config, to_jsonc};
use termfetch_studio_v2::domain::color::ColorValue;
use termfetch_studio_v2::domain::logo::builtin_logos;
use termfetch_studio_v2::domain::preset::{DEMO_DATA, ModuleKind, Preset, get_builtin_presets};
use termfetch_studio_v2::domain::settings::Settings;
use termfetch_studio_v2::domain::theme::get_builtin_themes;
use termfetch_studio_v2::imaging::TermGeometry;
use termfetch_studio_v2::paths::Paths;

fn sandbox(name: &str) -> (Paths, PathBuf) {
    let root = std::env::temp_dir().join(format!("tfs-it-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    (Paths::with_root(&root), root)
}

fn parse(text: &str) -> serde_json::Value {
    let v = jsonc_parser::parse_to_value(text, &Default::default())
        .expect("valid JSONC")
        .expect("non-empty");
    // Round-trip through serde_json for convenient indexing.
    serde_json::from_str(&json_of(&v)).unwrap()
}

fn json_of(v: &jsonc_parser::JsonValue) -> String {
    use jsonc_parser::JsonValue::*;
    match v {
        Null => "null".into(),
        Boolean(b) => b.to_string(),
        Number(n) => n.to_string(),
        String(s) => serde_json::to_string(s.as_ref()).unwrap(),
        Array(a) => format!("[{}]", a.iter().map(json_of).collect::<Vec<_>>().join(",")),
        Object(o) => format!(
            "{{{}}}",
            o.clone()
                .into_iter()
                .map(|(k, v)| format!("{}:{}", serde_json::to_string(&k).unwrap(), json_of(&v)))
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}

fn modules(cfg: &serde_json::Value) -> Vec<&serde_json::Value> {
    cfg["modules"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m.is_object())
        .collect()
}

#[test]
fn image_logo_config_is_valid_and_written_atomically_to_both_targets() {
    let (paths, root) = sandbox("apply");
    let themes = get_builtin_themes();
    let presets = get_builtin_presets();
    let logos = builtin_logos(&paths);
    let latte = themes.iter().find(|t| t.id == "catppuccin-latte").unwrap();
    let pikachu = logos.iter().find(|l| l.id == "pikachu").unwrap();
    let geo = TermGeometry::from_window(200, 50, 2000, 1000);

    let written = config::apply(
        latte,
        &presets[0],
        pikachu,
        &Settings::default(),
        &paths,
        &geo,
    )
    .unwrap();
    assert_eq!(written.len(), 2);
    let a = std::fs::read_to_string(&written[0]).unwrap();
    let b = std::fs::read_to_string(&written[1]).unwrap();
    assert_eq!(a, b);
    assert!(a.starts_with("// TermFetch Studio"));

    let cfg = parse(&a);
    let logo = &cfg["logo"];
    let src = logo["source"].as_str().unwrap();
    assert!(
        src.starts_with(paths.images_dir().to_str().unwrap()),
        "image copied into config dir: {src}"
    );
    assert!(std::path::Path::new(src).is_file());
    let (w, h) = (
        logo["width"].as_u64().unwrap(),
        logo["height"].as_u64().unwrap(),
    );
    // pikachu.png aspect with 10x20 cells: displayed ratio must match within 5%.
    let (pw, ph) =
        termfetch_studio_v2::imaging::image_dimensions(std::path::Path::new(src)).unwrap();
    let shown = (w as f64 * 10.0) / (h as f64 * 20.0);
    assert!(
        (shown / (pw as f64 / ph as f64) - 1.0).abs() <= 0.05,
        "{w}x{h} for {pw}x{ph}"
    );
    assert!(w <= 36 && h <= 22);

    // Theme group colours land in keyColor.
    let os = modules(&cfg)
        .into_iter()
        .find(|m| m["type"] == "os")
        .unwrap()
        .clone();
    assert_eq!(os["keyColor"], latte.groups.os.as_str());
    assert_eq!(os["outputColor"], latte.text.as_str());
    assert_eq!(os["keyWidth"], 10);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn title_has_no_key_and_demo_mode_controls_formats() {
    let themes = get_builtin_themes();
    let full = &get_builtin_presets()[0];
    let mut s = Settings {
        demo_mode: true,
        ..Default::default()
    };

    let demo = parse(
        &to_jsonc(
            &build_config(&themes[0], full, &LogoSpec::Auto, &s),
            &themes[0],
            full,
            &s,
        )
        .unwrap(),
    );
    let title = modules(&demo)
        .into_iter()
        .find(|m| m["type"] == "title")
        .unwrap()
        .clone();
    assert!(
        title.get("key").is_none(),
        "an empty key makes fastfetch print 'Title ➜'"
    );
    assert!(title["format"].as_str().unwrap().contains("cyberdeck"));
    let os = modules(&demo)
        .into_iter()
        .find(|m| m["type"] == "os")
        .unwrap()
        .clone();
    assert_eq!(os["format"], DEMO_DATA.os);

    s.demo_mode = false;
    let real = parse(
        &to_jsonc(
            &build_config(&themes[0], full, &LogoSpec::Auto, &s),
            &themes[0],
            full,
            &s,
        )
        .unwrap(),
    );
    assert!(
        modules(&real)
            .iter()
            .filter(|m| m["type"] != "title" && m["type"] != "custom")
            .all(|m| m.get("format").is_none())
    );
    let title = modules(&real)
        .into_iter()
        .find(|m| m["type"] == "title")
        .unwrap()
        .clone();
    assert!(title["format"].as_str().unwrap().contains("{user-name}"));
}

#[test]
fn colour_settings_override_theme() {
    let themes = get_builtin_themes();
    let p = Preset::custom(&[ModuleKind::Cpu, ModuleKind::LocalIp]);
    let mut s = Settings {
        icon_color: ColorValue::Ansi(34),
        text_color: ColorValue::Hex("#c0c0c0".into()),
        ..Default::default()
    };
    s.group_colors.network = ColorValue::Hex("#ff8c00".into());
    let cfg = parse(
        &to_jsonc(
            &build_config(&themes[0], &p, &LogoSpec::Auto, &s),
            &themes[0],
            &p,
            &s,
        )
        .unwrap(),
    );
    let ms = modules(&cfg);
    let cpu = ms.iter().find(|m| m["type"] == "cpu").unwrap();
    assert_eq!(cpu["keyColor"], "34");
    assert_eq!(cpu["outputColor"], "#c0c0c0");
    let ip = ms.iter().find(|m| m["type"] == "localip").unwrap();
    assert_eq!(ip["keyColor"], "#ff8c00");
    assert_eq!(ip["outputColor"], "#ff8c00");
    // cpu and localip are different groups -> separated by a break.
    let all = cfg["modules"].as_array().unwrap();
    let ci = all.iter().position(|m| m["type"] == "cpu").unwrap();
    assert_eq!(all[ci + 1], "break");
}

#[test]
fn ascii_logo_uses_file_type_with_theme_colour() {
    let (paths, root) = sandbox("ascii");
    let themes = get_builtin_themes();
    let logos = builtin_logos(&paths);
    let tux = logos.iter().find(|l| l.id == "tux").unwrap();
    let s = Settings::default();
    let geo = TermGeometry::from_window(120, 40, 0, 0);
    let spec = config::resolve_logo_spec(tux, &s, &paths, &geo).unwrap();
    let LogoSpec::Text { source } = &spec else {
        panic!("{spec:?}")
    };
    assert_eq!(std::fs::read_to_string(source).unwrap(), tux.art);
    let cfg = build_config(&themes[0], &get_builtin_presets()[1], &spec, &s);
    assert_eq!(cfg.logo.kind, "file");
    assert_eq!(cfg.logo.color.as_ref().unwrap()["1"], themes[0].primary);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn foreign_fastfetch_config_is_backed_up_before_first_overwrite() {
    let (paths, root) = sandbox("foreign");
    std::fs::create_dir_all(paths.fastfetch_config.parent().unwrap()).unwrap();
    std::fs::write(&paths.fastfetch_config, "{ // mine\n}").unwrap();
    config::write_targets(&paths, "// TermFetch Studio\n{}").unwrap();
    let saved: Vec<_> = std::fs::read_dir(paths.backups_dir())
        .unwrap()
        .flatten()
        .collect();
    assert_eq!(saved.len(), 1);
    assert_eq!(
        std::fs::read_to_string(saved[0].path()).unwrap(),
        "{ // mine\n}"
    );
    // Our own config is not backed up again.
    config::write_targets(&paths, "// TermFetch Studio\n{}").unwrap();
    assert_eq!(std::fs::read_dir(paths.backups_dir()).unwrap().count(), 1);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn generated_config_runs_in_fastfetch_when_available() {
    if termfetch_studio_v2::adapters::fastfetch::runner::version().is_none() {
        return;
    }
    let (paths, root) = sandbox("ff");
    let themes = get_builtin_themes();
    let presets = get_builtin_presets();
    let logos = builtin_logos(&paths);
    let geo = TermGeometry::from_window(160, 48, 1600, 960);
    for preset in &presets {
        let s = Settings {
            demo_mode: true,
            ..Settings::default()
        };
        let text = config::render(&themes[0], preset, &logos[0], &s, &paths, &geo).unwrap();
        let file = root.join(format!("{}.jsonc", preset.id));
        std::fs::write(&file, text).unwrap();
        let out = termfetch_studio_v2::adapters::fastfetch::runner::fastfetch_cmd()
            .arg("-c")
            .arg(&file)
            .arg("--pipe")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}: {}",
            preset.id,
            String::from_utf8_lossy(&out.stderr)
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains("cyberdeck"), "{}: {stdout}", preset.id);
        assert!(
            !stdout.contains("Title"),
            "{}: title must not have a key",
            preset.id
        );
        assert!(stdout.contains(DEMO_DATA.os), "{}: {stdout}", preset.id);
    }
    let _ = std::fs::remove_dir_all(root);
}
