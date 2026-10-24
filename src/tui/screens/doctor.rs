use crate::adapters::doctor::Level;
use crate::app::state::AppState;
use crate::tui::widgets::list::{panel, render_rows, Row};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    Frame,
};

pub fn render_doctor(f: &mut Frame, area: Rect, state: &mut AppState) {
    let rows: Vec<Row> = state
        .doctor
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let color = match c.level {
                Level::Ok => Color::Green,
                Level::Info => Color::Cyan,
                Level::Warn => Color::Yellow,
                Level::Fail => Color::Red,
            };
            Row::Item(
                Line::from(vec![
                    Span::styled(format!("{} ", c.level.symbol()), Style::new().fg(color).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{:<17}", c.name), Style::new().fg(Color::White)),
                    Span::styled(c.detail.clone(), Style::new().fg(Color::Gray)),
                ]),
                i,
            )
        })
        .collect();
    let block = panel("\u{f04d9} Doctor · r to re-run", state);
    let cursor = state.doctor_cursor;
    render_rows(f, area, state, block, rows, cursor);
}
