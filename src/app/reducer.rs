use crate::adapters::backup;
use crate::adapters::doctor;
use crate::adapters::fastfetch::config;
use crate::app::actions::Action;
use crate::app::state::{
    AppState, DashCursor, DashRow, Dropdown, Effect, Focus, Hit, OPTION_APPLY, OPTION_ROWS,
    PreviewOutput, QUICK_ACTIONS, SETTINGS_ROWS, Screen, StatusKind, TOOLS,
};
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
    if state.editor.is_some() && !matches!(action, Action::Tick | Action::Resize) {
        match action {
            Action::Back => {
                state.editor = None;
            }
            Action::Text(c) if !c.is_control() => {
                if state.input.chars().count() < 100 {
                    state.input.push(c);
                }
            }
            Action::Erase => {
                state.input.pop();
            }
            Action::Activate => {
                let kind = state.editor.take().unwrap();
                if kind == "search" {
                    state.theme_search = state.input.clone();
                    state.grid_scroll = 0;
                    select_filtered(state);
                } else if kind == "rename" && !state.input.trim().is_empty() {
                    let i = state.preset_idx();
                    state.presets[i].name = state.input.trim().into();
                    persist_presets(state);
                } else if kind == "delete" && state.input.eq_ignore_ascii_case("yes") {
                    let i = state.preset_idx();
                    if state.presets.len() > 1 {
                        state.presets.remove(i);
                        state.settings.preset = state.presets[0].id.clone();
                        persist_presets(state);
                    }
                }
            }
            _ => {}
        }
        return;
    }
    if state.preview_focus {
        match action {
            Action::Up | Action::ScrollUp | Action::PageUp => {
                state.preview_scroll = state
                    .preview_scroll
                    .saturating_sub(if action == Action::PageUp { 10 } else { 1 });
                return;
            }
            Action::Down | Action::ScrollDown | Action::PageDown => {
                state.preview_scroll = state
                    .preview_scroll
                    .saturating_add(if action == Action::PageDown { 10 } else { 1 });
                return;
            }
            Action::Home => {
                state.preview_scroll = 0;
                return;
            }
            _ => {}
        }
    }
    // Overlays swallow input first.
    if state.help_open && !matches!(action, Action::Tick | Action::Resize | Action::Quit) {
        if matches!(
            action,
            Action::ToggleHelp | Action::Back | Action::Activate | Action::Click { .. }
        ) {
            state.help_open = false;
        }
        return;
    }
    if state.dropdown.is_some() && dropdown_input(state, &action) {
        return;
    }
    match action {
        Action::FocusPreview => {
            state.preview_focus = !state.preview_focus;
            state.dropdown = None;
        }
        Action::FilterThemes => {
            if state.screen == Screen::Themes {
                state.theme_filter = (state.theme_filter + 1) % 6;
                state.grid_scroll = 0;
                select_filtered(state);
            }
        }
        Action::SearchThemes => {
            if state.screen == Screen::Themes {
                state.editor = Some("search".into());
                state.input = state.theme_search.clone();
            }
        }
        Action::Text(_) | Action::Erase => {}
        Action::Quit => quit(state),
        Action::Back if state.preview_focus => state.preview_focus = false,
        Action::Back => match state.focus {
            Focus::Sidebar => quit(state),
            Focus::Content | Focus::Options => state.focus = Focus::Sidebar,
        },
        Action::ToggleFocus => cycle_focus(state, true),
        Action::FocusPrev => cycle_focus(state, false),
        Action::Goto(screen) => goto(state, screen),
        Action::Number(n) => {
            let n = usize::from(n);
            if state.screen == Screen::Dashboard
                && state.focus == Focus::Content
                && (1..=QUICK_ACTIONS).contains(&n)
            {
                state.dash = DashCursor {
                    row: DashRow::Quick,
                    col: n - 1,
                };
                quick_action(state, n - 1);
            } else if let Some(&screen) = Screen::ALL.get(n.wrapping_sub(1)) {
                goto(state, screen);
            }
        }
        Action::ToggleHelp => state.help_open = !state.help_open,
        Action::CycleOutput => {
            let i = state.option_index(3);
            set_option(state, 3, (i + 1) % PreviewOutput::ALL.len());
        }
        Action::Up => navigate(state, -1),
        Action::Down => navigate(state, 1),
        Action::PageUp => navigate(state, -(PAGE as isize)),
        Action::PageDown => navigate(state, PAGE as isize),
        Action::Home => navigate(state, -10_000),
        Action::End => navigate(state, 10_000),
        Action::Left => horizontal(state, false),
        Action::Right => horizontal(state, true),
        Action::Activate => activate(state),
        Action::Toggle => {
            if state.screen == Screen::Modules && state.focus == Focus::Content {
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
        Action::ToggleDemo if state.screen == Screen::Presets && state.focus == Focus::Content => {
            let mut p = state.current_preset().clone();
            let mut n = 1;
            while state.presets.iter().any(|p| p.id == format!("user-{n}")) {
                n += 1;
            }
            p.id = format!("user-{n}");
            p.name = format!("{} copy", p.name);
            state.settings.preset = p.id.clone();
            state.presets.push(p);
            persist_presets(state);
        }
        Action::ToggleDemo => toggle_demo(state),
        Action::CycleLogoSize => cycle_logo_size(state, true),
        Action::RunFastfetch => run_fastfetch(state),
        Action::Refresh => match state.screen {
            Screen::Presets => {
                state.editor = Some("rename".into());
                state.input = state.current_preset().name.clone();
            }
            Screen::Doctor => run_doctor(state),
            Screen::Backup => {
                state.refresh_backups();
                state.set_status(
                    StatusKind::Info,
                    format!("{} backups found", state.backups.len()),
                );
            }
            Screen::Logos => {
                let logo = state.current_logo().clone();
                if let Some(p) = logo.file_path.as_deref() {
                    state.pool.invalidate(p);
                    state.pool.request(p, true);
                    if let Some(k) = state.kitty.as_mut() {
                        state
                            .effects
                            .extend(k.invalidate(p).into_iter().map(Effect::WriteTerminal));
                    }
                }
            }
            _ => {}
        },
        Action::Delete => {
            if state.screen == Screen::Presets {
                if state.presets.len() > 1 {
                    state.editor = Some("delete".into());
                    state.input.clear();
                } else {
                    state.set_status(StatusKind::Info, "At least one preset must remain");
                }
            } else if state.screen == Screen::Backup {
                delete_backup(state);
            }
        }
        Action::Undo => {
            if let Some(prev) = state.undo_stack.pop() {
                state.redo_stack.push(state.settings.clone());
                state.settings = prev;
                state.sync_custom_preset();
                state.set_status(StatusKind::Info, "↺ Undid last change");
            } else {
                state.set_status(StatusKind::Info, "Nothing to undo");
            }
        }
        Action::Redo => {
            if let Some(next) = state.redo_stack.pop() {
                state.undo_stack.push(state.settings.clone());
                state.settings = next;
                state.sync_custom_preset();
                state.set_status(StatusKind::Info, "↻ Redid change");
            } else {
                state.set_status(StatusKind::Info, "Nothing to redo");
            }
        }
        Action::Click { col, row } => click(state, col, row),
        Action::ScrollUp => navigate(state, -1),
        Action::ScrollDown => navigate(state, 1),
        Action::Resize => state.geo = TermGeometry::detect(),
        Action::Tick => tick(state),
    }
}

/// Whether the Preview Options card was drawn in the last frame (it is
/// hidden on narrow terminals, so `Tab` must skip it).
fn options_visible(state: &AppState) -> bool {
    state.hits.iter().any(|(_, h)| matches!(h, Hit::Option(_)))
}

fn cycle_focus(state: &mut AppState, forward: bool) {
    let mut next = if forward {
        state.focus.next()
    } else {
        state.focus.prev()
    };
    if next == Focus::Options && !options_visible(state) {
        next = if forward { next.next() } else { next.prev() };
    }
    state.focus = next;
}

fn run_fastfetch(state: &mut AppState) {
    if state.fastfetch_version.is_some() {
        state.effects.push(Effect::RunFastfetch);
    } else {
        state.set_status(StatusKind::Error, "fastfetch is not installed - see Doctor");
    }
}

fn cycle_logo_size(state: &mut AppState, forward: bool) {
    state.push_undo();
    let s = state.settings.logo_size;
    state.settings.logo_size = if forward { s.next() } else { s.next().next() };
    let size = state.settings.logo_size;
    let (w, h) = size.max_cells();
    state.set_status(
        StatusKind::Info,
        format!(
            "Logo size: {} (max {w}x{h} cells) - Enter to apply",
            size.label()
        ),
    );
}

/// Keys while a Preview Options dropdown is open. Returns whether the action was consumed.
fn dropdown_input(state: &mut AppState, action: &Action) -> bool {
    let Some(dd) = state.dropdown.clone() else {
        return false;
    };
    let len = state.option_len(dd.field);
    let cur = state.option_index(dd.field);
    let delta = match action {
        Action::Up | Action::Left | Action::ScrollUp => -1,
        Action::Down | Action::Right | Action::ScrollDown => 1,
        Action::PageUp => -(PAGE as isize),
        Action::PageDown => PAGE as isize,
        Action::Home => -10_000,
        Action::End => 10_000,
        Action::Activate | Action::Toggle | Action::ToggleFocus | Action::FocusPrev => {
            state.dropdown = None;
            return true;
        }
        Action::Back => {
            set_option(state, dd.field, dd.original);
            state.dropdown = None;
            return true;
        }
        Action::Click { .. } => return false,
        _ => return false,
    };
    set_option(state, dd.field, step(cur, len, delta, true));
    true
}

/// Select choice `i` of a Preview Options field (previews, does not apply).
fn set_option(state: &mut AppState, field: usize, i: usize) {
    match field {
        0 => {
            if let Some(t) = state.themes.get(i) {
                state.settings.theme = t.id.clone();
                let name = t.name.clone();
                state.set_status(
                    StatusKind::Info,
                    format!("Previewing theme {name} - A to apply"),
                );
            }
        }
        1 => {
            if let Some(p) = state.presets.get(i) {
                state.settings.preset = p.id.clone();
                let name = p.name.clone();
                state.set_status(
                    StatusKind::Info,
                    format!("Previewing preset {name} - A to apply"),
                );
            }
        }
        2 if i < state.logos.len() => select_logo(state, i),
        3 => state.preview_output = PreviewOutput::ALL.get(i).copied().unwrap_or_default(),
        _ => {}
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

/// Jump to a screen with the center stage focused.
fn open(state: &mut AppState, screen: Screen) {
    goto(state, screen);
    state.focus = Focus::Content;
}

fn navigate(state: &mut AppState, delta: isize) {
    match state.focus {
        Focus::Sidebar => {
            let idx = step(state.screen.index(), Screen::ALL.len(), delta, true);
            goto(state, Screen::ALL[idx]);
            return;
        }
        Focus::Options => {
            state.options_cursor = step(state.options_cursor, OPTION_ROWS, delta, true);
            return;
        }
        Focus::Content => {}
    }
    match state.screen {
        Screen::Themes => {
            // Vertical moves jump a whole row of the card grid.
            let cols = state.grid_cols.max(1) as isize;
            let list = state.filtered_themes();
            if !list.is_empty() {
                let pos = list
                    .iter()
                    .position(|&i| i == state.theme_idx())
                    .unwrap_or(0);
                let i = step(pos, list.len(), delta.saturating_mul(cols), false);
                set_option(state, 0, list[i]);
            }
        }
        Screen::Presets => {
            let i = step(state.preset_idx(), state.presets.len(), delta, true);
            set_option(state, 1, i);
        }
        Screen::Logos => {
            let i = step(state.logo_idx(), state.logos.len(), delta, true);
            select_logo(state, i);
        }
        Screen::Modules => {
            state.module_cursor = step(state.module_cursor, state.module_rows().len(), delta, true);
        }
        Screen::Settings => {
            state.color_cursor = step(state.color_cursor, SETTINGS_ROWS, delta, true)
        }
        Screen::Backup => {
            state.backup_cursor = step(state.backup_cursor, state.backups.len() + 1, delta, true)
        }
        Screen::Doctor => {
            state.doctor_cursor = step(state.doctor_cursor, state.doctor.len(), delta, false)
        }
        Screen::Dashboard => dash_vertical(state, delta),
    }
}

fn dash_vertical(state: &mut AppState, delta: isize) {
    let recent = state.recent_theme_indices().len();
    let d = &mut state.dash;
    match (d.row, delta.signum()) {
        (DashRow::Quick, 1) => {
            *d = DashCursor {
                row: DashRow::Recent,
                col: d.col.min(recent.saturating_sub(1)),
            }
        }
        (DashRow::Recent, 1) => {
            *d = DashCursor {
                row: DashRow::Tools,
                col: 0,
            }
        }
        (DashRow::Recent, -1) => {
            *d = DashCursor {
                row: DashRow::Quick,
                col: d.col.min(QUICK_ACTIONS - 1),
            }
        }
        (DashRow::Tools, -1) if d.col == 0 => {
            *d = DashCursor {
                row: DashRow::Recent,
                col: state.recent_offset.min(recent.saturating_sub(1)),
            }
        }
        (DashRow::Tools, s) => d.col = (d.col as isize + s).clamp(0, TOOLS as isize - 1) as usize,
        _ => {}
    }
}

fn horizontal(state: &mut AppState, forward: bool) {
    let dir: isize = if forward { 1 } else { -1 };
    match state.focus {
        Focus::Sidebar => {
            if forward {
                state.focus = Focus::Content;
            }
            return;
        }
        Focus::Options => {
            let f = state.options_cursor;
            if f < OPTION_APPLY {
                let i = step(state.option_index(f), state.option_len(f), dir, true);
                set_option(state, f, i);
            }
            return;
        }
        Focus::Content => {}
    }
    match state.screen {
        Screen::Settings => cycle_setting(state, forward),
        Screen::Themes => {
            let cols = state.grid_cols.max(1);
            let list = state.filtered_themes();
            if list.is_empty() {
                return;
            }
            let i = list
                .iter()
                .position(|&i| i == state.theme_idx())
                .unwrap_or(0);
            if !forward && i % cols == 0 {
                state.focus = Focus::Sidebar;
            } else {
                set_option(state, 0, list[step(i, list.len(), dir, false)]);
            }
        }
        Screen::Dashboard => {
            let len = match state.dash.row {
                DashRow::Quick => QUICK_ACTIONS,
                DashRow::Recent => state.recent_theme_indices().len(),
                DashRow::Tools => 1,
            };
            let col = state.dash.col as isize + dir;
            if col < 0 {
                state.focus = Focus::Sidebar;
            } else if state.dash.row != DashRow::Tools {
                state.dash.col = (col as usize).min(len.saturating_sub(1));
            }
        }
        _ => {
            if !forward {
                state.focus = Focus::Sidebar;
            } else if options_visible(state) {
                state.focus = Focus::Options;
            }
        }
    }
}

fn select_logo(state: &mut AppState, i: usize) {
    let logo = state.logos[i].clone();
    state.settings.logo = logo.id.clone();
    if let Some(p) = logo.existing_image() {
        state.pool.request(p, true);
    }
    state.set_status(
        StatusKind::Info,
        format!("Previewing logo {} - Enter to apply", logo.name),
    );
}

fn quick_action(state: &mut AppState, i: usize) {
    match i {
        0 => open(state, Screen::Themes),
        1 => open(state, Screen::Presets),
        2 => open(state, Screen::Logos),
        _ => {
            if apply(state) {
                run_fastfetch(state);
            }
        }
    }
}

fn tool_action(state: &mut AppState, i: usize) {
    match i {
        0 => open(state, Screen::Doctor),
        1 => {
            let c = doctor::shell_integration();
            let hint = if c.level == doctor::Level::Ok {
                String::new()
            } else {
                " - add `fastfetch` to ~/.zshrc, ~/.bashrc or ~/.config/fish/config.fish".into()
            };
            let kind = if c.level == doctor::Level::Ok {
                StatusKind::Ok
            } else {
                StatusKind::Info
            };
            state.set_status(kind, format!("Shell integration: {}{hint}", c.detail));
        }
        2 => open(state, Screen::Backup),
        _ => {
            open(state, Screen::Backup);
            let dir = state.paths.backups_dir();
            state.set_status(StatusKind::Info, format!("Export: create a backup and copy its folder from {}; import: drop a backup folder there and restore it", dir.display()));
        }
    }
}

/// Apply the theme of a Recent Themes card.
fn apply_recent(state: &mut AppState, pos: usize) {
    let Some(&ti) = state.recent_theme_indices().get(pos) else {
        return;
    };
    state.settings.theme = state.themes[ti].id.clone();
    if apply(state) {
        // The applied theme moves to the front of the row.
        state.dash = DashCursor {
            row: DashRow::Recent,
            col: 0,
        };
        state.recent_offset = 0;
    }
}

fn activate(state: &mut AppState) {
    match state.focus {
        Focus::Sidebar => {
            state.focus = Focus::Content;
            return;
        }
        Focus::Options => {
            if state.options_cursor == OPTION_APPLY {
                apply(state);
            } else {
                let field = state.options_cursor;
                state.dropdown = Some(Dropdown {
                    field,
                    original: state.option_index(field),
                });
            }
            return;
        }
        Focus::Content => {}
    }
    match state.screen {
        Screen::Dashboard => match state.dash.row {
            DashRow::Quick => quick_action(state, state.dash.col),
            DashRow::Recent => apply_recent(state, state.dash.col),
            DashRow::Tools => tool_action(state, state.dash.col),
        },
        Screen::Modules => toggle_module(state),
        Screen::Settings => match state.color_cursor {
            7 => {
                state.push_undo();
                state.settings.reset_colors();
                state.set_status(
                    StatusKind::Info,
                    "Colours reset to theme defaults - a to apply",
                );
            }
            _ => cycle_setting(state, true),
        },
        Screen::Backup => {
            if state.backup_cursor == 0 {
                create_backup(state);
            } else {
                restore_backup(state);
            }
        }
        Screen::Doctor => run_doctor(state),
        Screen::Themes | Screen::Presets | Screen::Logos => {
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
        state.push_undo();
        state.settings.custom_modules.retain(|&x| x != m);
    } else {
        state.push_undo();
        state.settings.custom_modules.push(m);
    }
    // Keep the cursor on the toggled module.
    state.module_cursor = state
        .module_rows()
        .iter()
        .position(|&(x, _)| x == m)
        .unwrap_or(0);
    after_module_edit(
        state,
        format!(
            "{} {}",
            if enabled { "Disabled" } else { "Enabled" },
            m.label()
        ),
    );
}

fn move_module(state: &mut AppState, up: bool) {
    let rows = state.module_rows();
    let Some(&(m, true)) = rows.get(state.module_cursor) else {
        state.set_status(StatusKind::Warn, "Only enabled modules can be reordered");
        return;
    };
    let Some(i) = state.settings.custom_modules.iter().position(|&x| x == m) else {
        return;
    };
    let j = if up {
        i.checked_sub(1)
    } else {
        (i + 1 < state.settings.custom_modules.len()).then_some(i + 1)
    };
    if let Some(j) = j {
        state.push_undo();
        state.settings.custom_modules.swap(i, j);
        state.module_cursor = j;
        after_module_edit(
            state,
            format!("Moved {} {}", m.label(), if up { "up" } else { "down" }),
        );
    }
}

fn after_module_edit(state: &mut AppState, msg: String) {
    state.settings.preset = "custom".into();
    state.sync_custom_preset();
    state.set_status(
        StatusKind::Info,
        format!("{msg} - Custom preset selected, a to apply"),
    );
}

fn cycle_setting(state: &mut AppState, forward: bool) {
    state.push_undo();
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
        9 => toggle_demo(state),
        10 => cycle_logo_size(state, forward),
        11 => s.run_fastfetch_on_exit = !s.run_fastfetch_on_exit,
        12 => s.ui_follow_theme = !s.ui_follow_theme,
        13 => open(state, Screen::Doctor),
        14 => open(state, Screen::Backup),
        _ => {}
    }
}

/// Persist settings and write both fastfetch configs.
fn write_all(state: &mut AppState, settings: &Settings) -> anyhow::Result<Vec<std::path::PathBuf>> {
    let theme = state
        .themes
        .iter()
        .find(|t| t.id == settings.theme)
        .unwrap_or(&state.themes[0])
        .clone();
    let preset = state
        .presets
        .iter()
        .find(|p| p.id == settings.preset)
        .unwrap_or(&state.presets[0])
        .clone();
    let logo = state
        .logos
        .iter()
        .find(|l| l.id == settings.logo)
        .unwrap_or(&state.logos[0])
        .clone();
    settings.save(&state.paths.settings_file())?;
    config::apply(&theme, &preset, &logo, settings, &state.paths, &state.geo)
}

/// Leaving the TUI commits whatever is on screen: unsaved edits are written to
/// settings.json and both fastfetch configs so the next `fastfetch` shows them.
fn quit(state: &mut AppState) {
    if state.unsaved() {
        // If the user only previewed a theme or preset without applying it,
        // and made no other modifications, do not overwrite their system configs!
        if (state.is_previewing_theme()
            || state.settings.preset != state.saved.preset
            || state.settings.logo != state.saved.logo)
            && !state.has_explicit_modifications()
        {
            state.settings.theme = state.saved.theme.clone();
            state.settings.preset = state.saved.preset.clone();
            state.settings.logo = state.saved.logo.clone();
            state.running = false;
            return;
        }
        let ok = apply(state);
        let (kind, msg) = state.status.clone();
        let msg = if ok {
            msg.replacen("✓ Applied", "✓ Saved on exit:", 1)
        } else {
            format!("Changes were NOT saved - {msg}")
        };
        state.exit_message = Some((kind, msg));
    }
    state.running = false;
}

/// Returns whether the settings were written.
fn apply(state: &mut AppState) -> bool {
    let _ = crate::adapters::backup::create_backup(&state.paths);
    let theme = state.settings.theme.clone();
    state.settings.remember_theme(&theme);
    let settings = state.settings.clone();
    match write_all(state, &settings) {
        Ok(targets) => {
            state.saved = settings;
            state.undo_stack.clear();
            state.redo_stack.clear();
            if state.saved.apply_terminal_colors {
                let osc = state.current_theme().osc_sequences();
                state.effects.push(Effect::WriteTerminal(osc));
            }
            let list = targets
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(" + ");
            let (theme, preset) = (
                state.current_theme().name.clone(),
                state.current_preset().name.clone(),
            );
            state.set_status(
                StatusKind::Ok,
                format!("✓ Applied {theme} · {preset} → {list}"),
            );
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
            let what = if on {
                "ON - privacy-safe mock specs"
            } else {
                "OFF - real system specs"
            };
            let tail = if regenerate {
                " (fastfetch config regenerated)"
            } else {
                ""
            };
            state.set_status(
                if on { StatusKind::Ok } else { StatusKind::Warn },
                format!("Preview mode {what}{tail}"),
            );
        }
        Err(e) => state.set_status(
            StatusKind::Error,
            format!("Could not save preview mode: {e:#}"),
        ),
    }
    if !on {
        state.start_real_probe();
    }
}

fn create_backup(state: &mut AppState) {
    match backup::create_backup(&state.paths) {
        Ok(entry) => {
            state.refresh_backups();
            state.set_status(
                StatusKind::Ok,
                format!(
                    "✓ Backup {} created ({} files)",
                    entry.name,
                    entry.files.len()
                ),
            );
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
            state.set_status(
                StatusKind::Ok,
                format!(
                    "✓ Restored {n} files from {} (previous state saved as pre-restore backup)",
                    entry.name
                ),
            );
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
    let fails = state
        .doctor
        .iter()
        .filter(|c| c.level == doctor::Level::Fail)
        .count();
    let warns = state
        .doctor
        .iter()
        .filter(|c| c.level == doctor::Level::Warn)
        .count();
    let kind = if fails > 0 {
        StatusKind::Error
    } else if warns > 0 {
        StatusKind::Warn
    } else {
        StatusKind::Ok
    };
    state.set_status(
        kind,
        format!(
            "Doctor: {} checks, {fails} failed, {warns} warnings",
            state.doctor.len()
        ),
    );
}

fn click(state: &mut AppState, col: u16, row: u16) {
    let inside = |r: &ratatui::layout::Rect| {
        col >= r.x && col < r.x + r.width && row >= r.y && row < r.y + r.height
    };
    let hit = state
        .hits
        .iter()
        .rev()
        .find(|(r, _)| inside(r))
        .map(|&(_, h)| h);
    if let Some(dd) = state.dropdown.clone() {
        if let Some(Hit::DropItem(i)) = hit {
            set_option(state, dd.field, i);
        }
        state.dropdown = None;
        return;
    }
    if let Some(hit) = hit {
        match hit {
            Hit::Nav(screen) => {
                state.focus = Focus::Sidebar;
                goto(state, screen);
            }
            Hit::Quick(i) => {
                state.focus = Focus::Content;
                state.dash = DashCursor {
                    row: DashRow::Quick,
                    col: i,
                };
                quick_action(state, i);
            }
            Hit::Recent(i) => {
                state.focus = Focus::Content;
                state.dash = DashCursor {
                    row: DashRow::Recent,
                    col: i,
                };
                apply_recent(state, i);
            }
            Hit::RecentScroll => {
                let len = state.recent_theme_indices().len();
                let n = state.recent_visible.max(1);
                state.recent_offset = if state.recent_offset + n >= len {
                    0
                } else {
                    state.recent_offset + 1
                };
            }
            Hit::ViewAll => open(state, Screen::Themes),
            Hit::Tool(i) => {
                state.focus = Focus::Content;
                state.dash = DashCursor {
                    row: DashRow::Tools,
                    col: i,
                };
                tool_action(state, i);
            }
            Hit::Option(i) => {
                state.focus = Focus::Options;
                state.options_cursor = i;
                activate(state);
            }
            Hit::FocusPreview => state.preview_focus = !state.preview_focus,
            Hit::ThemeFilter => {
                state.theme_filter = (state.theme_filter + 1) % 6;
                state.grid_scroll = 0;
                select_filtered(state);
            }
            Hit::ThemeCard(i) => {
                state.focus = Focus::Content;
                if state.theme_idx() == i {
                    apply(state);
                } else {
                    set_option(state, 0, i);
                }
            }
            Hit::DropItem(_) => {}
        }
        return;
    }
    let Some(idx) = state.click.hit(col, row) else {
        return;
    };
    state.focus = Focus::Content;
    match state.screen {
        Screen::Presets => set_option(state, 1, idx),
        Screen::Logos => select_logo(state, idx),
        Screen::Modules => state.module_cursor = idx,
        Screen::Settings => state.color_cursor = idx,
        Screen::Backup => state.backup_cursor = idx,
        Screen::Doctor => state.doctor_cursor = idx,
        Screen::Themes | Screen::Dashboard => {}
    }
}

fn tick(state: &mut AppState) {
    let clock = state.sysinfo.clock();
    if clock != state.clock {
        state.clock = clock;
        state.dirty = true;
    }
    if let Some(k) = state.kitty.as_mut() {
        let uploads = k.poll();
        if !uploads.is_empty() {
            state
                .effects
                .extend(uploads.into_iter().map(Effect::WriteTerminal));
            state.dirty = true;
        }
    }
    let done = state.pool.poll();
    if let Some(sel) = state.current_logo().existing_image()
        && done.iter().any(|p| p == sel)
    {
        state.dirty = true;
    }
    if let Some(rx) = &state.real_rx
        && let Ok(info) = rx.try_recv()
    {
        state.real_info = Some(info);
        state.real_rx = None;
        state.dirty = true;
    }
}

fn select_filtered(state: &mut AppState) {
    let list = state.filtered_themes();
    if !list.contains(&state.theme_idx())
        && let Some(&i) = list.first()
    {
        set_option(state, 0, i);
    }
}
fn persist_presets(state: &mut AppState) {
    let custom = serde_json::json!({"presets": &state.presets});
    let result = serde_json::to_vec_pretty(&custom)
        .map_err(anyhow::Error::from)
        .and_then(|bytes| {
            crate::adapters::fsutil::atomic_write(
                &state.paths.config_dir.join("presets.json"),
                &bytes,
            )
        });
    if let Err(e) = result {
        state.set_status(StatusKind::Error, format!("Cannot save presets: {e}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::Paths;

    fn app() -> (AppState, std::path::PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "tfs-reducer-{}-{}",
            std::process::id(),
            rand_suffix()
        ));
        let paths = Paths::with_root(&root);
        let geo = TermGeometry::from_window(160, 48, 1600, 960);
        (AppState::new(paths, Settings::default(), geo), root)
    }

    fn rand_suffix() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
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
        reduce(&mut s, Action::Goto(Screen::Settings));
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
        assert!(
            matches!(&s.exit_message, Some((StatusKind::Ok, m)) if m.contains("Saved on exit")),
            "{:?}",
            s.exit_message
        );
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
        reduce(&mut s, Action::Goto(Screen::Settings));
        s.focus = Focus::Content;
        reduce(&mut s, Action::Right);
        assert_ne!(
            s.settings.icon_color,
            crate::domain::color::ColorValue::Theme
        );
        s.color_cursor = 7;
        reduce(&mut s, Action::Activate);
        assert_eq!(
            s.settings.icon_color,
            crate::domain::color::ColorValue::Theme
        );
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
        s.backup_cursor = s
            .backups
            .iter()
            .position(|b| !b.name.contains("pre-restore"))
            .unwrap()
            + 1;
        reduce(&mut s, Action::Activate);
        assert_eq!(s.settings.theme, original_theme, "{:?}", s.status);
        assert!(s.backups.iter().any(|b| b.name.contains("pre-restore")));
        let _ = std::fs::remove_dir_all(root);
    }
}
