use crate::app::state::AppState;
use crate::domain::color::{ColorValue, SemanticGroup};
use crate::domain::style;
use crate::tui::widgets::list::{Row, panel, render_rows, screen_frame};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

fn value_size(pal: &crate::tui::palette::Palette, v: &str) -> Span<'static> {
    Span::styled(format!("◀ {v:<13} ▶"), Style::new().fg(pal.fg).add_modifier(Modifier::BOLD))
}

fn swatch(c: Color) -> Span<'static> {
    Span::styled("██", Style::new().fg(c))
}

pub fn render_settings(f: &mut Frame, area: Rect, state: &mut AppState) {
    let area = screen_frame(f, area, state);
    let theme = state.current_theme().clone();
    let s = state.settings.clone();
    let fg = theme.text_color();
    let pal = state.palette();
    let value = |v: &ColorValue| Span::styled(format!("◀ {:<13} ▶", v.label()), Style::new().fg(pal.fg).add_modifier(Modifier::BOLD));
    let label = |t: &str| Span::styled(format!("{t:<14}"), Style::new().fg(pal.muted));

    let mut rows = vec![Row::Header("Colors".into())];
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
    rows.push(Row::Header("Terminal".into()));
    rows.push(Row::Item(Line::from(Span::styled("↺  Reset to theme colours", Style::new().fg(pal.warn))), 7));
    let osc = if s.apply_terminal_colors { "on" } else { "off" };
    rows.push(Row::Item(
        Line::from(vec![
            label("Terminal bg/fg"),
            Span::styled(format!("{osc} "), Style::new().fg(pal.fg).add_modifier(Modifier::BOLD)),
            Span::styled("(OSC 10/11 on apply)", Style::new().fg(pal.dim)),
        ]),
        8,
    ));
    let on_off =
        |b: bool, on: &str, off: &str| Span::styled(format!("{} ", if b { on } else { off }), Style::new().fg(if b { pal.ok } else { pal.muted }).add_modifier(Modifier::BOLD));
    rows.push(Row::Header("Preview".into()));
    rows.push(Row::Item(
        Line::from(vec![
            label("Preview mode"),
            on_off(s.demo_mode, "ON", "OFF"),
            Span::styled(if s.demo_mode { "privacy-safe mock specs (d)" } else { "real system specs (d)" }, Style::new().fg(pal.dim)),
        ]),
        9,
    ));
    let (mw, mh) = s.logo_size.max_cells();
    rows.push(Row::Item(
        Line::from(vec![label("Logo size"), value_size(&pal, s.logo_size.label()), Span::styled(format!(" max {mw}x{mh} cells (s)"), Style::new().fg(pal.dim))]),
        10,
    ));
    rows.push(Row::Header("Shell Integration".into()));
    rows.push(Row::Item(
        Line::from(vec![label("Exit fastfetch"), on_off(s.run_fastfetch_on_exit, "on", "off"), Span::styled("run fastfetch after quitting", Style::new().fg(pal.dim))]),
        11,
    ));
    rows.push(Row::Header("Appearance".into()));
    rows.push(Row::Item(
        Line::from(vec![
            label("Studio chrome"),
            value_size(&pal, if s.ui_follow_theme { "Follow theme" } else { "Studio indigo" }),
            Span::styled(" UI colours", Style::new().fg(pal.dim)),
        ]),
        12,
    ));
    rows.push(Row::Header("Advanced".into()));
    rows.push(Row::Item(Line::from("System diagnostics →"), 13));
    rows.push(Row::Item(Line::from("Backup & restore →"), 14));
    let block = panel("\u{f0493} Settings · colours & preferences", state);
    let cursor = state.color_cursor;
    render_rows(f, area, state, block, rows, cursor);
}
