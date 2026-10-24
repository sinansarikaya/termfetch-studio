//! Chrome colours for the whole TUI, derived from the active theme so every
//! label stays legible: the UI is painted on the theme background and all
//! text is contrast-checked against it (dark ink on light themes, light ink
//! on dark themes).

use crate::domain::contrast::{self, best_on, color, ensure, mix, Rgb, AA, AAA, UI};
use crate::domain::theme::Theme;
use ratatui::style::{Color, Style};

// Base hues for status colours; `ensure` darkens/lightens them per theme.
const GREEN: Rgb = (0x3f, 0xb9, 0x50);
const YELLOW: Rgb = (0xe3, 0xb3, 0x41);
const RED: Rgb = (0xf8, 0x51, 0x49);
const CYAN: Rgb = (0x39, 0xc5, 0xcf);
const MAGENTA: Rgb = (0xbc, 0x8c, 0xff);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub light: bool,
    /// Theme background, painted under the whole UI.
    pub bg: Color,
    /// Body text (>= 7:1).
    pub fg: Color,
    /// Labels and descriptions (>= 6:1, softer than `fg`).
    pub muted: Color,
    /// Hints, section headers, counters (>= 4.5:1).
    pub dim: Color,
    /// Unfocused borders (>= 3:1).
    pub border: Color,
    /// Theme primary as text / focused border (>= 4.5:1).
    pub primary: Color,
    /// Fill behind the selected row; text on it keeps its contrast.
    pub sel_bg: Color,
    pub ok: Color,
    pub warn: Color,
    pub error: Color,
    pub info: Color,
    pub special: Color,
}

impl Palette {
    pub fn from_theme(theme: &Theme) -> Self {
        let bg = contrast::to_rgb(theme.background_color()).unwrap_or((0x1e, 0x1e, 0x2e));
        let light = contrast::is_light(bg);
        let ink = best_on(bg);
        let text = contrast::to_rgb(theme.text_color()).unwrap_or(ink);
        let primary = contrast::to_rgb(theme.primary_color()).unwrap_or(text);
        let fg = ensure(text, bg, AAA);
        let sel_bg = mix(bg, fg, if light { 0.10 } else { 0.14 });
        // Selected rows sit on `sel_bg`, so check against the darker/lighter of the two.
        let status = |c: Rgb| color(ensure(ensure(c, bg, AA), sel_bg, AA));
        Self {
            light,
            bg: color(bg),
            fg: color(fg),
            muted: color(ensure(ensure(mix(fg, bg, 0.2), bg, 6.0), sel_bg, AA)),
            dim: color(ensure(ensure(mix(fg, bg, 0.45), bg, AA), sel_bg, AA)),
            border: color(ensure(mix(fg, bg, 0.6), bg, UI)),
            primary: status(primary),
            sel_bg: color(sel_bg),
            ok: status(GREEN),
            warn: status(YELLOW),
            error: status(RED),
            info: status(CYAN),
            special: status(MAGENTA),
        }
    }

    /// Text colour that reads on a filled `fill` (pills, badges).
    pub fn on(fill: Color) -> Color {
        contrast::to_rgb(fill).map_or(Color::Black, |c| color(best_on(c)))
    }

    /// Filled pill style: `fill` background with black/white text.
    pub fn pill(fill: Color) -> Style {
        Style::new().fg(Self::on(fill)).bg(fill)
    }

    /// Theme-provided colour (group icon etc.) nudged until it is visible
    /// (>= 3:1) on both the background and the selected-row fill.
    pub fn visible(&self, c: Color) -> Color {
        self.lift(c, UI)
    }

    /// Theme-provided colour used as text (>= 4.5:1 on background and selection).
    pub fn readable(&self, c: Color) -> Color {
        self.lift(c, AA)
    }

    fn lift(&self, c: Color, min: f64) -> Color {
        match (contrast::to_rgb(c), contrast::to_rgb(self.bg), contrast::to_rgb(self.sel_bg)) {
            (Some(c), Some(bg), Some(sel)) => color(ensure(ensure(c, bg, min), sel, min)),
            _ => c,
        }
    }
}
