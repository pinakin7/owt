//! Pure `update()` tests — the bulk of TUI coverage per `docs/design/tui-client.md`
//! § Test plan. No terminal: construct `AppState`, feed `Msg`s, assert state + `Cmd`s.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use owt_client::config::EffectiveConfig;
use owt_tui::app::{AppState, Breakpoint, Cmd, Msg, Route};
use owt_tui::{data, update};

fn app() -> AppState {
    AppState::new(EffectiveConfig::default())
}

fn press(code: KeyCode) -> Msg {
    Msg::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn ch(c: char) -> Msg {
    press(KeyCode::Char(c))
}

#[test]
fn starts_on_search() {
    let s = app();
    assert_eq!(s.route(), &Route::Search);
    assert_eq!(s.route_stack.len(), 1);
}

#[test]
fn colon_opens_palette_and_esc_closes_it() {
    let mut s = app();
    update(&mut s, ch(':'));
    assert!(s.palette.open);
    assert_eq!(s.palette.prefix, ':');
    update(&mut s, press(KeyCode::Esc));
    assert!(!s.palette.open);
}

#[test]
fn slash_also_opens_palette() {
    let mut s = app();
    update(&mut s, ch('/'));
    assert!(s.palette.open);
    assert_eq!(s.palette.prefix, '/');
}

#[test]
fn palette_typing_and_invalid_submit_keeps_open_with_error() {
    let mut s = app();
    update(&mut s, ch('/'));
    for c in "frobnicate".chars() {
        update(&mut s, ch(c));
    }
    assert_eq!(s.palette.buffer, "frobnicate");
    update(&mut s, press(KeyCode::Enter));
    assert!(s.palette.open, "invalid command stays open");
    assert!(s.palette.error.is_some());
}

#[test]
fn palette_open_command_navigates_and_loads() {
    let mut s = app();
    update(&mut s, ch(':'));
    for c in "open will-fed-cut".chars() {
        update(&mut s, ch(c));
    }
    let cmds = update(&mut s, press(KeyCode::Enter));
    assert!(!s.palette.open, "valid command closes the palette");
    assert!(matches!(s.route(), Route::MarketDetail { slug } if slug == "will-fed-cut"));
    // Opening a market loads it and subscribes to its live topics (state/book/trades).
    assert!(matches!(&cmds[0], Cmd::Load(Route::MarketDetail { .. })));
    let subs = cmds
        .iter()
        .filter(|c| matches!(c, Cmd::Subscribe(_)))
        .count();
    assert_eq!(subs, 3, "market open subscribes to its three live topics");
}

#[test]
fn enter_on_search_result_pushes_route_and_esc_pops() {
    let mut s = app();
    // Load fixture results, then open the first (a market) with Enter.
    update(
        &mut s,
        Msg::Loaded(owt_tui::app::Loaded::Search {
            hits: data::search("fed"),
        }),
    );
    let cmds = update(&mut s, press(KeyCode::Enter));
    assert_eq!(s.route_stack.len(), 2);
    assert!(matches!(s.route(), Route::MarketDetail { .. }));
    assert!(matches!(&cmds[0], Cmd::Load(_)));

    // Esc goes back.
    update(&mut s, press(KeyCode::Esc));
    assert_eq!(s.route_stack.len(), 1);
    assert_eq!(s.route(), &Route::Search);
}

#[test]
fn esc_at_root_does_not_pop() {
    let mut s = app();
    update(&mut s, press(KeyCode::Esc));
    assert_eq!(s.route_stack.len(), 1);
}

#[test]
fn q_quits_and_emits_quit_cmd() {
    let mut s = app();
    let cmds = update(&mut s, ch('q'));
    assert!(s.should_quit);
    assert!(cmds.contains(&Cmd::Quit));
}

#[test]
fn ctrl_c_quits_even_with_palette_open() {
    let mut s = app();
    update(&mut s, ch(':'));
    let cmds = update(
        &mut s,
        Msg::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
    );
    assert!(s.should_quit);
    assert!(cmds.contains(&Cmd::Quit));
}

#[test]
fn resize_sets_breakpoint() {
    let mut s = app();
    update(&mut s, Msg::Resize(200, 60));
    assert_eq!(s.breakpoint(), Breakpoint::Wide);
    update(&mut s, Msg::Resize(90, 24));
    assert_eq!(s.breakpoint(), Breakpoint::Narrow);
    update(&mut s, Msg::Resize(70, 20));
    assert_eq!(s.breakpoint(), Breakpoint::TooSmall);
}

#[test]
fn question_mark_toggles_help() {
    let mut s = app();
    update(&mut s, ch('?'));
    assert!(s.help_open);
    update(&mut s, ch('?'));
    assert!(!s.help_open);
}

#[test]
fn gg_jumps_to_top() {
    let mut s = app();
    update(
        &mut s,
        Msg::Loaded(owt_tui::app::Loaded::Search {
            hits: data::search("x"),
        }),
    );
    // move down twice, then gg back to top
    update(&mut s, ch('j'));
    update(&mut s, ch('j'));
    assert_eq!(s.screens.search.selected, 2);
    update(&mut s, ch('g'));
    assert!(s.pending_g);
    update(&mut s, ch('g'));
    assert!(!s.pending_g);
    assert_eq!(s.screens.search.selected, 0);
}

#[test]
fn loaded_market_populates_screen() {
    let mut s = app();
    update(&mut s, Msg::Loaded(data::market("will-fed-cut")));
    assert!(s.screens.market_detail.detail.is_some());
    assert_eq!(s.screens.market_detail.trades.len(), 3);
}

#[test]
fn alert_command_reports_v1() {
    let mut s = app();
    update(&mut s, ch(':'));
    for c in "alert foo".chars() {
        update(&mut s, ch(c));
    }
    update(&mut s, press(KeyCode::Enter));
    assert!(s.toasts.iter().any(|t| t.text.contains("v1")));
}
