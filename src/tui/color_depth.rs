//! Terminals without 24-bit colour. Apple's Terminal.app before macOS 26 misdraws
//! `38;2`/`48;2` sequences, so every RGB cell is mapped to the nearest xterm-256 colour.

use crate::domain::color::nearest_256;
use ratatui::buffer::Buffer;
use ratatui::style::Color;
use std::sync::OnceLock;

/// Whether the running terminal needs RGB colours reduced to the 256-colour palette.
pub fn needs_256() -> bool {
    static NEEDS: OnceLock<bool> = OnceLock::new();
    *NEEDS.get_or_init(|| {
        let env = |k: &str| std::env::var(k).unwrap_or_default();
        lacks_truecolor(&env("TERM_PROGRAM"), &env("COLORTERM"))
    })
}

pub fn lacks_truecolor(program: &str, colorterm: &str) -> bool {
    // Terminal.app advertises truecolor through COLORTERM once it supports it.
    program == "Apple_Terminal" && !matches!(colorterm, "truecolor" | "24bit")
}

fn reduce(c: &mut Color) {
    if let Color::Rgb(r, g, b) = *c {
        *c = Color::Indexed(nearest_256(r, g, b));
    }
}

/// Replace every RGB colour in `buf` with its 256-colour neighbour.
pub fn downsample(buf: &mut Buffer) {
    for cell in buf.content.iter_mut() {
        reduce(&mut cell.fg);
        reduce(&mut cell.bg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_apple_terminal_without_truecolor_is_reduced() {
        assert!(lacks_truecolor("Apple_Terminal", ""));
        assert!(!lacks_truecolor("Apple_Terminal", "truecolor"));
        assert!(!lacks_truecolor("iTerm.app", ""));
        assert!(!lacks_truecolor("ghostty", "truecolor"));
    }

    #[test]
    fn downsample_rewrites_only_rgb_cells() {
        let mut buf = Buffer::empty(ratatui::layout::Rect::new(0, 0, 2, 1));
        buf.content[0].fg = Color::Rgb(255, 0, 0);
        buf.content[1].fg = Color::Blue;
        downsample(&mut buf);
        assert_eq!(buf.content[0].fg, Color::Indexed(196));
        assert_eq!(buf.content[1].fg, Color::Blue);
    }
}
