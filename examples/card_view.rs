use ratatui::{buffer::Buffer, layout::Rect};
use termfetch_studio_v2::{
    domain::theme::get_builtin_themes, tui::widgets::preview::buffer_to_ansi,
    tui::widgets::ui::draw_thumbnail,
};

fn main() {
    let themes = get_builtin_themes();
    for (i, theme) in themes.iter().enumerate() {
        // Exact size in theme card: 20 cols x 3 lines (6 halfblock rows)
        let area = Rect::new(0, 0, 20, 3);
        let mut buf = Buffer::empty(area);
        draw_thumbnail(&mut buf, area, theme);
        println!("=== {:02}. {} ===", i + 1, theme.name);
        println!("{}", buffer_to_ansi(&buf));
    }
}
