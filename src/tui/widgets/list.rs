//! Scrollable list with optional section headers and mouse hit-testing.

use crate::app::state::{AppState, ClickMap, Focus};
use crate::tui::widgets::ui::{card, fill, line_at};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, List, ListItem},
    Frame,
};

pub enum Row<'a> {
    Header(String),
    Item(Line<'a>, usize),
}

/// Card for the center stage; its border lights up while the content has focus.
pub fn panel<'a>(title: impl Into<String>, state: &AppState) -> Block<'a> {
    let pal = state.palette();
    card(&pal, state.focus == Focus::Content).title(Line::from(vec![
        Span::styled(" ", Style::new().bg(pal.bg)),
        Span::styled(format!("{} ", title.into()), Style::new().fg(pal.primary).bg(pal.bg).add_modifier(Modifier::BOLD)),
    ]))
}

/// Center-stage page header (icon + title, subtitle) on the canvas; returns
/// the remaining content area.
pub fn screen_frame(f: &mut Frame, area: Rect, state: &AppState) -> Rect {
    let pal = state.palette();
    fill(f.buffer_mut(), area, Style::new().bg(pal.bg).fg(pal.fg));
    let area = Rect { x: area.x + 1, width: area.width.saturating_sub(2), ..area };
    if area.height < 10 {
        return area;
    }
    let s = state.screen;
    line_at(f.buffer_mut(), area, 0, Line::from(vec![
        Span::styled(format!("{}  ", s.icon()), Style::new().fg(pal.primary).bg(pal.bg)),
        Span::styled(s.title(), Style::new().fg(pal.fg).bg(pal.bg).add_modifier(Modifier::BOLD)),
    ]));
    line_at(f.buffer_mut(), area, 1, Line::from(Span::styled(s.subtitle(), Style::new().fg(pal.muted).bg(pal.bg))));
    Rect { y: area.y + 3, height: area.height - 3, ..area }
}

pub fn render_rows(f: &mut Frame, area: Rect, state: &mut AppState, block: Block, rows: Vec<Row>, selected: usize) {
    let pal = state.palette();
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
