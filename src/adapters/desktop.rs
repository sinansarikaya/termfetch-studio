//! Linux application-menu entry (freedesktop `.desktop` file) with a "Settings" action.

use anyhow::Result;
use std::path::{Path, PathBuf};

/// The entry for `exe`. Exec lines are quoted so paths with spaces work.
pub fn entry(exe: &Path) -> String {
    let exe = exe.display().to_string().replace('"', r#"\""#);
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=TermFetch Studio\n\
         Comment=Customize your terminal, your way\n\
         Exec=\"{exe}\"\n\
         Terminal=true\n\
         Categories=Utility;\n\
         Actions=Settings;\n\
         \n\
         [Desktop Action Settings]\n\
         Name=Settings\n\
         Exec=\"{exe}\" settings\n"
    )
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
    let text = entry(exe);
    if std::fs::read_to_string(path).is_ok_and(|t| t == text) {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, text)?;
    Ok(true)
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
