//! Export the actual halfblock thumbnail engine as a nearest-neighbor atlas.
//! cargo run --example theme_gallery -- docs/ui-theme-gallery.png
use image::{Rgb, RgbImage};
use ratatui::{buffer::Buffer, layout::Rect, style::Color};
use termfetch_studio_v2::{domain::theme::get_builtin_themes, tui::widgets::ui::draw_thumbnail};
fn main() {
    let themes = get_builtin_themes();
    let mut atlas = RgbImage::from_pixel(6 * 204, 5 * 108, Rgb([14, 17, 29]));
    for (i, theme) in themes.iter().enumerate() {
        let area = Rect::new(0, 0, 64, 16);
        let mut buf = Buffer::empty(area);
        draw_thumbnail(&mut buf, area, theme);
        for y in 0..32u16 {
            for x in 0..64u16 {
                let cell = &buf[(x, y / 2)];
                if let Color::Rgb(r, g, b) = if y % 2 == 0 { cell.fg } else { cell.bg } {
                    for dy in 0..3 {
                        for dx in 0..3 {
                            atlas.put_pixel((i as u32 % 6) * 204 + 6 + u32::from(x) * 3 + dx, (i as u32 / 6) * 108 + 6 + u32::from(y) * 3 + dy, Rgb([r, g, b]));
                        }
                    }
                }
            }
        }
        println!("{}: {}", i + 1, theme.name);
    }
    atlas.save(std::env::args().nth(1).unwrap_or_else(|| "theme-gallery.png".into())).expect("save atlas");
}
