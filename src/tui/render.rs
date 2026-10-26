use crate::adapters::fastfetch::config::LOGO_PADDING_TOP;
use crate::app::state::{AppState, PreviewOutput, Screen};
use crate::imaging::kitty::ImageKey;
use crate::imaging::sixel::{ImageKey as SixelImage, Slot};
use crate::tui::screens::{
    backup::render_backup, dashboard::render_dashboard, doctor::render_doctor, logos::render_logos,
    modules::render_modules, presets::render_presets, settings::render_settings,
    themes::render_themes,
};
use crate::tui::widgets::footer::render_footer;
use crate::tui::widgets::header::render_header;
use crate::tui::widgets::options::{options_height, render_options};
use crate::tui::widgets::preview::{
    ImageState, LogoRaster, PreviewModel, draw_preview_chrome, draw_preview_scrolled, inner_area,
    rgb,
};
use crate::tui::widgets::sidebar::render_sidebar;
use crate::tui::widgets::ui::{card, fill, key_badge, line_at};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};

/// Column widths for the three-pane layout at a given terminal width:
/// (sidebar, preview); the center stage gets the rest.
fn columns(w: u16) -> (u16, u16) {
    match w {
        // Very wide / standard developer terminals (180+ cols):
        // Preserve 44% of the full terminal for preview, including ultrawide screens.
        w if w >= 180 => (24, w * 44 / 100),
        // Wide terminals (150-179 cols):
        w if w >= 150 => (24, w * 44 / 100),
        // Medium terminals (125-149 cols):
        w if w >= 125 => (22, w * 44 / 100),
        // Compact terminals (100-124 cols):
        w if w >= 100 => (22, w * 44 / 100),
        // Narrow terminals (72-99 cols): sidebar + center only, preview collapsed.
        w if w >= 72 => (22, 0),
        _ => (0, 0),
    }
}

pub fn render_ui(f: &mut Frame, state: &mut AppState) {
    // Paint the canvas ourselves: legibility must not depend on whatever
    // background the host terminal happens to have.
    let pal = state.palette();
    let full = f.area();
    fill(f.buffer_mut(), full, Style::new().bg(pal.bg).fg(pal.fg));
    state.hits.clear();
    state.preview_body = Rect::default();
    let bars = if full.height >= 24 { 3 } else { 1 };
    let [top, body, bottom] = Layout::vertical([
        Constraint::Length(bars),
        Constraint::Min(4),
        Constraint::Length(bars),
    ])
    .areas(full);
    render_header(f, top, state);

    if state.preview_focus {
        render_preview_pane(f, body, state);
        render_footer(f, bottom, state);
        render_dialog(f, state);
        return;
    }
    let (side_w, preview_w) = columns(body.width);
    let [side, center, preview] = Layout::horizontal([
        Constraint::Length(side_w),
        Constraint::Min(20),
        Constraint::Length(preview_w),
    ])
    .areas(body);
    if side_w > 0 {
        render_sidebar(f, side, state);
    }
    render_screen(
        f,
        Rect {
            y: center.y + 1,
            height: center.height.saturating_sub(1),
            ..center
        },
        state,
    );
    if preview_w > 0 {
        let col = Rect {
            x: preview.x + 1,
            width: preview.width.saturating_sub(2),
            y: preview.y + 1,
            height: preview.height.saturating_sub(1),
        };
        let opt_h = options_height(col.height);
        let [pv, opts] =
            Layout::vertical([Constraint::Min(4), Constraint::Length(opt_h)]).areas(col);
        render_preview_pane(f, pv, state);
        if opt_h > 0 {
            render_options(f, opts, state);
        }
    }
    render_footer(f, bottom, state);
    render_dialog(f, state);
    if state.help_open {
        render_help(f, state);
    }
}

fn render_screen(f: &mut Frame, area: Rect, state: &mut AppState) {
    state.click = Default::default();
    match state.screen {
        Screen::Dashboard => render_dashboard(f, area, state),
        Screen::Themes => render_themes(f, area, state),
        Screen::Presets => render_presets(f, area, state),
        Screen::Logos => render_logos(f, area, state),
        Screen::Modules => render_modules(f, area, state),
        Screen::Settings => render_settings(f, area, state),
        Screen::Backup => render_backup(f, area, state),
        Screen::Doctor => render_doctor(f, area, state),
    }
}

pub fn image_state(state: &AppState) -> ImageState {
    let logo = state.current_logo();
    if !logo.is_image() {
        return ImageState::NotImage;
    }
    match logo.file_path.as_deref() {
        Some(p) => match state.pool.get(p) {
            Some(img) => ImageState::Ready(img.dims),
            None if state.pool.is_pending(p) => ImageState::Loading,
            None => ImageState::Missing,
        },
        None => ImageState::Missing,
    }
}

fn preview_model(state: &AppState, image: ImageState) -> PreviewModel<'_> {
    PreviewModel {
        theme: state.current_theme(),
        preset: state.current_preset(),
        logo: state.current_logo(),
        settings: &state.settings,
        info: state.real_info.as_ref(),
        info_loading: state.real_rx.is_some(),
        image,
        geo: state.geo,
        fill: state.block_logos,
    }
}

/// Kitty rendition of `path` shown in `cells` at the terminal's cell pixel size.
fn kitty_key(state: &AppState, path: &str, cells: (u16, u16)) -> ImageKey {
    let cell = state.geo.cell;
    ImageKey {
        path: path.to_string(),
        cells,
        px: (u32::from(cells.0) * cell.w, u32::from(cells.1) * cell.h),
    }
}

/// Pre-upload every decoded image logo at the size the preview would show it,
/// so browsing the gallery only switches placeholder ids.
fn kitty_prefetch(state: &mut AppState, avail: (u16, u16)) {
    let keys: Vec<ImageKey> = state
        .logos
        .iter()
        .filter_map(|l| l.existing_image())
        .filter_map(|p| state.pool.get(p).map(|img| (p, img.dims)))
        .map(|(p, dims)| {
            let cells = preview_model(state, ImageState::Ready(dims)).logo_cells(avail);
            kitty_key(state, p, cells)
        })
        .collect();
    if let Some(k) = state.kitty.as_mut() {
        keys.iter().for_each(|key| k.request(key, false));
    }
}

fn render_preview_pane(f: &mut Frame, area: Rect, state: &mut AppState) {
    let control = Rect {
        x: area.right().saturating_sub(7),
        y: area.y,
        width: 6.min(area.width),
        height: 1,
    };
    state
        .hits
        .push((control, crate::app::state::Hit::FocusPreview));
    state.preview_body = Block::new().borders(Borders::ALL).inner(area);
    if state.preview_output == PreviewOutput::Config {
        render_config_output(f, area, state);
        let pal = state.palette();
        line_at(
            f.buffer_mut(),
            control,
            0,
            Line::from(Span::styled(
                "[⛶] f",
                Style::new().fg(pal.primary).bg(pal.surface),
            )),
        );
        return;
    }
    let image = image_state(state);
    let inner = inner_area(area);
    let avail = (inner.width, inner.height);
    // Size first (immutable borrow), then fetch/cached-rasterise the logo.
    let cells = preview_model(state, image).logo_cells(avail);
    let path = state.current_logo().file_path.clone();
    let mut kitty_id = None;
    if let (ImageState::Ready(_), Some(p), true) = (image, path.as_deref(), state.kitty.is_some()) {
        let key = kitty_key(state, p, cells);
        let k = state.kitty.as_mut().expect("checked above");
        kitty_id = k.id(&key);
        if kitty_id.is_none() {
            k.request(&key, true);
        }
        kitty_prefetch(state, avail);
    }
    // Sixel is next when the terminal has no Kitty graphics; half blocks are the last fallback.
    let mut sixel_on = false;
    if let (None, ImageState::Ready(_), Some(p)) = (kitty_id, image, path.as_deref())
        && state.sixel.enabled
    {
        let cell = state.geo.cell;
        let bg = rgb(preview_model(state, image).bg());
        let slot = Slot {
            x: inner.x,
            y: inner.y + LOGO_PADDING_TOP,
            image: SixelImage {
                path: p.to_string(),
                px: (u32::from(cells.0) * cell.w, u32::from(cells.1) * cell.h),
                bg,
            },
        };
        sixel_on = state.sixel.want(slot);
    }
    // Half blocks are the fallback until the full-resolution upload is in the terminal.
    let grid = match (image, kitty_id, path) {
        (ImageState::Ready(_), None, Some(p)) if !sixel_on => state.pool.grid(&p, cells.0, cells.1),
        _ => None,
    };
    let raster = match (kitty_id, grid.as_deref()) {
        (Some(id), _) => LogoRaster::Kitty(id),
        (None, _) if sixel_on => LogoRaster::Sixel,
        (None, Some(g)) => LogoRaster::Blocks(g),
        (None, None) => LogoRaster::None,
    };
    let pal = state.palette();
    let model = preview_model(state, image);
    draw_preview_scrolled(
        f.buffer_mut(),
        area,
        &model,
        raster,
        Some(&pal),
        if state.preview_focus {
            state.preview_scroll
        } else {
            0
        },
    );
    line_at(
        f.buffer_mut(),
        control,
        0,
        Line::from(Span::styled(
            "[⛶] f",
            Style::new().fg(pal.primary).bg(pal.surface),
        )),
    );
}

/// The config `Enter`/`a` would write, shown in the preview box.
fn render_config_output(f: &mut Frame, area: Rect, state: &mut AppState) {
    let pal = state.palette();
    let stale = state
        .config_cache
        .as_ref()
        .is_none_or(|(s, _)| *s != state.settings);
    if stale {
        let (t, p, l) = (
            state.current_theme(),
            state.current_preset(),
            state.current_logo(),
        );
        let text = crate::adapters::fastfetch::config::render_dry(
            t,
            p,
            l,
            &state.settings,
            &state.paths,
            &state.geo,
        )
        .unwrap_or_else(|e| format!("// cannot render config: {e:#}"));
        state.config_cache = Some((state.settings.clone(), text));
    }
    let body = draw_preview_chrome(
        f.buffer_mut(),
        area,
        &pal,
        &state.current_preset().name,
        state.settings.demo_mode,
        false,
    );
    fill(
        f.buffer_mut(),
        body,
        Style::new().bg(pal.surface).fg(pal.fg),
    );
    let text = state
        .config_cache
        .as_ref()
        .map(|(_, t)| t.as_str())
        .unwrap_or_default();
    let body = Rect {
        x: body.x + 1,
        width: body.width.saturating_sub(1),
        ..body
    };
    for (i, l) in text.lines().take(usize::from(body.height)).enumerate() {
        let line = match l.split_once(':') {
            Some((k, v)) if k.trim_start().starts_with('"') => Line::from(vec![
                Span::styled(format!("{k}:"), Style::new().fg(pal.link)),
                Span::styled(v.to_string(), Style::new().fg(pal.fg)),
            ]),
            _ if l.trim_start().starts_with("//") => {
                Line::from(Span::styled(l.to_string(), Style::new().fg(pal.dim)))
            }
            _ => Line::from(Span::styled(l.to_string(), Style::new().fg(pal.muted))),
        };
        line_at(f.buffer_mut(), body, i as u16, line);
    }
}

const HELP: [(&str, &str); 18] = [
    (
        "Tab / Shift+Tab",
        "next / previous section (menu → stage → options)",
    ),
    ("↑↓ ←→  hjkl", "move; the live preview follows instantly"),
    ("Enter", "select: open, apply or toggle the focused item"),
    ("a / A", "apply theme to system (both fastfetch configs)"),
    (
        "1-8",
        "jump to a screen (Dashboard stage: 1-4 quick actions)",
    ),
    ("p", "run the real fastfetch with the current selection"),
    ("i", "install fastfetch (press twice)"),
    (
        "d / r / x",
        "Presets: duplicate / rename / confirm deletion",
    ),
    ("s", "cycle logo size"),
    (
        "f / Esc",
        "expand preview / return; ↑↓ scroll full system values",
    ),
    ("c / /", "Themes: category / search"),
    ("Space  J/K", "Modules: toggle / reorder"),
    ("x / r", "Backup: delete · Doctor: re-run"),
    ("o", "cycle preview output (terminal / config)"),
    ("u / U", "undo / redo changes (Ctrl+Z / Ctrl+Y)"),
    ("Mouse", "click cards, fields and menu; wheel scrolls"),
    ("? / Esc", "toggle this help · Esc goes back"),
    ("q", "quit (unsaved changes are applied first)"),
];

fn render_help(f: &mut Frame, state: &AppState) {
    let pal = state.palette();
    let screen = f.area();
    let w = 70.min(screen.width.saturating_sub(4));
    let h = (HELP.len() as u16 + 4).min(screen.height.saturating_sub(2));
    let area = Rect {
        x: screen.x + (screen.width - w) / 2,
        y: screen.y + (screen.height - h) / 2,
        width: w,
        height: h,
    };
    fill(f.buffer_mut(), area, Style::new().bg(pal.surface));
    let block = card(&pal, true).title(Line::from(Span::styled(
        " ? Keyboard shortcuts ",
        Style::new()
            .fg(pal.primary)
            .bg(pal.bg)
            .add_modifier(Modifier::BOLD),
    )));
    let inner = block.inner(area);
    block.render(area, f.buffer_mut());
    let inner = Rect {
        x: inner.x + 1,
        y: inner.y + 1,
        width: inner.width.saturating_sub(2),
        height: inner.height.saturating_sub(1),
    };
    for (i, (k, d)) in HELP.iter().enumerate() {
        let mut badge = key_badge(&pal, k);
        badge.content = format!(" {k:<15} ").into();
        line_at(
            f.buffer_mut(),
            inner,
            i as u16,
            Line::from(vec![
                badge,
                Span::styled(format!("  {d}"), Style::new().fg(pal.muted)),
            ]),
        );
    }
}

fn render_dialog(f: &mut Frame, state: &AppState) {
    let Some(kind) = state.editor.as_deref() else {
        return;
    };
    let pal = state.palette();
    let full = f.area();
    let w = full.width.saturating_sub(4).min(64);
    let h = full.height.min(6);
    let area = Rect::new(
        full.x + (full.width - w) / 2,
        full.y + (full.height - h) / 2,
        w,
        h,
    );
    fill(f.buffer_mut(), area, Style::new().bg(pal.surface));
    let title = match kind {
        "search" => "Search themes",
        "rename" => "Rename preset",
        _ => "Delete preset · type yes to confirm",
    };
    let block = card(&pal, true).title(format!(" {title} "));
    let inner = block.inner(area);
    block.render(area, f.buffer_mut());
    line_at(
        f.buffer_mut(),
        inner,
        0,
        Line::from(Span::styled(
            format!("> {}▏", state.input),
            Style::new().fg(pal.fg),
        )),
    );
    line_at(
        f.buffer_mut(),
        inner,
        2,
        Line::from(Span::styled(
            "Enter confirm · Esc cancel",
            Style::new().fg(pal.muted),
        )),
    );
}
