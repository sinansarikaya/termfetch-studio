//! Scrollable list with optional section headers and mouse hit-testing.

use crate::app::state::{AppState, ClickMap, Focus};
use crate::tui::palette::Palette;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem},
    Frame,
};

pub enum Row<'a> {
    Header(String),
    Item(Line<'a>, usize),
}

pub fn panel<'a>(title: impl Into<String>, state: &AppState) -> Block<'a> {
    let pal = Palette::from_theme(state.current_theme());
    let primary = pal.primary;
    let border = if state.focus == Focus::Content { primary } else { pal.border };
    Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(border))
        .title(Span::styled(format!(" {} ", title.into()), Style::new().fg(primary).add_modifier(Modifier::BOLD)))
}

pub fn render_rows(f: &mut Frame, area: Rect, state: &mut AppState, block: Block, rows: Vec<Row>, selected: usize) {
    let pal = Palette::from_theme(state.current_theme());
    let (highlight_bg, primary) = (pal.sel_bg, pal.primary);
    let mut map = Vec::with_capacity(rows.len());
    let mut sel_row = 0;
    let items: Vec<ListItem> = rows
        .into_iter()
        .enumerate()
        .map(|(row, r)| match r {
            Row::Header(t) => {
                map.push(None);
                ListItem::new(Line::from(Span::styled(
                    format!("── {t} "),
                    Style::new().fg(pal.dim).add_modifier(Modifier::BOLD),
                )))
            }
            Row::Item(line, idx) => {
                map.push(Some(idx));
                if idx == selected {
                    sel_row = row;
                }
                ListItem::new(line)
            }
        })
        .collect();

    let inner = block.inner(area);
    let list = List::new(items)
        .block(block)
        .highlight_style(Style::new().bg(highlight_bg).add_modifier(Modifier::BOLD))
        .highlight_symbol(Line::from(Span::styled("▌", Style::new().fg(primary))))
        .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);
    let ls = &mut state.list_states[state.screen.index()];
    ls.select(Some(sel_row));
    f.render_stateful_widget(list, area, ls);
    // Map rows past the 1-column highlight gutter too: the whole row is clickable.
    state.click = ClickMap { area: inner, offset: ls.offset(), rows: map };
}
