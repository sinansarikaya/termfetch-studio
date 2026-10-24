//! "Preview Options" card under the live preview: four dropdown fields and
//! the primary "Apply Theme to System" button.

use crate::app::state::{AppState, Focus, Hit, OPTION_APPLY};
use crate::tui::palette::Palette;
use crate::tui::widgets::ui::{ellipsize, fill, line_at, titled_card, width};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
    Frame,
};

const FIELDS: [(&str, &str); 4] = [("\u{f03d8}", "Theme"), ("\u{f0249}", "Preset"), ("\u{f02d4}", "Logo"), ("\u{f0169}", "Output")];
const LABEL_W: u16 = 11;
const DROP_MAX: usize = 10;

/// Rows the card needs for a given amount of space.
pub fn options_height(avail: u16) -> u16 {
    match avail {
        h if h >= 38 => 14,
        h if h >= 26 => 9,
        h if h >= 20 => 7,
        _ => 0,
    }
}

pub fn render_options(f: &mut Frame, area: Rect, state: &mut AppState) {
    let pal = state.palette();
    let focused = state.focus == Focus::Options;
    let block = titled_card(&pal, "\u{f0493}", pal.primary, "Preview Options", focused);
    let inner = block.inner(area);
    block.render(area, f.buffer_mut());
    if inner.height < 2 || inner.width < 12 {
        return;
    }
    let inner = Rect { x: inner.x + 1, width: inner.width.saturating_sub(2), ..inner };
    // Spacious: blank rows between fields and a 3-row button.
    let (step, top, button_h) = if inner.height >= 12 { (2u16, 1u16, 3u16) } else if inner.height >= 7 { (1, 0, if inner.height >= 8 { 3 } else { 1 }) } else { (1, 0, 1) };
    let colors = [pal.special, pal.info, pal.orange, pal.ok];

    let mut field_rects = [Rect::default(); 4];
    for (i, (icon, label)) in FIELDS.iter().enumerate() {
        let dy = top + i as u16 * step;
        if dy + button_h >= inner.height {
            break;
        }
        let row = Rect { y: inner.y + dy, height: 1, ..inner };
        let sel = focused && state.options_cursor == i;
        let field_x = inner.x + LABEL_W.min(inner.width / 3);
        let field = Rect { x: field_x, width: inner.right().saturating_sub(field_x), ..row };
        field_rects[i] = field;
        line_at(f.buffer_mut(), row, 0, Line::from(vec![
            Span::styled(format!("{icon}  "), Style::new().fg(colors[i]).bg(pal.surface)),
            Span::styled(*label, Style::new().fg(if sel { pal.fg } else { pal.muted }).bg(pal.surface)),
        ]));
        let value = state.option_label(i, state.option_index(i));
        let style = if sel { Style::new().fg(pal.primary).bg(pal.sel_bg).add_modifier(Modifier::BOLD) } else { Style::new().fg(pal.fg).bg(pal.raised) };
        fill(f.buffer_mut(), field, style);
        let fw = usize::from(field.width);
        let text = ellipsize(&value, fw.saturating_sub(5));
        let pad = fw.saturating_sub(width(&text) as usize + 4);
        line_at(f.buffer_mut(), field, 0, Line::from(vec![
            Span::styled(format!(" {text}"), style),
            Span::styled(" ".repeat(pad), style),
            Span::styled("\u{f0140} ", style),
        ]));
        state.hits.push((row, Hit::Option(i)));
    }

    // Apply button, bottom-aligned.
    let by = inner.bottom().saturating_sub(button_h);
    let button = Rect { y: by, height: button_h, ..inner };
    let sel = focused && state.options_cursor == OPTION_APPLY;
    let btn_fill = if sel { pal.accent2 } else { pal.accent_fill };
    let st = Palette::pill(btn_fill).add_modifier(Modifier::BOLD);
    fill(f.buffer_mut(), button, st);
    let label = "▶  Apply Theme to System";
    let key = " A ";
    let bw = button.width;
    let lx = (bw.saturating_sub(width(label))) / 2;
    let mid = Rect { y: button.y + button_h / 2, height: 1, ..button };
    let mut spans = vec![Span::styled(" ".repeat(usize::from(lx)), st), Span::styled(label, st)];
    let used = lx + width(label);
    if used + 5 <= bw {
        spans.push(Span::styled(" ".repeat(usize::from(bw - used - 4)), st));
        spans.push(Span::styled(key, Style::new().fg(btn_fill).bg(Palette::on(btn_fill)).add_modifier(Modifier::BOLD)));
    }
    line_at(f.buffer_mut(), mid, 0, Line::from(spans));
    state.hits.push((button, Hit::Option(OPTION_APPLY)));

    if let Some(dd) = state.dropdown.clone() {
        render_dropdown(f, field_rects[dd.field.min(3)], dd.field, state);
    }
}

/// Floating list under (or above) the field it belongs to.
fn render_dropdown(f: &mut Frame, field: Rect, field_idx: usize, state: &mut AppState) {
    if field.width == 0 {
        return;
    }
    let pal = state.palette();
    let screen = f.area();
    let len = state.option_len(field_idx);
    let rows = len.min(DROP_MAX) as u16;
    let h = rows + 2;
    let below = screen.bottom().saturating_sub(field.bottom());
    let y = if below >= h { field.bottom() } else { field.y.saturating_sub(h) };
    let area = Rect { x: field.x.saturating_sub(1), y, width: (field.width + 2).min(screen.right().saturating_sub(field.x.saturating_sub(1))), height: h.min(screen.height) }.intersection(screen);
    let block = crate::tui::widgets::ui::card(&pal, true);
    let inner = block.inner(area);
    fill(f.buffer_mut(), area, Style::new().bg(pal.surface));
    block.render(area, f.buffer_mut());
    let cur = state.option_index(field_idx);
    let offset = cur.saturating_sub(usize::from(inner.height).saturating_sub(1)).min(len.saturating_sub(usize::from(inner.height)));
    let w = usize::from(inner.width);
    for row in 0..inner.height {
        let i = offset + usize::from(row);
        if i >= len {
            break;
        }
        let sel = i == cur;
        let style = if sel { Style::new().fg(pal.primary).bg(pal.sel_bg).add_modifier(Modifier::BOLD) } else { Style::new().fg(pal.fg).bg(pal.surface) };
        let text = ellipsize(&state.option_label(field_idx, i), w.saturating_sub(3));
        let mark = if sel { "▌" } else { " " };
        let pad = w.saturating_sub(width(&text) as usize + 2);
        line_at(f.buffer_mut(), inner, row, Line::from(vec![
            Span::styled(mark, Style::new().fg(pal.primary).bg(if sel { pal.sel_bg } else { pal.surface })),
            Span::styled(format!(" {text}"), style),
            Span::styled(" ".repeat(pad), style),
        ]));
        state.hits.push((Rect { y: inner.y + row, height: 1, ..inner }, Hit::DropItem(i)));
    }
}
