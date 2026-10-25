//! Host facts for Windows when fastfetch is not installed, so the dashboard and
//! live preview still show real values. One CIM query per probe, run off the UI thread.

use crate::adapters::fastfetch::runner::RealInfo;
use crate::domain::preset::ModuleKind;
use base64::Engine;
use serde::Deserialize;
use std::collections::HashMap;

/// Fields the PowerShell query returns; anything missing stays blank.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Facts {
    pub os: Option<String>,
    pub kernel: Option<String>,
    pub host: Option<String>,
    pub cpu: Option<String>,
    pub gpu: Option<String>,
    pub memory: Option<String>,
    pub uptime: Option<String>,
    pub display: Option<String>,
}

/// Single-line query; sent with `-EncodedCommand` so no shell quoting is involved.
const QUERY: &str = "$os=Get-CimInstance Win32_OperatingSystem;\
$cs=Get-CimInstance Win32_ComputerSystem;\
$cpu=Get-CimInstance Win32_Processor|Select-Object -First 1;\
$vc=@(Get-CimInstance Win32_VideoController);\
$gpu=$vc|Where-Object {$_.Name -notmatch 'Virtual|Basic|Remote'}|Select-Object -First 1;\
if(!$gpu){$gpu=$vc|Select-Object -First 1};\
$lit=$vc|Where-Object {$_.CurrentHorizontalResolution}|Select-Object -First 1;\
$up=(Get-Date)-$os.LastBootUpTime;\
[pscustomobject]@{\
os=(($os.Caption -replace '^Microsoft ','')+' '+$os.OSArchitecture);\
kernel=$os.Version;\
host=((@($cs.Manufacturer,$cs.Model)-join ' ').Trim());\
cpu=\"$($cpu.Name.Trim()) ($($cpu.NumberOfLogicalProcessors) threads)\";\
gpu=$gpu.Name;\
memory=\"$([math]::Round(($os.TotalVisibleMemorySize-$os.FreePhysicalMemory)/1MB,1)) GiB / $([math]::Round($os.TotalVisibleMemorySize/1MB,1)) GiB\";\
uptime=\"$($up.Days)d $($up.Hours)h $($up.Minutes)m\";\
display=\"$($lit.CurrentHorizontalResolution)x$($lit.CurrentVerticalResolution)\"\
}|ConvertTo-Json -Compress";

/// Parse the JSON printed by the query.
pub fn parse_facts(json: &str) -> Option<Facts> {
    serde_json::from_str(json.trim()).ok()
}

/// Map the facts onto the same module keys fastfetch fills, dropping blank values.
pub fn to_info(
    facts: Facts,
    terminal: Option<String>,
    shell: Option<String>,
    title: Option<String>,
) -> RealInfo {
    let mut values = HashMap::new();
    let mut put = |m: ModuleKind, v: Option<String>| {
        if let Some(v) = v.map(|v| v.trim().to_string()).filter(|v| !v.is_empty()) {
            values.insert(m, v);
        }
    };
    put(ModuleKind::Os, facts.os);
    put(ModuleKind::Kernel, facts.kernel);
    put(ModuleKind::Host, facts.host);
    put(ModuleKind::Cpu, facts.cpu);
    put(ModuleKind::Gpu, facts.gpu);
    put(ModuleKind::Memory, facts.memory);
    put(ModuleKind::Uptime, facts.uptime);
    put(ModuleKind::Display, facts.display);
    put(ModuleKind::Terminal, terminal);
    put(ModuleKind::Shell, shell);
    RealInfo { title, values }
}

/// Real values for the modules this fallback knows. Empty off Windows.
pub fn probe() -> RealInfo {
    #[cfg(windows)]
    {
        let env = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        let title = match (env("USERNAME"), env("COMPUTERNAME")) {
            (Some(user), Some(host)) => Some(format!("{user}@{host}")),
            _ => None,
        };
        let terminal = env("WT_SESSION")
            .map(|_| "Windows Terminal".to_string())
            .or_else(|| env("TERM_PROGRAM"));
        let shell = env("PSModulePath")
            .map(|_| "PowerShell".to_string())
            .or_else(|| env("COMSPEC").and_then(|c| c.rsplit('\\').next().map(str::to_string)));
        query_powershell()
            .and_then(|json| parse_facts(&json))
            .map(|facts| to_info(facts, terminal, shell, title))
            .unwrap_or_default()
    }
    #[cfg(not(windows))]
    {
        RealInfo::default()
    }
}

#[cfg(windows)]
fn query_powershell() -> Option<String> {
    use std::process::{Command, Stdio};
    let utf16: Vec<u8> = QUERY.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let encoded = base64::engine::general_purpose::STANDARD.encode(utf16);
    let out = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-EncodedCommand", &encoded])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_query_output_into_module_values() {
        let json = r#"{"os":"Windows 11 Home 64-bit","kernel":"10.0.26300","host":"Dell XPS 13","cpu":"Intel Core i7 (16 threads)","gpu":"","memory":"8.1 GiB / 16.0 GiB","uptime":"0d 3h 12m","display":"1920x1080"}"#;
        let facts = parse_facts(json).expect("valid json");
        let info = to_info(
            facts,
            Some("Windows Terminal".into()),
            None,
            Some("me@box".into()),
        );
        assert_eq!(info.title.as_deref(), Some("me@box"));
        assert_eq!(info.get(ModuleKind::Os), Some("Windows 11 Home 64-bit"));
        assert_eq!(
            info.get(ModuleKind::Cpu),
            Some("Intel Core i7 (16 threads)")
        );
        assert_eq!(info.get(ModuleKind::Gpu), None, "blank values are dropped");
        assert_eq!(info.get(ModuleKind::Terminal), Some("Windows Terminal"));
        assert_eq!(info.get(ModuleKind::Shell), None);
    }

    #[test]
    fn rejects_non_json_output() {
        assert!(parse_facts("Get-CimInstance : Access denied").is_none());
    }
}
