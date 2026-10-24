//! WCAG 2.x relative luminance / contrast helpers. The TUI derives every
//! chrome colour from the active theme background through these, so light
//! themes get dark text and dark themes get light text.

use ratatui::style::Color;

pub type Rgb = (u8, u8, u8);

pub const BLACK: Rgb = (0, 0, 0);
pub const WHITE: Rgb = (255, 255, 255);

/// Body text (WCAG AAA).
pub const AAA: f64 = 7.0;
/// Secondary text (WCAG AA).
pub const AA: f64 = 4.5;
/// Borders, icons and other non-text UI (WCAG 1.4.11).
pub const UI: f64 = 3.0;

fn channel(c: u8) -> f64 {
    let c = f64::from(c) / 255.0;
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

/// Relative luminance in `0.0..=1.0`.
pub fn luminance((r, g, b): Rgb) -> f64 {
    0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b)
}

/// Contrast ratio in `1.0..=21.0`.
pub fn contrast(a: Rgb, b: Rgb) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// A background is "light" when black text reads better on it than white text.
pub fn is_light(bg: Rgb) -> bool {
    contrast(bg, BLACK) > contrast(bg, WHITE)
}

/// Linear blend: `t = 0` gives `a`, `t = 1` gives `b`.
pub fn mix(a: Rgb, b: Rgb, t: f64) -> Rgb {
    let l = |x: u8, y: u8| (f64::from(x) + (f64::from(y) - f64::from(x)) * t).round().clamp(0.0, 255.0) as u8;
    (l(a.0, b.0), l(a.1, b.1), l(a.2, b.2))
}

/// `fg` itself when it already reaches `min` against `bg`, otherwise the
/// least-changed blend of `fg` towards black (light bg) or white (dark bg)
/// that does. Keeps the hue while guaranteeing legibility.
pub fn ensure(fg: Rgb, bg: Rgb, min: f64) -> Rgb {
    if contrast(fg, bg) >= min {
        return fg;
    }
    let target = if is_light(bg) { BLACK } else { WHITE };
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..20 {
        let mid = (lo + hi) / 2.0;
        if contrast(mix(fg, target, mid), bg) >= min { hi = mid } else { lo = mid }
    }
    mix(fg, target, hi)
}

/// Black or white, whichever reads better on `bg` (text on a filled pill).
pub fn best_on(bg: Rgb) -> Rgb {
    if is_light(bg) { BLACK } else { WHITE }
}

pub fn to_rgb(c: Color) -> Option<Rgb> {
    match c {
        Color::Rgb(r, g, b) => Some((r, g, b)),
        _ => None,
    }
}

pub fn color((r, g, b): Rgb) -> Color {
    Color::Rgb(r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_ratios() {
        assert!((contrast(BLACK, WHITE) - 21.0).abs() < 1e-9);
        assert!((contrast((119, 119, 119), WHITE) - 4.48).abs() < 0.01);
        assert!(is_light((0xfd, 0xf6, 0xe3)));
        assert!(!is_light((0x1e, 0x1e, 0x2e)));
    }

    #[test]
    fn ensure_reaches_target_on_both_polarities() {
        for bg in [(0xfd, 0xf6, 0xe3), (0xff, 0xff, 0xff), (0x1e, 0x1e, 0x2e), (0, 0, 0)] {
            for fg in [(255, 255, 255), (200, 200, 200), (0xb5, 0x89, 0x00), (0, 0, 0), (0x26, 0x8b, 0xd2)] {
                let out = ensure(fg, bg, AAA);
                assert!(contrast(out, bg) >= AAA - 0.01, "{fg:?} on {bg:?} -> {out:?}");
            }
        }
        // Already readable colours are untouched.
        assert_eq!(ensure((0x07, 0x36, 0x42), (0xfd, 0xf6, 0xe3), AA), (0x07, 0x36, 0x42));
    }
}
