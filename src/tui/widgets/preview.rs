//! Live preview that mirrors fastfetch's layout cell for cell: logo with
//! `padding.top = 2` / `padding.right = 4`, module keys padded to `keyWidth`,
//! the same separator and the same resolved colours as the generated config.

use crate::adapters::fastfetch::config::{
    DOT, KEY_WIDTH, LOGO_PADDING_RIGHT, LOGO_PADDING_TOP, SEPARATOR, module_key,
};
use crate::adapters::fastfetch::runner::RealInfo;
use crate::domain::logo::LogoItem;
use crate::domain::preset::{DEMO_DATA, Preset};
use crate::domain::settings::Settings;
use crate::domain::style;
use crate::domain::theme::Theme;
use crate::imaging::{BlockGrid, TermGeometry, config_logo_cells, fit_image_cells, kitty};
use crate::tui::palette::Palette;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Widget};
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageState {
    NotImage,
    Ready((u32, u32)),
    Loading,
    Missing,
}

/// How an image logo is painted into the preview.
#[derive(Debug, Clone, Copy)]
pub enum LogoRaster<'a> {
    /// Not rasterised yet.
    None,
    /// Quadrant block characters (any terminal).
    Blocks(&'a BlockGrid),
    /// Full-resolution image already uploaded to the terminal under this id.
    Kitty(u32),
    /// Sixel image placed on top of the blank logo cells after the frame.
    Sixel,
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
    /// Block-drawn logo that grows to the height of the info text.
    pub fill: bool,
}

pub fn rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (0, 0, 0),
    }
}

impl PreviewModel<'_> {
    fn fg(&self) -> Color {
        self.theme.text_color()
    }

    pub fn bg(&self) -> Color {
        self.theme.background_color()
    }

    /// Logo size in cells for an inner preview area of `avail` (w, h).
    /// Images use exactly the box written to the config, shrunk only when the
    /// preview pane is smaller - always with the image's aspect ratio.
    pub fn logo_cells(&self, avail: (u16, u16)) -> (u16, u16) {
        match self.image {
            ImageState::Ready(dims) => {
                let (mut cw, mut ch) =
                    config_logo_cells(dims, self.settings.logo_size.max_cells(), &self.geo);
                if self.fill {
                    // As tall as the info text, as `termfetch run` draws it in the shell.
                    let rows = (self.lines().len() as u16).max(ch);
                    (cw, ch) = fit_image_cells(dims.0, dims.1, self.geo.cell, avail.0, rows);
                }
                let info_width = self
                    .lines()
                    .iter()
                    .map(Line::width)
                    .max()
                    .unwrap_or(30)
                    .min(46)
                    // Text wraps; the logo keeps at least half of a narrow pane.
                    .min(usize::from(avail.0 / 2)) as u16;
                let max_w = cw.min(
                    avail
                        .0
                        .saturating_sub(info_width + LOGO_PADDING_RIGHT)
                        .max(1),
                );
                let max_h = ch.min(avail.1.saturating_sub(LOGO_PADDING_TOP).max(1));
                if (max_w, max_h) == (cw, ch) {
                    (cw, ch)
                } else {
                    fit_image_cells(dims.0, dims.1, self.geo.cell, max_w, max_h)
                }
            }
            ImageState::Loading | ImageState::Missing => {
                let (w, h) = self.settings.logo_size.max_cells();
                (
                    w.min(avail.0 / 2).max(1),
                    h.min(avail.1.saturating_sub(LOGO_PADDING_TOP)).max(1),
                )
            }
            ImageState::NotImage => {
                let lines = self.logo.art.lines();
                let w = lines.clone().map(UnicodeWidthStr::width).max().unwrap_or(0);
                (w as u16, self.logo.art.lines().count() as u16)
            }
        }
    }

    /// `None` when fastfetch reported nothing: like fastfetch, the line is left out.
    fn value(&self, m: crate::domain::preset::ModuleKind) -> Option<String> {
        if self.settings.demo_mode {
            return Some(m.demo_value().to_string());
        }
        match self.info.and_then(|i| i.get(m)) {
            Some(v) => Some(v.to_string()),
            None if self.info_loading || self.info.is_none() => Some("…".into()),
            None => None,
        }
    }

    fn title(&self) -> (String, String) {
        let raw = if self.settings.demo_mode {
            DEMO_DATA.user_host.to_string()
        } else {
            self.info.and_then(|i| i.title.clone()).unwrap_or_else(|| {
                format!(
                    "{}@…",
                    std::env::var("USER").unwrap_or_else(|_| "user".into())
                )
            })
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
        for (i, c) in style::dot_colors(self.theme, self.settings)
            .iter()
            .enumerate()
        {
            if i > 0 {
                spans.push(Span::styled(" ", Style::new().bg(bg)));
            }
            spans.push(Span::styled(
                DOT,
                Style::new()
                    .fg(c.tui_color(fg))
                    .bg(bg)
                    .add_modifier(Modifier::BOLD),
            ));
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

        let mut first = true;
        for section in &self.preset.sections {
            let shown: Vec<_> = section
                .iter()
                .filter_map(|&m| self.value(m).map(|v| (m, v)))
                .collect();
            if shown.is_empty() {
                continue;
            }
            if !std::mem::take(&mut first) {
                out.push(Line::default());
            }
            for (m, value) in shown {
                let colors = style::module_colors(self.theme, self.settings, m.group());
                let key = module_key(m);
                let lead = key.width() + SEPARATOR.width();
                // fastfetch jumps to column `keyWidth` (1-based) before the value.
                let pad = usize::from(KEY_WIDTH - 1).saturating_sub(lead);
                out.push(Line::from(vec![
                    Span::styled(
                        key,
                        Style::new()
                            .fg(colors.key.tui_color(fg))
                            .bg(bg)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(SEPARATOR, base),
                    Span::styled(" ".repeat(pad), base),
                    Span::styled(value, Style::new().fg(colors.output.tui_color(fg)).bg(bg)),
                ]));
            }
        }
        out.push(Line::default());
        out.push(if self.preset.color_dots {
            self.dots()
        } else {
            Line::default()
        });
        out
    }
}

/// Paint `grid` with quadrant blocks at (x, y), alpha-blended onto `bg`.
pub fn draw_blocks(buf: &mut Buffer, x: u16, y: u16, clip: Rect, grid: &BlockGrid, bg: Color) {
    let bg_rgb = rgb(bg);
    for cy in 0..grid.h {
        for cx in 0..grid.w {
            let (px, py) = (x + cx, y + cy);
            if px >= clip.right() || py >= clip.bottom() {
                continue;
            }
            if grid.pixels(cx, cy).iter().all(|p| p[3] == 0) {
                continue;
            }
            let Some(block) = grid.cell(cx, cy, Some(bg_rgb)) else {
                continue;
            };
            let (fr, fg, fb) = block.fg;
            let (br, bgc, bb) = block.bg.unwrap_or(bg_rgb);
            if let Some(cell) = buf.cell_mut((px, py)) {
                cell.set_char(block.ch)
                    .set_fg(Color::Rgb(fr, fg, fb))
                    .set_bg(Color::Rgb(br, bgc, bb));
            }
        }
    }
}

/// Paint a `w` x `h` Kitty placeholder block for image `id` at (x, y).
pub fn draw_kitty_placeholders(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    clip: Rect,
    (w, h): (u16, u16),
    id: u32,
    bg: Color,
) {
    let fg = kitty::id_color(id);
    for cy in 0..h {
        for cx in 0..w {
            let (px, py) = (x + cx, y + cy);
            if px >= clip.right() || py >= clip.bottom() {
                continue;
            }
            if let Some(cell) = buf.cell_mut((px, py)) {
                cell.reset();
                cell.set_symbol(&kitty::placeholder(cy, cx))
                    .set_fg(fg)
                    .set_bg(bg);
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

/// Studio chrome around the live preview: `● Live Preview · <preset>` with
/// fullscreen / preview-mode icons. Returns the inner (terminal) rect.
pub fn draw_preview_chrome(
    buf: &mut Buffer,
    area: Rect,
    pal: &Palette,
    preset: &str,
    demo: bool,
    focused: bool,
) -> Rect {
    let bg = Style::new().bg(pal.bg);
    let eye = if demo {
        ("\u{f0208} ", pal.ok)
    } else {
        ("\u{f0209} ", pal.warn)
    };
    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(
            Style::new()
                .fg(if focused { pal.primary } else { pal.border })
                .bg(pal.bg),
        )
        .title(Line::from(vec![
            Span::styled(" ● ", Style::new().fg(pal.ok).bg(pal.bg)),
            Span::styled(
                "Live Preview ",
                Style::new()
                    .fg(pal.fg)
                    .bg(pal.bg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("· {preset} "),
                Style::new().fg(pal.muted).bg(pal.bg),
            ),
        ]))
        .title(
            Line::from(vec![
                Span::styled(" \u{f0293}  ", Style::new().fg(pal.muted).bg(pal.bg)),
                Span::styled(eye.0, Style::new().fg(eye.1).bg(pal.bg)),
                Span::styled(" ", bg),
            ])
            .right_aligned(),
        );
    let inner = block.inner(area);
    block.render(area, buf);
    inner
}

/// Draw the full preview pane. `raster` must have been produced at
/// `model.logo_cells(..)` for image logos. With `chrome`, the pane gets the
/// studio frame and the emulated terminal is painted inside it.
pub fn draw_preview(
    buf: &mut Buffer,
    area: Rect,
    model: &PreviewModel,
    raster: LogoRaster,
    chrome: Option<&Palette>,
) {
    draw_preview_scrolled(buf, area, model, raster, chrome, 0);
}

/// Wrap long system values and scroll their full content in Focus Preview.
pub fn draw_preview_scrolled(
    buf: &mut Buffer,
    area: Rect,
    model: &PreviewModel,
    raster: LogoRaster,
    chrome: Option<&Palette>,
    scroll: u16,
) {
    let (bg, fg) = (model.bg(), model.fg());
    let area = area.intersection(buf.area);
    let body = match chrome {
        Some(pal) => draw_preview_chrome(
            buf,
            area,
            pal,
            &model.preset.name,
            model.settings.demo_mode,
            false,
        ),
        None => area,
    };
    buf.set_style(body, Style::new().bg(bg).fg(fg));
    for y in body.top()..body.bottom() {
        for x in body.left()..body.right() {
            if let Some(c) = buf.cell_mut((x, y)) {
                c.set_char(' ');
            }
        }
    }
    let inner = if chrome.is_some() {
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
    let logo_style = Style::new()
        .fg(style::logo_color(model.theme).tui_color(fg))
        .bg(bg);
    match (model.image, raster) {
        (ImageState::Ready(_), LogoRaster::Blocks(g)) => draw_blocks(buf, lx, ly, inner, g, bg),
        (ImageState::Ready(_), LogoRaster::Kitty(id)) => {
            draw_kitty_placeholders(buf, lx, ly, inner, (lw, lh), id, bg)
        }
        // Blank cells: the sixel image is written over them once the frame is out.
        (ImageState::Ready(_), LogoRaster::Sixel) => {}
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
            let msg = if state == ImageState::Missing {
                "image not found"
            } else {
                "decoding…"
            };
            let y = ly + lh / 2;
            if y < inner.bottom() {
                let x = lx + lw.saturating_sub(msg.width() as u16) / 2;
                buf.set_stringn(
                    x,
                    y,
                    msg,
                    usize::from(lw),
                    Style::new().fg(Palette::from_theme(model.theme).dim).bg(bg),
                );
            }
        }
    }

    let mx = lx + lw + LOGO_PADDING_RIGHT;
    if mx >= inner.right() {
        return;
    }
    let width = inner.right() - mx;
    let lines = wrap_info_lines(model.lines(), usize::from(width));
    let wrapped_rows = lines.len();
    let scroll = scroll.min(
        wrapped_rows
            .saturating_sub(usize::from(inner.height))
            .min(u16::MAX as usize) as u16,
    );
    Paragraph::new(lines)
        .scroll((scroll, 0))
        .render(Rect::new(mx, inner.y, width, inner.height), buf);
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
                out.push_str(&format!(
                    "\x1b[0;{};{}{}m",
                    sgr(cell.fg, true),
                    sgr(cell.bg, false),
                    if bold { ";1" } else { "" }
                ));
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

/// Hard-wrap without dropping long, unbroken hardware identifiers or paths.
fn wrap_info_lines(lines: Vec<Line<'static>>, width: usize) -> Vec<Line<'static>> {
    let mut out = Vec::new();
    for line in lines {
        let mut row = Vec::new();
        let mut used = 0;
        for span in line.spans {
            for ch in span.content.chars() {
                let cells = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
                if used + cells > width.max(1) && !row.is_empty() {
                    out.push(Line::from(std::mem::take(&mut row)));
                    used = 0;
                }
                row.push(Span::styled(ch.to_string(), span.style));
                used += cells;
            }
        }
        out.push(Line::from(row));
    }
    out
}

#[cfg(test)]
mod wrapping_tests {
    use super::*;
    #[test]
    fn long_identifiers_and_unicode_survive_wrapping() {
        let input = "CPU-abcdefghijklmnopqrstuvwxyz界界";
        let lines = wrap_info_lines(vec![Line::from(input)], 8);
        assert!(lines.iter().all(|l| l.width() <= 8));
        let restored: String = lines
            .iter()
            .flat_map(|l| l.spans.iter())
            .map(|s| s.content.as_ref())
            .collect();
        assert_eq!(restored, input);
    }
}
