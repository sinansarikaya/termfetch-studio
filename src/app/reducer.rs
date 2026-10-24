use crate::adapters::backup;
use crate::adapters::doctor;
use crate::adapters::fastfetch::config;
use crate::app::actions::Action;
use crate::app::state::{AppState, Effect, Focus, Screen, StatusKind, COLOR_ROWS};
use crate::domain::color::SemanticGroup;
use crate::domain::settings::Settings;
use crate::imaging::TermGeometry;

const PAGE: usize = 10;

fn step(idx: usize, len: usize, delta: isize, wrap: bool) -> usize {
    if len == 0 {
        return 0;
    }
    let n = len as isize;
    let next = idx as isize + delta;
    if wrap && delta.abs() == 1 {
        next.rem_euclid(n) as usize
    } else {
        next.clamp(0, n - 1) as usize
    }
}

pub fn reduce(state: &mut AppState, action: Action) {
    if action != Action::Tick {
        state.dirty = true;
    }
    match action {
        Action::Quit => quit(state),
        Action::Back => match state.focus {
            Focus::Content => state.focus = Focus::Sidebar,
            Focus::Sidebar => quit(state),
        },
        Action::ToggleFocus => {
            state.focus = match state.focus {
                Focus::Sidebar => Focus::Content,
                Focus::Content => Focus::Sidebar,
            }
        }
        Action::Goto(screen) => goto(state, screen),
        Action::Up => navigate(state, -1),
        Action::Down => navigate(state, 1),
        Action::PageUp => navigate(state, -(PAGE as isize)),
        Action::PageDown => navigate(state, PAGE as isize),
        Action::Home => navigate(state, -10_000),
        Action::End => navigate(state, 10_000),
        Action::Left => {
            if state.focus == Focus::Content && state.screen == Screen::Colors {
                cycle_color(state, false);
            } else {
                state.focus = Focus::Sidebar;
            }
        }
        Action::Right => {
            if state.focus == Focus::Content && state.screen == Screen::Colors {
                cycle_color(state, true);
            } else {
                state.focus = Focus::Content;
            }
        }
        Action::Activate => activate(state),
        Action::Toggle => {
            if state.screen == Screen::Modules {
                toggle_module(state);
            } else {
                activate(state);
            }
        }
        Action::MoveItem { up } => {
            if state.screen == Screen::Modules {
                move_module(state, up);
            }
        }
        Action::Apply => {
            apply(state);
        }
        Action::ToggleDemo => toggle_demo(state),
        Action::CycleLogoSize => {
            state.settings.logo_size = state.settings.logo_size.next();
            let size = state.settings.logo_size;
            let (w, h) = size.max_cells();
            state.set_status(StatusKind::Info, format!("Logo size: {} (max {w}x{h} cells) - Enter to apply", size.label()));
        }
        Action::RunFastfetch => {
            if state.fastfetch_version.is_some() {
                state.effects.push(Effect::RunFastfetch);
            } else {
                state.set_status(StatusKind::Error, "fastfetch is not installed - see Doctor");
            }
        }
        Action::Refresh => match state.screen {
            Screen::Doctor => run_doctor(state),
            Screen::Backup => {
                state.refresh_backups();
                state.set_status(StatusKind::Info, format!("{} backups found", state.backups.len()));
            }
            Screen::Logos => {
                let logo = state.current_logo().clone();
                if let Some(p) = logo.file_path.as_deref() {
                    state.pool.invalidate(p);
                    state.pool.request(p, true);
                    if let Some(k) = state.kitty.as_mut() {
                        state.effects.extend(k.invalidate(p).into_iter().map(Effect::WriteTerminal));
                    }
                }
            }
            _ => {}
        },
        Action::Delete => {
            if state.screen == Screen::Backup {
                delete_backup(state);
            }
        }
        Action::Click { col, row } => click(state, col, row),
        Action::ScrollUp => navigate(state, -1),
        Action::ScrollDown => navigate(state, 1),
        Action::Resize => state.geo = TermGeometry::detect(),
        Action::Tick => tick(state),
    }
}

fn goto(state: &mut AppState, screen: Screen) {
    state.screen = screen;
    match screen {
        Screen::Doctor if state.doctor.is_empty() => run_doctor(state),
        Screen::Backup => state.refresh_backups(),
        _ => {}
    }
}

fn navigate(state: &mut AppState, delta: isize) {
    if state.focus == Focus::Sidebar {
        let idx = step(state.screen.index(), Screen::ALL.len(), delta, true);
        goto(state, Screen::ALL[idx]);
        return;
    }
    match state.screen {
        Screen::Themes => {
            let i = step(state.theme_idx(), state.themes.len(), delta, true);
            state.settings.theme = state.themes[i].id.clone();
            let name = state.themes[i].name.clone();
            state.set_status(StatusKind::Info, format!("Previewing theme {name} - Enter to apply"));
        }
        Screen::Presets => {
            let i = step(state.preset_idx(), state.presets.len(), delta, true);
            state.settings.preset = state.presets[i].id.clone();
            let name = state.presets[i].name.clone();
            state.set_status(StatusKind::Info, format!("Previewing preset {name} - Enter to apply"));
        }
        Screen::Logos => {
            let i = step(state.logo_idx(), state.logos.len(), delta, true);
            select_logo(state, i);
        }
        Screen::Modules => {
            state.module_cursor = step(state.module_cursor, state.module_rows().len(), delta, true);
        }
        Screen::Colors => state.color_cursor = step(state.color_cursor, COLOR_ROWS, delta, true),
        Screen::Backup => state.backup_cursor = step(state.backup_cursor, state.backups.len() + 1, delta, true),
        Screen::Doctor => state.doctor_cursor = step(state.doctor_cursor, state.doctor.len(), delta, false),
        Screen::Dashboard => {}
    }
}

fn select_logo(state: &mut AppState, i: usize) {
    let logo = state.logos[i].clone();
    state.settings.logo = logo.id.clone();
    if let Some(p) = logo.existing_image() {
        state.pool.request(p, true);
    }
    state.set_status(StatusKind::Info, format!("Previewing logo {} - Enter to apply", logo.name));
}

fn activate(state: &mut AppState) {
    if state.focus == Focus::Sidebar {
        state.focus = Focus::Content;
        return;
    }
    match state.screen {
        Screen::Modules => toggle_module(state),
        Screen::Colors => match state.color_cursor {
            7 => {
                state.settings.reset_colors();
                state.set_status(StatusKind::Info, "Colours reset to theme defaults - a to apply");
            }
            8 => {
                state.settings.apply_terminal_colors = !state.settings.apply_terminal_colors;
            }
            _ => cycle_color(state, true),
        },
        Screen::Backup => {
            if state.backup_cursor == 0 {
                create_backup(state);
            } else {
                restore_backup(state);
            }
        }
        Screen::Doctor => run_doctor(state),
        _ => {
            apply(state);
        }
    }
}

fn toggle_module(state: &mut AppState) {
    let rows = state.module_rows();
    let Some(&(m, enabled)) = rows.get(state.module_cursor) else {
        return;
    };
    if enabled {
        if state.settings.custom_modules.len() == 1 {
            state.set_status(StatusKind::Warn, "Keep at least one module enabled");
            return;
        }
        state.settings.custom_modules.retain(|&x| x != m);
    } else {
        state.settings.custom_modules.push(m);
    }
    // Keep the cursor on the toggled module.
    state.module_cursor = state.module_rows().iter().position(|&(x, _)| x == m).unwrap_or(0);
    after_module_edit(state, format!("{} {}", if enabled { "Disabled" } else { "Enabled" }, m.label()));
}

fn move_module(state: &mut AppState, up: bool) {
    let rows = state.module_rows();
    let Some(&(m, true)) = rows.get(state.module_cursor) else {
        state.set_status(StatusKind::Warn, "Only enabled modules can be reordered");
        return;
    };
    let list = &mut state.settings.custom_modules;
    let Some(i) = list.iter().position(|&x| x == m) else { return };
    let j = if up { i.checked_sub(1) } else { (i + 1 < list.len()).then_some(i + 1) };
    if let Some(j) = j {
        list.swap(i, j);
        state.module_cursor = j;
        after_module_edit(state, format!("Moved {} {}", m.label(), if up { "up" } else { "down" }));
    }
}

fn after_module_edit(state: &mut AppState, msg: String) {
    state.settings.preset = "custom".into();
    state.sync_custom_preset();
    state.set_status(StatusKind::Info, format!("{msg} - Custom preset selected, a to apply"));
}

fn cycle_color(state: &mut AppState, forward: bool) {
    let s = &mut state.settings;
    match state.color_cursor {
        0 => s.icon_color = s.icon_color.cycle(forward, false),
        1 => s.text_color = s.text_color.cycle(forward, false),
        r @ 2..=6 => {
            let g = SemanticGroup::ALL[r - 2];
            let v = s.group_colors.get_mut(g);
            *v = v.cycle(forward, true);
        }
        8 => s.apply_terminal_colors = !s.apply_terminal_colors,
        _ => {}
    }
}

/// Persist settings and write both fastfetch configs.
fn write_all(state: &mut AppState, settings: &Settings) -> anyhow::Result<Vec<std::path::PathBuf>> {
    let theme = state.themes.iter().find(|t| t.id == settings.theme).unwrap_or(&state.themes[0]).clone();
    let preset = state.presets.iter().find(|p| p.id == settings.preset).unwrap_or(&state.presets[0]).clone();
    let logo = state.logos.iter().find(|l| l.id == settings.logo).unwrap_or(&state.logos[0]).clone();
    settings.save(&state.paths.settings_file())?;
    config::apply(&theme, &preset, &logo, settings, &state.paths, &state.geo)
}

/// Leaving the TUI commits whatever is on screen: unsaved edits are written to
/// settings.json and both fastfetch configs so the next `fastfetch` shows them.
fn quit(state: &mut AppState) {
    if state.unsaved() {
        let ok = apply(state);
        let (kind, msg) = state.status.clone();
        let msg = if ok { msg.replacen("✓ Applied", "✓ Saved on exit:", 1) } else { format!("Changes were NOT saved - {msg}") };
        state.exit_message = Some((kind, msg));
    }
    state.running = false;
}

/// Returns whether the settings were written.
fn apply(state: &mut AppState) -> bool {
    let settings = state.settings.clone();
    match write_all(state, &settings) {
        Ok(targets) => {
            state.saved = settings;
            if state.saved.apply_terminal_colors {
                let osc = state.current_theme().osc_sequences();
                state.effects.push(Effect::WriteTerminal(osc));
            }
            let list = targets.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(" + ");
            let (theme, preset) = (state.current_theme().name.clone(), state.current_preset().name.clone());
            state.set_status(StatusKind::Ok, format!("✓ Applied {theme} · {preset} → {list}"));
            true
        }
        Err(e) => {
            state.set_status(StatusKind::Error, format!("Apply failed: {e:#}"));
            false
        }
    }
}

/// Demo mode is a persisted setting: flip it, save it and regenerate the
/// active config (like the installer's `toggle_demo_mode`) without committing
/// other unsaved edits.
fn toggle_demo(state: &mut AppState) {
    let on = !state.settings.demo_mode;
    state.settings.demo_mode = on;
    let mut saved = state.saved.clone();
    saved.demo_mode = on;
    let regenerate = state.paths.studio_config().exists();
    let result = if regenerate {
        write_all(state, &saved).map(|_| ())
    } else {
        saved.save(&state.paths.settings_file())
    };
    match result {
        Ok(()) => {
            state.saved = saved;
            let what = if on { "ON - privacy-safe mock specs" } else { "OFF - real system specs" };
            let tail = if regenerate { " (fastfetch config regenerated)" } else { "" };
            state.set_status(if on { StatusKind::Ok } else { StatusKind::Warn }, format!("Preview mode {what}{tail}"));
        }
        Err(e) => state.set_status(StatusKind::Error, format!("Could not save preview mode: {e:#}")),
    }
    if !on {
        state.start_real_probe();
    }
}

fn create_backup(state: &mut AppState) {
    match backup::create_backup(&state.paths) {
        Ok(entry) => {
            state.refresh_backups();
            state.set_status(StatusKind::Ok, format!("✓ Backup {} created ({} files)", entry.name, entry.files.len()));
        }
        Err(e) => state.set_status(StatusKind::Error, format!("Backup failed: {e:#}")),
    }
}

fn restore_backup(state: &mut AppState) {
    let Some(entry) = state.backups.get(state.backup_cursor - 1).cloned() else {
        return;
    };
    match backup::restore_backup(&state.paths, &entry) {
        Ok(n) => {
            let restored = Settings::load_or_default(&state.paths.settings_file());
            state.settings = restored.clone();
            state.saved = restored;
            state.sync_custom_preset();
            state.refresh_backups();
            state.set_status(StatusKind::Ok, format!("✓ Restored {n} files from {} (previous state saved as pre-restore backup)", entry.name));
        }
        Err(e) => state.set_status(StatusKind::Error, format!("Restore failed: {e:#}")),
    }
}

fn delete_backup(state: &mut AppState) {
    if state.backup_cursor == 0 {
        return;
    }
    let Some(entry) = state.backups.get(state.backup_cursor - 1).cloned() else {
        return;
    };
    match backup::delete_backup(&state.paths, &entry) {
        Ok(()) => {
            state.refresh_backups();
            state.set_status(StatusKind::Info, format!("Deleted {}", entry.name));
        }
        Err(e) => state.set_status(StatusKind::Error, format!("Delete failed: {e:#}")),
    }
}

fn run_doctor(state: &mut AppState) {
    state.doctor = doctor::run_checks(&state.paths);
    state.doctor_cursor = 0;
    let fails = state.doctor.iter().filter(|c| c.level == doctor::Level::Fail).count();
    let warns = state.doctor.iter().filter(|c| c.level == doctor::Level::Warn).count();
    let kind = if fails > 0 { StatusKind::Error } else if warns > 0 { StatusKind::Warn } else { StatusKind::Ok };
    state.set_status(kind, format!("Doctor: {} checks, {fails} failed, {warns} warnings", state.doctor.len()));
}

fn click(state: &mut AppState, col: u16, row: u16) {
    let s = state.sidebar_area;
    if s.width > 0 && col >= s.x && col < s.x + s.width && row > s.y && row < s.y + s.height - 1 {
        if let Some(&screen) = Screen::ALL.get(usize::from(row - s.y - 1)) {
            state.focus = Focus::Sidebar;
            goto(state, screen);
        }
        return;
    }
    let Some(idx) = state.click.hit(col, row) else {
        return;
    };
    state.focus = Focus::Content;
    match state.screen {
        Screen::Themes => state.settings.theme = state.themes[idx].id.clone(),
        Screen::Presets => state.settings.preset = state.presets[idx].id.clone(),
        Screen::Logos => select_logo(state, idx),
        Screen::Modules => state.module_cursor = idx,
        Screen::Colors => state.color_cursor = idx,
        Screen::Backup => state.backup_cursor = idx,
        Screen::Doctor => state.doctor_cursor = idx,
        Screen::Dashboard => {}
    }
}

fn tick(state: &mut AppState) {
    if let Some(k) = state.kitty.as_mut() {
        let uploads = k.poll();
        if !uploads.is_empty() {
            state.effects.extend(uploads.into_iter().map(Effect::WriteTerminal));
            state.dirty = true;
        }
    }
    let done = state.pool.poll();
    if let Some(sel) = state.current_logo().existing_image() {
        if done.iter().any(|p| p == sel) {
            state.dirty = true;
        }
    }
    if let Some(rx) = &state.real_rx {
        if let Ok(info) = rx.try_recv() {
            state.real_info = Some(info);
            state.real_rx = None;
            state.dirty = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::Paths;

    fn app() -> (AppState, std::path::PathBuf) {
        let root = std::env::temp_dir().join(format!("tfs-reducer-{}-{}", std::process::id(), rand_suffix()));
        let paths = Paths::with_root(&root);
        let geo = TermGeometry::from_window(160, 48, 1600, 960);
        (AppState::new(paths, Settings::default(), geo), root)
    }

    fn rand_suffix() -> u128 {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    }

    #[test]
    fn browsing_changes_preview_but_not_saved_state() {
        let (mut s, root) = app();
        reduce(&mut s, Action::Goto(Screen::Themes));
        reduce(&mut s, Action::ToggleFocus);
        reduce(&mut s, Action::Down);
        assert_ne!(s.settings.theme, s.saved.theme);
        assert!(s.unsaved());
        reduce(&mut s, Action::Apply);
        assert!(!s.unsaved(), "{:?}", s.status);
        assert!(s.paths.studio_config().exists());
        assert!(s.paths.fastfetch_config.exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn quit_autosaves_unsaved_changes_to_both_configs() {
        let (mut s, root) = app();
        reduce(&mut s, Action::Goto(Screen::Colors));
        s.focus = Focus::Content;
        reduce(&mut s, Action::Right);
        reduce(&mut s, Action::Goto(Screen::Themes));
        reduce(&mut s, Action::Down);
        assert!(s.unsaved());
        let (theme, icon) = (s.settings.theme.clone(), s.settings.icon_color.clone());
        reduce(&mut s, Action::Quit);
        assert!(!s.running);
        assert!(!s.unsaved(), "{:?}", s.status);
        let persisted = Settings::load(&s.paths.settings_file()).unwrap();
        assert_eq!((persisted.theme, persisted.icon_color), (theme, icon));
        let studio = std::fs::read_to_string(s.paths.studio_config()).unwrap();
        let fastfetch = std::fs::read_to_string(&s.paths.fastfetch_config).unwrap();
        assert_eq!(studio, fastfetch);
        assert!(matches!(&s.exit_message, Some((StatusKind::Ok, m)) if m.contains("Saved on exit")), "{:?}", s.exit_message);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn quit_without_changes_writes_nothing() {
        let (mut s, root) = app();
        reduce(&mut s, Action::Back);
        assert!(!s.running);
        assert!(s.exit_message.is_none());
        assert!(!s.paths.fastfetch_config.exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn demo_toggle_persists_and_regenerates() {
        let (mut s, root) = app();
        reduce(&mut s, Action::Apply);
        let before = std::fs::read_to_string(s.paths.studio_config()).unwrap();
        assert!(before.contains(crate::domain::preset::DEMO_DATA.os));
        reduce(&mut s, Action::ToggleDemo);
        assert!(!s.settings.demo_mode && !s.saved.demo_mode);
        let after = std::fs::read_to_string(s.paths.studio_config()).unwrap();
        assert!(!after.contains("\"format\": \"Arch Linux x86_64\""));
        assert!(!Settings::load(&s.paths.settings_file()).unwrap().demo_mode);
        reduce(&mut s, Action::ToggleDemo);
        assert!(s.settings.demo_mode);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn module_builder_toggle_and_reorder() {
        let (mut s, root) = app();
        reduce(&mut s, Action::Goto(Screen::Modules));
        s.focus = Focus::Content;
        let first = s.settings.custom_modules[0];
        reduce(&mut s, Action::Down);
        reduce(&mut s, Action::MoveItem { up: true });
        assert_eq!(s.settings.custom_modules[1], first);
        assert_eq!(s.settings.preset, "custom");
        let n = s.settings.custom_modules.len();
        reduce(&mut s, Action::End);
        reduce(&mut s, Action::Toggle);
        assert_eq!(s.settings.custom_modules.len(), n + 1);
        assert_eq!(s.current_preset().module_count(), n + 1);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn colors_cycle_and_reset() {
        let (mut s, root) = app();
        reduce(&mut s, Action::Goto(Screen::Colors));
        s.focus = Focus::Content;
        reduce(&mut s, Action::Right);
        assert_ne!(s.settings.icon_color, crate::domain::color::ColorValue::Theme);
        s.color_cursor = 7;
        reduce(&mut s, Action::Activate);
        assert_eq!(s.settings.icon_color, crate::domain::color::ColorValue::Theme);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn backup_restore_roundtrip() {
        let (mut s, root) = app();
        reduce(&mut s, Action::Apply);
        reduce(&mut s, Action::Goto(Screen::Backup));
        s.focus = Focus::Content;
        reduce(&mut s, Action::Activate);
        assert_eq!(s.backups.len(), 1, "{:?}", s.status);
        // Change and apply something else, then restore.
        let original_theme = s.saved.theme.clone();
        s.settings.theme = "nord".into();
        reduce(&mut s, Action::Apply);
        s.screen = Screen::Backup;
        s.refresh_backups();
        s.backup_cursor = s.backups.iter().position(|b| !b.name.contains("pre-restore")).unwrap() + 1;
        reduce(&mut s, Action::Activate);
        assert_eq!(s.settings.theme, original_theme, "{:?}", s.status);
        assert!(s.backups.iter().any(|b| b.name.contains("pre-restore")));
        let _ = std::fs::remove_dir_all(root);
    }
}
