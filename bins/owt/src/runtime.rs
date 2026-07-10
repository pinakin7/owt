//! The event loop: the impure driver around the pure `update`/`view` core
//! (`docs/design/tui-client.md` § Architecture).
//!
//! Crossterm's async `EventStream`, a tick timer, and an `mpsc` of `Msg`s (fed by
//! command execution) are merged into one stream. Each `Msg` runs `update`; the
//! returned `Cmd`s are executed here; the frame is redrawn only when state is dirty or
//! on a tick. In the skeleton, `Cmd::Load` resolves against the fixture seam
//! (`crate::data`); live REST/WS wiring replaces those bodies in the follow-up.

use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyEventKind};
use futures::StreamExt;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{Duration, interval};

use crate::app::{AppState, Cmd, Msg, update};
use crate::{data, terminal, view};

/// Enter the terminal, run the loop, and restore on exit (success or error).
pub async fn run(mut state: AppState) -> Result<()> {
    let mut term = terminal::enter()?;
    let result = event_loop(&mut state, &mut term).await;
    // Restore before surfacing any loop error, so the message lands on a clean screen.
    let restored = terminal::restore();
    result.and(restored)
}

async fn event_loop(state: &mut AppState, term: &mut terminal::Tui) -> Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Msg>();
    let mut events = EventStream::new();
    let tick_ms = state.cfg.ui.tick_ms.max(16);
    let mut ticker = interval(Duration::from_millis(tick_ms));

    let size = term.size()?;
    state.size = (size.width, size.height);

    // Load the starting route and draw the first frame.
    tx.send(Msg::Loaded(data::load(state.route()))).ok();
    term.draw(|f| view(f, state))?;
    state.dirty = false;

    loop {
        let msg = tokio::select! {
            Some(m) = rx.recv() => m,
            _ = ticker.tick() => Msg::Tick,
            maybe = events.next() => match maybe {
                Some(Ok(event)) => match to_msg(event) {
                    Some(m) => m,
                    None => continue,
                },
                Some(Err(_)) | None => continue,
            }
        };

        for cmd in update(state, msg) {
            execute(state, cmd, &tx);
        }

        if state.should_quit {
            break;
        }
        if state.dirty {
            term.draw(|f| view(f, state))?;
            state.dirty = false;
        }
    }

    Ok(())
}

/// Translate a crossterm event into a `Msg`, dropping ones we don't act on.
fn to_msg(event: Event) -> Option<Msg> {
    match event {
        // Ignore key-release/repeat (Windows emits them) to avoid double-firing.
        Event::Key(key) if key.kind == KeyEventKind::Press => Some(Msg::Key(key)),
        Event::Resize(w, h) => Some(Msg::Resize(w, h)),
        _ => None,
    }
}

/// Perform a side effect. This is the seam the live SDK plugs into.
fn execute(state: &mut AppState, cmd: Cmd, tx: &UnboundedSender<Msg>) {
    match cmd {
        Cmd::Load(route) => {
            // Fixture load is synchronous and infallible; the live version spawns a
            // task calling owt-client and sends Loaded / Failed when it completes.
            tx.send(Msg::Loaded(data::load(&route))).ok();
        }
        Cmd::Subscribe(_) | Cmd::Unsubscribe(_) => {
            // Live WS subscriptions arrive with the SDK wiring (follow-up).
        }
        Cmd::Export { format, path } => {
            let where_to = if path.is_empty() {
                "the working directory".to_owned()
            } else {
                path
            };
            state.info(format!(
                "export ({format}) → {where_to} lands with live data"
            ));
        }
        Cmd::Quit => state.should_quit = true,
    }
}
