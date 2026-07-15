//! Search / browse screen: results list + preview pane
//! (`docs/product/prd-mvp.md` § Screen inventory 1).

use owt_api_types::dto::{HitKind, SearchHit};
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState, Paragraph, Wrap};

use crate::app::route::Route;
use crate::screens::{OpenResult, select_down, select_up};
use crate::theme::Theme;
use crate::widgets::pane_block;

/// Search screen state.
#[derive(Debug, Default)]
pub struct SearchState {
    /// The current query text (mirrors the palette / search input).
    pub query: String,
    /// Result hits.
    pub hits: Vec<SearchHit>,
    /// Selected row.
    pub selected: usize,
}

impl SearchState {
    /// Replace results, clamping the selection.
    pub fn set_hits(&mut self, hits: Vec<SearchHit>) {
        self.hits = hits;
        self.selected = self.selected.min(self.hits.len().saturating_sub(1));
    }

    /// Move selection up.
    pub fn up(&mut self) {
        self.selected = select_up(self.selected);
    }

    /// Move selection down.
    pub fn down(&mut self) {
        self.selected = select_down(self.selected, self.hits.len());
    }

    /// Jump to top.
    pub fn top(&mut self) {
        self.selected = 0;
    }

    /// Jump to bottom.
    pub fn bottom(&mut self) {
        self.selected = self.hits.len().saturating_sub(1);
    }

    /// Open the selected hit.
    pub fn open(&self) -> OpenResult {
        let Some(hit) = self.hits.get(self.selected) else {
            return OpenResult::None;
        };
        match hit.kind {
            HitKind::Market => OpenResult::Route(Route::MarketDetail {
                slug: hit.slug.clone(),
            }),
            HitKind::Event => OpenResult::Route(Route::EventWorkspace {
                slug: hit.slug.clone(),
            }),
            HitKind::Entity => OpenResult::Route(Route::TopicView {
                id: hit.slug.clone(),
            }),
            HitKind::News => OpenResult::Toast(format!("open article: {}", hit.title)),
            _ => OpenResult::None,
        }
    }
}

/// Render the search screen.
pub fn render(frame: &mut Frame<'_>, area: Rect, state: &SearchState, theme: &Theme) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    let items: Vec<ListItem<'_>> = state
        .hits
        .iter()
        .map(|h| {
            let tag = kind_tag(h.kind);
            ListItem::new(Line::from(vec![
                Span::styled(format!("{tag:<7}"), Style::default().fg(theme.dim)),
                Span::styled(h.title.clone(), Style::default().fg(theme.fg)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(pane_block("results", true, theme))
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ");
    let mut ls = ListState::default();
    if !state.hits.is_empty() {
        ls.select(Some(state.selected));
    }
    frame.render_stateful_widget(list, cols[0], &mut ls);

    render_preview(frame, cols[1], state, theme);
}

fn render_preview(frame: &mut Frame<'_>, area: Rect, state: &SearchState, theme: &Theme) {
    let block = pane_block("preview", false, theme);
    let lines = match state.hits.get(state.selected) {
        Some(h) => vec![
            Line::from(Span::styled(
                h.title.clone(),
                Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                format!("{} · {}", kind_tag(h.kind), h.slug),
                Style::default().fg(theme.dim),
            )),
            Line::raw(""),
            Line::from(Span::styled(
                h.snippet.clone().unwrap_or_default(),
                Style::default().fg(theme.fg),
            )),
            Line::raw(""),
            Line::from(Span::styled(
                "Enter to open",
                Style::default().fg(theme.accent),
            )),
        ],
        None => vec![Line::from(Span::styled(
            "no results — try loosening filters",
            Style::default().fg(theme.dim),
        ))],
    };
    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: true }),
        area,
    );
}

fn kind_tag(kind: HitKind) -> &'static str {
    match kind {
        HitKind::Market => "market",
        HitKind::Event => "event",
        HitKind::News => "news",
        HitKind::Entity => "topic",
        _ => "?",
    }
}
