//! Every glyph the TUI draws must be legible on every built-in theme,
//! light themes included.

use ratatui::{Terminal, backend::TestBackend, style::Color};
use termfetch_studio_v2::app::actions::Action;
use termfetch_studio_v2::app::reducer::reduce;
use termfetch_studio_v2::app::state::{AppState, Focus, Screen};
use termfetch_studio_v2::domain::contrast::{AA, AAA, UI, contrast, is_light, to_rgb};
use termfetch_studio_v2::domain::settings::Settings;
use termfetch_studio_v2::domain::theme::{Theme, ThemeCategory, get_builtin_themes};
use termfetch_studio_v2::imaging::TermGeometry;
use termfetch_studio_v2::paths::Paths;
use termfetch_studio_v2::tui::palette::Palette;
use termfetch_studio_v2::tui::render::render_ui;

const W: u16 = 180;
const H: u16 = 45;

fn rgb(c: Color) -> (u8, u8, u8) {
    to_rgb(c).unwrap_or_else(|| panic!("named colour {c:?} depends on the terminal palette"))
}

fn assert_palette(id: &str, p: &Palette) {
    let bg = rgb(p.bg);
    assert_eq!(p.light, is_light(bg), "{id}");
    for (sname, surface) in [
        ("bg", p.bg),
        ("surface", p.surface),
        ("raised", p.raised),
        ("selection", p.sel_bg),
    ] {
        let s = rgb(surface);
        assert!(contrast(rgb(p.fg), s) >= AAA - 0.01, "{id} fg on {sname}");
        for (name, c) in [
            ("muted", p.muted),
            ("dim", p.dim),
            ("primary", p.primary),
            ("accent2", p.accent2),
            ("link", p.link),
            ("ok", p.ok),
            ("warn", p.warn),
            ("error", p.error),
            ("info", p.info),
            ("special", p.special),
            ("orange", p.orange),
        ] {
            assert!(
                contrast(rgb(c), s) >= AA - 0.01,
                "{id} {name} on {sname}: {:.2}",
                contrast(rgb(c), s)
            );
        }
    }
    assert!(contrast(rgb(p.border), bg) >= UI - 0.01, "{id} border");
    assert!(
        contrast(rgb(Palette::on(p.accent_fill)), rgb(p.accent_fill)) >= AA - 0.01,
        "{id} button text"
    );
    // Light canvases get dark ink, dark canvases light ink.
    assert_eq!(p.light, !is_light(rgb(p.fg)), "{id} ink polarity");
}

#[test]
fn palette_meets_wcag_on_all_themes() {
    assert_palette("studio", &Palette::studio());
    assert!(!Palette::studio().light);
    for t in get_builtin_themes() {
        assert_palette(&t.id, &Palette::from_theme(&t));
    }
}

#[test]
fn light_themes_are_detected_as_light() {
    for t in get_builtin_themes()
        .iter()
        .filter(|t| t.category == ThemeCategory::Light || t.id.contains("latte"))
    {
        assert!(Palette::from_theme(t).light, "{} should be light", t.id);
    }
}

/// Swatches deliberately show other themes' / user-picked colours: palette
/// dots, half-block theme thumbnails, and the Settings screen's `██ sample` previews.
/// Theme art on the cards: a mini fastfetch in some theme's own colours,
/// reproduced verbatim like the live preview.
fn is_theme_art(themes: &[Theme], cell: &ratatui::buffer::Cell) -> bool {
    ["/", "\\", "_", "├", "▪", "→", "━"].contains(&cell.symbol())
        && themes.iter().any(|t| {
            let g = &t.groups;
            cell.bg == t.background_color()
                && [
                    &t.primary,
                    &g.os,
                    &g.desktop,
                    &g.hardware,
                    &g.network,
                    &g.time,
                ]
                .iter()
                .any(|c| cell.fg == Theme::parse_color(c))
        })
}

fn is_swatch(
    themes: &[Theme],
    screen: Screen,
    buf: &ratatui::buffer::Buffer,
    x: u16,
    y: u16,
) -> bool {
    let sym = buf[(x, y)].symbol();
    if sym == "●" || sym == "▀" || sym == "▄" || is_theme_art(themes, &buf[(x, y)]) {
        return true;
    }
    match screen {
        Screen::Settings => {
            sym == "█"
                || (x.saturating_sub(5)..=x).any(|s| {
                    (0..6).all(|i| {
                        s + i < W && buf[(s + i, y)].symbol() == &"sample"[i as usize..=i as usize]
                    })
                })
        }
        _ => false,
    }
}

#[test]
fn every_cell_is_legible_on_every_theme_and_screen() {
    let root = std::env::temp_dir().join(format!("tfs-contrast-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut failures = Vec::new();
    let themes = get_builtin_themes();
    assert_eq!(
        themes.len(),
        40,
        "every built-in theme is audited, with both chrome modes"
    );
    for (theme, follow) in themes.iter().flat_map(|t| [(t, false), (t, true)]) {
        let settings = Settings {
            theme: theme.id.clone(),
            ui_follow_theme: follow,
            // Mock specs keep the rendered text independent of this machine.
            demo_mode: true,
            ..Settings::default()
        };
        let geo = TermGeometry::from_window(W, H, 1800, 900);
        let mut state = AppState::new(Paths::with_root(&root), settings, geo);
        let mut t = Terminal::new(TestBackend::new(W, H)).unwrap();
        for screen in Screen::ALL {
            for focus in [Focus::Sidebar, Focus::Content, Focus::Options] {
                reduce(&mut state, Action::Goto(screen));
                state.focus = focus;
                t.draw(|f| render_ui(f, &mut state)).unwrap();
                let buf = t.backend().buffer();
                let text: String = buf.content().iter().map(|c| c.symbol()).collect();
                assert!(
                    text.contains("◉ PREVIEW")
                        && text.contains("Live Preview")
                        && text.contains("Apply Theme to System")
                        && !text.contains("DEMO"),
                    "{} {screen:?}",
                    theme.id
                );
                // Everything except the emulated terminal inside the live
                // preview, which reproduces fastfetch's colours verbatim.
                let body = state.preview_body;
                assert!(body.width > 20 && body.height > 10, "{body:?}");
                let cells = (0..H)
                    .flat_map(|y| (0..W).map(move |x| (x, y)))
                    .filter(|&(x, y)| {
                        !(x >= body.x && x < body.right() && y >= body.y && y < body.bottom())
                    });
                for (x, y) in cells {
                    {
                        let cell = &buf[(x, y)];
                        let sym = cell.symbol();
                        if sym.trim().is_empty() || is_swatch(&themes, screen, buf, x, y) {
                            continue;
                        }
                        let ratio = contrast(rgb(cell.fg), rgb(cell.bg));
                        // Text needs WCAG AA; box-drawing borders and icon glyphs need the UI minimum.
                        let text = sym
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || c.is_ascii_punctuation());
                        let min = if text { AA } else { UI };
                        if ratio < min - 0.01 {
                            failures.push(format!(
                                "{} follow={follow} {:?} {focus:?} ({x},{y}) {sym:?} {ratio:.2}",
                                theme.id, screen
                            ));
                        }
                    }
                }
            }
        }
    }
    let _ = std::fs::remove_dir_all(root);
    assert!(
        failures.is_empty(),
        "{} illegible cells:\n{}",
        failures.len(),
        failures
            .iter()
            .take(40)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
