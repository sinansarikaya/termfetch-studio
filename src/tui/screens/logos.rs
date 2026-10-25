use crate::app::state::AppState;
use crate::domain::logo::{LogoGroup, LogoKind};
use crate::tui::widgets::list::{Row, panel, render_rows, screen_frame};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

pub fn render_logos(f: &mut Frame, area: Rect, state: &mut AppState) {
    let area = screen_frame(f, area, state);
    let [list_area, info_area] =
        Layout::vertical([Constraint::Min(8), Constraint::Length(6)]).areas(area);
    let selected = state.logo_idx();
    let theme_name = state.current_theme().name.clone();
    let pal = state.palette();
    let primary = pal.primary;
    let mut rows = Vec::new();
    let mut last: Option<LogoGroup> = None;
    for (i, l) in state.logos.iter().enumerate() {
        if last != Some(l.group) {
            last = Some(l.group);
            rows.push(Row::Header(l.group.as_str().to_string()));
        }
        let (badge, color) = match l.kind {
            LogoKind::Ascii => ("ASCII", pal.info),
            LogoKind::PresetImage => ("PNG", pal.ok),
            LogoKind::CustomImage => ("CUSTOM", pal.special),
        };
        let status = match l.file_path.as_deref() {
            Some(p) if l.is_image() && state.pool.get(p).is_some() => {
                Span::styled(" ●", Style::new().fg(pal.ok))
            }
            Some(p) if l.is_image() && state.pool.is_pending(p) => {
                Span::styled(" ◌", Style::new().fg(pal.warn))
            }
            Some(_) if l.is_image() => Span::styled(" ✗", Style::new().fg(pal.error)),
            _ => Span::raw(""),
        };
        let name_style = if i == selected {
            Style::new().fg(primary).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(pal.fg)
        };
        let mut spans = vec![
            Span::styled(format!("{badge:<7}"), Style::new().fg(color)),
            Span::styled(l.name.clone(), name_style),
            status,
        ];
        if l.id == state.saved.logo {
            spans.push(Span::styled("  ✓", Style::new().fg(pal.ok)));
        }
        rows.push(Row::Item(Line::from(spans), i));
    }
    let decoded = state.pool.decoded_count();
    let block = panel(format!("\u{f02e9} Logos · {decoded} images decoded"), state);
    render_rows(f, list_area, state, block, rows, selected);

    let logo = state.current_logo();
    let size = state.settings.logo_size;
    let (mw, mh) = size.max_cells();
    let size_badge = |s: crate::domain::settings::LogoSize| format!("[ {} ] ", s.label());
    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                "Logo Scale: ",
                Style::new().fg(pal.muted).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                size_badge(crate::domain::settings::LogoSize::Small),
                if size == crate::domain::settings::LogoSize::Small {
                    Style::new().fg(primary).add_modifier(Modifier::BOLD)
                } else {
                    Style::new().fg(pal.dim)
                },
            ),
            Span::styled(
                size_badge(crate::domain::settings::LogoSize::Medium),
                if size == crate::domain::settings::LogoSize::Medium {
                    Style::new().fg(primary).add_modifier(Modifier::BOLD)
                } else {
                    Style::new().fg(pal.dim)
                },
            ),
            Span::styled(
                size_badge(crate::domain::settings::LogoSize::Large),
                if size == crate::domain::settings::LogoSize::Large {
                    Style::new().fg(primary).add_modifier(Modifier::BOLD)
                } else {
                    Style::new().fg(pal.dim)
                },
            ),
            Span::styled(
                format!("(max {mw}x{mh} cells) · press 's' to cycle"),
                Style::new().fg(pal.dim),
            ),
        ]),
        Line::default(),
    ];
    if let Some(p) = logo.file_path.as_deref().filter(|_| logo.is_image()) {
        let filename = std::path::Path::new(p)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(p);
        let detail = match state.pool.get(p) {
            Some(img) => format!(
                "✓ Image Ready: {}x{} px · File: {} · Path: {}",
                img.dims.0, img.dims.1, filename, p
            ),
            None => match state.pool.error(p) {
                Some(_) if logo.kind == LogoKind::CustomImage => {
                    format!("Import instruction: Place your image at {p} and press 'r' to reload.")
                }
                Some(e) => format!("Image load warning for {filename}: {e}"),
                None => format!("Decoding {filename} in background…"),
            },
        };
        lines.push(Line::from(Span::styled(detail, Style::new().fg(pal.dim))));
    } else {
        lines.push(Line::from(vec![
            Span::styled("✓ ASCII Format: ", Style::new().fg(pal.ok)),
            Span::styled(
                format!(
                    "{} lines of text · Colored automatically with theme primary ({})",
                    logo.art.lines().count(),
                    theme_name
                ),
                Style::new().fg(pal.dim),
            ),
        ]));
    }
    f.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .block(panel("Logo Details & Sizing", state)),
        info_area,
    );
}
