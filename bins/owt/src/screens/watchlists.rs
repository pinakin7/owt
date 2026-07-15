//! Watchlists screen: named lists as tabs + compact live rows
//! (`docs/product/prd-mvp.md` F4).

use owt_api_types::dto::WatchlistView;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Cell, Row, Table, TableState, Tabs};

use crate::app::route::Route;
use crate::fmt::{cents, signed_cents, usd_compact};
use crate::screens::{OpenResult, select_down, select_up};
use crate::theme::Theme;
use crate::widgets::pane_block;

/// Watchlists screen state.
#[derive(Debug, Default)]
pub struct WatchlistsState {
    /// All named lists.
    pub lists: Vec<WatchlistView>,
    /// Active list index.
    pub active: usize,
    /// Selected row within the active list.
    pub selected: usize,
}

impl WatchlistsState {
    /// Replace the lists.
    pub fn set(&mut self, lists: Vec<WatchlistView>) {
        self.lists = lists;
        self.active = self.active.min(self.lists.len().saturating_sub(1));
        self.selected = 0;
    }

    fn active_len(&self) -> usize {
        self.lists.get(self.active).map_or(0, |l| l.rows.len())
    }

    /// Jump to list `n` (1-based); clamps and resets the row selection.
    pub fn jump(&mut self, n: u8) {
        let idx = (n.saturating_sub(1)) as usize;
        if idx < self.lists.len() {
            self.active = idx;
            self.selected = 0;
        }
    }

    /// Move selection up.
    pub fn up(&mut self) {
        self.selected = select_up(self.selected);
    }

    /// Move selection down.
    pub fn down(&mut self) {
        self.selected = select_down(self.selected, self.active_len());
    }

    /// Jump to top.
    pub fn top(&mut self) {
        self.selected = 0;
    }

    /// Jump to bottom.
    pub fn bottom(&mut self) {
        self.selected = self.active_len().saturating_sub(1);
    }

    /// Open the selected row's market.
    pub fn open(&self) -> OpenResult {
        match self
            .lists
            .get(self.active)
            .and_then(|l| l.rows.get(self.selected))
        {
            Some(r) => OpenResult::Route(Route::MarketDetail {
                slug: r.market_id.to_string(),
            }),
            None => OpenResult::None,
        }
    }
}

/// Render the watchlists screen.
pub fn render(frame: &mut Frame<'_>, area: Rect, state: &WatchlistsState, theme: &Theme) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(3)])
        .split(area);

    let titles: Vec<Line<'_>> = state
        .lists
        .iter()
        .enumerate()
        .map(|(i, l)| Line::from(format!(" {}:{} ", i + 1, l.name)))
        .collect();
    let tabs = Tabs::new(titles)
        .select(state.active)
        .style(Style::default().fg(theme.dim))
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(tabs, rows[0]);

    render_table(frame, rows[1], state, theme);
}

fn render_table(frame: &mut Frame<'_>, area: Rect, state: &WatchlistsState, theme: &Theme) {
    let header = Row::new(
        ["Market", "Price", "Δ24h", "Spread", "Volume"]
            .into_iter()
            .map(|h| Cell::from(h).style(Style::default().fg(theme.dim))),
    );

    let data_rows: Vec<Row<'_>> = state
        .lists
        .get(state.active)
        .map(|l| {
            l.rows
                .iter()
                .map(|r| {
                    let chg_color = if r.chg_24h >= 0.0 {
                        theme.good
                    } else {
                        theme.bad
                    };
                    Row::new(vec![
                        Cell::from(r.title.clone()).style(Style::default().fg(theme.fg)),
                        Cell::from(cents(r.price)).style(Style::default().fg(theme.fg)),
                        Cell::from(signed_cents(r.chg_24h)).style(Style::default().fg(chg_color)),
                        Cell::from(cents(r.spread)).style(Style::default().fg(theme.fg)),
                        Cell::from(usd_compact(r.volume)).style(Style::default().fg(theme.fg)),
                    ])
                })
                .collect()
        })
        .unwrap_or_default();

    let widths = [
        Constraint::Percentage(44),
        Constraint::Percentage(14),
        Constraint::Percentage(14),
        Constraint::Percentage(14),
        Constraint::Percentage(14),
    ];
    let table = Table::new(data_rows, widths)
        .header(header)
        .block(pane_block("watchlist", true, theme))
        .row_highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ");
    let mut ts = TableState::default();
    if state.active_len() > 0 {
        ts.select(Some(state.selected));
    }
    frame.render_stateful_widget(table, area, &mut ts);
}
