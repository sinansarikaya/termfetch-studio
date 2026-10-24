use crate::app::state::{AppState, Screen, StatusKind};
use crate::tui::screens::{
    backup::render_backup, colors::render_colors, dashboard::render_dashboard, doctor::render_doctor,
    logos::render_logos, modules::render_modules, presets::render_presets, themes::render_themes,
};
use crate::tui::widgets::preview::{draw_preview, inner_area, ImageState, PreviewModel};
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

fn render_preview_pane(f: &mut Frame, area: Rect, state: &mut AppState) {
    let image = image_state(state);
    let inner = inner_area(area);
    // Size first (immutable borrow), then fetch/cached-rasterise the grid.
    let cells = {
        let model = PreviewModel {
            theme: state.current_theme(),
            preset: state.current_preset(),
            logo: state.current_logo(),
            settings: &state.settings,
            info: state.real_info.as_ref(),
            info_loading: state.real_rx.is_some(),
            image,
            geo: state.geo,
        };
        model.logo_cells((inner.width, inner.height))
    };
    let grid = match (image, state.current_logo().file_path.clone()) {
        (ImageState::Ready(_), Some(p)) => state.pool.grid(&p, cells.0, cells.1),
        _ => None,
    };
    let model = PreviewModel {
        theme: state.current_theme(),
        preset: state.current_preset(),
        logo: state.current_logo(),
        settings: &state.settings,
        info: state.real_info.as_ref(),
        info_loading: state.real_rx.is_some(),
        image,
        geo: state.geo,
    };
    draw_preview(f.buffer_mut(), area, &model, grid.as_deref(), true);
}

fn pill(text: String, fg: Color, bg: Color) -> Span<'static> {
    Span::styled(text, Style::new().fg(fg).bg(bg).add_modifier(Modifier::BOLD))
}

fn render_top_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let theme = state.current_theme();
    let primary = theme.primary_color();
    let left = Line::from(vec![
        pill(" \u{f018d} TermFetch Studio ".into(), theme.background_color(), primary),
        Span::styled(" v2 ", Style::new().fg(Color::DarkGray)),
        Span::styled(format!(" {} {} ", state.screen.icon(), state.screen.title()), Style::new().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]);
    let mut right = Vec::new();
    if state.unsaved() {
        right.push(pill(" ● unsaved ".into(), Color::Black, Color::Yellow));
        right.push(Span::raw(" "));
    }
    right.push(if state.settings.demo_mode {
        pill(" 󰈈 DEMO ".into(), Color::Black, Color::Green)
    } else {
        pill(" 󰈉 REAL ".into(), Color::Black, Color::LightRed)
    });
    right.push(Span::raw(" "));
    f.render_widget(Paragraph::new(left), area);
    f.render_widget(Paragraph::new(Line::from(right).right_aligned()), area);
}

fn render_bottom_bar(f: &mut Frame, area: Rect, state: &AppState) {
    let (kind, msg) = &state.status;
    let color = match kind {
        StatusKind::Info => Color::White,
        StatusKind::Ok => Color::Green,
        StatusKind::Warn => Color::Yellow,
        StatusKind::Error => Color::Red,
    };
    let hints = format!("{} · q quit ", state.screen.hints());
    let hint_w = hints.chars().count() as u16;
    let [msg_area, hint_area] = Layout::horizontal([Constraint::Min(10), Constraint::Length(hint_w.min(area.width / 2))]).areas(area);
    f.render_widget(Paragraph::new(Span::styled(format!(" {msg}"), Style::new().fg(color))), msg_area);
    f.render_widget(Paragraph::new(Line::from(Span::styled(hints, Style::new().fg(Color::DarkGray))).right_aligned()), hint_area);
}
