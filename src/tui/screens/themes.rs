//! Themes: a scrollable grid of thumbnail cards.

use crate::app::state::{AppState, Focus, Hit};
use crate::tui::palette::Palette;
use crate::tui::widgets::list::{panel, screen_frame};
use crate::tui::widgets::ui::{card, draw_thumbnail, ellipsize, fill, line_at, theme_dots};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

const CARD_MIN_W: u16 = 20;
const CARD_H: u16 = 10;

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
    let cols = usize::from((inner.width + 1) / (CARD_MIN_W + 1)).max(1);
    let card_h = if inner.height >= CARD_H {
        CARD_H
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
            let block = card(&pal, sel)
                .style(Style::new().bg(pal.raised).fg(pal.fg))
                .border_style(Style::new().fg(border).bg(pal.surface));
            let ti = block.inner(rect);
            block.render(rect, f.buffer_mut());
            let thumb_h = ti.height.saturating_sub(2).min(6);
            draw_thumbnail(
                f.buffer_mut(),
                Rect {
                    height: thumb_h,
                    ..ti
                },
                t,
            );
            if t.id == applied && ti.width >= 10 && thumb_h > 0 {
                line_at(
                    f.buffer_mut(),
                    Rect {
                        x: ti.right() - 10,
                        width: 10,
                        ..ti
                    },
                    0,
                    Line::from(Span::styled(
                        " ✓ applied",
                        Palette::pill(pal.accent_fill).add_modifier(Modifier::BOLD),
                    )),
                );
            }
            let tw = usize::from(ti.width.saturating_sub(1));
            let name_bg = if sel { pal.sel_bg } else { pal.raised };
            let name_row = Rect {
                y: ti.y + thumb_h,
                height: 1,
                ..ti
            };
            fill(
                f.buffer_mut(),
                name_row.intersection(ti),
                Style::new().bg(name_bg),
            );
            line_at(
                f.buffer_mut(),
                ti,
                thumb_h,
                Line::from(Span::styled(
                    format!(" {}", ellipsize(&t.name, tw.saturating_sub(1))),
                    Style::new()
                        .fg(if sel { pal.primary } else { pal.fg })
                        .bg(name_bg)
                        .add_modifier(Modifier::BOLD),
                )),
            );
            let mut dots = vec![Span::raw(" ")];
            dots.extend(theme_dots(t, tw >= 17));
            line_at(f.buffer_mut(), ti, thumb_h + 1, Line::from(dots));
            state.hits.push((rect, Hit::ThemeCard(theme_index)));
        }
    }
}
