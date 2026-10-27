//! Dashboard center stage: welcome banner, quick actions, recent themes,
//! system overview and tools.

use crate::app::state::{AppState, DashRow, Focus, Hit, QUICK_ACTIONS};
use crate::domain::preset::ModuleKind;
use crate::tui::palette::Palette;
use crate::tui::widgets::ui::{
    THEME_CARD_H, THEME_CARD_W, card, draw_theme_card, ellipsize, fill, line_at, width,
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

pub struct QuickAction {
    pub icon: &'static str,
    pub title: &'static str,
    pub desc: &'static str,
}

pub const QUICK: [QuickAction; QUICK_ACTIONS] = [
    QuickAction {
        icon: "\u{f03d8}",
        title: "Edit Theme",
        desc: "Browse & edit palette",
    },
    QuickAction {
        icon: "\u{f0249}",
        title: "Manage Presets",
        desc: "Organize module sets",
    },
    QuickAction {
        icon: "\u{f02d4}",
        title: "Change Logo",
        desc: "ASCII & image logos",
    },
    QuickAction {
        icon: "\u{f040a}",
        title: "Apply & Preview",
        desc: "Live terminal preview",
    },
];

pub const TOOLS: [QuickAction; crate::app::state::TOOLS] = [
    QuickAction {
        icon: "\u{f0147}",
        title: "System Diagnostics",
        desc: "Check dependencies and system info",
    },
    QuickAction {
        icon: "\u{f018d}",
        title: "Shell Integration",
        desc: "Configure your shell (zsh, bash, fish)",
    },
    QuickAction {
        icon: "\u{f01bc}",
        title: "Backup & Restore",
        desc: "Manage your configurations",
    },
    QuickAction {
        icon: "\u{f0552}",
        title: "Import / Export",
        desc: "Share themes and presets",
    },
];

fn quick_colors(pal: &Palette) -> [Color; 4] {
    [pal.special, pal.info, pal.orange, pal.ok]
}

/// Greedy word wrap into at most `lines` lines of `w` cells.
fn wrap(text: &str, w: usize, lines: usize) -> Vec<String> {
    let mut out: Vec<String> = vec![String::new()];
    for word in text.split_whitespace() {
        let cur = out.last_mut().expect("non-empty");
        if cur.is_empty() {
            cur.push_str(word);
        } else if width(cur) as usize + 1 + width(word) as usize <= w {
            cur.push(' ');
            cur.push_str(word);
        } else {
            out.push(word.to_string());
        }
    }
    if out.len() > lines {
        let rest = out[lines - 1..].join(" ");
        out.truncate(lines - 1);
        out.push(rest);
    }
    out.into_iter().map(|l| ellipsize(&l, w)).collect()
}

/// Section header inside a card: icon + bold title (+ muted subtitle row).
fn section_header(
    buf: &mut ratatui::buffer::Buffer,
    area: Rect,
    pal: &Palette,
    icon: &str,
    icon_c: Color,
    title: &str,
    subtitle: Option<&str>,
) {
    line_at(
        buf,
        area,
        0,
        Line::from(vec![
            Span::styled(format!("{icon} "), Style::new().fg(icon_c)),
            Span::styled(
                title.to_string(),
                Style::new().fg(pal.fg).add_modifier(Modifier::BOLD),
            ),
        ]),
    );
    if let Some(s) = subtitle {
        line_at(
            buf,
            area,
            1,
            Line::from(Span::styled(format!("   {s}"), Style::new().fg(pal.muted))),
        );
    }
}

pub fn render_dashboard(f: &mut Frame, area: Rect, state: &mut AppState) {
    let pal = state.palette();
    fill(f.buffer_mut(), area, Style::new().bg(pal.bg).fg(pal.fg));
    let area = Rect {
        x: area.x + 1,
        width: area.width.saturating_sub(2),
        ..area
    };
    let roomy = area.height >= 36;
    let (welcome_h, mut quick_h, recent_h) = match area.height {
        h if h >= 36 => (3, 9, THEME_CARD_H + 4),
        h if h >= 24 => (3, 7, 9),
        _ => (2, 6, 7),
    };
    if area.width < 68 {
        quick_h = quick_h.max(10);
    }
    let [welcome, quick, recent, bottom] = Layout::vertical([
        Constraint::Length(welcome_h),
        Constraint::Length(quick_h),
        Constraint::Length(recent_h),
        Constraint::Min(0),
    ])
    .areas(area);

    // Welcome banner.
    let buf = f.buffer_mut();
    line_at(
        buf,
        welcome,
        0,
        Line::from(vec![
            Span::styled(
                "👋 Welcome to ",
                Style::new()
                    .fg(pal.fg)
                    .bg(pal.bg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "TermFetch Studio",
                Style::new()
                    .fg(pal.accent2)
                    .bg(pal.bg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "!",
                Style::new()
                    .fg(pal.fg)
                    .bg(pal.bg)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    );
    line_at(
        buf,
        welcome,
        1,
        Line::from(Span::styled(
            format!(
                "{} · {} · {}",
                state.current_theme().name,
                state.current_preset().name,
                // `fastfetch --version` already starts with "fastfetch".
                state
                    .fastfetch_version
                    .as_deref()
                    .unwrap_or("fastfetch missing")
            ),
            Style::new().fg(pal.muted).bg(pal.bg),
        )),
    );

    render_quick(f, quick, state, &pal, roomy);
    render_recent(f, recent, state, &pal, roomy);
    if bottom.height >= 4 {
        let [left, right] =
            Layout::horizontal([Constraint::Percentage(48), Constraint::Percentage(52)])
                .spacing(1)
                .areas(bottom);
        render_overview(f, left, state, &pal);
        render_tools(f, right, state, &pal);
    }
}

fn dash_focus(state: &AppState, row: DashRow) -> Option<usize> {
    (state.focus == Focus::Content && state.dash.row == row).then_some(state.dash.col)
}

fn render_quick(f: &mut Frame, area: Rect, state: &mut AppState, pal: &Palette, roomy: bool) {
    if area.height < 3 {
        return;
    }
    let block = card(pal, false);
    let inner = block.inner(area);
    block.render(area, f.buffer_mut());
    let inner = Rect {
        x: inner.x + 1,
        width: inner.width.saturating_sub(2),
        ..inner
    };
    let header_h = if roomy { 2 } else { 1 };
    section_header(
        f.buffer_mut(),
        inner,
        pal,
        "⚡",
        pal.warn,
        "Quick Actions",
        roomy.then_some("Get started with the most common tasks."),
    );
    let tiles = Rect {
        y: inner.y + header_h,
        height: inner.height.saturating_sub(header_h),
        ..inner
    };
    if tiles.height < 3 {
        return;
    }
    let cols: Vec<Rect> = if tiles.width >= 64 {
        Layout::horizontal([Constraint::Ratio(1, 4); 4])
            .spacing(1)
            .split(tiles)
            .to_vec()
    } else {
        Layout::vertical([Constraint::Ratio(1, 2); 2])
            .split(tiles)
            .iter()
            .flat_map(|&row| {
                Layout::horizontal([Constraint::Ratio(1, 2); 2])
                    .spacing(1)
                    .split(row)
                    .to_vec()
            })
            .collect()
    };
    let focus = dash_focus(state, DashRow::Quick);
    let colors = quick_colors(pal);
    for (i, (qa, &rect)) in QUICK.iter().zip(cols.iter()).enumerate() {
        let sel = focus == Some(i);
        let bg = if sel { pal.sel_bg } else { pal.raised };
        let block = card(pal, sel)
            .style(Style::new().bg(bg).fg(pal.fg))
            .border_style(
                Style::new()
                    .fg(if sel { pal.primary } else { pal.border })
                    .bg(pal.surface),
            );
        let t = block.inner(rect);
        block.render(rect, f.buffer_mut());
        let tw = usize::from(t.width);
        let badge = Span::styled(
            format!(" {} ", i + 1),
            Style::new()
                .fg(pal.fg)
                .bg(pal.surface)
                .add_modifier(Modifier::BOLD),
        );
        let icon = Span::styled(
            qa.icon,
            Style::new()
                .fg(colors[i])
                .bg(bg)
                .add_modifier(Modifier::BOLD),
        );
        let buf = f.buffer_mut();
        line_at(buf, t, 0, Line::from(icon));
        line_at(
            buf,
            t,
            0,
            Line::from(vec![badge, Span::styled(" ", Style::new().bg(bg))]).right_aligned(),
        );
        let title_y = if t.height >= 5 { 2 } else { 1 };
        line_at(
            buf,
            t,
            title_y,
            Line::from(Span::styled(
                ellipsize(qa.title, tw),
                Style::new()
                    .fg(if sel { pal.primary } else { pal.fg })
                    .bg(bg)
                    .add_modifier(Modifier::BOLD),
            )),
        );
        let desc_lines = usize::from(t.height.saturating_sub(title_y + 1)).min(2);
        if desc_lines > 0 {
            for (k, l) in wrap(qa.desc, tw.saturating_sub(1), desc_lines)
                .into_iter()
                .enumerate()
            {
                line_at(
                    buf,
                    t,
                    title_y + 1 + k as u16,
                    Line::from(Span::styled(l, Style::new().fg(pal.muted).bg(bg))),
                );
            }
        }
        state.hits.push((rect, Hit::Quick(i)));
    }
}

fn render_recent(f: &mut Frame, area: Rect, state: &mut AppState, pal: &Palette, roomy: bool) {
    if area.height < 3 {
        return;
    }
    let block = card(pal, false);
    let inner = block.inner(area);
    block.render(area, f.buffer_mut());
    let inner = Rect {
        x: inner.x + 1,
        width: inner.width.saturating_sub(2),
        ..inner
    };
    let header_h = if roomy { 2 } else { 1 };
    section_header(
        f.buffer_mut(),
        inner,
        pal,
        "\u{f051a}",
        pal.accent2,
        "Recent Themes",
        roomy.then_some("Quickly switch between your favorite themes."),
    );
    let view_all = "View All →";
    line_at(
        f.buffer_mut(),
        inner,
        0,
        Line::from(Span::styled(
            view_all,
            Style::new().fg(pal.link).add_modifier(Modifier::BOLD),
        ))
        .right_aligned(),
    );
    let va_w = width(view_all);
    state.hits.push((
        Rect {
            x: inner.right().saturating_sub(va_w),
            y: inner.y,
            width: va_w,
            height: 1,
        },
        Hit::ViewAll,
    ));

    let tiles = Rect {
        y: inner.y + header_h,
        height: inner.height.saturating_sub(header_h),
        ..inner
    };
    if tiles.height < 3 {
        return;
    }
    let list = state.recent_theme_indices();
    let n = (usize::from(tiles.width.saturating_sub(1)) / usize::from(THEME_CARD_W + 1))
        .clamp(1, 4)
        .min(list.len().max(1));
    state.recent_visible = n;
    let focus = dash_focus(state, DashRow::Recent);
    if let Some(col) = focus {
        if col < state.recent_offset {
            state.recent_offset = col;
        } else if col >= state.recent_offset + n {
            state.recent_offset = col + 1 - n;
        }
    }
    state.recent_offset = state.recent_offset.min(list.len().saturating_sub(n));
    let more = state.recent_offset + n < list.len();
    let row = Rect {
        width: tiles.width.saturating_sub(2),
        ..tiles
    };
    let cols = Layout::horizontal(vec![Constraint::Ratio(1, n as u32); n])
        .spacing(1)
        .split(row);
    for (k, &rect) in cols.iter().enumerate() {
        let pos = state.recent_offset + k;
        let Some(&ti) = list.get(pos) else { break };
        let theme = &state.themes[ti];
        let active = theme.id == state.settings.theme;
        let sel = focus == Some(pos);
        let border = if sel {
            pal.accent2
        } else if active {
            pal.primary
        } else {
            pal.border
        };
        draw_theme_card(
            f.buffer_mut(),
            rect,
            pal,
            theme,
            border,
            sel,
            active.then_some(" ✓ "),
        );
        state.hits.push((rect, Hit::Recent(pos)));
    }
    if more || state.recent_offset > 0 {
        let arrow = Rect {
            x: tiles.right().saturating_sub(1),
            y: tiles.y + tiles.height / 2,
            width: 1,
            height: 1,
        };
        line_at(
            f.buffer_mut(),
            arrow,
            0,
            Line::from(Span::styled(
                if more { "›" } else { "‹" },
                Style::new().fg(pal.fg).add_modifier(Modifier::BOLD),
            )),
        );
        state.hits.push((arrow, Hit::RecentScroll));
    }
}

fn render_overview(f: &mut Frame, area: Rect, state: &AppState, pal: &Palette) {
    let block = card(pal, false);
    let inner = block.inner(area);
    block.render(area, f.buffer_mut());
    let inner = Rect {
        x: inner.x + 1,
        width: inner.width.saturating_sub(2),
        ..inner
    };
    section_header(
        f.buffer_mut(),
        inner,
        pal,
        "\u{f02fc}",
        pal.info,
        "System Overview",
        None,
    );
    let os_icon = if state.settings.demo_mode {
        "\u{f08c7}"
    } else {
        state.sysinfo.distro_icon()
    };
    let rows: [(&str, Color, &str, ModuleKind); 10] = [
        (os_icon, pal.info, "OS", ModuleKind::Os),
        ("\u{f0322}", pal.error, "Host", ModuleKind::Host),
        ("\u{f061a}", pal.orange, "Kernel", ModuleKind::Kernel),
        ("\u{f0954}", pal.warn, "Uptime", ModuleKind::Uptime),
        ("\u{f07b7}", pal.link, "Terminal", ModuleKind::Terminal),
        ("\u{f048d}", pal.special, "Shell", ModuleKind::Shell),
        ("󰻠", pal.info, "CPU", ModuleKind::Cpu),
        ("󰢮", pal.special, "GPU", ModuleKind::Gpu),
        ("󰍛", pal.ok, "Memory", ModuleKind::Memory),
        ("󰍹", pal.orange, "Display", ModuleKind::Display),
    ];
    let gap = 1;
    let w = usize::from(inner.width);
    for (i, (icon, c, label, m)) in rows.iter().enumerate() {
        let value = if state.settings.demo_mode {
            m.demo_value().to_string()
        } else {
            state
                .real_info
                .as_ref()
                .and_then(|r| r.get(*m))
                .map(str::to_string)
                .unwrap_or_else(|| {
                    if state.real_rx.is_some() {
                        "…".into()
                    } else {
                        "-".into()
                    }
                })
        };
        line_at(
            f.buffer_mut(),
            inner,
            gap + i as u16,
            Line::from(vec![
                Span::styled(format!("{icon}  "), Style::new().fg(*c)),
                Span::styled(format!("{label:<10}"), Style::new().fg(pal.muted)),
                Span::styled(
                    ellipsize(&value, w.saturating_sub(13)),
                    Style::new().fg(pal.fg),
                ),
            ]),
        );
    }
}

fn render_tools(f: &mut Frame, area: Rect, state: &mut AppState, pal: &Palette) {
    let block = card(pal, false);
    let inner = block.inner(area);
    block.render(area, f.buffer_mut());
    let inner = Rect {
        x: inner.x + 1,
        width: inner.width.saturating_sub(2),
        ..inner
    };
    section_header(
        f.buffer_mut(),
        inner,
        pal,
        "\u{f0ad}",
        pal.muted,
        "Tools & Utilities",
        None,
    );
    let focus = dash_focus(state, DashRow::Tools);
    let per = if inner.height >= 9 { 2 } else { 1 };
    let colors = [pal.warn, pal.info, pal.special, pal.link];
    let w = usize::from(inner.width);
    for (i, tool) in TOOLS.iter().enumerate() {
        let dy = 1 + i as u16 * per;
        if dy >= inner.height {
            break;
        }
        let sel = focus == Some(i);
        let bg = if sel { pal.sel_bg } else { pal.surface };
        let rect = Rect {
            y: inner.y + dy,
            height: per.min(inner.height - dy),
            ..inner
        };
        fill(f.buffer_mut(), rect, Style::new().bg(bg));
        let buf = f.buffer_mut();
        line_at(
            buf,
            rect,
            0,
            Line::from(vec![
                Span::styled(
                    format!(" {}  ", tool.icon),
                    Style::new().fg(colors[i]).bg(bg),
                ),
                Span::styled(
                    ellipsize(tool.title, w.saturating_sub(7)),
                    Style::new()
                        .fg(if sel { pal.primary } else { pal.fg })
                        .bg(bg)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        );
        if per == 2 {
            line_at(
                buf,
                rect,
                1,
                Line::from(Span::styled(
                    format!("    {}", ellipsize(tool.desc, w.saturating_sub(7))),
                    Style::new().fg(pal.dim).bg(bg),
                )),
            );
        }
        line_at(
            buf,
            rect,
            0,
            Line::from(Span::styled("› ", Style::new().fg(pal.muted).bg(bg))).right_aligned(),
        );
        state.hits.push((rect, Hit::Tool(i)));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn wrap_respects_width_and_line_count() {
        assert_eq!(
            super::wrap("Change colors and style", 14, 2),
            vec!["Change colors", "and style"]
        );
        let l = super::wrap("a b c d e f g h", 3, 2);
        assert_eq!(l.len(), 2);
        assert!(l.iter().all(|s| super::width(s) <= 3));
    }
}
