//! Topic (entity) page: entity header + `[odds board | linked news | related]` +
//! forecast strip placeholder (`docs/design/tui-client.md` § Screens; ADR-0011/0012).
//! The forecast pane is v1 (F14) — shown here as a disclosed placeholder.

use owt_api_types::EntityKind;
use owt_api_types::dto::EntityView;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState, Paragraph, Wrap};

use crate::app::route::Route;
use crate::fmt::cents;
use crate::screens::{OpenResult, select_down, select_up};
use crate::theme::Theme;
use crate::widgets::pane_block;

/// Topic view state.
#[derive(Debug, Default)]
pub struct TopicViewState {
    /// Entity id / slug.
    pub id: String,
    /// Loaded entity composite.
    pub entity: Option<EntityView>,
    /// Focused pane (0 odds, 1 news, 2 related).
    pub focus: usize,
    /// Selection within the focused list.
    pub selected: usize,
}

impl TopicViewState {
    /// Populate from a loaded entity.
    pub fn set(&mut self, entity: EntityView) {
        self.id = entity.entity_id.to_string();
        self.entity = Some(entity);
        self.selected = 0;
    }

    fn focused_len(&self) -> usize {
        let Some(e) = &self.entity else { return 0 };
        match self.focus {
            0 => e.odds.len(),
            1 => e.news.len(),
            _ => e.related.len(),
        }
    }

    /// Cycle focus forward.
    pub fn focus_next(&mut self) {
        self.focus = (self.focus + 1) % 3;
        self.selected = 0;
    }

    /// Cycle focus backward.
    pub fn focus_prev(&mut self) {
        self.focus = (self.focus + 2) % 3;
        self.selected = 0;
    }

    /// Move selection up.
    pub fn up(&mut self) {
        self.selected = select_up(self.selected);
    }

    /// Move selection down.
    pub fn down(&mut self) {
        self.selected = select_down(self.selected, self.focused_len());
    }

    /// Jump to top.
    pub fn top(&mut self) {
        self.selected = 0;
    }

    /// Jump to bottom.
    pub fn bottom(&mut self) {
        self.selected = self.focused_len().saturating_sub(1);
    }

    /// Open the current selection.
    pub fn open(&self) -> OpenResult {
        let Some(e) = &self.entity else {
            return OpenResult::None;
        };
        match self.focus {
            0 => match e.odds.get(self.selected) {
                Some(o) => OpenResult::Route(Route::MarketDetail {
                    slug: o.market_id.to_string(),
                }),
                None => OpenResult::None,
            },
            1 => match e.news.get(self.selected) {
                Some(n) => OpenResult::Toast(format!("open article: {}", n.headline)),
                None => OpenResult::None,
            },
            _ => match e.related.get(self.selected) {
                Some(r) => OpenResult::Route(Route::TopicView {
                    id: r.entity_id.to_string(),
                }),
                None => OpenResult::None,
            },
        }
    }
}

/// Render the topic page.
pub fn render(frame: &mut Frame<'_>, area: Rect, state: &TopicViewState, theme: &Theme) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // entity header
            Constraint::Min(3),    // three columns
            Constraint::Length(1), // forecast strip (v1 placeholder)
        ])
        .split(area);

    render_header(frame, rows[0], state, theme);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(rows[1]);
    render_odds(frame, cols[0], state, theme);
    render_news(frame, cols[1], state, theme);
    render_related(frame, cols[2], state, theme);

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "Forecast: arrives in v1 — statistical model only, always disclosed (ADR-0012)",
            Style::default().fg(theme.dim),
        ))),
        rows[2],
    );
}

fn render_header(frame: &mut Frame<'_>, area: Rect, state: &TopicViewState, theme: &Theme) {
    let lines = match &state.entity {
        Some(e) => vec![
            Line::from(vec![
                Span::styled(
                    e.name.clone(),
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("   {}", kind_label(e.kind)),
                    Style::default().fg(theme.dim),
                ),
            ]),
            Line::from(Span::styled(
                e.description.clone().unwrap_or_default(),
                Style::default().fg(theme.fg),
            )),
        ],
        None => vec![Line::from(Span::styled(
            "loading…",
            Style::default().fg(theme.dim),
        ))],
    };
    frame.render_widget(
        Paragraph::new(lines)
            .block(pane_block("topic", false, theme))
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_odds(frame: &mut Frame<'_>, area: Rect, state: &TopicViewState, theme: &Theme) {
    let focused = state.focus == 0;
    let items: Vec<ListItem<'_>> = state
        .entity
        .as_ref()
        .map(|e| {
            e.odds
                .iter()
                .map(|o| {
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("{:>6}  ", cents(o.probability)),
                            Style::default().fg(theme.good),
                        ),
                        Span::styled(o.label.clone(), Style::default().fg(theme.fg)),
                    ]))
                })
                .collect()
        })
        .unwrap_or_default();
    render_selectable(
        frame,
        area,
        items,
        "Odds board",
        focused,
        state.selected,
        theme,
    );
}

fn render_news(frame: &mut Frame<'_>, area: Rect, state: &TopicViewState, theme: &Theme) {
    let focused = state.focus == 1;
    let items: Vec<ListItem<'_>> = state
        .entity
        .as_ref()
        .map(|e| {
            e.news
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
                .collect()
        })
        .unwrap_or_default();
    render_selectable(
        frame,
        area,
        items,
        "Linked news",
        focused,
        state.selected,
        theme,
    );
}

fn render_related(frame: &mut Frame<'_>, area: Rect, state: &TopicViewState, theme: &Theme) {
    let focused = state.focus == 2;
    let items: Vec<ListItem<'_>> = state
        .entity
        .as_ref()
        .map(|e| {
            e.related
                .iter()
                .map(|r| {
                    ListItem::new(Line::from(Span::styled(
                        r.name.clone(),
                        Style::default().fg(theme.fg),
                    )))
                })
                .collect()
        })
        .unwrap_or_default();
    render_selectable(
        frame,
        area,
        items,
        "Related",
        focused,
        state.selected,
        theme,
    );
}

fn render_selectable(
    frame: &mut Frame<'_>,
    area: Rect,
    items: Vec<ListItem<'_>>,
    title: &str,
    focused: bool,
    selected: usize,
    theme: &Theme,
) {
    let has_items = !items.is_empty();
    let list = List::new(items)
        .block(pane_block(title, focused, theme))
        .highlight_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ");
    let mut ls = ListState::default();
    if focused && has_items {
        ls.select(Some(selected));
    }
    frame.render_stateful_widget(list, area, &mut ls);
}

fn kind_label(kind: EntityKind) -> &'static str {
    match kind {
        EntityKind::Person => "Person",
        EntityKind::Org => "Org",
        EntityKind::Topic => "Topic",
        EntityKind::Place => "Place",
        _ => "Entity",
    }
}
