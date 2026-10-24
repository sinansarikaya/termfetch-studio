//! Every glyph the TUI draws must be legible on every built-in theme,
//! light themes included.

use ratatui::{backend::TestBackend, style::Color, Terminal};
use termfetch_studio_v2::app::actions::Action;
use termfetch_studio_v2::app::reducer::reduce;
use termfetch_studio_v2::app::state::{AppState, Focus, Screen};
use termfetch_studio_v2::domain::contrast::{contrast, is_light, to_rgb, AA, AAA, UI};
use termfetch_studio_v2::domain::settings::Settings;
use termfetch_studio_v2::domain::theme::{get_builtin_themes, ThemeCategory};
use termfetch_studio_v2::imaging::TermGeometry;
use termfetch_studio_v2::paths::Paths;
use termfetch_studio_v2::tui::palette::Palette;
use termfetch_studio_v2::tui::render::render_ui;

const W: u16 = 180;
const H: u16 = 45;
/// Left of the live preview pane at 180 columns (menu 26 + content 61).
const PREVIEW_X: u16 = 87;

fn rgb(c: Color) -> (u8, u8, u8) {
    to_rgb(c).unwrap_or_else(|| panic!("named colour {c:?} depends on the terminal palette"))
}

#[test]
fn palette_meets_wcag_on_all_themes() {
    for t in get_builtin_themes() {
        let p = Palette::from_theme(&t);
        let bg = rgb(p.bg);
        let sel = rgb(p.sel_bg);
        assert_eq!(p.light, is_light(bg), "{}", t.id);
        assert!(contrast(rgb(p.fg), bg) >= AAA - 0.01, "{} fg", t.id);
        for (name, c) in [("muted", p.muted), ("dim", p.dim), ("primary", p.primary), ("ok", p.ok), ("warn", p.warn), ("error", p.error), ("info", p.info), ("special", p.special)] {
            assert!(contrast(rgb(c), bg) >= AA - 0.01, "{} {name} on bg: {:.2}", t.id, contrast(rgb(c), bg));
            assert!(contrast(rgb(c), sel) >= AA - 0.01, "{} {name} on selection: {:.2}", t.id, contrast(rgb(c), sel));
        }
        assert!(contrast(rgb(p.border), bg) >= UI - 0.01, "{} border", t.id);
        // Light themes get dark ink, dark themes light ink.
        assert_eq!(p.light, !is_light(rgb(p.fg)), "{} ink polarity", t.id);
    }
}

#[test]
fn light_themes_are_detected_as_light() {
    for t in get_builtin_themes().iter().filter(|t| t.category == ThemeCategory::Light || t.id.contains("latte")) {
        assert!(Palette::from_theme(t).light, "{} should be light", t.id);
    }
}

/// Swatches deliberately show other themes' / user-picked colours: the
/// theme list dots, and the Colors screen's `██ sample` previews.
fn is_swatch(screen: Screen, buf: &ratatui::buffer::Buffer, x: u16, y: u16) -> bool {
    let sym = buf[(x, y)].symbol();
    match screen {
        Screen::Themes => sym == "●",
        Screen::Colors => sym == "█" || (x.saturating_sub(5)..=x).any(|s| (0..6).all(|i| s + i < W && buf[(s + i, y)].symbol() == &"sample"[i as usize..=i as usize])),
        _ => false,
    }
}

#[test]
fn every_cell_is_legible_on_every_theme_and_screen() {
    let root = std::env::temp_dir().join(format!("tfs-contrast-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut failures = Vec::new();
    let themes = get_builtin_themes();
    assert_eq!(themes.len(), 30, "every built-in theme is audited");
    for theme in &themes {
        let settings = Settings { theme: theme.id.clone(), ..Settings::default() };
        let geo = TermGeometry::from_window(W, H, 1800, 900);
        let mut state = AppState::new(Paths::with_root(&root), settings, geo);
        let mut t = Terminal::new(TestBackend::new(W, H)).unwrap();
        for screen in Screen::ALL {
            for focus in [Focus::Sidebar, Focus::Content] {
                reduce(&mut state, Action::Goto(screen));
                state.focus = focus;
                t.draw(|f| render_ui(f, &mut state)).unwrap();
                let buf = t.backend().buffer();
                let text: String = buf.content().iter().map(|c| c.symbol()).collect();
                assert!(text.contains("◉ PREVIEW") && text.contains("[d] preview") && !text.contains("DEMO"), "{} {screen:?}", theme.id);
                // Whole UI left of the preview, plus the preview's own chrome
                // (title row and the bottom badge row); its body emulates fastfetch verbatim.
                let cells = (0..H).flat_map(|y| (0..W).map(move |x| (x, y))).filter(|&(x, y)| x < PREVIEW_X || y == 0 || y == 1 || y == H - 2 || y == H - 1);
                for (x, y) in cells {
                    {
                        let cell = &buf[(x, y)];
                        let sym = cell.symbol();
                        if sym.trim().is_empty() || is_swatch(screen, buf, x, y) {
                            continue;
                        }
                        let ratio = contrast(rgb(cell.fg), rgb(cell.bg));
                        // Text needs WCAG AA; box-drawing borders and icon glyphs need the UI minimum.
                        let text = sym.chars().all(|c| c.is_ascii_alphanumeric() || c.is_ascii_punctuation());
                        let min = if text { AA } else { UI };
                        if ratio < min - 0.01 {
                            failures.push(format!("{} {:?} ({x},{y}) {sym:?} {ratio:.2}", theme.id, screen));
                        }
                    }
                }
            }
        }
    }
    let _ = std::fs::remove_dir_all(root);
    assert!(failures.is_empty(), "{} illegible cells:\n{}", failures.len(), failures.iter().take(40).cloned().collect::<Vec<_>>().join("\n"));
}
