use crate::app::state::AppState;
use crate::tui::widgets::list::{panel, render_rows, screen_frame, Row};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};

pub fn render_presets(f: &mut Frame, area: Rect, state: &mut AppState) {
    let area = screen_frame(f, area, state);
    let [list_area, info_area] = Layout::vertical([Constraint::Min(8), Constraint::Length(9)]).areas(area);
    let selected = state.preset_idx();
    let pal = state.palette();
    let primary = pal.primary;
    let rows: Vec<Row> = state
        .presets
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let mut spans = vec![
                Span::styled(format!("{:<16}", p.name), if i == selected { Style::new().fg(primary).add_modifier(Modifier::BOLD) } else { Style::new().fg(pal.fg) }),
                Span::styled(format!("{:>2} modules", p.module_count()), Style::new().fg(pal.dim)),
            ];
            if p.id == state.saved.preset {
                spans.push(Span::styled("  ✓ applied", Style::new().fg(pal.ok)));
            }
            Row::Item(Line::from(spans), i)
        })
        .collect();
    let block = panel("\u{f0645} Presets", state);
    render_rows(f, list_area, state, block, rows, selected);

    let p = state.current_preset();
    let theme = state.current_theme();
    let mut lines = vec![
        Line::from(Span::styled(p.description.clone(), Style::new().fg(pal.muted))),
        Line::default(),
    ];
    for section in &p.sections {
        let mut spans = Vec::new();
        for m in section {
            let c = pal.visible(crate::domain::theme::Theme::parse_color(m.group().theme_hex(theme)));
            spans.push(Span::styled(format!("{} ", m.icon()), Style::new().fg(c)));
            spans.push(Span::styled(format!("{}  ", m.label()), Style::new().fg(pal.fg)));
        }
        lines.push(Line::from(spans));
    }
    if p.id == "custom" {
        lines.push(Line::from(Span::styled("Edit this preset in Module Builder (5)", Style::new().fg(pal.dim))));
    }
    let info = Paragraph::new(lines).wrap(Wrap { trim: true }).block(panel("Modules", state));
    f.render_widget(info, info_area);
}
