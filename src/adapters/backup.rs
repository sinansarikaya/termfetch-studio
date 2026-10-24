//! Backup & Restore of studio settings and both fastfetch configs.
//! A backup is a directory `backups/backup_<UTC timestamp>/` with plain files.

use crate::adapters::fsutil::{atomic_write, timestamp_utc};
use crate::paths::Paths;
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;

/// (file name inside a backup, live location)
fn members(paths: &Paths) -> [(&'static str, PathBuf); 3] {
    [
        ("settings.json", paths.settings_file()),
        ("fastfetch.jsonc", paths.studio_config()),
        ("fastfetch-config.jsonc", paths.fastfetch_config.clone()),
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupEntry {
    pub name: String,
    pub path: PathBuf,
    pub files: Vec<String>,
}

fn valid_name(name: &str) -> bool {
    name.starts_with("backup_")
        && !name.contains(['/', '\\'])
        && name != "."
        && !name.contains("..")
}

pub fn create_backup(paths: &Paths) -> Result<BackupEntry> {
    create_named(paths, "backup")
}

fn create_named(paths: &Paths, label: &str) -> Result<BackupEntry> {
    let root = paths.backups_dir();
    let stamp = timestamp_utc();
    let mut name = format!("backup_{stamp}");
    if label != "backup" {
        name.push('_');
        name.push_str(label);
    }
    let mut dir = root.join(&name);
    let mut n = 1;
    while dir.exists() {
        n += 1;
        dir = root.join(format!("{name}-{n}"));
    }
    let mut files = Vec::new();
    for (file, src) in members(paths) {
        if let Ok(bytes) = fs::read(&src) {
            atomic_write(&dir.join(file), &bytes)?;
            files.push(file.to_string());
        }
    }
    if files.is_empty() {
        bail!("nothing to back up yet (no settings or fastfetch config found)");
    }
    Ok(BackupEntry {
        name: dir.file_name().unwrap_or_default().to_string_lossy().into_owned(),
        path: dir,
        files,
    })
}

/// Newest first.
pub fn list_backups(paths: &Paths) -> Vec<BackupEntry> {
    let Ok(rd) = fs::read_dir(paths.backups_dir()) else {
        return Vec::new();
    };
    let known: Vec<&str> = members(paths).iter().map(|(f, _)| *f).collect();
    let mut out: Vec<BackupEntry> = rd
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            if !valid_name(&name) {
                return None;
            }
            let files: Vec<String> = known
                .iter()
                .filter(|f| e.path().join(f).is_file())
                .map(|f| (*f).to_string())
                .collect();
            Some(BackupEntry { name, path: e.path(), files })
        })
        .collect();
    out.sort_by(|a, b| b.name.cmp(&a.name));
    out
}

pub fn find_backup(paths: &Paths, name: &str) -> Result<BackupEntry> {
    if !valid_name(name) {
        bail!("invalid backup name '{name}'");
    }
    list_backups(paths)
        .into_iter()
        .find(|b| b.name == name)
        .with_context(|| format!("backup '{name}' not found"))
}

/// Restore files from `entry`. The current state is saved first as a
/// `_pre-restore` backup so a restore can always be undone.
pub fn restore_backup(paths: &Paths, entry: &BackupEntry) -> Result<usize> {
    if !valid_name(&entry.name) {
        bail!("invalid backup name '{}'", entry.name);
    }
    let _ = create_named(paths, "pre-restore");
    let mut restored = 0;
    // Only known member names are read; nothing in the backup can redirect a write.
    for (file, dest) in members(paths) {
        let src = entry.path.join(file);
        if src.is_file() {
            let bytes = fs::read(&src).with_context(|| format!("reading {}", src.display()))?;
            atomic_write(&dest, &bytes)?;
            restored += 1;
        }
    }
    if restored == 0 {
        bail!("backup '{}' contains no restorable files", entry.name);
    }
    Ok(restored)
}

pub fn delete_backup(paths: &Paths, entry: &BackupEntry) -> Result<()> {
    if !valid_name(&entry.name) || entry.path.parent() != Some(paths.backups_dir().as_path()) {
        bail!("refusing to delete '{}'", entry.path.display());
    }
    fs::remove_dir_all(&entry.path).with_context(|| format!("deleting {}", entry.path.display()))
}
