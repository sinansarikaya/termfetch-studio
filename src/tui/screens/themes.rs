//! Themes: a scrollable grid of thumbnail cards.

use crate::app::state::{AppState, Focus, Hit};
use crate::tui::palette::Palette;
use crate::tui::widgets::list::{panel, screen_frame};
use crate::tui::widgets::ui::{card, draw_thumbnail, ellipsize, fill, line_at, theme_dots};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
    Frame,
};

const CARD_MIN_W: u16 = 20;
const CARD_H: u16 = 7;

pub fn render_themes(f: &mut Frame, area: Rect, state: &mut AppState) {
    let pal = state.palette();
    let content = screen_frame(f, area, state);
    let selected = state.theme_idx();
    let applied = state.saved.theme.clone();
    let theme = state.current_theme();
    let block = panel(format!("\u{f03d8} Themes · {} of {} · {}", selected + 1, state.themes.len(), theme.category.as_str()), state);
    let inner = block.inner(content);
    block.render(content, f.buffer_mut());
    if inner.width < 6 || inner.height < 3 {
        return;
    }
    let inner = Rect { x: inner.x + 1, width: inner.width.saturating_sub(2), ..inner };
    let cols = usize::from((inner.width + 1) / (CARD_MIN_W + 1)).max(1);
    let card_h = if inner.height >= CARD_H { CARD_H } else { inner.height };
    let rows_visible = usize::from(inner.height / card_h).max(1);
    state.grid_cols = cols;
    let sel_row = selected / cols;
    if sel_row < state.grid_scroll {
        state.grid_scroll = sel_row;
    } else if sel_row >= state.grid_scroll + rows_visible {
        state.grid_scroll = sel_row + 1 - rows_visible;
    }
    let focused = state.focus == Focus::Content;
    for r in 0..rows_visible {
        let row_idx = state.grid_scroll + r;
        let row = Rect { y: inner.y + r as u16 * card_h, height: card_h, ..inner };
        let rects = Layout::horizontal(vec![Constraint::Ratio(1, cols as u32); cols]).spacing(1).split(row);
        for (c, &rect) in rects.iter().enumerate() {
            let i = row_idx * cols + c;
            let Some(t) = state.themes.get(i) else { break };
            let sel = i == selected;
            let border = if sel && focused { pal.accent2 } else if sel { pal.primary } else { pal.border };
            let block = card(&pal, sel).style(Style::new().bg(pal.raised).fg(pal.fg)).border_style(Style::new().fg(border).bg(pal.surface));
            let ti = block.inner(rect);
            block.render(rect, f.buffer_mut());
            let thumb_h = ti.height.saturating_sub(2).min(3);
            draw_thumbnail(f.buffer_mut(), Rect { height: thumb_h, ..ti }, t);
            if t.id == applied && ti.width >= 4 && thumb_h > 0 {
                line_at(f.buffer_mut(), Rect { x: ti.right() - 3, width: 3, ..ti }, 0, Line::from(Span::styled(" ✓ ", Palette::pill(pal.accent_fill).add_modifier(Modifier::BOLD))));
            }
            let tw = usize::from(ti.width.saturating_sub(1));
            let name_bg = if sel { pal.sel_bg } else { pal.raised };
            let name_row = Rect { y: ti.y + thumb_h, height: 1, ..ti };
            fill(f.buffer_mut(), name_row.intersection(ti), Style::new().bg(name_bg));
            line_at(f.buffer_mut(), ti, thumb_h, Line::from(Span::styled(format!(" {}", ellipsize(&t.name, tw.saturating_sub(1))), Style::new().fg(if sel { pal.primary } else { pal.fg }).bg(name_bg).add_modifier(Modifier::BOLD))));
            let mut dots = vec![Span::raw(" ")];
            dots.extend(theme_dots(t, tw >= 17));
            line_at(f.buffer_mut(), ti, thumb_h + 1, Line::from(dots));
            state.hits.push((rect, Hit::ThemeCard(i)));
        }
    }
}
