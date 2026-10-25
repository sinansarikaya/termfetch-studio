//! Shell integration: a marked block that runs fastfetch whenever a shell starts.
//! The block is owned by the app: it is added once and replaced when its content changes.

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

fn quote(p: &Path) -> String {
    p.display().to_string().replace('\'', "''")
}

/// PowerShell block. Windows Terminal gets the sixel logo of the studio config; the
/// classic console cannot show sixel, so it gets fastfetch's own logo instead.
/// `exe` is the full fastfetch path when fastfetch is not on PATH (WinGet portable installs).
fn powershell_line(exe: Option<&Path>, config: &Path) -> String {
    let (guard, cmd) = match exe {
        None => (
            "Get-Command fastfetch -ErrorAction SilentlyContinue".to_string(),
            "fastfetch".to_string(),
        ),
        Some(p) => (
            format!("Test-Path -LiteralPath '{}'", quote(p)),
            format!("& '{}'", quote(p)),
        ),
    };
    let cfg = quote(config);
    format!(
        "if ({guard}) {{ if ($env:WT_SESSION) {{ {cmd} -c '{cfg}' }} else {{ {cmd} -c '{cfg}' --logo-type builtin }} }}"
    )
}

fn posix_line(exe: Option<&Path>, config: &Path) -> String {
    let cmd = match exe {
        None => "fastfetch".to_string(),
        Some(p) => format!("'{}'", p.display().to_string().replace('\'', r"'\''")),
    };
    let cfg = config.display().to_string().replace('\'', r"'\''");
    format!("{cmd} -c '{cfg}'")
}

/// Startup files that should run fastfetch on this system.
pub fn targets(
    os: Os,
    home: &Path,
    documents: Option<&Path>,
    shell: &str,
    exe: Option<&Path>,
    config: &Path,
) -> Vec<Target> {
    let mut out = Vec::new();
    if os == Os::Windows {
        if let Some(docs) = documents {
            let line = powershell_line(exe, config);
            out.push(Target {
                path: docs
                    .join("PowerShell")
                    .join("Microsoft.PowerShell_profile.ps1"),
                line: line.clone(),
            });
            // Windows PowerShell 5.1 only when it is actually set up on this machine.
            let legacy = docs.join("WindowsPowerShell");
            if legacy.is_dir() {
                out.push(Target {
                    path: legacy.join("Microsoft.PowerShell_profile.ps1"),
                    line,
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
            line: posix_line(exe, config),
        });
    }
    out
}

/// Write the block to `path`: appended when missing, replaced when its content differs.
/// Returns whether the file changed.
pub fn add_block(path: &Path, line: &str) -> io::Result<bool> {
    let existing = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e),
    };
    let block = format!("{BEGIN}\n{line}\n{END}\n");
    let text = match (existing.find(BEGIN), existing.find(END)) {
        (Some(start), Some(end)) if end > start => {
            let after = end + END.len();
            let after = existing[after..]
                .find('\n')
                .map_or(existing.len(), |i| after + i + 1);
            format!("{}{block}{}", &existing[..start], &existing[after..])
        }
        _ => {
            let mut text = existing.clone();
            if !text.is_empty() && !text.ends_with('\n') {
                text.push('\n');
            }
            text.push_str(&block);
            text
        }
    };
    if text == existing {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, text)?;
    Ok(true)
}

/// Add or refresh the block in every startup file of this system. Returns the files changed.
pub fn integrate(config: &Path) -> io::Result<Vec<PathBuf>> {
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
        config,
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

    const CFG: &str = r"C:\Users\x\.config\termfetch-studio\fastfetch.jsonc";

    #[test]
    fn windows_targets_follow_documents_folder() {
        let docs = Path::new("C:/Users/x/OneDrive/Documents");
        let t = targets(
            Os::Windows,
            Path::new("C:/Users/x"),
            Some(docs),
            "",
            None,
            Path::new(CFG),
        );
        assert_eq!(
            t[0].path,
            docs.join("PowerShell")
                .join("Microsoft.PowerShell_profile.ps1")
        );
        assert!(t.iter().all(|t| t.line.contains("fastfetch")));
    }

    #[test]
    fn powershell_picks_the_logo_per_host() {
        let line = powershell_line(None, Path::new(CFG));
        assert!(line.contains("Get-Command fastfetch"));
        assert!(line.contains("if ($env:WT_SESSION)"));
        assert!(line.contains("--logo-type builtin"));
        let exe = Path::new(r"C:\WinGet\fastfetch.exe");
        let line = powershell_line(Some(exe), Path::new(CFG));
        assert!(line.contains(r"Test-Path -LiteralPath 'C:\WinGet\fastfetch.exe'"));
        assert!(line.contains(r"& 'C:\WinGet\fastfetch.exe' -c"));
    }

    #[test]
    fn posix_targets_follow_the_login_shell() {
        let home = Path::new("/home/u");
        let cfg = Path::new("/home/u/.config/termfetch-studio/fastfetch.jsonc");
        let zsh = targets(Os::Linux, home, None, "/bin/zsh", None, cfg);
        assert_eq!(zsh[0].path, home.join(".zshrc"));
        assert_eq!(
            zsh[0].line,
            "fastfetch -c '/home/u/.config/termfetch-studio/fastfetch.jsonc'"
        );
        assert_eq!(
            targets(Os::Linux, home, None, "/usr/bin/fish", None, cfg)[0].path,
            home.join(".config/fish/config.fish")
        );
        assert_eq!(
            targets(Os::Linux, home, None, "/bin/bash", None, cfg)[0].path,
            home.join(".bashrc")
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
    fn stale_block_is_replaced_and_surrounding_text_kept() {
        let dir = temp("replace");
        let file = dir.join("profile.ps1");
        fs::write(&file, format!("before\n{BEGIN}\nold line\n{END}\nafter\n")).unwrap();
        assert!(add_block(&file, "new line").unwrap());
        assert_eq!(
            fs::read_to_string(&file).unwrap(),
            format!("before\n{BEGIN}\nnew line\n{END}\nafter\n")
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn creates_missing_profile_directories() {
        let dir = temp("create");
        let file = dir
            .join("PowerShell")
            .join("Microsoft.PowerShell_profile.ps1");
        assert!(add_block(&file, &powershell_line(None, Path::new(CFG))).unwrap());
        assert!(
            fs::read_to_string(&file)
                .unwrap()
                .contains("Get-Command fastfetch")
        );
        let _ = fs::remove_dir_all(dir);
    }
}
