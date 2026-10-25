use ratatui::{buffer::Buffer, layout::Rect, style::Color};
use termfetch_studio_v2::{domain::theme::get_builtin_themes, tui::widgets::ui::draw_thumbnail};

fn main() {
    let themes = get_builtin_themes();
    for (i, theme) in themes.iter().enumerate() {
        let area = Rect::new(0, 0, 20, 3);
        let mut buf = Buffer::empty(area);
        draw_thumbnail(&mut buf, area, theme);
        println!("=== {:02}. {} ({}) ===", i + 1, theme.name, theme.id);
        for row in 0..6 {
            let cy = row / 2;
            let top = (row % 2) == 0;
            let mut line = String::new();
            for cx in 0..20 {
                let cell = &buf[(cx, cy)];
                let col = if top { cell.fg } else { cell.bg };
                match col {
                    Color::Rgb(r, g, b) => {
                        // print colored block
                        line.push_str(&format!("\x1b[38;2;{r};{g};{b}m█\x1b[0m"));
                    }
                    _ => line.push('?'),
                }
            }
            println!("  r{}: {}", row, line);
        }
    }
}
