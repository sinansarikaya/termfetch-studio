//! Windows Terminal color scheme: new tabs take the applied theme's background and text.
//! Only the scheme entry and `profiles.defaults.colorScheme` are touched; the original
//! file is kept next to it on the first change.

use crate::adapters::fsutil::atomic_write;
use anyhow::{Context, Result, bail};
use serde_json::{Map, Value, json};
use std::path::{Path, PathBuf};

pub const SCHEME: &str = "TermFetch Studio";
pub const PROFILE: &str = "TermFetch";
/// Stable id, so re-running setup updates the same profile.
pub const PROFILE_GUID: &str = "{6f1d2c9e-3b1a-4e5f-9a7c-7e0d5b2a1c44}";
const BACKUP_SUFFIX: &str = ".termfetch-backup";

/// The colors a scheme needs from a theme.
#[derive(Debug, Clone)]
pub struct Colors {
    pub background: String,
    pub foreground: String,
    pub cursor: String,
    pub selection: String,
}

/// Windows Terminal's settings files (stable and preview packages).
pub fn settings_files() -> Vec<PathBuf> {
    let Some(local) = std::env::var_os("LOCALAPPDATA") else {
        return Vec::new();
    };
    let packages = PathBuf::from(local).join("Packages");
    [
        "Microsoft.WindowsTerminal_8wekyb3d8bbwe",
        "Microsoft.WindowsTerminalPreview_8wekyb3d8bbwe",
    ]
    .iter()
    .map(|pkg| packages.join(pkg).join("LocalState").join("settings.json"))
    .filter(|p| p.is_file())
    .collect()
}

/// The 16 ANSI colors Windows Terminal ships with, so the scheme is complete.
const ANSI: [(&str, &str); 16] = [
    ("black", "#0C0C0C"),
    ("red", "#C50F1F"),
    ("green", "#13A10E"),
    ("yellow", "#C19C00"),
    ("blue", "#0037DA"),
    ("purple", "#881798"),
    ("cyan", "#3A96DD"),
    ("white", "#CCCCCC"),
    ("brightBlack", "#767676"),
    ("brightRed", "#E74856"),
    ("brightGreen", "#16C60C"),
    ("brightYellow", "#F9F1A5"),
    ("brightBlue", "#3B78FF"),
    ("brightPurple", "#B4009E"),
    ("brightCyan", "#61D6D6"),
    ("brightWhite", "#F2F2F2"),
];

fn scheme(c: &Colors) -> Value {
    let mut s = json!({
        "name": SCHEME,
        "background": c.background,
        "foreground": c.foreground,
        "cursorColor": c.cursor,
        "selectionBackground": c.selection,
    });
    for (name, hex) in ANSI {
        s[name] = json!(hex);
    }
    s
}

/// Edit a Windows Terminal settings file in place. `edit` changes the parsed JSON; the
/// file is only rewritten when the result differs. The original is kept on the first change.
/// A file that is not a JSON object is left alone.
fn edit_settings(
    path: &Path,
    edit: impl FnOnce(&mut Map<String, Value>) -> Result<()>,
) -> Result<bool> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let mut root: Value =
        serde_json::from_str(&text).context("Windows Terminal settings are not plain JSON")?;
    let Some(obj) = root.as_object_mut() else {
        bail!("Windows Terminal settings are not an object");
    };
    edit(obj)?;
    let updated = serde_json::to_string_pretty(&root)?;
    if updated == text {
        return Ok(false);
    }
    let backup = PathBuf::from(format!("{}{BACKUP_SUFFIX}", path.display()));
    if !backup.exists() {
        std::fs::write(&backup, &text).with_context(|| format!("writing {}", backup.display()))?;
    }
    atomic_write(path, updated.as_bytes())?;
    Ok(true)
}

/// The `profiles.defaults` object, created when missing.
fn profile_defaults(obj: &mut Map<String, Value>) -> Result<&mut Map<String, Value>> {
    obj.entry("profiles")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .context("\"profiles\" is not an object")?
        .entry("defaults")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .context("\"profiles.defaults\" is not an object")
}

/// Add or update the scheme and make it the default for every profile.
pub fn apply_scheme(path: &Path, colors: &Colors) -> Result<bool> {
    let entry = scheme(colors);
    edit_settings(path, |obj| {
        let schemes = obj
            .entry("schemes")
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .context("\"schemes\" is not a list")?;
        match schemes.iter().position(|s| s["name"] == SCHEME) {
            Some(i) => schemes[i] = entry,
            None => schemes.push(entry),
        }
        profile_defaults(obj)?.insert("colorScheme".into(), json!(SCHEME));
        Ok(())
    })
}

/// Add or update the "TermFetch" profile, so it can be opened from the new-tab menu.
pub fn ensure_profile(path: &Path, exe: &Path) -> Result<bool> {
    let commandline = format!("\"{}\"", exe.display());
    let icon = path.with_file_name("termfetch-studio-icon.png");
    let bytes = include_bytes!("../../assets/brand/termfetch-64.png");
    let profile = json!({
        "name": PROFILE,
        "guid": PROFILE_GUID,
        "commandline": commandline,
        "hidden": false,
        "icon": &icon,
    });
    edit_settings(path, |obj| {
        let list = obj
            .entry("profiles")
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .context("\"profiles\" is not an object")?
            .entry("list")
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .context("\"profiles.list\" is not a list")?;
        match std::fs::read(&icon) {
            Ok(data) if data == bytes => {}
            Ok(_) => bail!("{} exists and is not ours; left in place", icon.display()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => std::fs::write(&icon, bytes)?,
            Err(e) => return Err(e.into()),
        }

        match list.iter().position(|p| p["guid"] == PROFILE_GUID) {
            Some(i) => list[i] = profile,
            None => list.push(profile),
        }
        Ok(())
    })
}

/// Remove the app's profile and scheme, and the default scheme if it is ours.
/// Returns whether the file changed.
pub fn remove(path: &Path) -> Result<bool> {
    let changed = edit_settings(path, |obj| {
        if let Some(list) = obj
            .get_mut("profiles")
            .and_then(|p| p.get_mut("list"))
            .and_then(Value::as_array_mut)
        {
            list.retain(|p| p["guid"] != PROFILE_GUID);
        }
        if let Some(schemes) = obj.get_mut("schemes").and_then(Value::as_array_mut) {
            schemes.retain(|s| s["name"] != SCHEME);
        }
        if let Some(defaults) = obj
            .get_mut("profiles")
            .and_then(|p| p.get_mut("defaults"))
            .and_then(Value::as_object_mut)
            .filter(|d| d.get("colorScheme") == Some(&json!(SCHEME)))
        {
            defaults.remove("colorScheme");
        }
        Ok(())
    })?;
    let icon = path.with_file_name("termfetch-studio-icon.png");
    if std::fs::read(&icon)
        .is_ok_and(|data| data == include_bytes!("../../assets/brand/termfetch-64.png"))
    {
        std::fs::remove_file(icon)?;
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn colors() -> Colors {
        Colors {
            background: "#1e1e2e".into(),
            foreground: "#cdd6f4".into(),
            cursor: "#89b4fa".into(),
            selection: "#45475a".into(),
        }
    }

    fn temp_settings(name: &str, text: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tfs-wt-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        std::fs::write(&path, text).unwrap();
        path
    }

    #[test]
    fn adds_scheme_sets_default_and_keeps_the_rest() {
        let path = temp_settings(
            "add",
            r#"{"defaultProfile":"x","profiles":{"defaults":{"font":{"size":12}},"list":[{"name":"PS"}]},"schemes":[]}"#,
        );
        assert!(apply_scheme(&path, &colors()).unwrap());
        let v: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(v["defaultProfile"], "x");
        assert_eq!(v["profiles"]["list"][0]["name"], "PS");
        assert_eq!(v["profiles"]["defaults"]["font"]["size"], 12);
        assert_eq!(v["profiles"]["defaults"]["colorScheme"], SCHEME);
        let s = &v["schemes"][0];
        assert_eq!(s["name"], SCHEME);
        assert_eq!(s["background"], "#1e1e2e");
        assert_eq!(s["brightWhite"], "#F2F2F2");
        assert!(
            path.with_file_name("settings.json.termfetch-backup")
                .exists()
        );
    }

    #[test]
    fn second_run_is_a_no_op_and_updates_in_place() {
        let path = temp_settings("twice", r#"{"schemes":[],"profiles":{"list":[]}}"#);
        assert!(apply_scheme(&path, &colors()).unwrap());
        assert!(!apply_scheme(&path, &colors()).unwrap());
        let mut changed = colors();
        changed.background = "#000000".into();
        assert!(apply_scheme(&path, &changed).unwrap());
        let v: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let schemes = v["schemes"].as_array().unwrap();
        assert_eq!(schemes.len(), 1, "scheme replaced, not duplicated");
        assert_eq!(schemes[0]["background"], "#000000");
    }

    #[test]
    fn profile_is_added_once_and_other_profiles_stay() {
        let path = temp_settings(
            "profile",
            r#"{"profiles":{"list":[{"name":"Windows PowerShell","guid":"{x}"}]}}"#,
        );
        let exe =
            Path::new(r"C:\Users\x\AppData\Local\Programs\TermFetch Studio\termfetch-studio.exe");
        assert!(ensure_profile(&path, exe).unwrap());
        assert!(!ensure_profile(&path, exe).unwrap());
        let v: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let list = v["profiles"]["list"].as_array().unwrap();
        assert_eq!(list.len(), 2, "existing profile kept, ours added once");
        assert_eq!(list[0]["name"], "Windows PowerShell");
        assert_eq!(list[1]["name"], PROFILE);
        assert_eq!(list[1]["commandline"], format!("\"{}\"", exe.display()));
    }

    #[test]
    fn remove_takes_only_our_profile_and_scheme() {
        let path = temp_settings(
            "remove",
            r#"{"profiles":{"defaults":{"colorScheme":"TermFetch Studio"},"list":[{"name":"PS","guid":"{x}"},{"name":"TermFetch","guid":"{6f1d2c9e-3b1a-4e5f-9a7c-7e0d5b2a1c44}"}]},"schemes":[{"name":"TermFetch Studio"},{"name":"Other"}]}"#,
        );
        assert!(remove(&path).unwrap());
        let v: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(v["profiles"]["list"].as_array().unwrap().len(), 1);
        assert_eq!(v["profiles"]["list"][0]["name"], "PS");
        assert_eq!(v["schemes"].as_array().unwrap().len(), 1);
        assert_eq!(v["schemes"][0]["name"], "Other");
        assert!(v["profiles"]["defaults"].get("colorScheme").is_none());
    }

    #[test]
    fn invalid_json_is_left_untouched() {
        let text = "{ not json";
        let path = temp_settings("invalid", text);
        assert!(apply_scheme(&path, &colors()).is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), text);
    }
}
