//! Shell integration: a marked block that runs fastfetch whenever a shell starts.
//! The block is added once and only touches the lines between its markers.

use crate::adapters::install::{Os, current_os, on_path};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const BEGIN: &str = "# >>> termfetch-studio >>>";
pub const END: &str = "# <<< termfetch-studio <<<";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub path: PathBuf,
    pub line: String,
}

/// The startup line for a shell. `exe` is the full fastfetch path when fastfetch
/// is not on PATH (WinGet portable installs get no PATH entry), `None` otherwise.
fn powershell_line(exe: Option<&Path>) -> String {
    match exe {
        // Guarded so a missing fastfetch does not print an error in every new PowerShell.
        None => "if (Get-Command fastfetch -ErrorAction SilentlyContinue) { fastfetch }".into(),
        Some(p) => {
            let q = p.display().to_string().replace('\'', "''");
            format!("if (Test-Path -LiteralPath '{q}') {{ & '{q}' }}")
        }
    }
}

fn posix_line(exe: Option<&Path>) -> String {
    match exe {
        None => "fastfetch".into(),
        Some(p) => format!("'{}'", p.display().to_string().replace('\'', r"'\''")),
    }
}

/// Startup files that should run fastfetch on this system.
pub fn targets(
    os: Os,
    home: &Path,
    documents: Option<&Path>,
    shell: &str,
    exe: Option<&Path>,
) -> Vec<Target> {
    let mut out = Vec::new();
    if os == Os::Windows {
        if let Some(docs) = documents {
            out.push(Target {
                path: docs
                    .join("PowerShell")
                    .join("Microsoft.PowerShell_profile.ps1"),
                line: powershell_line(exe),
            });
            // Windows PowerShell 5.1 only when it is actually set up on this machine.
            let legacy = docs.join("WindowsPowerShell");
            if legacy.is_dir() {
                out.push(Target {
                    path: legacy.join("Microsoft.PowerShell_profile.ps1"),
                    line: powershell_line(exe),
                });
            }
        }
    } else {
        let path = if shell.ends_with("zsh") {
            home.join(".zshrc")
        } else if shell.ends_with("fish") {
            home.join(".config").join("fish").join("config.fish")
        } else {
            home.join(".bashrc")
        };
        out.push(Target {
            path,
            line: posix_line(exe),
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
    let exe = if on_path("fastfetch") {
        None
    } else {
        crate::adapters::fastfetch::runner::find_fastfetch()
    };
    let mut changed = Vec::new();
    for target in targets(
        current_os(),
        &home,
        documents.as_deref(),
        &shell,
        exe.as_deref(),
    ) {
        if add_block(&target.path, &target.line)? {
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
        let t = targets(Os::Windows, Path::new("C:/Users/x"), Some(docs), "", None);
        assert_eq!(
            t[0].path,
            docs.join("PowerShell")
                .join("Microsoft.PowerShell_profile.ps1")
        );
        assert!(t.iter().all(|t| t.line.contains("fastfetch")));
    }

    #[test]
    fn powershell_uses_full_path_when_not_on_path() {
        let exe = Path::new(r"C:\Users\x\WinGet\fastfetch.exe");
        assert_eq!(
            powershell_line(Some(exe)),
            r"if (Test-Path -LiteralPath 'C:\Users\x\WinGet\fastfetch.exe') { & 'C:\Users\x\WinGet\fastfetch.exe' }"
        );
        assert!(powershell_line(None).contains("Get-Command fastfetch"));
    }

    #[test]
    fn posix_targets_follow_the_login_shell() {
        let home = Path::new("/home/u");
        assert_eq!(
            targets(Os::Linux, home, None, "/bin/zsh", None)[0].path,
            home.join(".zshrc")
        );
        assert_eq!(
            targets(Os::Linux, home, None, "/usr/bin/fish", None)[0].path,
            home.join(".config/fish/config.fish")
        );
        assert_eq!(
            targets(Os::Linux, home, None, "/bin/bash", None)[0].path,
            home.join(".bashrc")
        );
        assert_eq!(
            posix_line(Some(Path::new("/opt/it's/fastfetch"))),
            r"'/opt/it'\''s/fastfetch'"
        );
    }

    #[test]
    fn block_is_added_once_and_keeps_existing_lines() {
        let dir = temp("block");
        let file = dir.join("profile.ps1");
        fs::write(&file, "Set-Alias ll ls").unwrap();
        assert!(add_block(&file, "fastfetch").unwrap());
        assert!(
            !add_block(&file, "fastfetch").unwrap(),
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
        assert!(add_block(&file, &powershell_line(None)).unwrap());
        assert!(
            fs::read_to_string(&file)
                .unwrap()
                .contains("Get-Command fastfetch")
        );
        let _ = fs::remove_dir_all(dir);
    }
}
