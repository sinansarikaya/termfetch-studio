//! Application-menu launchers: a freedesktop `.desktop` entry on Linux and a double-clickable
//! `.command` file on macOS. Both only ever replace or remove files that we wrote.

use anyhow::{Result, bail};
use std::path::{Path, PathBuf};

/// The entry for `exe`. Exec lines are quoted so paths with spaces work.
pub fn entry(exe: &Path) -> String {
    let exe = exe.display().to_string().replace('"', r#"\""#);
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=TermFetch\n\
         Comment=Customize your terminal, your way\n\
         Exec=\"{exe}\"\n\
         Icon=termfetch-studio\n\
         Terminal=true\n\
         Categories=Utility;\n\
         Actions=Settings;\n\
         \n\
         [Desktop Action Settings]\n\
         Name=Settings\n\
         Exec=\"{exe}\" settings\n"
    )
}

/// Embedded source also works for a standalone executable.
pub const ICON: &str = include_str!("../../assets/brand/termfetch-app-icon.svg");

fn write_icon(path: &Path) -> Result<bool> {
    match std::fs::read(path) {
        Ok(data) if data == ICON.as_bytes() => return Ok(false),
        Ok(_) => bail!("{} exists and is not ours; left in place", path.display()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, ICON)?;
    Ok(true)
}

/// Where the entry lives for the current user (XDG data home).
pub fn path() -> Option<PathBuf> {
    let dirs = directories::BaseDirs::new()?;
    Some(
        dirs.data_dir()
            .join("applications")
            .join("termfetch-studio.desktop"),
    )
}

/// Write the entry when it differs. Returns whether the file changed.
pub fn write(path: &Path, exe: &Path) -> Result<bool> {
    // Keep the icon beside this user-owned launcher; package icons use the theme name.
    let icon = path.with_extension("svg");
    let icon_changed = write_icon(&icon)?;
    let text = entry(exe).replace("Icon=termfetch-studio", &format!("Icon={}", icon.display()));
    if std::fs::read_to_string(path).is_ok_and(|t| t == text) {
        return Ok(icon_changed);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, text)?;
    Ok(true)
}

/// macOS: a `.command` file in ~/Applications; double-clicking it opens TermFetch in Terminal.
pub const COMMAND_MARK: &str = "# termfetch-studio launcher";

pub fn command_text() -> String {
    format!("#!/bin/sh\n{COMMAND_MARK}\nexec termfetch-studio\n")
}

pub fn command_path() -> Option<PathBuf> {
    let dirs = directories::BaseDirs::new()?;
    Some(
        dirs.home_dir()
            .join("Applications")
            .join("TermFetch.command"),
    )
}

/// Write the launcher, executable. Returns whether the file changed.
pub fn write_command(path: &Path) -> Result<bool> {
    let text = command_text();
    if std::fs::read_to_string(path).is_ok_and(|t| t == text) {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, text)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))?;
    }
    Ok(true)
}

/// Remove a file only when it is ours (it carries our marker). Returns whether it was removed.
pub fn remove_if_ours(path: &Path, marker: &str) -> Result<bool> {
    match std::fs::read_to_string(path) {
        Ok(text) if text.contains(marker) => {
            std::fs::remove_file(path)?;
            if path
                .file_name()
                .is_some_and(|n| n == "termfetch-studio.desktop")
            {
                let icon = path.with_extension("svg");
                if std::fs::read(&icon).is_ok_and(|data| data == ICON.as_bytes()) {
                    std::fs::remove_file(icon)?;
                }
            }
            Ok(true)
        }
        Ok(_) => bail!("{} exists and is not ours; left in place", path.display()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_has_main_exec_and_settings_action() {
        let text = entry(Path::new("/home/u/bin/termfetch-studio"));
        assert!(text.contains("Exec=\"/home/u/bin/termfetch-studio\"\n"));
        assert!(text.contains("Terminal=true"));
        assert!(text.contains("Actions=Settings;"));
        assert!(text.contains("[Desktop Action Settings]\nName=Settings\nExec=\"/home/u/bin/termfetch-studio\" settings\n"));
    }

    #[test]
    fn quotes_in_paths_are_escaped() {
        let text = entry(Path::new(r#"/opt/a"b/termfetch-studio"#));
        assert!(text.contains(r#"Exec="/opt/a\"b/termfetch-studio""#));
    }

    #[test]
    fn command_launcher_is_written_once_and_only_removed_when_ours() {
        let dir = std::env::temp_dir().join(format!("tfs-command-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let file = dir.join("TermFetch.command");
        assert!(write_command(&file).unwrap());
        assert!(!write_command(&file).unwrap());
        assert!(command_text().contains("exec termfetch-studio"));
        assert!(remove_if_ours(&file, COMMAND_MARK).unwrap());
        assert!(!remove_if_ours(&file, COMMAND_MARK).unwrap());
        std::fs::write(
            &file,
            "#!/bin/sh
echo mine
",
        )
        .unwrap();
        assert!(
            remove_if_ours(&file, COMMAND_MARK).is_err(),
            "someone else's file stays"
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn write_is_idempotent() {
        let dir = std::env::temp_dir().join(format!("tfs-desktop-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let file = dir.join("termfetch-studio.desktop");
        let exe = Path::new("/usr/bin/termfetch-studio");
        assert!(write(&file, exe).unwrap());
        assert!(!write(&file, exe).unwrap());
        let _ = std::fs::remove_dir_all(dir);
    }
}
