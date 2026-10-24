//! Bottom bar: boxed key badges, the latest status message and `q Quit`.

use crate::app::state::{AppState, StatusKind};
use crate::tui::widgets::ui::{ellipsize, fill, key_badge, line_at, width};
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};

const KEYS: [(&str, &str); 6] = [
    ("?", "Navigate"),
    ("↕", "Move"),
    ("Enter", "Select"),
    ("Tab", "Next Section"),
    ("d", "Demo Mode"),
    ("p", "Run Fastfetch"),
];

pub fn render_footer(f: &mut Frame, area: Rect, state: &AppState) {
    let pal = state.palette();
    let buf = f.buffer_mut();
    fill(buf, area, Style::new().bg(pal.bg).fg(pal.fg));
    let row = if area.height >= 3 {
        Block::new()
            .borders(Borders::TOP)
            .border_style(Style::new().fg(pal.border).bg(pal.bg))
            .render(area, buf);
        Rect {
            x: area.x + 1,
            y: area.y + 1 + (area.height - 2) / 2,
            width: area.width.saturating_sub(2),
            height: 1,
        }
    } else {
        area
    };
    let bg = Style::new().bg(pal.bg);
    let gap = Span::styled("   ", bg);

    let quit_label = if state.unsaved() {
        " Save & Quit"
    } else {
        " Quit"
    };
    let quit = vec![
        key_badge(&pal, "q"),
        Span::styled(quit_label, Style::new().fg(pal.muted).bg(pal.bg)),
        Span::styled(" ", bg),
    ];
    let quit_w: u16 = quit.iter().map(|s| width(&s.content)).sum();

    let mut spans = vec![Span::styled(" ", bg)];
    let mut used = 1u16;
    let mut extra: Vec<(&str, &str)> = state.screen.extra_keys().to_vec();
    if !state.undo_stack.is_empty() {
        extra.push(("u", "Undo"));
    }
    if !state.redo_stack.is_empty() {
        extra.push(("U", "Redo"));
    }
    let keys = KEYS.iter().copied().chain(extra);
    for (k, label) in keys {
        let item = [
            key_badge(&pal, k),
            Span::styled(format!(" {label}"), Style::new().fg(pal.muted).bg(pal.bg)),
            gap.clone(),
        ];
        let w: u16 = item.iter().map(|s| width(&s.content)).sum();
        if used + w + quit_w + 12 > row.width {
            break;
        }
        used += w;
        spans.extend(item);
    }

    // Status message in the space that is left.
    let (kind, msg) = &state.status;
    let color = match kind {
        StatusKind::Info => pal.fg,
        StatusKind::Ok => pal.ok,
        StatusKind::Warn => pal.warn,
        StatusKind::Error => pal.error,
    };
    let room = usize::from(row.width.saturating_sub(used + quit_w + 2));
    if room >= 10 {
        spans.push(Span::styled(
            ellipsize(msg, room),
            Style::new().fg(color).bg(pal.bg),
        ));
    }
    line_at(buf, row, 0, Line::from(spans));
    if quit_w < row.width {
        line_at(buf, row, 0, Line::from(quit).right_aligned());
    }
}
