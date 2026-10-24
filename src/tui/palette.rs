//! Chrome colours for the whole TUI. The default "Studio" look is a deep navy
//! canvas with indigo/violet accents; optionally the chrome follows the active
//! theme instead. Either way every text colour is contrast-checked against
//! every surface it can be drawn on (canvas, cards, raised cards, selection).

use crate::domain::contrast::{self, AA, AAA, Rgb, UI, best_on, color, ensure, mix};
use crate::domain::theme::Theme;
use ratatui::style::{Color, Style};

// Base hues for status colours; `ensure` darkens/lightens them per surface.
const GREEN: Rgb = (0x4a, 0xde, 0x80);
const YELLOW: Rgb = (0xfb, 0xbf, 0x24);
const RED: Rgb = (0xf8, 0x71, 0x71);
const BLUE: Rgb = (0x60, 0xa5, 0xfa);
const MAGENTA: Rgb = (0xc0, 0x84, 0xfc);
const ORANGE: Rgb = (0xfb, 0x92, 0x3c);
const PINK: Rgb = (0xf4, 0x72, 0xb6);
const CYAN: Rgb = (0x67, 0xe8, 0xf9);

// Studio canvas.
const STUDIO_BG: Rgb = (0x0b, 0x0f, 0x1a);
const STUDIO_SURFACE: Rgb = (0x11, 0x17, 0x26);
const STUDIO_RAISED: Rgb = (0x17, 0x1e, 0x31);
const STUDIO_FG: Rgb = (0xe8, 0xeb, 0xf4);
const STUDIO_MUTED: Rgb = (0xa6, 0xae, 0xc4);
const STUDIO_DIM: Rgb = (0x8a, 0x93, 0xab);
const STUDIO_BORDER: Rgb = (0x5a, 0x63, 0x80);
const STUDIO_ACCENT: Rgb = (0xa7, 0x8b, 0xfa);
const STUDIO_ACCENT_FILL: Rgb = (0x5b, 0x45, 0xe0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub light: bool,
    /// Canvas painted under the whole UI.
    pub bg: Color,
    /// Card / panel fill.
    pub surface: Color,
    /// Raised elements on cards: inputs, key badges, inner tiles.
    pub raised: Color,
    /// Fill behind the selected row / focused tile.
    pub sel_bg: Color,
    /// Body text (>= 7:1 on the canvas).
    pub fg: Color,
    /// Labels and descriptions (>= 4.5:1 everywhere).
    pub muted: Color,
    /// Hints, section headers, counters (>= 4.5:1 everywhere).
    pub dim: Color,
    /// Unfocused borders (>= 3:1 on the canvas).
    pub border: Color,
    /// Accent as text / focused border (>= 4.5:1 everywhere).
    pub primary: Color,
    /// Filled accent: active nav pill and the primary button (text via [`Palette::on`]).
    pub accent_fill: Color,
    /// Second accent (welcome highlight).
    pub accent2: Color,
    /// Tagline / link colour.
    pub link: Color,
    pub ok: Color,
    pub warn: Color,
    pub error: Color,
    pub info: Color,
    pub special: Color,
    pub orange: Color,
}

/// Raise `c` until it reaches `min` on every surface (all share one polarity,
/// so each step only increases the contrast against the earlier surfaces).
fn on_all(c: Rgb, surfaces: &[Rgb], min: f64) -> Rgb {
    surfaces.iter().fold(c, |acc, &s| ensure(acc, s, min))
}

struct Base {
    bg: Rgb,
    surface: Rgb,
    raised: Rgb,
    sel: Rgb,
    fg: Rgb,
    muted: Rgb,
    dim: Rgb,
    border: Rgb,
    accent: Rgb,
    accent_fill: Rgb,
    accent2: Rgb,
    link: Rgb,
}

impl Palette {
    /// The built-in indigo look from the TermFetch Studio mock-up.
    pub fn studio() -> Self {
        let sel = mix(STUDIO_RAISED, STUDIO_ACCENT_FILL, 0.28);
        Self::build(Base {
            bg: STUDIO_BG,
            surface: STUDIO_SURFACE,
            raised: STUDIO_RAISED,
            sel,
            fg: STUDIO_FG,
            muted: STUDIO_MUTED,
            dim: STUDIO_DIM,
            border: STUDIO_BORDER,
            accent: STUDIO_ACCENT,
            accent_fill: STUDIO_ACCENT_FILL,
            accent2: PINK,
            link: CYAN,
        })
    }

    /// Chrome derived from the theme: dark ink on light themes, light ink on dark ones.
    pub fn from_theme(theme: &Theme) -> Self {
        let bg = contrast::to_rgb(theme.background_color()).unwrap_or(STUDIO_BG);
        let ink = best_on(bg);
        let text = contrast::to_rgb(theme.text_color()).unwrap_or(ink);
        let fg = ensure(text, bg, AAA);
        let light = contrast::is_light(bg);
        let primary = contrast::to_rgb(theme.primary_color()).unwrap_or(text);
        let secondary = contrast::to_rgb(theme.secondary_color()).unwrap_or(primary);
        let accent = contrast::to_rgb(theme.accent_color()).unwrap_or(primary);
        Self::build(Base {
            bg,
            surface: mix(bg, fg, if light { 0.04 } else { 0.05 }),
            raised: mix(bg, fg, if light { 0.08 } else { 0.10 }),
            sel: mix(bg, fg, if light { 0.13 } else { 0.17 }),
            fg,
            muted: mix(fg, bg, 0.2),
            dim: mix(fg, bg, 0.4),
            border: mix(fg, bg, 0.6),
            accent: primary,
            accent_fill: primary,
            accent2: secondary,
            link: accent,
        })
    }

    pub fn for_settings(follow_theme: bool, theme: &Theme) -> Self {
        if follow_theme {
            Self::from_theme(theme)
        } else {
            Self::studio()
        }
    }

    fn build(b: Base) -> Self {
        let all = [b.bg, b.surface, b.raised, b.sel];
        let text = |c: Rgb| color(on_all(c, &all, AA));
        Self {
            light: contrast::is_light(b.bg),
            bg: color(b.bg),
            surface: color(b.surface),
            raised: color(b.raised),
            sel_bg: color(b.sel),
            fg: color(on_all(b.fg, &all, AAA)),
            muted: color(on_all(b.muted, &all, 6.0)),
            dim: text(b.dim),
            border: color(on_all(b.border, &[b.bg, b.surface], UI)),
            primary: text(b.accent),
            accent_fill: color(b.accent_fill),
            accent2: text(b.accent2),
            link: text(b.link),
            ok: text(GREEN),
            warn: text(YELLOW),
            error: text(RED),
            info: text(BLUE),
            special: text(MAGENTA),
            orange: text(ORANGE),
        }
    }

    /// Text colour that reads on a filled `fill` (pills, badges, buttons).
    pub fn on(fill: Color) -> Color {
        contrast::to_rgb(fill).map_or(Color::Black, |c| color(best_on(c)))
    }

    /// Filled pill style: `fill` background with black/white text.
    pub fn pill(fill: Color) -> Style {
        Style::new().fg(Self::on(fill)).bg(fill)
    }

    /// Theme-provided colour (group icon etc.) nudged until it is visible
    /// (>= 3:1) on every surface.
    pub fn visible(&self, c: Color) -> Color {
        self.lift(c, UI)
    }

    /// Theme-provided colour used as text (>= 4.5:1 on every surface).
    pub fn readable(&self, c: Color) -> Color {
        self.lift(c, AA)
    }

    fn lift(&self, c: Color, min: f64) -> Color {
        let surfaces: Vec<Rgb> = [self.bg, self.surface, self.raised, self.sel_bg]
            .into_iter()
            .filter_map(contrast::to_rgb)
            .collect();
        match contrast::to_rgb(c) {
            Some(c) => color(on_all(c, &surfaces, min)),
            None => c,
        }
    }
}
