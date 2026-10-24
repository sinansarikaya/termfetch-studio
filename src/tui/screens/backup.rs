use crate::app::state::AppState;
use crate::tui::widgets::list::{panel, render_rows, Row};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};

pub fn render_backup(f: &mut Frame, area: Rect, state: &mut AppState) {
    let [list_area, info_area] = Layout::vertical([Constraint::Min(6), Constraint::Length(6)]).areas(area);
    let primary = state.current_theme().primary_color();
    let mut rows = vec![Row::Item(
        Line::from(Span::styled("＋ Create new backup", Style::new().fg(primary).add_modifier(Modifier::BOLD))),
        0,
    )];
    rows.push(Row::Header(format!("Backups · {}", state.backups.len())));
    if state.backups.is_empty() {
        rows.push(Row::Header("none yet".into()));
    }
    for (i, b) in state.backups.iter().enumerate() {
        let tag = if b.name.contains("pre-restore") { Span::styled("  auto", Style::new().fg(Color::Yellow)) } else { Span::raw("") };
        rows.push(Row::Item(
            Line::from(vec![
                Span::styled("󰁯 ", Style::new().fg(Color::Cyan)),
                Span::styled(b.name.clone(), Style::new().fg(Color::White)),
                Span::styled(format!("  {} files", b.files.len()), Style::new().fg(Color::DarkGray)),
                tag,
            ]),
            i + 1,
        ));
    }
    let block = panel("\u{f006f} Backup & Restore", state);
    let cursor = state.backup_cursor;
    render_rows(f, list_area, state, block, rows, cursor);

    let detail = match state.backup_cursor.checked_sub(1).and_then(|i| state.backups.get(i)) {
        Some(b) => format!("Enter restores {} ({}). Your current files are saved first as a pre-restore backup. x deletes.", b.name, b.files.join(", ")),
        None => format!("Saves settings.json and both fastfetch configs into {}", state.paths.backups_dir().display()),
    };
    f.render_widget(
        Paragraph::new(Span::styled(detail, Style::new().fg(Color::Gray))).wrap(Wrap { trim: true }).block(panel("Info", state)),
        info_area,
    );
}
