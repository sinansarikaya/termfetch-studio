//! Running fastfetch: version detection, real-spec probing for the preview
//! and the interactive live run.

use crate::adapters::fsutil::atomic_write;
use crate::domain::preset::ModuleKind;
use anyhow::{Context, Result};
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const FIELD_SEP: char = '\u{1f}';

/// Locate the fastfetch executable across platforms (PATH + standard package manager locations).
pub fn find_fastfetch() -> Option<PathBuf> {
    if let Some(explicit) = std::env::var_os("FASTFETCH_BIN") {
        let p = PathBuf::from(explicit);
        if p.is_file() {
            return Some(p);
        }
    }

    let exe_name = if cfg!(windows) {
        "fastfetch.exe"
    } else {
        "fastfetch"
    };

    // 1. Search PATH
    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(exe_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    // 2. Platform-specific standard package manager locations
    #[cfg(target_os = "macos")]
    {
        let mac_candidates = [
            "/opt/homebrew/bin/fastfetch",
            "/usr/local/bin/fastfetch",
            "/opt/local/bin/fastfetch",
        ];
        for path in mac_candidates {
            let p = PathBuf::from(path);
            if p.is_file() {
                return Some(p);
            }
        }
    }

    #[cfg(windows)]
    {
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            let winget = PathBuf::from(local)
                .join("Microsoft")
                .join("WinGet")
                .join("Links")
                .join("fastfetch.exe");
            if winget.is_file() {
                return Some(winget);
            }
        }
        if let Some(user) = std::env::var_os("USERPROFILE") {
            let scoop = PathBuf::from(user)
                .join("scoop")
                .join("shims")
                .join("fastfetch.exe");
            if scoop.is_file() {
                return Some(scoop);
            }
        }
        let choco = PathBuf::from(r"C:\ProgramData\chocolatey\bin\fastfetch.exe");
        if choco.is_file() {
            return Some(choco);
        }
        let prog = PathBuf::from(r"C:\Program Files\Fastfetch\fastfetch.exe");
        if prog.is_file() {
            return Some(prog);
        }
    }

    #[cfg(unix)]
    {
        let unix_candidates = [
            "/usr/bin/fastfetch",
            "/usr/local/bin/fastfetch",
            "/bin/fastfetch",
        ];
        for path in unix_candidates {
            let p = PathBuf::from(path);
            if p.is_file() {
                return Some(p);
            }
        }
        if let Some(home) = std::env::var_os("HOME") {
            let home_p = PathBuf::from(home);
            let local_bin = home_p.join(".local/bin/fastfetch");
            if local_bin.is_file() {
                return Some(local_bin);
            }
            let nix_bin = home_p.join(".nix-profile/bin/fastfetch");
            if nix_bin.is_file() {
                return Some(nix_bin);
            }
        }
    }

    None
}

/// Create a `Command` targeting fastfetch (either resolved binary or fallback to "fastfetch").
pub fn fastfetch_cmd() -> Command {
    if let Some(bin) = find_fastfetch() {
        Command::new(bin)
    } else {
        Command::new("fastfetch")
    }
}

pub fn version() -> Option<String> {
    let out = fastfetch_cmd().arg("--version").output().ok()?;
    out.status.success().then(|| {
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .next()
            .unwrap_or("fastfetch")
            .trim()
            .to_string()
    })
}

/// Real values shown in the preview when demo mode is off.
#[derive(Debug, Clone, Default)]
pub struct RealInfo {
    pub title: Option<String>,
    pub values: HashMap<ModuleKind, String>,
}

impl RealInfo {
    pub fn get(&self, m: ModuleKind) -> Option<&str> {
        self.values.get(&m).map(String::as_str)
    }
}

/// Parse `type<US>value` lines produced by [`probe_config`].
pub fn parse_probe_output(text: &str) -> RealInfo {
    let mut info = RealInfo::default();
    for line in text.lines() {
        let Some((key, value)) = line.split_once(FIELD_SEP) else {
            continue;
        };
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        if key == "title" {
            info.title.get_or_insert_with(|| value.to_string());
        } else if let Some(m) = ModuleKind::from_type(key) {
            // Multi-instance modules (several disks/GPUs): keep the first, like a compact fetch.
            info.values.entry(m).or_insert_with(|| value.to_string());
        }
    }
    info
}

fn probe_config(modules: &[ModuleKind]) -> serde_json::Value {
    let mut list = vec![
        json!({"type": "title", "format": format!("title{FIELD_SEP}{{user-name}}@{{host-name}}")}),
    ];
    list.extend(
        modules
            .iter()
            .map(|m| json!({"type": m.fastfetch_type(), "key": m.fastfetch_type()})),
    );
    json!({
        "logo": {"type": "none"},
        "display": {"separator": FIELD_SEP.to_string(), "pipe": true},
        "modules": list,
    })
}

/// Ask fastfetch for real values of `modules`. Blocking; run off the UI thread.
pub fn probe_real_info(modules: &[ModuleKind], scratch: &Path) -> Result<RealInfo> {
    let cfg = serde_json::to_vec(&probe_config(modules))?;
    atomic_write(scratch, &cfg)?;
    let out = fastfetch_cmd()
        .arg("-c")
        .arg(scratch)
        .arg("--pipe")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .context("running fastfetch")?;
    Ok(parse_probe_output(&String::from_utf8_lossy(&out.stdout)))
}

/// Run fastfetch with `config` attached to the current terminal.
pub fn run_live(config: &Path) -> Result<std::process::ExitStatus> {
    fastfetch_cmd()
        .arg("-c")
        .arg(config)
        .status()
        .context("failed to launch fastfetch (is it installed and in PATH?)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_probe_lines() {
        let text = "title\u{1f}me@box\nos\u{1f}Arch Linux\ndisk\u{1f}1 TiB\ndisk\u{1f}2 TiB\nbogus\u{1f}x\nnoise\n";
        let info = parse_probe_output(text);
        assert_eq!(info.title.as_deref(), Some("me@box"));
        assert_eq!(info.get(ModuleKind::Os), Some("Arch Linux"));
        assert_eq!(info.get(ModuleKind::Disk), Some("1 TiB"));
        assert_eq!(info.values.len(), 2);
    }
}
