use crate::app::state::AppState;
use crate::tui::palette::Palette;
use crate::tui::widgets::list::panel;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};

fn kv<'a>(pal: &Palette, k: &'a str, v: String, vc: Color) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{k:<13}"), Style::new().fg(pal.muted)),
        Span::styled(v, Style::new().fg(vc).add_modifier(Modifier::BOLD)),
    ])
}

pub fn render_dashboard(f: &mut Frame, area: Rect, state: &mut AppState) {
    let theme = state.current_theme();
    let pal = Palette::from_theme(theme);
    let primary = pal.primary;
    let [top, keys] = Layout::vertical([Constraint::Length(12), Constraint::Min(6)]).areas(area);

    let ff = match &state.fastfetch_version {
        Some(v) => (v.clone(), pal.ok),
        None => ("not installed".into(), pal.error),
    };
    let demo = if state.settings.demo_mode { ("ON · privacy-safe mock specs".into(), pal.ok) } else { ("OFF · real system specs".into(), pal.warn) };
    let unsaved = if state.unsaved() { ("unsaved changes · Enter to apply".into(), pal.warn) } else { ("in sync with fastfetch config".into(), pal.ok) };
    let lines = vec![
        kv(&pal, "Theme", theme.name.clone(), primary),
        kv(&pal, "Preset", format!("{} ({} modules)", state.current_preset().name, state.current_preset().module_count()), primary),
        kv(&pal, "Logo", format!("{} · {}", state.current_logo().name, state.settings.logo_size.label()), primary),
        kv(&pal, "Colors", format!("icons {} · text {}", state.settings.icon_color.label(), state.settings.text_color.label()), pal.fg),
        Line::default(),
        kv(&pal, "Preview mode", demo.0, demo.1),
        kv(&pal, "State", unsaved.0, unsaved.1),
        kv(&pal, "Fastfetch", ff.0, ff.1),
        kv(&pal, "Cell size", format!("{}x{} px{}", state.geo.cell.w, state.geo.cell.h, if state.geo.cell_reported { "" } else { " (assumed)" }), pal.fg),
        kv(&pal, "Catalogue", format!("{} themes · {} presets · {} logos", state.themes.len(), state.presets.len(), state.logos.len()), pal.fg),
    ];
    f.render_widget(Paragraph::new(lines).block(panel("\u{f056e} Dashboard", state)), top);

    let key = |k: &'static str, d: &'static str| Line::from(vec![Span::styled(format!("{k:<12}"), Style::new().fg(primary).add_modifier(Modifier::BOLD)), Span::styled(d, Style::new().fg(pal.muted))]);
    let help = vec![
        key("Tab / ←→", "switch between menu and content"),
        key("↑↓ / jk", "browse; the preview updates instantly"),
        key("Enter / a", "apply: save settings + write both fastfetch configs"),
        key("p", "run real fastfetch with the current selection"),
        key("d", "toggle privacy preview mode (mock specs, persisted)"),
        key("1-8", "jump to a screen · mouse click & wheel work too"),
        key("q / Esc", "quit"),
    ];
    f.render_widget(Paragraph::new(help).wrap(Wrap { trim: true }).block(panel("Keys", state)), keys);
}
