//! Live preview that mirrors fastfetch's layout cell for cell: logo with
//! `padding.top = 2` / `padding.right = 4`, module keys padded to `keyWidth`,
//! the same separator and the same resolved colours as the generated config.

use crate::adapters::fastfetch::config::{module_key, DOT, KEY_WIDTH, LOGO_PADDING_RIGHT, LOGO_PADDING_TOP, SEPARATOR};
use crate::adapters::fastfetch::runner::RealInfo;
use crate::domain::logo::LogoItem;
use crate::domain::preset::{Preset, DEMO_DATA};
use crate::domain::settings::Settings;
use crate::domain::style;
use crate::domain::theme::Theme;
use crate::imaging::{blend, config_logo_cells, fit_image_cells, HalfblockGrid, TermGeometry};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Widget};
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageState {
    NotImage,
    Ready((u32, u32)),
    Loading,
    Missing,
}

pub struct PreviewModel<'a> {
    pub theme: &'a Theme,
    pub preset: &'a Preset,
    pub logo: &'a LogoItem,
    pub settings: &'a Settings,
    pub info: Option<&'a RealInfo>,
    pub info_loading: bool,
    pub image: ImageState,
    pub geo: TermGeometry,
}

fn rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (0, 0, 0),
    }
}

impl PreviewModel<'_> {
    fn fg(&self) -> Color {
        self.theme.text_color()
    }

    fn bg(&self) -> Color {
        self.theme.background_color()
    }

    /// Logo size in cells for an inner preview area of `avail` (w, h).
    /// Images use exactly the box written to the config, shrunk only when the
    /// preview pane is smaller - always with the image's aspect ratio.
    pub fn logo_cells(&self, avail: (u16, u16)) -> (u16, u16) {
        match self.image {
            ImageState::Ready(dims) => {
                let (cw, ch) = config_logo_cells(dims, self.settings.logo_size.max_cells(), &self.geo);
                let max_w = cw.min((avail.0 / 2).max(1));
                let max_h = ch.min(avail.1.saturating_sub(LOGO_PADDING_TOP).max(1));
                if (max_w, max_h) == (cw, ch) {
                    (cw, ch)
                } else {
                    fit_image_cells(dims.0, dims.1, self.geo.cell, max_w, max_h)
                }
            }
            ImageState::Loading | ImageState::Missing => {
                let (w, h) = self.settings.logo_size.max_cells();
                (w.min(avail.0 / 2).max(1), h.min(avail.1.saturating_sub(LOGO_PADDING_TOP)).max(1))
            }
            ImageState::NotImage => {
                let lines = self.logo.art.lines();
                let w = lines.clone().map(UnicodeWidthStr::width).max().unwrap_or(0);
                (w as u16, self.logo.art.lines().count() as u16)
            }
        }
    }

    fn value(&self, m: crate::domain::preset::ModuleKind) -> String {
        if self.settings.demo_mode {
            return m.demo_value().to_string();
        }
        match self.info.and_then(|i| i.get(m)) {
            Some(v) => v.to_string(),
            None if self.info_loading || self.info.is_none() => "…".into(),
            None => "-".into(),
        }
    }

    fn title(&self) -> (String, String) {
        let raw = if self.settings.demo_mode {
            DEMO_DATA.user_host.to_string()
        } else {
            self.info
                .and_then(|i| i.title.clone())
                .unwrap_or_else(|| format!("{}@…", std::env::var("USER").unwrap_or_else(|_| "user".into())))
        };
        match raw.split_once('@') {
            Some((u, h)) => (u.to_string(), h.to_string()),
            None => (raw, String::new()),
        }
    }

    fn dots(&self) -> Line<'static> {
        let bg = self.bg();
        let fg = self.fg();
        let mut spans = Vec::new();
        for (i, c) in style::dot_colors(self.theme, self.settings).iter().enumerate() {
            if i > 0 {
                spans.push(Span::styled(" ", Style::new().bg(bg)));
            }
            spans.push(Span::styled(DOT, Style::new().fg(c.tui_color(fg)).bg(bg).add_modifier(Modifier::BOLD)));
        }
        Line::from(spans)
    }

    /// The module column, line by line, exactly as fastfetch prints it.
    pub fn lines(&self) -> Vec<Line<'static>> {
        let (bg, fg) = (self.bg(), self.fg());
        let base = Style::new().bg(bg).fg(fg);
        let mut out: Vec<Line<'static>> = vec![Line::default(), Line::default()];

        let tc = style::title_color(self.theme).tui_color(fg);
        let text_c = style::text_color(self.theme, self.settings).tui_color(fg);
        let (user, host) = self.title();
        let title_style = Style::new().fg(tc).bg(bg).add_modifier(Modifier::BOLD);
        out.push(Line::from(vec![
            Span::styled(user, title_style),
            Span::styled("@", Style::new().fg(text_c).bg(bg)),
            Span::styled(host, title_style),
        ]));
        out.push(Line::default());
        if self.preset.color_dots {
            out.push(self.dots());
            out.push(Line::default());
        }

        for (i, section) in self.preset.sections.iter().enumerate() {
            if i > 0 {
                out.push(Line::default());
            }
            for &m in section {
                let colors = style::module_colors(self.theme, self.settings, m.group());
                let key = module_key(m);
                let lead = key.width() + SEPARATOR.width();
                // fastfetch jumps to column `keyWidth` (1-based) before the value.
                let pad = usize::from(KEY_WIDTH - 1).saturating_sub(lead);
                out.push(Line::from(vec![
                    Span::styled(key, Style::new().fg(colors.key.tui_color(fg)).bg(bg).add_modifier(Modifier::BOLD)),
                    Span::styled(SEPARATOR, base),
                    Span::styled(" ".repeat(pad), base),
                    Span::styled(self.value(m), Style::new().fg(colors.output.tui_color(fg)).bg(bg)),
                ]));
            }
        }
        out.push(Line::default());
        out.push(if self.preset.color_dots { self.dots() } else { Line::default() });
        out
    }
}

/// Paint `grid` with upper-half blocks at (x, y), alpha-blended onto `bg`.
pub fn draw_halfblocks(buf: &mut Buffer, x: u16, y: u16, clip: Rect, grid: &HalfblockGrid, bg: Color) {
    let bg_rgb = rgb(bg);
    for cy in 0..grid.h {
        for cx in 0..grid.w {
            let (px, py) = (x + cx, y + cy);
            if px >= clip.right() || py >= clip.bottom() {
                continue;
            }
            let (top, bottom) = grid.cell(cx, cy);
            if top[3] == 0 && bottom[3] == 0 {
                continue;
            }
            let (tr, tg, tb) = blend(top, bg_rgb);
            let (br, bgc, bb) = blend(bottom, bg_rgb);
            if let Some(cell) = buf.cell_mut((px, py)) {
                cell.set_char('▀').set_fg(Color::Rgb(tr, tg, tb)).set_bg(Color::Rgb(br, bgc, bb));
            }
        }
    }
}

/// Inner content rect (inside the border, with a one-column margin).
pub fn inner_area(area: Rect) -> Rect {
    let inner = Block::new().borders(Borders::ALL).inner(area);
    Rect {
        x: inner.x + 1,
        width: inner.width.saturating_sub(1),
        ..inner
    }
}

/// Draw the full preview pane. `grid` must have been rasterised at
/// `model.logo_cells(..)` for image logos.
pub fn draw_preview(buf: &mut Buffer, area: Rect, model: &PreviewModel, grid: Option<&HalfblockGrid>, show_chrome: bool) {
    let (bg, fg) = (model.bg(), model.fg());
    let primary = model.theme.primary_color();
    buf.set_style(area, Style::new().bg(bg).fg(fg));
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(c) = buf.cell_mut((x, y)) {
                c.set_char(' ');
            }
        }
    }

    let inner = if show_chrome {
        let demo = if model.settings.demo_mode { " demo " } else { " real specs " };
        Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(primary).bg(bg))
            .title(Line::from(vec![
                Span::styled(" \u{f02e9} Live Preview ", Style::new().fg(primary).bg(bg).add_modifier(Modifier::BOLD)),
                Span::styled(format!("· {} ", model.preset.name), Style::new().fg(fg).bg(bg)),
            ]))
            .title(Line::from(Span::styled(format!(" {} ", model.theme.name), Style::new().fg(fg).bg(bg))).right_aligned())
            .title_bottom(Line::from(vec![
                Span::styled(demo, Style::new().fg(bg).bg(if model.settings.demo_mode { model.theme.accent_color() } else { model.theme.secondary_color() })),
                Span::styled(" p ", Style::new().fg(primary).bg(bg).add_modifier(Modifier::BOLD)),
                Span::styled("run fastfetch ", Style::new().fg(fg).bg(bg)),
            ]).right_aligned())
            .render(area, buf);
        inner_area(area)
    } else {
        area
    };
    if inner.width < 4 || inner.height < 3 {
        return;
    }

    let (lw, lh) = model.logo_cells((inner.width, inner.height));
    let lx = inner.x;
    let ly = inner.y + LOGO_PADDING_TOP;
    let logo_style = Style::new().fg(style::logo_color(model.theme).tui_color(fg)).bg(bg);
    match (model.image, grid) {
        (ImageState::Ready(_), Some(g)) => draw_halfblocks(buf, lx, ly, inner, g, bg),
        (ImageState::NotImage, _) => {
            for (i, l) in model.logo.art.lines().enumerate() {
                let y = ly + i as u16;
                if y >= inner.bottom() {
                    break;
                }
                buf.set_stringn(lx, y, l, usize::from(inner.width), logo_style);
            }
        }
        (state, _) => {
            let msg = if state == ImageState::Missing { "image not found" } else { "decoding…" };
            let y = ly + lh / 2;
            if y < inner.bottom() {
                let x = lx + lw.saturating_sub(msg.width() as u16) / 2;
                buf.set_stringn(x, y, msg, usize::from(lw), Style::new().fg(Color::DarkGray).bg(bg));
            }
        }
    }

    let mx = lx + lw + LOGO_PADDING_RIGHT;
    if mx >= inner.right() {
        return;
    }
    let width = inner.right() - mx;
    for (i, line) in model.lines().iter().enumerate() {
        let y = inner.y + i as u16;
        if y >= inner.bottom() {
            break;
        }
        buf.set_line(mx, y, line, width);
    }
}

/// Convert a rendered buffer to a string with ANSI SGR colours (CLI `--preview`).
pub fn buffer_to_ansi(buf: &Buffer) -> String {
    fn sgr(c: Color, fg: bool) -> String {
        let base = if fg { 30 } else { 40 };
        match c {
            Color::Rgb(r, g, b) => format!("{};2;{r};{g};{b}", if fg { 38 } else { 48 }),
            Color::Reset => format!("{}", if fg { 39 } else { 49 }),
            Color::Black => format!("{base}"),
            Color::Red => format!("{}", base + 1),
            Color::Green => format!("{}", base + 2),
            Color::Yellow => format!("{}", base + 3),
            Color::Blue => format!("{}", base + 4),
            Color::Magenta => format!("{}", base + 5),
            Color::Cyan => format!("{}", base + 6),
            Color::Gray => format!("{}", base + 7),
            Color::DarkGray => format!("{}", base + 60),
            Color::LightRed => format!("{}", base + 61),
            Color::LightGreen => format!("{}", base + 62),
            Color::LightYellow => format!("{}", base + 63),
            Color::LightBlue => format!("{}", base + 64),
            Color::LightMagenta => format!("{}", base + 65),
            Color::LightCyan => format!("{}", base + 66),
            Color::White => format!("{}", base + 67),
            Color::Indexed(i) => format!("{};5;{i}", if fg { 38 } else { 48 }),
        }
    }
    let area = buf.area;
    let mut out = String::new();
    for y in area.top()..area.bottom() {
        let mut last: Option<(Color, Color, bool)> = None;
        let mut skip = 0usize;
        for x in area.left()..area.right() {
            if skip > 0 {
                skip -= 1;
                continue;
            }
            let cell = &buf[(x, y)];
            let bold = cell.modifier.contains(Modifier::BOLD);
            let key = (cell.fg, cell.bg, bold);
            if last != Some(key) {
                out.push_str(&format!("\x1b[0;{};{}{}m", sgr(cell.fg, true), sgr(cell.bg, false), if bold { ";1" } else { "" }));
                last = Some(key);
            }
            let sym = cell.symbol();
            out.push_str(sym);
            skip = sym.width().saturating_sub(1);
        }
        out.push_str("\x1b[0m\n");
    }
    out
}
