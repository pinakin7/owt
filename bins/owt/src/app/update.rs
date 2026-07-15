//! The pure state transition: `update(&mut AppState, Msg) -> Vec<Cmd>`.
//!
//! No I/O happens here — key handling, navigation, palette dispatch, and screen
//! mutation are all pure functions of the current state and the incoming message.
//! The runtime performs the returned `Cmd`s and feeds results back as `Msg`s.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::AppState;
use crate::app::cmd::Cmd;
use crate::app::msg::{Loaded, Msg};
use crate::app::route::Route;
use crate::input::Action;
use crate::palette::{Arg, Command, Verb};
use crate::screens::OpenResult;

/// A movement within the focused screen's active list.
#[derive(Debug, Clone, Copy)]
enum Move {
    Up,
    Down,
    Top,
    Bottom,
}

/// Apply a message, returning side effects for the runtime.
pub fn update(state: &mut AppState, msg: Msg) -> Vec<Cmd> {
    match msg {
        Msg::Resize(w, h) => {
            state.size = (w, h);
            state.dirty = true;
            Vec::new()
        }
        Msg::Tick => {
            // Draw-on-tick keeps live panes fresh without redrawing per message.
            state.dirty = true;
            Vec::new()
        }
        Msg::Conn(conn) => {
            state.conn = conn;
            state.dirty = true;
            Vec::new()
        }
        Msg::Failed { what, error } => {
            state.error(format!("{what}: {error}"));
            state.dirty = true;
            Vec::new()
        }
        Msg::Loaded(loaded) => {
            apply_loaded(state, loaded);
            state.dirty = true;
            Vec::new()
        }
        Msg::Key(key) => on_key(state, key),
    }
}

fn apply_loaded(state: &mut AppState, loaded: Loaded) {
    match loaded {
        Loaded::Search { hits } => state.screens.search.set_hits(hits),
        Loaded::Market {
            detail,
            book,
            trades,
            news,
        } => state.screens.market_detail.set(*detail, book, trades, news),
        Loaded::Event(event) => state.screens.event_workspace.set(*event),
        Loaded::Entity(entity) => state.screens.topic_view.set(*entity),
        Loaded::Watchlists(lists) => state.screens.watchlists.set(lists),
    }
}

fn on_key(state: &mut AppState, key: KeyEvent) -> Vec<Cmd> {
    state.dirty = true;

    // Ctrl-C always quits, regardless of mode.
    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        state.should_quit = true;
        return vec![Cmd::Quit];
    }

    if state.palette.open {
        return on_palette_key(state, key);
    }

    if state.help_open {
        match key.code {
            KeyCode::Esc | KeyCode::Char('?') => state.help_open = false,
            KeyCode::Char('q') => {
                state.should_quit = true;
                return vec![Cmd::Quit];
            }
            _ => {}
        }
        return Vec::new();
    }

    match state.keymap.resolve(&key) {
        Some(action) => apply_action(state, action),
        None => {
            state.pending_g = false;
            Vec::new()
        }
    }
}

fn on_palette_key(state: &mut AppState, key: KeyEvent) -> Vec<Cmd> {
    match key.code {
        KeyCode::Esc => {
            state.palette.close();
            Vec::new()
        }
        KeyCode::Enter => match state.palette.submit() {
            Ok(command) => {
                let cmds = dispatch_command(state, command);
                state.palette.close();
                cmds
            }
            // Parse error is recorded on the palette and rendered inline; stay open.
            Err(_) => Vec::new(),
        },
        KeyCode::Backspace => {
            state.palette.backspace();
            Vec::new()
        }
        KeyCode::Char(c) => {
            state.palette.push(c);
            Vec::new()
        }
        _ => Vec::new(),
    }
}

fn apply_action(state: &mut AppState, action: Action) -> Vec<Cmd> {
    // The `gg` chord: a lone `g` arms; a second `g` jumps to top.
    if action == Action::GPrefix {
        if state.pending_g {
            state.pending_g = false;
            movement(state, Move::Top);
        } else {
            state.pending_g = true;
        }
        return Vec::new();
    }
    state.pending_g = false;

    match action {
        Action::OpenPalette(prefix) => {
            state.palette.open(prefix);
            Vec::new()
        }
        Action::Help => {
            state.help_open = true;
            Vec::new()
        }
        Action::Quit => {
            state.should_quit = true;
            vec![Cmd::Quit]
        }
        Action::Back => back(state),
        Action::FocusNext | Action::Right => {
            screen_focus(state, true);
            Vec::new()
        }
        Action::FocusPrev | Action::Left => {
            screen_focus(state, false);
            Vec::new()
        }
        Action::Up => {
            movement(state, Move::Up);
            Vec::new()
        }
        Action::Down => {
            movement(state, Move::Down);
            Vec::new()
        }
        Action::Top => {
            movement(state, Move::Top);
            Vec::new()
        }
        Action::Bottom => {
            movement(state, Move::Bottom);
            Vec::new()
        }
        Action::Open => on_open(state),
        Action::ToggleWatch => {
            state.info("watch toggled (local; syncs when the server is live)");
            Vec::new()
        }
        Action::Jump(n) => jump_watchlist(state, n),
        Action::GPrefix => unreachable!("handled above"),
    }
}

fn movement(state: &mut AppState, mv: Move) {
    match state.route().clone() {
        Route::Search => {
            let s = &mut state.screens.search;
            match mv {
                Move::Up => s.up(),
                Move::Down => s.down(),
                Move::Top => s.top(),
                Move::Bottom => s.bottom(),
            }
        }
        Route::MarketDetail { .. } => {
            let s = &mut state.screens.market_detail;
            match mv {
                Move::Up => s.up(),
                Move::Down => s.down(),
                Move::Top => s.top(),
                Move::Bottom => s.bottom(),
            }
        }
        Route::EventWorkspace { .. } => {
            let s = &mut state.screens.event_workspace;
            match mv {
                Move::Up => s.up(),
                Move::Down => s.down(),
                Move::Top => s.top(),
                Move::Bottom => s.bottom(),
            }
        }
        Route::TopicView { .. } => {
            let s = &mut state.screens.topic_view;
            match mv {
                Move::Up => s.up(),
                Move::Down => s.down(),
                Move::Top => s.top(),
                Move::Bottom => s.bottom(),
            }
        }
        Route::Watchlists => {
            let s = &mut state.screens.watchlists;
            match mv {
                Move::Up => s.up(),
                Move::Down => s.down(),
                Move::Top => s.top(),
                Move::Bottom => s.bottom(),
            }
        }
    }
}

fn screen_focus(state: &mut AppState, forward: bool) {
    match state.route().clone() {
        Route::MarketDetail { .. } => {
            let s = &mut state.screens.market_detail;
            if forward {
                s.focus_next();
            } else {
                s.focus_prev();
            }
        }
        Route::EventWorkspace { .. } => {
            let s = &mut state.screens.event_workspace;
            if forward {
                s.focus_next();
            } else {
                s.focus_prev();
            }
        }
        Route::TopicView { .. } => {
            let s = &mut state.screens.topic_view;
            if forward {
                s.focus_next();
            } else {
                s.focus_prev();
            }
        }
        // Single-pane screens: focus cycling is a no-op.
        Route::Search | Route::Watchlists => {}
    }
}

fn on_open(state: &mut AppState) -> Vec<Cmd> {
    let result = match state.route().clone() {
        Route::Search => state.screens.search.open(),
        Route::MarketDetail { .. } => state.screens.market_detail.open(),
        Route::EventWorkspace { .. } => state.screens.event_workspace.open(),
        Route::TopicView { .. } => state.screens.topic_view.open(),
        Route::Watchlists => state.screens.watchlists.open(),
    };
    match result {
        OpenResult::Route(route) => open_route(state, route),
        OpenResult::Toast(text) => {
            state.info(text);
            Vec::new()
        }
        OpenResult::None => Vec::new(),
    }
}

/// Pop the current route, unsubscribing any live topics it held. Returns the unsubscribe
/// commands (empty when already at the root or the screen had no subscriptions).
fn back(state: &mut AppState) -> Vec<Cmd> {
    let leaving = state.route().clone();
    if state.pop_route() {
        route_topics(&leaving)
            .into_iter()
            .map(Cmd::Unsubscribe)
            .collect()
    } else {
        Vec::new()
    }
}

fn jump_watchlist(state: &mut AppState, n: u8) -> Vec<Cmd> {
    let need_load = state.screens.watchlists.lists.is_empty();
    if !matches!(state.route(), Route::Watchlists) {
        state.push_route(Route::Watchlists);
    }
    state.screens.watchlists.jump(n);
    if need_load {
        vec![Cmd::Load(Route::Watchlists)]
    } else {
        Vec::new()
    }
}

fn dispatch_command(state: &mut AppState, command: Command) -> Vec<Cmd> {
    match command.verb {
        Verb::Open => open_route(
            state,
            Route::MarketDetail {
                slug: match first_value(&command.args) {
                    Some(s) => s,
                    None => {
                        state.error("open needs a market or event slug");
                        return Vec::new();
                    }
                },
            },
        ),
        Verb::Topic => open_route(
            state,
            Route::TopicView {
                id: match first_value(&command.args) {
                    Some(s) => s,
                    None => {
                        state.error("topic needs a query");
                        return Vec::new();
                    }
                },
            },
        ),
        Verb::News => open_route(state, Route::Search),
        Verb::Watch => {
            state.info("watch toggled (local; syncs when the server is live)");
            Vec::new()
        }
        Verb::Unwatch => {
            state.info("unwatched (local)");
            Vec::new()
        }
        Verb::Pin => {
            state.info("pinned (local)");
            Vec::new()
        }
        Verb::Compare => {
            state.info("compare: side-by-side panes arrive with live data");
            Vec::new()
        }
        Verb::View => {
            state.info("saved views persist server-side (arrives with live data)");
            Vec::new()
        }
        Verb::Export => {
            let (format, path) = export_args(&command.args);
            vec![Cmd::Export { format, path }]
        }
        Verb::Alert => {
            state.info("alerts arrive in v1");
            Vec::new()
        }
        Verb::Help => {
            state.help_open = true;
            Vec::new()
        }
        Verb::Quit => {
            state.should_quit = true;
            vec![Cmd::Quit]
        }
    }
}

fn open_route(state: &mut AppState, route: Route) -> Vec<Cmd> {
    state.push_route(route.clone());
    let mut cmds = vec![Cmd::Load(route.clone())];
    cmds.extend(route_topics(&route).into_iter().map(Cmd::Subscribe));
    cmds
}

/// The live WS topics a route wants subscribed while it is on screen. Only markets carry
/// live subscriptions today (state/book/tape); other screens are REST-only.
fn route_topics(route: &Route) -> Vec<String> {
    match route {
        Route::MarketDetail { slug } => vec![
            format!("market:{slug}:state"),
            format!("market:{slug}:book"),
            format!("market:{slug}:trades"),
        ],
        _ => Vec::new(),
    }
}

/// The first ident/quoted argument's value, if any.
fn first_value(args: &[Arg]) -> Option<String> {
    args.iter().find_map(|a| match a {
        Arg::Ident(s) | Arg::Quoted(s) => Some(s.clone()),
        Arg::Filter { .. } => None,
    })
}

/// Parse `/export [json|csv] [path]` into `(format, path)`.
fn export_args(args: &[Arg]) -> (String, String) {
    let mut format = "json".to_owned();
    let mut path = String::new();
    for arg in args {
        if let Arg::Ident(s) | Arg::Quoted(s) = arg {
            if s == "json" || s == "csv" {
                format = s.clone();
            } else {
                path = s.clone();
            }
        }
    }
    (format, path)
}
