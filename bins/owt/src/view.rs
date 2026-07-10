//! The top-level `view`: header + active screen + status bar, with the palette, help,
//! and toast overlays on top. Layout recomputes from the route + breakpoint every
//! frame — there are no absolute coordinates in state, so a resize can never corrupt
//! the display (`docs/design/tui-client.md` § Resize/degradation).

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::app::{AppState, Breakpoint, Route};
use crate::screens;
use crate::widgets;

/// Render the whole UI from `state`. Pure over `AppState`.
pub fn view(frame: &mut Frame<'_>, state: &AppState) {
    let area = frame.area();
    let theme = &state.theme;

    if state.breakpoint() == Breakpoint::TooSmall {
        let msg = vec![
            Line::from(Span::styled(
                "terminal too small",
                Style::default().fg(theme.warn).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                format!(
                    "{}×{} — owt needs at least 80×24",
                    state.size.0, state.size.1
                ),
                Style::default().fg(theme.dim),
            )),
        ];
        frame.render_widget(Paragraph::new(msg).alignment(Alignment::Center), area);
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // header
            Constraint::Min(3),    // content
            Constraint::Length(1), // status
        ])
        .split(area);

    widgets::header_bar(frame, rows[0], state, theme);
    render_content(frame, rows[1], state);
    widgets::status_bar(frame, rows[2], state, theme);

    if state.palette.open {
        widgets::palette_overlay(frame, area, &state.palette, theme);
    }
    if state.help_open {
        widgets::help_overlay(frame, area, theme);
    }
    widgets::render_toasts(frame, rows[1], &state.toasts, theme);
}

fn render_content(frame: &mut Frame<'_>, area: ratatui::layout::Rect, state: &AppState) {
    let theme = &state.theme;
    let breakpoint = state.breakpoint();
    match state.route() {
        Route::Search => screens::search::render(frame, area, &state.screens.search, theme),
        Route::MarketDetail { .. } => screens::market_detail::render(
            frame,
            area,
            &state.screens.market_detail,
            theme,
            breakpoint,
        ),
        Route::EventWorkspace { .. } => {
            screens::event_workspace::render(frame, area, &state.screens.event_workspace, theme)
        }
        Route::TopicView { .. } => {
            screens::topic_view::render(frame, area, &state.screens.topic_view, theme)
        }
        Route::Watchlists => {
            screens::watchlists::render(frame, area, &state.screens.watchlists, theme)
        }
    }
}
