//! Renders all TUI screens and theme swatch thumbnails to a JSON representation
//! for pixel-perfect screenshot generation.
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use serde::Serialize;
use termfetch_studio_v2::app::state::{AppState, Focus, Screen};
use termfetch_studio_v2::domain::contrast::to_rgb;
use termfetch_studio_v2::domain::settings::Settings;
use termfetch_studio_v2::domain::theme::get_builtin_themes;
use termfetch_studio_v2::imaging::TermGeometry;
use termfetch_studio_v2::paths::Paths;
use termfetch_studio_v2::tui::render::render_ui;
use termfetch_studio_v2::tui::widgets::ui::draw_thumbnail;

#[derive(Serialize)]
struct CellDump {
    x: u16,
    y: u16,
    symbol: String,
    fg: (u8, u8, u8),
    bg: (u8, u8, u8),
}

#[derive(Serialize)]
struct ScreenDump {
    name: String,
    width: u16,
    height: u16,
    cells: Vec<CellDump>,
}

fn color_to_rgb(c: Color, fallback: (u8, u8, u8)) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Black => (0, 0, 0),
        Color::Red => (205, 49, 49),
        Color::Green => (13, 188, 121),
        Color::Yellow => (229, 229, 16),
        Color::Blue => (36, 114, 200),
        Color::Magenta => (188, 63, 188),
        Color::Cyan => (17, 168, 205),
        Color::White => (229, 229, 229),
        Color::Gray | Color::DarkGray => (102, 102, 102),
        Color::LightRed => (241, 76, 76),
        Color::LightGreen => (35, 209, 139),
        Color::LightYellow => (245, 245, 67),
        Color::LightBlue => (59, 142, 234),
        Color::LightMagenta => (214, 112, 214),
        Color::LightCyan => (41, 184, 219),
        _ => to_rgb(c).unwrap_or(fallback),
    }
}

fn dump_buffer(name: &str, buf: &Buffer) -> ScreenDump {
    let mut cells = Vec::new();
    let area = buf.area;
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let cell = &buf[(x, y)];
            cells.push(CellDump {
                x,
                y,
                symbol: cell.symbol().to_string(),
                fg: color_to_rgb(cell.fg, (220, 220, 220)),
                bg: color_to_rgb(cell.bg, (15, 17, 26)),
            });
        }
    }
    ScreenDump {
        name: name.to_string(),
        width: area.width,
        height: area.height,
        cells,
    }
}

fn main() {
    let width = 150u16;
    let height = 42u16;
    let mut dumps = Vec::new();

    let screens = [
        ("dashboard", Screen::Dashboard, false),
        ("themes", Screen::Themes, false),
        ("presets", Screen::Presets, false),
        ("logos", Screen::Logos, false),
        ("modules", Screen::Modules, false),
        ("settings", Screen::Settings, false),
        ("preview-focus", Screen::Themes, true),
    ];

    for (name, screen, focus_preview) in screens {
        let temp_dir = std::env::temp_dir().join(format!("tfs-qa-dump-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let paths = Paths::with_root(&temp_dir);
        let settings = Settings {
            demo_mode: true,
            ..Default::default()
        };
        let geo = TermGeometry::detect();
        let mut state = AppState::new(paths, settings, geo);
        state.screen = screen;
        state.focus = Focus::Content;
        if focus_preview {
            state.preview_focus = true;
        }

        let mut term = Terminal::new(TestBackend::new(width, height)).unwrap();
        term.draw(|f| render_ui(f, &mut state)).unwrap();
        dumps.push(dump_buffer(name, term.backend().buffer()));
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    // Dump Gallery Thumbnails (26x6 each)
    let themes = get_builtin_themes();
    for t in &themes {
        let area = Rect::new(0, 0, 26, 6);
        let mut buf = Buffer::empty(area);
        draw_thumbnail(&mut buf, area, t);
        dumps.push(dump_buffer(&format!("gallery_{}", t.id), &buf));
    }

    // Dump Sidebar Thumbnails (9x3 each)
    for t in &themes {
        let area = Rect::new(0, 0, 9, 3);
        let mut buf = Buffer::empty(area);
        draw_thumbnail(&mut buf, area, t);
        dumps.push(dump_buffer(&format!("compact_{}", t.id), &buf));
    }

    let json = serde_json::to_string(&dumps).unwrap();
    let out_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "screens.json".into());
    std::fs::write(&out_path, json).unwrap();
    println!("Dumping completed: written to {out_path}");
}
