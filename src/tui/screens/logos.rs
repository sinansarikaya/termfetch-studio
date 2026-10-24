use crate::app::state::AppState;
use crate::domain::logo::{LogoGroup, LogoKind};
use crate::tui::widgets::list::{panel, render_rows, Row};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};

pub fn render_logos(f: &mut Frame, area: Rect, state: &mut AppState) {
    let [list_area, info_area] = Layout::vertical([Constraint::Min(8), Constraint::Length(6)]).areas(area);
    let selected = state.logo_idx();
    let primary = state.current_theme().primary_color();
    let mut rows = Vec::new();
    let mut last: Option<LogoGroup> = None;
    for (i, l) in state.logos.iter().enumerate() {
        if last != Some(l.group) {
            last = Some(l.group);
            rows.push(Row::Header(l.group.as_str().to_string()));
        }
        let (badge, color) = match l.kind {
            LogoKind::Ascii => ("ASCII", Color::Cyan),
            LogoKind::PresetImage => ("PNG", Color::Green),
            LogoKind::CustomImage => ("CUSTOM", Color::Magenta),
        };
        let status = match l.file_path.as_deref() {
            Some(p) if l.is_image() && state.pool.get(p).is_some() => Span::styled(" ●", Style::new().fg(Color::Green)),
            Some(p) if l.is_image() && state.pool.is_pending(p) => Span::styled(" ◌", Style::new().fg(Color::Yellow)),
            Some(_) if l.is_image() => Span::styled(" ✗", Style::new().fg(Color::Red)),
            _ => Span::raw(""),
        };
        let name_style = if i == selected { Style::new().fg(primary).add_modifier(Modifier::BOLD) } else { Style::new().fg(Color::White) };
        let mut spans = vec![
            Span::styled(format!("{badge:<7}"), Style::new().fg(color)),
            Span::styled(l.name.clone(), name_style),
            status,
        ];
        if l.id == state.saved.logo {
            spans.push(Span::styled("  ✓", Style::new().fg(Color::Green)));
        }
        rows.push(Row::Item(Line::from(spans), i));
    }
    let decoded = state.pool.decoded_count();
    let block = panel(format!("\u{f02e9} Logos · {decoded} images decoded"), state);
    render_rows(f, list_area, state, block, rows, selected);

    let logo = state.current_logo();
    let size = state.settings.logo_size;
    let (mw, mh) = size.max_cells();
    let mut lines = vec![Line::from(vec![
        Span::styled("Size ", Style::new().fg(Color::Gray)),
        Span::styled(format!("{} ", size.label()), Style::new().fg(primary).add_modifier(Modifier::BOLD)),
        Span::styled(format!("(max {mw}x{mh} cells, aspect-locked) · s to change"), Style::new().fg(Color::DarkGray)),
    ])];
    if let Some(p) = logo.file_path.as_deref().filter(|_| logo.is_image()) {
        let detail = match state.pool.get(p) {
            Some(img) => format!("{}x{} px · {}", img.dims.0, img.dims.1, p),
            None => match state.pool.error(p) {
                Some(_) if logo.kind == LogoKind::CustomImage => format!("Place your image at {p}  (r to reload)"),
                Some(e) => format!("Cannot load {p}: {e}"),
                None => format!("Decoding {p}…"),
            },
        };
        lines.push(Line::from(Span::styled(detail, Style::new().fg(Color::DarkGray))));
    } else {
        lines.push(Line::from(Span::styled("ASCII art, written to the config dir and coloured with the theme primary", Style::new().fg(Color::DarkGray))));
    }
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }).block(panel("Details", state)), info_area);
}
