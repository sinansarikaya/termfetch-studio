use crate::app::state::{AppState, Screen, StatusKind};
use crate::tui::screens::{
    backup::render_backup, colors::render_colors, dashboard::render_dashboard, doctor::render_doctor,
    logos::render_logos, modules::render_modules, presets::render_presets, themes::render_themes,
};
use crate::imaging::kitty::ImageKey;
use crate::tui::widgets::preview::{draw_preview, inner_area, ImageState, LogoRaster, PreviewModel};
use crate::tui::palette::Palette;
use crate::tui::widgets::sidebar::render_sidebar;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

const SIDEBAR_W: u16 = 26;

pub fn render_ui(f: &mut Frame, state: &mut AppState) {
    // Paint the theme background ourselves: legibility must not depend on
    // whatever background the host terminal happens to have.
    let pal = Palette::from_theme(state.current_theme());
    let full = f.area();
    f.buffer_mut().set_style(full, Style::new().bg(pal.bg).fg(pal.fg));
    let [top, body, bottom] = Layout::vertical([Constraint::Length(1), Constraint::Min(8), Constraint::Length(1)]).areas(f.area());
    render_top_bar(f, top, state);

    let w = body.width;
    if w >= 110 {
        // Menu | content | wide live preview.
        let content_w = ((w - SIDEBAR_W) * 2 / 5).clamp(42, 62);
        let [side, content, preview] =
            Layout::horizontal([Constraint::Length(SIDEBAR_W), Constraint::Length(content_w), Constraint::Min(40)]).areas(body);
        render_sidebar(f, side, state);
        render_screen(f, content, state);
        render_preview_pane(f, preview, state);
    } else if w >= 72 {
        let [side, content] = Layout::horizontal([Constraint::Length(SIDEBAR_W), Constraint::Min(40)]).areas(body);
        render_sidebar(f, side, state);
        render_screen(f, content, state);
    } else {
        state.sidebar_area = Rect::default();
        render_screen(f, body, state);
    }
    render_bottom_bar(f, bottom, state);
}

fn render_screen(f: &mut Frame, area: Rect, state: &mut AppState) {
    state.click = Default::default();
    match state.screen {
        Screen::Dashboard => render_dashboard(f, area, state),
        Screen::Themes => render_themes(f, area, state),
        Screen::Presets => render_presets(f, area, state),
        Screen::Logos => render_logos(f, area, state),
        Screen::Modules => render_modules(f, area, state),
        Screen::Colors => render_colors(f, area, state),
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
    // Half blocks are the fallback until the full-resolution upload is in the terminal.
    let grid = match (image, kitty_id, path) {
        (ImageState::Ready(_), None, Some(p)) => state.pool.grid(&p, cells.0, cells.1),
        _ => None,
    };
    let raster = match (kitty_id, grid.as_deref()) {
        (Some(id), _) => LogoRaster::Kitty(id),
        (None, Some(g)) => LogoRaster::Halfblock(g),
        (None, None) => LogoRaster::None,
    };
    let model = preview_model(state, image);
    draw_preview(f.buffer_mut(), area, &model, raster, true);
}

fn pill(text: String, fill: Color) -> Span<'static> {
    Span::styled(text, Palette::pill(fill).add_modifier(Modifier::BOLD))
}

fn render_top_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let pal = Palette::from_theme(state.current_theme());
    let left = Line::from(vec![
        pill(" \u{f018d} TermFetch Studio ".into(), pal.primary),
        Span::styled(" v2 ", Style::new().fg(pal.dim)),
        Span::styled(format!(" {} {} ", state.screen.icon(), state.screen.title()), Style::new().fg(pal.fg).add_modifier(Modifier::BOLD)),
    ]);
    let mut right = Vec::new();
    if state.unsaved() {
        right.push(pill(" ● unsaved ".into(), pal.warn));
        right.push(Span::raw(" "));
    }
    right.push(if state.settings.demo_mode {
        pill(" ◉ PREVIEW ".into(), pal.ok)
    } else {
        pill(" ● REAL ".into(), pal.error)
    });
    right.push(Span::raw(" "));
    f.render_widget(Paragraph::new(left), area);
    f.render_widget(Paragraph::new(Line::from(right).right_aligned()), area);
}

fn render_bottom_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let pal = Palette::from_theme(state.current_theme());
    let (kind, msg) = &state.status;
    let color = match kind {
        StatusKind::Info => pal.fg,
        StatusKind::Ok => pal.ok,
        StatusKind::Warn => pal.warn,
        StatusKind::Error => pal.error,
    };
    let quit = if state.unsaved() { "q save & quit" } else { "q quit" };
    let hints = format!("{} · {quit} ", state.screen.hints());
    let hint_w = hints.chars().count() as u16;
    let [msg_area, hint_area] = Layout::horizontal([Constraint::Min(10), Constraint::Length(hint_w.min(area.width / 2))]).areas(area);
    f.render_widget(Paragraph::new(Span::styled(format!(" {msg}"), Style::new().fg(color))), msg_area);
    f.render_widget(Paragraph::new(Line::from(Span::styled(hints, Style::new().fg(pal.dim))).right_aligned()), hint_area);
}
