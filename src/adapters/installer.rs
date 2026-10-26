//! Windows install and uninstall. The PowerShell script is embedded in the exe, so the
//! exe alone can install itself; `scripts/install-windows.ps1` stays the single source.

use anyhow::{Context, Result};
use base64::Engine;
use std::process::Command;

const SCRIPT: &str = include_str!("../../scripts/install-windows.ps1");

/// Run the embedded script with `args` and return its exit code.
pub fn run_script(args: &[&str]) -> Result<i32> {
    let path = std::env::temp_dir().join(format!("termfetch-install-{}.ps1", std::process::id()));
    let icon = base64::engine::general_purpose::STANDARD
        .encode(include_bytes!("../../assets/brand/termfetch.ico"));
    let script = SCRIPT.replace("__TERMFETCH_ICON_BASE64__", &icon);
    std::fs::write(&path, script).with_context(|| format!("writing {}", path.display()))?;
    let status = Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(&path)
        .args(args)
        .status();
    let _ = std::fs::remove_file(&path);
    let status = status.context("running powershell")?;
    Ok(status.code().unwrap_or(1))
}
