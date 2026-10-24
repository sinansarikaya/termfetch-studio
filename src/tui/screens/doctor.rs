use crate::adapters::doctor::Level;
use crate::app::state::AppState;
use crate::tui::widgets::list::{panel, render_rows, screen_frame, Row};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    Frame,
};

pub fn render_doctor(f: &mut Frame, area: Rect, state: &mut AppState) {
    let area = screen_frame(f, area, state);
    let pal = state.palette();
    let rows: Vec<Row> = state
        .doctor
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let color = match c.level {
                Level::Ok => pal.ok,
                Level::Info => pal.info,
                Level::Warn => pal.warn,
                Level::Fail => pal.error,
            };
            Row::Item(
                Line::from(vec![
                    Span::styled(format!("{} ", c.level.symbol()), Style::new().fg(color).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{:<17}", c.name), Style::new().fg(pal.fg)),
                    Span::styled(c.detail.clone(), Style::new().fg(pal.muted)),
                ]),
                i,
            )
        })
        .collect();
    let block = panel("\u{f04d9} Doctor · r to re-run", state);
    let cursor = state.doctor_cursor;
    render_rows(f, area, state, block, rows, cursor);
}
