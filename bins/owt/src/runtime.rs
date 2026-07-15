//! The event loop: the impure driver around the pure `update`/`view` core
//! (`docs/design/tui-client.md` § Architecture).
//!
//! Crossterm's async `EventStream`, a tick timer, an `mpsc` of `Msg`s (fed by command
//! execution), and the `owt-client` WS session (connection-state + live frames) are
//! merged into one `select!`. Each `Msg` runs `update`; the returned `Cmd`s are executed
//! here — `Cmd::Load` spawns a `RestClient` fetch, `Cmd::Subscribe`/`Unsubscribe` drive
//! the WS session. The frame is redrawn only when state is dirty or on a tick.

use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyEventKind};
use futures::StreamExt;
use owt_api_types::ws::ServerFrame;
use owt_client::reconnect::ConnState;
use owt_client::{RestClient, WsClient, WsSession};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::watch;
use tokio::time::{Duration, interval};

use crate::app::route::Route;
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

/// The impure side of the loop: the SDK clients and the channel back into `update`.
struct Runtime {
    /// REST client, or `None` if the server URL was unusable (loads then fail loudly).
    rest: Option<RestClient>,
    /// The WS reconnect/resubscribe session, or `None` if the URL was unusable.
    ws: Option<WsSession>,
    /// Feeds `Msg`s (load results) back into the loop.
    tx: UnboundedSender<Msg>,
}

impl Runtime {
    /// Perform a side effect. This is the seam the live SDK plugs into.
    fn execute(&self, state: &mut AppState, cmd: Cmd) {
        match cmd {
            Cmd::Load(route) => self.load(state, route),
            Cmd::Subscribe(topic) => {
                if let Some(ws) = &self.ws {
                    ws.subscribe(topic);
                }
            }
            Cmd::Unsubscribe(topic) => {
                if let Some(ws) = &self.ws {
                    ws.unsubscribe(topic);
                }
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

    /// Spawn a task that fetches the data backing `route` and feeds `Loaded`/`Failed`
    /// back through the channel.
    fn load(&self, state: &AppState, route: Route) {
        let Some(rest) = self.rest.clone() else {
            self.tx
                .send(Msg::Failed {
                    what: route.label(),
                    error: "no server configured".to_owned(),
                })
                .ok();
            return;
        };
        let tx = self.tx.clone();
        let query = state.screens.search.query.clone();
        tokio::spawn(async move {
            let msg = match data::fetch(&rest, &route, &query).await {
                Ok(loaded) => Msg::Loaded(loaded),
                Err(e) => Msg::Failed {
                    what: route.label(),
                    error: e.to_string(),
                },
            };
            tx.send(msg).ok();
        });
    }
}

async fn event_loop(state: &mut AppState, term: &mut terminal::Tui) -> Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Msg>();

    // Build the SDK clients from config. A bad server URL degrades to offline rather than
    // aborting: REST loads surface `Failed` toasts and the WS branch stays idle.
    let rest = match RestClient::new(&state.cfg.server) {
        Ok(client) => Some(client),
        Err(e) => {
            state.error(format!("client init failed: {e}"));
            None
        }
    };
    let (ws, mut frames, mut conn_rx) = match WsClient::new(&state.cfg.server) {
        Ok(client) => {
            let (session, frames, conn) = WsSession::spawn(client);
            (Some(session), Some(frames), Some(conn))
        }
        Err(e) => {
            state.error(format!("ws init failed: {e}"));
            (None, None, None)
        }
    };
    let rt = Runtime {
        rest,
        ws,
        tx: tx.clone(),
    };

    let mut events = EventStream::new();
    let tick_ms = state.cfg.ui.tick_ms.max(16);
    let mut ticker = interval(Duration::from_millis(tick_ms));

    let size = term.size()?;
    state.size = (size.width, size.height);

    // Load the starting route and draw the first frame.
    rt.load(state, state.route().clone());
    term.draw(|f| view(f, state))?;
    state.dirty = false;

    loop {
        let msg = tokio::select! {
            Some(m) = rx.recv() => m,
            _ = ticker.tick() => Msg::Tick,
            conn = wait_conn(&mut conn_rx) => Msg::Conn(conn),
            frame = wait_frame(&mut frames) => { on_frame(&frame); continue; }
            maybe = events.next() => match maybe {
                Some(Ok(event)) => match to_msg(event) {
                    Some(m) => m,
                    None => continue,
                },
                Some(Err(_)) | None => continue,
            }
        };

        for cmd in update(state, msg) {
            rt.execute(state, cmd);
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

/// Await the next WS connection-state change, or never if the driver isn't running.
/// Cancel-safe (`watch::Receiver::changed`); a dropped sender retires the branch.
async fn wait_conn(conn: &mut Option<watch::Receiver<ConnState>>) -> ConnState {
    match conn {
        Some(rx) => {
            if rx.changed().await.is_ok() {
                *rx.borrow_and_update()
            } else {
                *conn = None;
                std::future::pending().await
            }
        }
        None => std::future::pending().await,
    }
}

/// Await the next live server frame, or never if the driver isn't running.
async fn wait_frame(frames: &mut Option<UnboundedReceiver<ServerFrame>>) -> ServerFrame {
    match frames {
        Some(rx) => match rx.recv().await {
            Some(frame) => frame,
            None => {
                *frames = None;
                std::future::pending().await
            }
        },
        None => std::future::pending().await,
    }
}

/// Handle a live server frame. This pass logs it; applying `Update`s to pane state is a
/// follow-up gated on the server defining per-topic payload shapes (ADR-0002).
fn on_frame(frame: &ServerFrame) {
    match frame {
        ServerFrame::Update { topic, .. } => {
            tracing::debug!(topic, "ws update frame (not yet applied to panes)");
        }
        ServerFrame::Error { topic, message } => {
            tracing::warn!(?topic, message, "ws error frame");
        }
        _ => {}
    }
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
