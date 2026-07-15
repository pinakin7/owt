//! Market detail screen: state header + `[book | tape | news]` columns + timeline
//! strip (`docs/product/prd-mvp.md` F2 wireframe). Columns collapse to a single
//! focused pane at the narrow breakpoint.

use owt_api_types::dto::{BookDto, MarketDetail, NewsDto, Side, TradeDto};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState, Paragraph};

use crate::app::Breakpoint;
use crate::fmt::{cents, hms, usd_compact};
use crate::screens::{OpenResult, select_down, select_up};
use crate::theme::Theme;
use crate::widgets::pane_block;

/// The three focusable panes.
const PANES: [&str; 3] = ["book", "tape", "news"];

/// Market detail screen state.
#[derive(Debug, Default)]
pub struct MarketDetailState {
    /// The market slug being shown.
    pub slug: String,
    /// State header.
    pub detail: Option<MarketDetail>,
    /// Order book.
    pub book: Option<BookDto>,
    /// Trade tape, newest first.
    pub trades: Vec<TradeDto>,
    /// Linked news.
    pub news: Vec<NewsDto>,
    /// Focused pane index (0 book, 1 tape, 2 news).
    pub focus: usize,
    /// Selection within the news pane.
    pub news_selected: usize,
}

impl MarketDetailState {
    /// Populate from a loaded bundle.
    pub fn set(
        &mut self,
        detail: MarketDetail,
        book: BookDto,
        trades: Vec<TradeDto>,
        news: Vec<NewsDto>,
    ) {
        self.slug = detail.market_id.to_string();
        self.detail = Some(detail);
        self.book = Some(book);
        self.trades = trades;
        self.news = news;
        self.news_selected = 0;
    }

    /// Cycle focus forward.
    pub fn focus_next(&mut self) {
        self.focus = (self.focus + 1) % PANES.len();
    }

    /// Cycle focus backward.
    pub fn focus_prev(&mut self) {
        self.focus = (self.focus + PANES.len() - 1) % PANES.len();
    }

    /// Move selection up (news pane only).
    pub fn up(&mut self) {
        if self.focus == 2 {
            self.news_selected = select_up(self.news_selected);
        }
    }

    /// Move selection down (news pane only).
    pub fn down(&mut self) {
        if self.focus == 2 {
            self.news_selected = select_down(self.news_selected, self.news.len());
        }
    }

    /// Jump to top of the news pane.
    pub fn top(&mut self) {
        if self.focus == 2 {
            self.news_selected = 0;
        }
    }

    /// Jump to bottom of the news pane.
    pub fn bottom(&mut self) {
        if self.focus == 2 {
            self.news_selected = self.news.len().saturating_sub(1);
        }
    }

    /// Open the current selection (news → open article).
    pub fn open(&self) -> OpenResult {
        if self.focus == 2
            && let Some(n) = self.news.get(self.news_selected)
        {
            return OpenResult::Toast(format!("open article: {}", n.headline));
        }
        OpenResult::None
    }
}

/// Render the market detail screen.
pub fn render(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &MarketDetailState,
    theme: &Theme,
    breakpoint: Breakpoint,
) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // header strip (price line + event/tags line)
            Constraint::Min(3),    // panes
            Constraint::Length(1), // timeline strip
        ])
        .split(area);

    render_header(frame, rows[0], state, theme);
    render_panes(frame, rows[1], state, theme, breakpoint);
    render_timeline_strip(frame, rows[2], state, theme);
}

fn render_header(frame: &mut Frame<'_>, area: Rect, state: &MarketDetailState, theme: &Theme) {
    let block = pane_block("market", false, theme);
    let lines = match &state.detail {
        Some(d) => {
            let yes = d.outcomes.first().map(|o| o.price).unwrap_or(d.last);
            vec![
                Line::from(vec![
                    Span::styled(
                        format!("{} YES", cents(yes)),
                        Style::default().fg(theme.good).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("   Spread {}", cents(d.spread)),
                        Style::default().fg(theme.fg),
                    ),
                    Span::styled(
                        format!("   Vol 24h {}", usd_compact(d.vol_24h)),
                        Style::default().fg(theme.fg),
                    ),
                    Span::styled(
                        format!("   Liquidity {}", d.liquidity),
                        Style::default().fg(theme.fg),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        format!("Event: {}   ", d.event_id),
                        Style::default().fg(theme.dim),
                    ),
                    Span::styled(
                        format!("Tags: {}", d.tags.join(" ")),
                        Style::default().fg(theme.dim),
                    ),
                ]),
            ]
        }
        None => vec![Line::from(Span::styled(
            "loading…",
            Style::default().fg(theme.dim),
        ))],
    };
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_panes(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &MarketDetailState,
    theme: &Theme,
    breakpoint: Breakpoint,
) {
    // Narrow / too-small: collapse the three columns to the single focused pane.
    if matches!(breakpoint, Breakpoint::Narrow | Breakpoint::TooSmall) {
        match state.focus {
            0 => render_book(frame, area, state, theme),
            1 => render_tape(frame, area, state, theme),
            _ => render_news(frame, area, state, theme),
        }
        return;
    }

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(35),
            Constraint::Percentage(35),
        ])
        .split(area);
    render_book(frame, cols[0], state, theme);
    render_tape(frame, cols[1], state, theme);
    render_news(frame, cols[2], state, theme);
}

fn render_book(frame: &mut Frame<'_>, area: Rect, state: &MarketDetailState, theme: &Theme) {
    let block = pane_block("Order Book", state.focus == 0, theme);
    let mut lines = Vec::new();
    if let Some(book) = &state.book {
        for lvl in &book.asks {
            lines.push(Line::from(Span::styled(
                format!("ask {:>5}  {:>6.0}", cents(lvl.price), lvl.size),
                Style::default().fg(theme.bad),
            )));
        }
        lines.push(Line::from(Span::styled(
            format!("mid {:>5}", cents(book.mid)),
            Style::default().fg(theme.dim).add_modifier(Modifier::BOLD),
        )));
        for lvl in &book.bids {
            lines.push(Line::from(Span::styled(
                format!("bid {:>5}  {:>6.0}", cents(lvl.price), lvl.size),
                Style::default().fg(theme.good),
            )));
        }
    }
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn render_tape(frame: &mut Frame<'_>, area: Rect, state: &MarketDetailState, theme: &Theme) {
    let block = pane_block("Trade Tape", state.focus == 1, theme);
    let items: Vec<ListItem<'_>> = state
        .trades
        .iter()
        .map(|t| {
            let (label, color) = match t.side {
                Side::Buy => ("BUY ", theme.good),
                Side::Sell => ("SELL", theme.bad),
                _ => ("    ", theme.dim),
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{}  ", hms(t.ts)), Style::default().fg(theme.dim)),
                Span::styled(format!("{label} "), Style::default().fg(color)),
                Span::styled(
                    format!("{:>5}  {:>4.0}", cents(t.price), t.size),
                    Style::default().fg(theme.fg),
                ),
            ]))
        })
        .collect();
    frame.render_widget(List::new(items).block(block), area);
}

fn render_news(frame: &mut Frame<'_>, area: Rect, state: &MarketDetailState, theme: &Theme) {
    let focused = state.focus == 2;
    let block = pane_block("Linked News", focused, theme);
    let items: Vec<ListItem<'_>> = state
        .news
        .iter()
        .map(|n| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{}: ", n.publisher),
                    Style::default().fg(theme.accent),
                ),
                Span::styled(n.headline.clone(), Style::default().fg(theme.fg)),
            ]))
        })
        .collect();
    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ");
    let mut ls = ListState::default();
    if focused && !state.news.is_empty() {
        ls.select(Some(state.news_selected));
    }
    frame.render_stateful_widget(list, area, &mut ls);
}

fn render_timeline_strip(
    frame: &mut Frame<'_>,
    area: Rect,
    state: &MarketDetailState,
    theme: &Theme,
) {
    let text = if state.detail.is_some() {
        "Timeline: Powell speech → +3.0¢ in 18m → 12 linked articles  (open the event for detail)"
    } else {
        ""
    };
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            text,
            Style::default().fg(theme.dim),
        ))),
        area,
    );
}
