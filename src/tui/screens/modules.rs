use crate::app::state::AppState;
use crate::domain::theme::Theme;
use crate::tui::widgets::list::{Row, panel, render_rows, screen_frame};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

pub fn render_modules(f: &mut Frame, area: Rect, state: &mut AppState) {
    let area = screen_frame(f, area, state);
    let [list_area, info_area] =
        Layout::vertical([Constraint::Min(8), Constraint::Length(6)]).areas(area);
    let theme = state.current_theme().clone();
    let pal = state.palette();
    let module_rows = state.module_rows();
    let enabled_count = state.settings.custom_modules.len();
    let mut rows = Vec::new();
    rows.push(Row::Header(format!(
        "Enabled · {enabled_count} (in display order)"
    )));
    for (i, (m, on)) in module_rows.iter().enumerate() {
        if i == enabled_count {
            rows.push(Row::Header("Available".into()));
        }
        let (mark, mark_style) = if *on {
            ("✓", Style::new().fg(pal.ok))
        } else {
            ("·", Style::new().fg(pal.dim))
        };
        let icon_c = pal.visible(Theme::parse_color(m.group().theme_hex(&theme)));
        let label_style = if *on {
            Style::new().fg(pal.fg).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(pal.muted)
        };
        let order = if *on {
            format!("{:>2}. ", i + 1)
        } else {
            "    ".into()
        };
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
    let active = if state.settings.preset == "custom" {
        " · active"
    } else {
        ""
    };
    let block = panel(format!("\u{f0169} Module Builder{active}"), state);
    let cursor = state.module_cursor;
    render_rows(f, list_area, state, block, rows, cursor);

    let sel = module_rows.get(state.module_cursor).copied();
    let mut lines = Vec::new();
    if let Some((m, on)) = sel {
        let icon_c = pal.visible(Theme::parse_color(m.group().theme_hex(&theme)));
        let status_span = if on {
            let pos = state
                .settings
                .custom_modules
                .iter()
                .position(|&x| x == m)
                .map(|p| p + 1)
                .unwrap_or(1);
            Span::styled(
                format!("  [✓ Enabled · Position #{pos}]"),
                Style::new().fg(pal.ok).add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled("  [· Available]", Style::new().fg(pal.dim))
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{} ", m.icon()), Style::new().fg(icon_c)),
            Span::styled(
                m.label(),
                Style::new().fg(pal.fg).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  ({})", m.group().label()),
                Style::new().fg(pal.muted),
            ),
            status_span,
        ]));
        lines.push(Line::from(Span::styled(
            m.description(),
            Style::new().fg(pal.dim),
        )));
        lines.push(Line::from(Span::styled(
            "Controls: Space to toggle · Shift+↑/↓ or K/J to reorder · a to apply",
            Style::new().fg(pal.muted),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "Select a module to view its description",
            Style::new().fg(pal.dim),
        )));
    }
    let detail_block = panel("Module Inspector", state);
    f.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .block(detail_block),
        info_area,
    );
}
