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

/// Block of the macOS app: its commands go on PATH and `termfetch-studio run` adapts
/// fastfetch to the terminal. Skipped quietly once the app has been deleted.
pub fn bundle_lines(bin_dir: &Path, fish: bool) -> String {
    let q = |p: &Path| {
        let p = p.display().to_string();
        if fish {
            format!("'{}'", p.replace('\\', r"\\").replace('\'', r"\'"))
        } else {
            format!("'{}'", p.replace('\'', r"'\''"))
        }
    };
    let (dir, exe) = (q(bin_dir), q(&bin_dir.join("termfetch-studio")));
    if fish {
        format!("if test -x {exe}\n    set -gx PATH {dir} $PATH\n    {exe} run\nend")
    } else {
        format!("if [ -x {exe} ]; then\n    export PATH={dir}:\"$PATH\"\n    {exe} run\nfi")
    }
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
        } else if os == Os::MacOs {
            // macOS terminals start login shells, which read ~/.bash_profile, not ~/.bashrc.
            home.join(".bash_profile")
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
    let bin_dir = crate::paths::app_bundle_bin_dir();
    let mut changed = Vec::new();
    for target in targets(
        current_os(),
        &home,
        documents.as_deref(),
        &shell,
        exe.as_deref(),
        config,
    ) {
        let line = match &bin_dir {
            Some(dir) => bundle_lines(dir, target.path.ends_with("config.fish")),
            None => target.line,
        };
        if add_block(&target.path, &line)? {
            changed.push(target.path);
        }
    }
    Ok(changed)
}

/// Remove the app's block from `path`. Returns whether the file changed.
pub fn remove_block(path: &Path) -> io::Result<bool> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e),
    };
    let (Some(start), Some(end)) = (text.find(BEGIN), text.find(END)) else {
        return Ok(false);
    };
    if end < start {
        return Ok(false);
    }
    let after = end + END.len();
    let after = text[after..]
        .find('\n')
        .map_or(text.len(), |i| after + i + 1);
    fs::write(path, format!("{}{}", &text[..start], &text[after..]))?;
    Ok(true)
}

/// Remove the app's block from every startup file it may have written to.
pub fn remove_all() -> io::Result<Vec<PathBuf>> {
    let dirs = directories::BaseDirs::new();
    let home = dirs
        .as_ref()
        .map(|b| b.home_dir().to_path_buf())
        .ok_or_else(|| io::Error::other("home directory unknown"))?;
    let documents =
        directories::UserDirs::new().and_then(|u| u.document_dir().map(Path::to_path_buf));
    let mut candidates = vec![
        home.join(".bashrc"),
        home.join(".bash_profile"),
        home.join(".zshrc"),
        home.join(".config").join("fish").join("config.fish"),
    ];
    if let Some(docs) = documents {
        candidates.push(
            docs.join("PowerShell")
                .join("Microsoft.PowerShell_profile.ps1"),
        );
        candidates.push(
            docs.join("WindowsPowerShell")
                .join("Microsoft.PowerShell_profile.ps1"),
        );
    }
    let mut removed = Vec::new();
    for path in candidates {
        if remove_block(&path)? {
            removed.push(path);
        }
    }
    Ok(removed)
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
        let mac_bash = targets(Os::MacOs, home, None, "/bin/bash", None, cfg);
        assert_eq!(mac_bash[0].path, home.join(".bash_profile"));
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

    // The app bundle exists only on macOS; Windows would join the path with `\`.
    #[cfg(unix)]
    #[test]
    fn app_bundle_block_puts_commands_on_path_and_runs_the_adapter() {
        let dir = Path::new("/Applications/TermFetch Studio.app/Contents/MacOS");
        assert_eq!(
            bundle_lines(dir, false),
            "if [ -x '/Applications/TermFetch Studio.app/Contents/MacOS/termfetch-studio' ]; then\n    \
             export PATH='/Applications/TermFetch Studio.app/Contents/MacOS':\"$PATH\"\n    \
             '/Applications/TermFetch Studio.app/Contents/MacOS/termfetch-studio' run\nfi"
        );
        let fish = bundle_lines(dir, true);
        assert!(fish.starts_with(
            "if test -x '/Applications/TermFetch Studio.app/Contents/MacOS/termfetch-studio'\n"
        ));
        assert!(
            fish.contains("set -gx PATH '/Applications/TermFetch Studio.app/Contents/MacOS' $PATH")
        );
        assert!(fish.ends_with(" run\nend"));
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
    fn remove_block_keeps_the_rest_of_the_file() {
        let dir = temp("remove");
        let file = dir.join("profile.ps1");
        fs::write(&file, format!("before\n{BEGIN}\nfastfetch\n{END}\nafter\n")).unwrap();
        assert!(remove_block(&file).unwrap());
        assert_eq!(fs::read_to_string(&file).unwrap(), "before\nafter\n");
        assert!(!remove_block(&file).unwrap(), "nothing left to remove");
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
