use crate::app::state::AppState;
use crate::domain::color::{ColorValue, SemanticGroup};
use crate::domain::style;
use crate::tui::widgets::list::{Row, panel, render_rows, screen_frame};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};

fn value_size(pal: &crate::tui::palette::Palette, v: &str) -> Span<'static> {
    Span::styled(
        format!("◀ {v:<13} ▶"),
        Style::new().fg(pal.fg).add_modifier(Modifier::BOLD),
    )
}

fn swatch(c: Color) -> Span<'static> {
    Span::styled("██", Style::new().fg(c))
}

pub fn render_settings(f: &mut Frame, area: Rect, state: &mut AppState) {
    let area = screen_frame(f, area, state);
    let [list_area, info_area] =
        Layout::vertical([Constraint::Min(8), Constraint::Length(6)]).areas(area);
    let theme = state.current_theme().clone();
    let s = state.settings.clone();
    let fg = theme.text_color();
    let pal = state.palette();
    let value = |v: &ColorValue| {
        Span::styled(
            format!("◀ {:<13} ▶", v.label()),
            Style::new().fg(pal.fg).add_modifier(Modifier::BOLD),
        )
    };
    let label = |t: &str| Span::styled(format!("{t:<14}"), Style::new().fg(pal.muted));

    let on_off = |b: bool, on: &str, off: &str| {
        if b {
            Span::styled(
                format!("[● {on} ] "),
                Style::new().fg(pal.ok).add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(format!("[○ {off}] "), Style::new().fg(pal.dim))
        }
    };

    let mut rows = vec![Row::Header("Appearance".into())];
    rows.push(Row::Item(
        Line::from(vec![
            label("Studio chrome"),
            value_size(
                &pal,
                if s.ui_follow_theme {
                    "Follow theme"
                } else {
                    "Studio indigo"
                },
            ),
            Span::styled(" UI chrome theme colors", Style::new().fg(pal.dim)),
        ]),
        12,
    ));

    rows.push(Row::Header("Preview".into()));
    rows.push(Row::Item(
        Line::from(vec![
            label("Preview mode"),
            on_off(s.demo_mode, "ON", "OFF"),
            Span::styled(
                if s.demo_mode {
                    "privacy-safe mock specs"
                } else {
                    "real system specs"
                },
                Style::new().fg(pal.dim),
            ),
        ]),
        9,
    ));
    let (mw, mh) = s.logo_size.max_cells();
    rows.push(Row::Item(
        Line::from(vec![
            label("Logo size"),
            value_size(&pal, s.logo_size.label()),
            Span::styled(
                format!(" max {mw}x{mh} cells (s)"),
                Style::new().fg(pal.dim),
            ),
        ]),
        10,
    ));

    rows.push(Row::Header("Colors".into()));
    let icon_c = style::module_colors(&theme, &s, SemanticGroup::Os)
        .key
        .tui_color(fg);
    rows.push(Row::Item(
        Line::from(vec![
            label("Icons"),
            value(&s.icon_color),
            Span::raw(" "),
            swatch(icon_c),
        ]),
        0,
    ));
    let text_c = style::text_color(&theme, &s).tui_color(fg);
    rows.push(Row::Item(
        Line::from(vec![
            label("Text"),
            value(&s.text_color),
            Span::raw(" "),
            swatch(text_c),
        ]),
        1,
    ));
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
    rows.push(Row::Item(
        Line::from(Span::styled(
            "↺  Reset to theme colours",
            Style::new().fg(pal.warn),
        )),
        7,
    ));

    rows.push(Row::Header("Terminal".into()));
    let osc = if s.apply_terminal_colors { "on" } else { "off" };
    rows.push(Row::Item(
        Line::from(vec![
            label("Terminal bg/fg"),
            Span::styled(
                format!("{osc} "),
                Style::new().fg(pal.fg).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "(sync host terminal via OSC 10/11 on apply)",
                Style::new().fg(pal.dim),
            ),
        ]),
        8,
    ));

    rows.push(Row::Header("Shell Integration".into()));
    rows.push(Row::Item(
        Line::from(vec![
            label("Exit fastfetch"),
            on_off(s.run_fastfetch_on_exit, "on", "off"),
            Span::styled(
                "run fastfetch after quitting termfetch",
                Style::new().fg(pal.dim),
            ),
        ]),
        11,
    ));

    rows.push(Row::Header("Advanced".into()));
    rows.push(Row::Item(Line::from("System diagnostics (Doctor) →"), 13));
    rows.push(Row::Item(
        Line::from("Backup & restore configurations →"),
        14,
    ));

    let block = panel("\u{f0493} Settings · preferences & configuration", state);
    let cursor = state.color_cursor;
    render_rows(f, list_area, state, block, rows, cursor);

    // Setting Inspector panel
    let inspector_lines = match state.color_cursor {
        0 => vec![
            Line::from(vec![
                Span::styled(
                    "Icons Colour: ",
                    Style::new().fg(pal.primary).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Controls module key/glyph colors in fastfetch",
                    Style::new().fg(pal.muted),
                ),
            ]),
            Line::from(Span::styled(
                "Select 'Theme' to follow the active palette, or force a custom color override.",
                Style::new().fg(pal.dim),
            )),
            Line::from(Span::styled(
                "Live Preview reflects changes immediately. Press 'a' to save to system.",
                Style::new().fg(pal.dim),
            )),
        ],
        1 => vec![
            Line::from(vec![
                Span::styled(
                    "Text Colour: ",
                    Style::new().fg(pal.primary).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Controls information text and value outputs",
                    Style::new().fg(pal.muted),
                ),
            ]),
            Line::from(Span::styled(
                "Ensures high contrast readability across terminal outputs.",
                Style::new().fg(pal.dim),
            )),
            Line::from(Span::styled(
                "Press Left/Right or Enter to cycle values.",
                Style::new().fg(pal.dim),
            )),
        ],
        r @ 2..=6 => {
            let g = SemanticGroup::ALL[r - 2];
            vec![
                Line::from(vec![
                    Span::styled(
                        format!("Group [{}]: ", g.label()),
                        Style::new().fg(pal.primary).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        "Applies to both key icons and values in this category",
                        Style::new().fg(pal.muted),
                    ),
                ]),
                Line::from(Span::styled(
                    "Overrides global icon/text colors for modules in this group.",
                    Style::new().fg(pal.dim),
                )),
                Line::from(Span::styled(
                    "Live Preview updates immediately in the right pane.",
                    Style::new().fg(pal.dim),
                )),
            ]
        }
        7 => vec![
            Line::from(vec![
                Span::styled(
                    "Reset Colours: ",
                    Style::new().fg(pal.warn).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Reverts all custom color overrides back to theme defaults",
                    Style::new().fg(pal.muted),
                ),
            ]),
            Line::from(Span::styled(
                "Press Enter or Space to restore default palette settings.",
                Style::new().fg(pal.dim),
            )),
        ],
        8 => vec![
            Line::from(vec![
                Span::styled(
                    "Terminal Synchronisation (OSC 10/11): ",
                    Style::new().fg(pal.primary).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Updates host terminal background & foreground",
                    Style::new().fg(pal.muted),
                ),
            ]),
            Line::from(Span::styled(
                "When applying, sends OSC escape sequences to match your terminal emulator to the theme.",
                Style::new().fg(pal.dim),
            )),
        ],
        9 => vec![
            Line::from(vec![
                Span::styled(
                    "Preview Mode (Demo Specs): ",
                    Style::new().fg(pal.primary).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Toggles privacy-safe mock hardware specs",
                    Style::new().fg(pal.muted),
                ),
            ]),
            Line::from(Span::styled(
                "Useful for recording videos or screenshots without revealing real IPs and serials.",
                Style::new().fg(pal.dim),
            )),
        ],
        10 => vec![
            Line::from(vec![
                Span::styled(
                    "Logo Sizing Scale: ",
                    Style::new().fg(pal.primary).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Sets maximum cell dimensions for ASCII and PNG logos",
                    Style::new().fg(pal.muted),
                ),
            ]),
            Line::from(Span::styled(
                "Small (24x12), Medium (34x18), or Large (48x24 cells). Aspect ratio is always locked.",
                Style::new().fg(pal.dim),
            )),
        ],
        11 => vec![
            Line::from(vec![
                Span::styled(
                    "Shell Integration: ",
                    Style::new().fg(pal.primary).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Runs fastfetch immediately after quitting termfetch",
                    Style::new().fg(pal.muted),
                ),
            ]),
            Line::from(Span::styled(
                "Allows instant verification of newly applied configurations in your shell.",
                Style::new().fg(pal.dim),
            )),
        ],
        12 => vec![
            Line::from(vec![
                Span::styled(
                    "Studio Chrome Interface Theme: ",
                    Style::new().fg(pal.primary).add_modifier(Modifier::BOLD),
                ),
                Span::styled("Switches Studio TUI appearance", Style::new().fg(pal.muted)),
            ]),
            Line::from(Span::styled(
                "Choose between 'Studio indigo' (consistent dark theme) and 'Follow theme' (adapts to active theme).",
                Style::new().fg(pal.dim),
            )),
        ],
        13 => vec![
            Line::from(vec![
                Span::styled(
                    "System Diagnostics: ",
                    Style::new().fg(pal.primary).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Inspects fastfetch installation and environment",
                    Style::new().fg(pal.muted),
                ),
            ]),
            Line::from(Span::styled(
                "Press Enter to navigate to the Doctor screen.",
                Style::new().fg(pal.dim),
            )),
        ],
        14 => vec![
            Line::from(vec![
                Span::styled(
                    "Backup & Restore: ",
                    Style::new().fg(pal.primary).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Safely save and restore configuration snapshots",
                    Style::new().fg(pal.muted),
                ),
            ]),
            Line::from(Span::styled(
                "Press Enter to navigate to the Backup & Restore screen.",
                Style::new().fg(pal.dim),
            )),
        ],
        _ => vec![Line::from(Span::styled(
            "Select a setting to view details",
            Style::new().fg(pal.dim),
        ))],
    };

    let detail_block = panel("Setting Inspector", state);
    f.render_widget(
        Paragraph::new(inspector_lines)
            .wrap(Wrap { trim: true })
            .block(detail_block),
        info_area,
    );
}
