//! Shared widgets: the header/status bars, focusable pane frames, and the palette,
//! help, and toast overlays. Screens compose these; nothing here holds state.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};

use crate::app::{AppState, ToastKind};
use crate::palette::PaletteState;
use crate::theme::Theme;

/// A bordered block for a pane, highlighted when focused.
pub fn pane_block(title: &str, focused: bool, theme: &Theme) -> Block<'static> {
    let border = if focused {
        theme.border_focus
    } else {
        theme.border
    };
    let title_style = if focused {
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.dim)
    };
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .title(Span::styled(format!(" {title} "), title_style))
}

/// The top header line: app name + current route, connection health on the right.
pub fn header_bar(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &Theme) {
    let route = state.route().label();
    let left = Line::from(vec![
        Span::styled(
            "owt ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(route, Style::default().fg(theme.fg)),
    ]);

    let conn = state.conn;
    let dot = Span::styled("●", Style::default().fg(theme.conn_color(conn)));
    let live = if conn.is_live() { "LIVE" } else { "STALE" };
    let right = Line::from(vec![
        Span::styled(live, Style::default().fg(theme.conn_color(conn))),
        Span::raw("  "),
        dot,
        Span::styled(
            format!(" ws: {}", conn.label()),
            Style::default().fg(theme.dim),
        ),
    ]);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);
    frame.render_widget(Paragraph::new(left), cols[0]);
    frame.render_widget(Paragraph::new(right).alignment(Alignment::Right), cols[1]);
}

/// The bottom status/hint line.
pub fn status_bar(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &Theme) {
    let hint = "  :cmd  ?help  Tab focus  Enter open  Esc back  q quit";
    let text = Line::from(vec![
        Span::styled(
            format!(" {} ", state.cfg.server.url),
            Style::default().fg(theme.dim),
        ),
        Span::styled(hint, Style::default().fg(theme.dim)),
    ]);
    frame.render_widget(Paragraph::new(text), area);
}

/// The command palette overlay (a single input line near the top).
pub fn palette_overlay(frame: &mut Frame<'_>, area: Rect, palette: &PaletteState, theme: &Theme) {
    let row = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: 3.min(area.height),
    };
    frame.render_widget(Clear, row);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .title(Span::styled(" command ", Style::default().fg(theme.accent)));
    let input = format!("{}{}", palette.prefix, palette.buffer);
    let mut lines = vec![Line::from(Span::styled(
        input,
        Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
    ))];
    if let Some(err) = &palette.error {
        lines.push(Line::from(Span::styled(
            format!("  ✗ {err}"),
            Style::default().fg(theme.bad),
        )));
    }
    frame.render_widget(Paragraph::new(lines).block(block), row);
}

/// The help overlay: keys and commands (`docs/design/tui-client.md` § Keybindings).
pub fn help_overlay(frame: &mut Frame<'_>, area: Rect, theme: &Theme) {
    let popup = centered_rect(70, 80, area);
    frame.render_widget(Clear, popup);
    let items = [
        ":  /       command palette",
        "h j k l    move within pane / lists",
        "Tab        cycle pane focus",
        "Enter      open selection",
        "Esc        close overlay / back",
        "g g / G    top / bottom of list",
        "w          toggle watch on selection",
        "1..9       jump to watchlist n",
        "?          this help",
        "q  Ctrl-C  quit",
        "",
        "commands:  /open /topic /watch /compare /news /export /view /help /quit",
    ];
    let lines: Vec<ListItem<'_>> = items
        .iter()
        .map(|s| ListItem::new(Line::from(Span::styled(*s, Style::default().fg(theme.fg)))))
        .collect();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .title(Span::styled(" help ", Style::default().fg(theme.accent)));
    frame.render_widget(List::new(lines).block(block), popup);
}

/// Render active toasts stacked at the bottom-right.
pub fn render_toasts(
    frame: &mut Frame<'_>,
    area: Rect,
    toasts: &[crate::app::Toast],
    theme: &Theme,
) {
    if toasts.is_empty() {
        return;
    }
    let shown: Vec<&crate::app::Toast> = toasts.iter().rev().take(3).collect();
    let h = (shown.len() as u16 + 2).min(area.height);
    let w = area.width.min(48);
    let rect = Rect {
        x: area.x + area.width.saturating_sub(w),
        y: area.y + area.height.saturating_sub(h),
        width: w,
        height: h,
    };
    frame.render_widget(Clear, rect);
    let lines: Vec<Line<'_>> = shown
        .into_iter()
        .rev()
        .map(|t| {
            let color = match t.kind {
                ToastKind::Info => theme.accent,
                ToastKind::Error => theme.bad,
            };
            Line::from(Span::styled(t.text.clone(), Style::default().fg(color)))
        })
        .collect();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.dim));
    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: true }),
        rect,
    );
}

/// A rectangle centered within `area`, sized as a percentage of it.
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(percent_y)])
        .flex(Flex::Center)
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(percent_x)])
        .flex(Flex::Center)
        .split(vertical[0])[0]
}
