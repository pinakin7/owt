//! Event workspace: query bar + `[market list | live timeline]`
//! (`docs/product/prd-mvp.md` F3 wireframe).

use owt_api_types::TimelineKind;
use owt_api_types::dto::EventDetail;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState, Paragraph};

use crate::app::route::Route;
use crate::fmt::hm;
use crate::screens::{OpenResult, select_down, select_up};
use crate::theme::Theme;
use crate::widgets::pane_block;

/// Event workspace state.
#[derive(Debug, Default)]
pub struct EventWorkspaceState {
    /// Event slug.
    pub slug: String,
    /// Loaded event.
    pub event: Option<EventDetail>,
    /// Focused pane (0 markets, 1 timeline).
    pub focus: usize,
    /// Selected market row.
    pub selected: usize,
}

impl EventWorkspaceState {
    /// Populate from a loaded event.
    pub fn set(&mut self, event: EventDetail) {
        self.slug = event.event_id.to_string();
        self.event = Some(event);
        self.selected = 0;
    }

    fn markets_len(&self) -> usize {
        self.event.as_ref().map_or(0, |e| e.markets.len())
    }

    /// Cycle focus forward.
    pub fn focus_next(&mut self) {
        self.focus = (self.focus + 1) % 2;
    }

    /// Cycle focus backward.
    pub fn focus_prev(&mut self) {
        self.focus = (self.focus + 1) % 2;
    }

    /// Move selection up (market list).
    pub fn up(&mut self) {
        if self.focus == 0 {
            self.selected = select_up(self.selected);
        }
    }

    /// Move selection down (market list).
    pub fn down(&mut self) {
        if self.focus == 0 {
            self.selected = select_down(self.selected, self.markets_len());
        }
    }

    /// Jump to top.
    pub fn top(&mut self) {
        self.selected = 0;
    }

    /// Jump to bottom.
    pub fn bottom(&mut self) {
        self.selected = self.markets_len().saturating_sub(1);
    }

    /// Open the selected member market.
    pub fn open(&self) -> OpenResult {
        match self
            .event
            .as_ref()
            .and_then(|e| e.markets.get(self.selected))
        {
            Some(m) => OpenResult::Route(Route::MarketDetail {
                slug: m.market_id.to_string(),
            }),
            None => OpenResult::None,
        }
    }
}

/// Render the event workspace.
pub fn render(frame: &mut Frame<'_>, area: Rect, state: &EventWorkspaceState, theme: &Theme) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3)])
        .split(area);

    let query = state
        .event
        .as_ref()
        .map(|e| e.title.clone())
        .unwrap_or_else(|| "loading…".to_owned());
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            format!("Query: {query}"),
            Style::default().fg(theme.fg),
        )))
        .block(pane_block("workspace", false, theme)),
        rows[0],
    );

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(rows[1]);

    render_markets(frame, cols[0], state, theme);
    render_timeline(frame, cols[1], state, theme);
}

fn render_markets(frame: &mut Frame<'_>, area: Rect, state: &EventWorkspaceState, theme: &Theme) {
    let focused = state.focus == 0;
    let items: Vec<ListItem<'_>> = state
        .event
        .as_ref()
        .map(|e| {
            e.markets
                .iter()
                .map(|m| {
                    ListItem::new(Line::from(Span::styled(
                        m.title.clone(),
                        Style::default().fg(theme.fg),
                    )))
                })
                .collect()
        })
        .unwrap_or_default();
    let list = List::new(items)
        .block(pane_block("Markets", focused, theme))
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ");
    let mut ls = ListState::default();
    if focused && state.markets_len() > 0 {
        ls.select(Some(state.selected));
    }
    frame.render_stateful_widget(list, area, &mut ls);
}

fn render_timeline(frame: &mut Frame<'_>, area: Rect, state: &EventWorkspaceState, theme: &Theme) {
    let items: Vec<ListItem<'_>> = state
        .event
        .as_ref()
        .map(|e| {
            e.timeline
                .iter()
                .map(|t| {
                    let marker = kind_marker(t.kind);
                    ListItem::new(Line::from(vec![
                        Span::styled(format!("{} ", hm(t.ts)), Style::default().fg(theme.dim)),
                        Span::styled(format!("{marker} "), Style::default().fg(theme.accent)),
                        Span::styled(t.title.clone(), Style::default().fg(theme.fg)),
                    ]))
                })
                .collect()
        })
        .unwrap_or_default();
    frame.render_widget(
        List::new(items).block(pane_block("Event timeline", state.focus == 1, theme)),
        area,
    );
}

fn kind_marker(kind: TimelineKind) -> &'static str {
    match kind {
        TimelineKind::PriceMove => "Δ",
        TimelineKind::News => "▪",
        TimelineKind::TradeBurst => "≈",
        TimelineKind::CommentSpike => "✦",
        TimelineKind::Resolution => "✔",
        _ => "•",
    }
}
