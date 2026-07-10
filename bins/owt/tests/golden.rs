//! Golden frames (`docs/design/tui-client.md` § Test plan step 2; the per-PR TUI E2E
//! gate in `docs/ops/testing-strategy.md`). Each screen is rendered from fixture state
//! via `ratatui::TestBackend` at three sizes and snapshotted with `insta`. Snapshots
//! capture the cell text (layout), which is deterministic given the fixtures.

use insta::assert_snapshot;
use owt_client::config::EffectiveConfig;
use owt_tui::app::{AppState, Msg, Route};
use owt_tui::{data, update, view};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;

/// Prepare an app parked on `route` with its fixtures loaded.
fn prepared(route: Route) -> AppState {
    let mut state = AppState::new(EffectiveConfig::default());
    state.route_stack = vec![route.clone()];
    update(&mut state, Msg::Loaded(data::load(&route)));
    state
}

/// Render a prepared state at `w`×`h` into a plain-text frame.
fn frame(state: &mut AppState, w: u16, h: u16) -> String {
    state.size = (w, h);
    let mut term = Terminal::new(TestBackend::new(w, h)).expect("test terminal");
    term.draw(|f| view(f, state)).expect("draw");
    buffer_to_string(term.backend().buffer())
}

fn buffer_to_string(buf: &Buffer) -> String {
    let area = buf.area;
    let mut out = String::with_capacity((area.width as usize + 1) * area.height as usize);
    for y in 0..area.height {
        for x in 0..area.width {
            if let Some(cell) = buf.cell((x, y)) {
                out.push_str(cell.symbol());
            }
        }
        // Trim trailing spaces so snapshots aren't padding-sensitive.
        while out.ends_with(' ') {
            out.pop();
        }
        out.push('\n');
    }
    out
}

macro_rules! golden_screen {
    ($test:ident, $route:expr, $name:literal) => {
        #[test]
        fn $test() {
            let mut s = prepared($route);
            assert_snapshot!(concat!($name, "_200x60"), frame(&mut s, 200, 60));
            assert_snapshot!(concat!($name, "_120x40"), frame(&mut s, 120, 40));
            assert_snapshot!(concat!($name, "_80x24"), frame(&mut s, 80, 24));
        }
    };
}

golden_screen!(search, Route::Search, "search");
golden_screen!(
    market_detail,
    Route::MarketDetail {
        slug: "will-fed-cut-rates-in-september".to_owned()
    },
    "market_detail"
);
golden_screen!(
    event_workspace,
    Route::EventWorkspace {
        slug: "federal-reserve-september-meeting".to_owned()
    },
    "event_workspace"
);
golden_screen!(
    topic_view,
    Route::TopicView {
        id: "ent-jerome-powell".to_owned()
    },
    "topic_view"
);
golden_screen!(watchlists, Route::Watchlists, "watchlists");

#[test]
fn help_overlay() {
    let mut s = prepared(Route::Search);
    s.help_open = true;
    assert_snapshot!("help_overlay_120x40", frame(&mut s, 120, 40));
}

#[test]
fn too_small_terminal_shows_hint() {
    let mut s = prepared(Route::Search);
    assert_snapshot!("too_small_70x20", frame(&mut s, 70, 20));
}
