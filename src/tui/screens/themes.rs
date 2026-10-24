use crate::app::state::AppState;
use crate::domain::color::SemanticGroup;
use crate::tui::widgets::list::{panel, render_rows, Row};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    Frame,
};

pub fn render_themes(f: &mut Frame, area: Rect, state: &mut AppState) {
    let selected = state.theme_idx();
    let applied = state.saved.theme.clone();
    let mut rows = Vec::new();
    let mut last = None;
    for (i, t) in state.themes.iter().enumerate() {
        if last != Some(t.category) {
            last = Some(t.category);
            rows.push(Row::Header(t.category.as_str().to_string()));
        }
        let mut spans = vec![Span::styled("  ", Style::new().bg(t.background_color()))];
        spans.push(Span::raw(" "));
        for g in SemanticGroup::ALL {
            spans.push(Span::styled("●", Style::new().fg(crate::domain::theme::Theme::parse_color(g.theme_hex(t)))));
        }
        spans.push(Span::raw(" "));
        let name_style = if i == selected { Style::new().fg(t.primary_color()).add_modifier(Modifier::BOLD) } else { Style::new().fg(Color::White) };
        spans.push(Span::styled(t.name.clone(), name_style));
        if t.id == applied {
            spans.push(Span::styled("  ✓ applied", Style::new().fg(Color::Green)));
        }
        rows.push(Row::Item(Line::from(spans), i));
    }
    let block = panel(format!("\u{f03d8} Themes ({})", state.themes.len()), state);
    render_rows(f, area, state, block, rows, selected);
}
