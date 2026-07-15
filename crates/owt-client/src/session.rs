//! The WebSocket reconnect + resubscribe driver (`docs/design/tui-client.md`
//! § Connection lifecycle).
//!
//! [`WsSession::spawn`] starts a background task that owns a [`WsClient`], keeps a
//! desired-topic registry, and maintains the connection: connect → resubscribe →
//! serve frames → on drop, back off (jittered) and retry. The task reports connection
//! health as [`ConnState`] and forwards [`ServerFrame`]s over channels, so the TUI
//! (or any script) drives its status bar and live panes without owning the socket.
//!
//! Only WS connection health is reported here (`Connected` while serving,
//! `Reconnecting` while (re)establishing); `Degraded`/`Offline` also depend on REST
//! health, which this task does not observe.

use std::collections::BTreeSet;
use std::time::{SystemTime, UNIX_EPOCH};

use owt_api_types::ws::ServerFrame;
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;

use crate::reconnect::{Backoff, ConnState};
use crate::ws::WsClient;

/// A subscription command issued to the running session task.
#[derive(Debug)]
enum WsCommand {
    Subscribe(String),
    Unsubscribe(String),
}

/// A running WS session: the sender side of a spawned reconnect driver.
///
/// Subscriptions are fire-and-forget (buffered on an unbounded channel), so they can be
/// issued while the [`ServerFrame`]/[`ConnState`] receivers are borrowed by an event
/// loop. Dropping the session shuts the driver task down.
#[derive(Debug)]
pub struct WsSession {
    cmd_tx: mpsc::UnboundedSender<WsCommand>,
    _task: JoinHandle<()>,
}

impl WsSession {
    /// Spawn the driver over `client`. Returns the session handle plus the frame and
    /// connection-state receivers. The watch starts at [`ConnState::Reconnecting`] (the
    /// driver's first act is to connect).
    pub fn spawn(
        client: WsClient,
    ) -> (
        WsSession,
        mpsc::UnboundedReceiver<ServerFrame>,
        watch::Receiver<ConnState>,
    ) {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let (frame_tx, frame_rx) = mpsc::unbounded_channel();
        let (conn_tx, conn_rx) = watch::channel(ConnState::Reconnecting);
        let task = tokio::spawn(run(client, cmd_rx, frame_tx, conn_tx));
        (
            WsSession {
                cmd_tx,
                _task: task,
            },
            frame_rx,
            conn_rx,
        )
    }

    /// Request a subscription (idempotent; the driver dedups against its registry).
    pub fn subscribe(&self, topic: impl Into<String>) {
        let _ = self.cmd_tx.send(WsCommand::Subscribe(topic.into()));
    }

    /// Drop a subscription.
    pub fn unsubscribe(&self, topic: impl Into<String>) {
        let _ = self.cmd_tx.send(WsCommand::Unsubscribe(topic.into()));
    }
}

/// The driver loop: (re)connect, resubscribe the desired set, serve frames, back off.
async fn run(
    client: WsClient,
    mut cmd_rx: mpsc::UnboundedReceiver<WsCommand>,
    frame_tx: mpsc::UnboundedSender<ServerFrame>,
    conn_tx: watch::Sender<ConnState>,
) {
    let backoff = Backoff::default();
    let mut topics: BTreeSet<String> = BTreeSet::new();
    let mut attempt: u32 = 0;

    loop {
        let mut conn = match client.connect().await {
            Ok(conn) => conn,
            Err(_) => {
                let _ = conn_tx.send(ConnState::Reconnecting);
                if !backoff_wait(&backoff, attempt, &mut cmd_rx, &mut topics).await {
                    return; // session dropped
                }
                attempt = attempt.saturating_add(1);
                continue;
            }
        };

        // Connected: reset the schedule and resubscribe the desired set.
        attempt = 0;
        let _ = conn_tx.send(ConnState::Connected);
        let mut broke = false;
        for topic in &topics {
            if conn.subscribe(topic).await.is_err() {
                broke = true;
                break;
            }
        }

        // Serve frames until the socket drops or a resubscribe failed.
        if !broke {
            broke = serve(&mut conn, &mut cmd_rx, &frame_tx, &mut topics).await;
        }

        // `serve` returns `false` only when the session handle was dropped.
        if !broke {
            return;
        }

        let _ = conn_tx.send(ConnState::Reconnecting);
        if !backoff_wait(&backoff, attempt, &mut cmd_rx, &mut topics).await {
            return;
        }
        attempt = attempt.saturating_add(1);
    }
}

/// Serve one live connection. Returns `true` if the connection dropped (caller should
/// reconnect) or `false` if the session handle was dropped (caller should stop).
async fn serve(
    conn: &mut crate::ws::WsConnection,
    cmd_rx: &mut mpsc::UnboundedReceiver<WsCommand>,
    frame_tx: &mpsc::UnboundedSender<ServerFrame>,
    topics: &mut BTreeSet<String>,
) -> bool {
    loop {
        tokio::select! {
            res = conn.next_frame() => match res {
                Ok(frame) => {
                    // A closed frame channel means the consumer is gone; stop.
                    if frame_tx.send(frame).is_err() {
                        return false;
                    }
                }
                Err(_) => return true, // disconnected -> reconnect
            },
            cmd = cmd_rx.recv() => match cmd {
                Some(WsCommand::Subscribe(topic)) => {
                    if topics.insert(topic.clone()) && conn.subscribe(&topic).await.is_err() {
                        return true;
                    }
                }
                Some(WsCommand::Unsubscribe(topic)) => {
                    if topics.remove(&topic) {
                        let _ = conn.unsubscribe(&topic).await;
                    }
                }
                None => return false, // session dropped -> stop
            },
        }
    }
}

/// Sleep for the jittered backoff delay while still absorbing subscription commands, so
/// the desired-topic set is current when we reconnect. Returns `false` if the session
/// handle was dropped during the wait.
async fn backoff_wait(
    backoff: &Backoff,
    attempt: u32,
    cmd_rx: &mut mpsc::UnboundedReceiver<WsCommand>,
    topics: &mut BTreeSet<String>,
) -> bool {
    let delay = Backoff::jitter(backoff.delay(attempt), jitter_sample());
    let sleep = tokio::time::sleep(delay);
    tokio::pin!(sleep);
    loop {
        tokio::select! {
            _ = &mut sleep => return true,
            cmd = cmd_rx.recv() => match cmd {
                Some(WsCommand::Subscribe(topic)) => { topics.insert(topic); }
                Some(WsCommand::Unsubscribe(topic)) => { topics.remove(&topic); }
                None => return false, // session dropped
            },
        }
    }
}

/// A rough uniform sample in `[0, 1)` for full-jitter. Sourced from the wall clock's
/// sub-second nanos so the driver needs no RNG dependency (reconnect.rs deliberately
/// leaves the RNG to the caller).
fn jitter_sample() -> f64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    f64::from(nanos) / 1_000_000_000.0
}
