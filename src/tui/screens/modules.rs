use crate::app::state::AppState;
use crate::domain::theme::Theme;
use crate::tui::widgets::list::{panel, render_rows, screen_frame, Row};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    Frame,
};

pub fn render_modules(f: &mut Frame, area: Rect, state: &mut AppState) {
    let area = screen_frame(f, area, state);
    let theme = state.current_theme().clone();
    let pal = state.palette();
    let module_rows = state.module_rows();
    let enabled_count = state.settings.custom_modules.len();
    let mut rows = Vec::new();
    rows.push(Row::Header(format!("Enabled · {enabled_count} (in display order)")));
    for (i, (m, on)) in module_rows.iter().enumerate() {
        if i == enabled_count {
            rows.push(Row::Header("Available".into()));
        }
        let (mark, mark_style) = if *on { ("✓", Style::new().fg(pal.ok)) } else { ("·", Style::new().fg(pal.dim)) };
        let icon_c = pal.visible(Theme::parse_color(m.group().theme_hex(&theme)));
        let label_style = if *on { Style::new().fg(pal.fg).add_modifier(Modifier::BOLD) } else { Style::new().fg(pal.muted) };
        let order = if *on { format!("{:>2}. ", i + 1) } else { "    ".into() };
        rows.push(Row::Item(
            Line::from(vec![
                Span::styled(format!("{mark} "), mark_style),
                Span::styled(order, Style::new().fg(pal.dim)),
                Span::styled(format!("{}  ", m.icon()), Style::new().fg(icon_c)),
                Span::styled(m.label(), label_style),
                Span::styled(format!("  {}", m.group().label()), Style::new().fg(pal.dim)),
            ]),
            i,
        ));
    }
    let active = if state.settings.preset == "custom" { " · active" } else { "" };
    let block = panel(format!("\u{f0169} Module Builder{active}"), state);
    let cursor = state.module_cursor;
    render_rows(f, area, state, block, rows, cursor);
}
