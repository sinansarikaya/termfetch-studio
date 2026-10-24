use crate::app::state::AppState;
use crate::tui::palette::Palette;
use crate::domain::color::{ColorValue, SemanticGroup};
use crate::domain::style;
use crate::tui::widgets::list::{panel, render_rows, Row};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    Frame,
};

fn swatch(c: Color) -> Span<'static> {
    Span::styled("██", Style::new().fg(c))
}

pub fn render_colors(f: &mut Frame, area: Rect, state: &mut AppState) {
    let theme = state.current_theme().clone();
    let s = state.settings.clone();
    let fg = theme.text_color();
    let pal = Palette::from_theme(&theme);
    let value = |v: &ColorValue| Span::styled(format!("◀ {:<13} ▶", v.label()), Style::new().fg(pal.fg).add_modifier(Modifier::BOLD));
    let label = |t: &str| Span::styled(format!("{t:<14}"), Style::new().fg(pal.muted));

    let mut rows = vec![Row::Header("Base colours".into())];
    let icon_c = style::module_colors(&theme, &s, SemanticGroup::Os).key.tui_color(fg);
    rows.push(Row::Item(Line::from(vec![label("Icons"), value(&s.icon_color), Span::raw(" "), swatch(icon_c)]), 0));
    let text_c = style::text_color(&theme, &s).tui_color(fg);
    rows.push(Row::Item(Line::from(vec![label("Text"), value(&s.text_color), Span::raw(" "), swatch(text_c)]), 1));
    rows.push(Row::Header("Group overrides (icon + text)".into()));
    for (i, g) in SemanticGroup::ALL.iter().enumerate() {
        let mc = style::module_colors(&theme, &s, *g);
        rows.push(Row::Item(
            Line::from(vec![
                label(g.label()),
                value(s.group_colors.get(*g)),
                Span::raw(" "),
                swatch(mc.key.tui_color(fg)),
                Span::styled(" sample", Style::new().fg(mc.output.tui_color(fg))),
            ]),
            i + 2,
        ));
    }
    rows.push(Row::Header("Actions".into()));
    rows.push(Row::Item(Line::from(Span::styled("↺  Reset to theme colours", Style::new().fg(pal.warn))), 7));
    let osc = if s.apply_terminal_colors { "on" } else { "off" };
    rows.push(Row::Item(
        Line::from(vec![label("Terminal bg/fg"), Span::styled(format!("{osc} "), Style::new().fg(pal.fg).add_modifier(Modifier::BOLD)), Span::styled("(OSC 10/11 on apply)", Style::new().fg(pal.dim))]),
        8,
    ));
    let block = panel("\u{f020a} Icon & Text Colors", state);
    let cursor = state.color_cursor;
    render_rows(f, area, state, block, rows, cursor);
}
