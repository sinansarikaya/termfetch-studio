//! Shell integration: a marked block that runs fastfetch whenever a shell starts.
//! The block is added once and only touches the lines between its markers.

use crate::adapters::install::{Os, current_os};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const BEGIN: &str = "# >>> termfetch-studio >>>";
pub const END: &str = "# <<< termfetch-studio <<<";
/// Guarded so a missing fastfetch does not print an error in every new PowerShell.
const POWERSHELL_LINE: &str =
    "if (Get-Command fastfetch -ErrorAction SilentlyContinue) { fastfetch }";
const POSIX_LINE: &str = "fastfetch";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub path: PathBuf,
    pub line: &'static str,
}

/// Startup files that should run fastfetch on this system.
pub fn targets(os: Os, home: &Path, documents: Option<&Path>, shell: &str) -> Vec<Target> {
    let mut out = Vec::new();
    if os == Os::Windows {
        if let Some(docs) = documents {
            out.push(Target {
                path: docs
                    .join("PowerShell")
                    .join("Microsoft.PowerShell_profile.ps1"),
                line: POWERSHELL_LINE,
            });
            // Windows PowerShell 5.1 only when it is actually set up on this machine.
            let legacy = docs.join("WindowsPowerShell");
            if legacy.is_dir() {
                out.push(Target {
                    path: legacy.join("Microsoft.PowerShell_profile.ps1"),
                    line: POWERSHELL_LINE,
                });
            }
        }
    } else if shell.ends_with("zsh") {
        out.push(Target {
            path: home.join(".zshrc"),
            line: POSIX_LINE,
        });
    } else if shell.ends_with("fish") {
        out.push(Target {
            path: home.join(".config").join("fish").join("config.fish"),
            line: POSIX_LINE,
        });
    } else {
        out.push(Target {
            path: home.join(".bashrc"),
            line: POSIX_LINE,
        });
    }
    out
}

/// Append the block to `path` unless it is already there. Returns whether it was written.
pub fn add_block(path: &Path, line: &str) -> io::Result<bool> {
    let existing = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e),
    };
    if existing.contains(BEGIN) {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut text = existing;
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str(&format!("{BEGIN}\n{line}\n{END}\n"));
    fs::write(path, text)?;
    Ok(true)
}

/// Add the block to every startup file of this system. Returns the files changed.
pub fn integrate() -> io::Result<Vec<PathBuf>> {
    let dirs = directories::BaseDirs::new();
    let home = dirs
        .as_ref()
        .map(|b| b.home_dir().to_path_buf())
        .ok_or_else(|| io::Error::other("home directory unknown"))?;
    let documents =
        directories::UserDirs::new().and_then(|u| u.document_dir().map(Path::to_path_buf));
    let shell = std::env::var("SHELL").unwrap_or_default();
    let mut changed = Vec::new();
    for target in targets(current_os(), &home, documents.as_deref(), &shell) {
        if add_block(&target.path, target.line)? {
            changed.push(target.path);
        }
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tfs-shell-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn windows_targets_follow_documents_folder() {
        let docs = Path::new("C:/Users/x/OneDrive/Documents");
        let t = targets(Os::Windows, Path::new("C:/Users/x"), Some(docs), "");
        assert_eq!(
            t[0].path,
            docs.join("PowerShell")
                .join("Microsoft.PowerShell_profile.ps1")
        );
        assert!(t.iter().all(|t| t.line.contains("fastfetch")));
    }

    #[test]
    fn posix_targets_follow_the_login_shell() {
        let home = Path::new("/home/u");
        assert_eq!(
            targets(Os::Linux, home, None, "/bin/zsh")[0].path,
            home.join(".zshrc")
        );
        assert_eq!(
            targets(Os::Linux, home, None, "/usr/bin/fish")[0].path,
            home.join(".config/fish/config.fish")
        );
        assert_eq!(
            targets(Os::Linux, home, None, "/bin/bash")[0].path,
            home.join(".bashrc")
        );
    }

    #[test]
    fn block_is_added_once_and_keeps_existing_lines() {
        let dir = temp("block");
        let file = dir.join("profile.ps1");
        fs::write(&file, "Set-Alias ll ls").unwrap();
        assert!(add_block(&file, POSIX_LINE).unwrap());
        assert!(
            !add_block(&file, POSIX_LINE).unwrap(),
            "second run is a no-op"
        );
        let text = fs::read_to_string(&file).unwrap();
        assert!(text.starts_with("Set-Alias ll ls\n"));
        assert_eq!(text.matches(BEGIN).count(), 1);
        assert!(text.contains(&format!("{BEGIN}\nfastfetch\n{END}\n")));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn creates_missing_profile_directories() {
        let dir = temp("create");
        let file = dir
            .join("PowerShell")
            .join("Microsoft.PowerShell_profile.ps1");
        assert!(add_block(&file, POWERSHELL_LINE).unwrap());
        assert!(fs::read_to_string(&file).unwrap().contains(POWERSHELL_LINE));
        let _ = fs::remove_dir_all(dir);
    }
}
