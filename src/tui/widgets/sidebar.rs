use crate::app::state::{AppState, Focus, Screen};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

pub fn render_sidebar(f: &mut Frame, area: Rect, state: &mut AppState) {
    state.sidebar_area = area;
    let theme = state.current_theme();
    let primary = theme.primary_color();
    let on_primary = theme.background_color();
    let focused = state.focus == Focus::Sidebar;

    let width = usize::from(area.width.saturating_sub(2));
    let lines: Vec<Line> = Screen::ALL
        .iter()
        .enumerate()
        .map(|(i, &screen)| {
            let label = format!(" {}  {}", screen.icon(), screen.title());
            let num = format!("{} ", i + 1);
            let pad = width.saturating_sub(unicode_width::UnicodeWidthStr::width(label.as_str()) + num.len());
            if screen == state.screen {
                let style = if focused {
                    Style::new().fg(on_primary).bg(primary).add_modifier(Modifier::BOLD)
                } else {
                    Style::new().fg(primary).bg(Color::Rgb(40, 42, 54)).add_modifier(Modifier::BOLD)
                };
                Line::from(vec![Span::styled(label, style), Span::styled(" ".repeat(pad), style), Span::styled(num, style)])
            } else {
                Line::from(vec![
                    Span::styled(label, Style::new().fg(Color::Gray)),
                    Span::raw(" ".repeat(pad)),
                    Span::styled(num, Style::new().fg(Color::DarkGray)),
                ])
            }
        })
        .collect();

    let border = if focused { Style::new().fg(primary) } else { Style::new().fg(Color::DarkGray) };
    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border)
        .title(Span::styled(" \u{f0aa2} Menu ", Style::new().fg(primary).add_modifier(Modifier::BOLD)));
    f.render_widget(Paragraph::new(lines).block(block), area);
}
