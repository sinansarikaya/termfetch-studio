//! Themes: a scrollable grid of thumbnail cards.

use crate::app::state::{AppState, Focus, Hit};
use crate::tui::widgets::list::{panel, screen_frame};
use crate::tui::widgets::ui::{THEME_CARD_H, THEME_CARD_W, draw_theme_card, line_at};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Widget,
};

pub fn render_themes(f: &mut Frame, area: Rect, state: &mut AppState) {
    let pal = state.palette();
    let content = screen_frame(f, area, state);
    let list = state.filtered_themes();
    let selected = list
        .iter()
        .position(|&i| i == state.theme_idx())
        .unwrap_or(0);
    let applied = state.saved.theme.clone();
    let categories = ["All", "Dark", "Light", "Community", "Pastel", "Gaming"];
    let block = panel(format!("Themes · {} results", list.len()), state);
    let inner = block.inner(content);
    block.render(content, f.buffer_mut());
    if inner.width < 6 || inner.height < 3 {
        return;
    }
    let filter = Rect { height: 1, ..inner };
    line_at(
        f.buffer_mut(),
        filter,
        0,
        Line::from(Span::styled(
            format!(
                " ◀ {} ▶ [c] · / Search: {}",
                categories[state.theme_filter], state.theme_search
            ),
            Style::new().fg(pal.primary),
        )),
    );
    state.hits.push((filter, Hit::ThemeFilter));
    let inner = Rect {
        x: inner.x + 1,
        y: inner.y + 2,
        width: inner.width.saturating_sub(2),
        height: inner.height.saturating_sub(2),
    };
    let cols = usize::from((inner.width + 1) / (THEME_CARD_W + 1)).max(1);
    let card_h = if inner.height >= THEME_CARD_H {
        THEME_CARD_H
    } else {
        inner.height
    };
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
        let row = Rect {
            y: inner.y + r as u16 * card_h,
            height: card_h,
            ..inner
        };
        let rects = Layout::horizontal(vec![Constraint::Ratio(1, cols as u32); cols])
            .spacing(1)
            .split(row);
        for (c, &rect) in rects.iter().enumerate() {
            let i = row_idx * cols + c;
            let Some(&theme_index) = list.get(i) else {
                break;
            };
            let t = &state.themes[theme_index];
            let sel = i == selected;
            let border = if sel && focused {
                pal.accent2
            } else if sel {
                pal.primary
            } else {
                pal.border
            };
            draw_theme_card(
                f.buffer_mut(),
                rect,
                &pal,
                t,
                border,
                sel,
                (t.id == applied).then_some(" ✓ applied"),
            );
            state.hits.push((rect, Hit::ThemeCard(theme_index)));
        }
    }
}
