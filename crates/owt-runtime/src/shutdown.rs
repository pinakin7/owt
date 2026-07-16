//! Graceful-shutdown coordination (ADR-0003 § Decision — supervised tasks drain
//! before exit; `docs/ops/deployment.md` § SIGTERM drain budget).
//!
//! A single [`watch`] channel carries a boolean "shutdown requested" flag. The flag
//! is *level-triggered*: a [`Shutdown`] handle cloned after the flag is set still
//! observes it, so a worker started late during teardown never hangs. Every
//! supervised worker holds a [`Shutdown`] and awaits [`Shutdown::cancelled`]; the OS
//! signal listener installed by [`install`] holds the [`ShutdownTrigger`] and flips
//! the flag on SIGINT/SIGTERM.

use tokio::sync::watch;

/// A cloneable observer handed to every supervised worker.
///
/// Backed by a `watch<bool>`: `false` = running, `true` = shutdown requested.
#[derive(Clone, Debug)]
pub struct Shutdown {
    rx: watch::Receiver<bool>,
}

/// The trigger side of a [`Shutdown`]. Held by the OS-signal listener (or a test).
#[derive(Clone, Debug)]
pub struct ShutdownTrigger {
    tx: watch::Sender<bool>,
}

impl Shutdown {
    /// Build an unwired handle + trigger pair (no OS signal wiring). For tests and
    /// embedding the runtime inside another process.
    pub fn channel() -> (ShutdownTrigger, Shutdown) {
        let (tx, rx) = watch::channel(false);
        (ShutdownTrigger { tx }, Shutdown { rx })
    }

    /// Resolve once shutdown has been requested. Returns immediately if it was already
    /// requested (level-triggered). Cancel-safe: sound to use as a `tokio::select!`
    /// branch. If every [`ShutdownTrigger`] has been dropped, this also resolves, so a
    /// worker awaiting it can never hang.
    pub async fn cancelled(&mut self) {
        if *self.rx.borrow() {
            return;
        }
        while self.rx.changed().await.is_ok() {
            if *self.rx.borrow() {
                return;
            }
        }
        // `changed()` errored => all senders dropped => treat as shutdown.
    }

    /// Non-blocking check for whether shutdown has been requested.
    pub fn is_shutdown(&self) -> bool {
        *self.rx.borrow()
    }
}

impl ShutdownTrigger {
    /// Request graceful shutdown. Idempotent — repeated calls are harmless.
    pub fn shutdown(&self) {
        // A send only fails if every receiver has been dropped; nothing left to notify.
        let _ = self.tx.send(true);
    }
}

/// Install the OS-signal listener and return a [`Shutdown`] for workers.
///
/// The first SIGINT/SIGTERM requests graceful shutdown (every [`Shutdown::cancelled`]
/// resolves). A **second** interrupt while draining forces an immediate
/// `process::exit(130)` — the double-Ctrl-C escape hatch when a worker won't drain.
///
/// Must be called from within a tokio runtime.
pub fn install() -> Shutdown {
    let (trigger, handle) = Shutdown::channel();
    tokio::spawn(listen(trigger));
    handle
}

#[cfg(unix)]
async fn listen(trigger: ShutdownTrigger) {
    use tokio::signal::unix::{SignalKind, signal};

    let mut sigint = match signal(SignalKind::interrupt()) {
        Ok(s) => s,
        Err(err) => {
            tracing::error!(error = %err, "failed to install SIGINT handler");
            return;
        }
    };
    let mut sigterm = match signal(SignalKind::terminate()) {
        Ok(s) => s,
        Err(err) => {
            tracing::error!(error = %err, "failed to install SIGTERM handler");
            return;
        }
    };

    tokio::select! {
        _ = sigint.recv() => {}
        _ = sigterm.recv() => {}
    }
    tracing::info!("shutdown signal received; draining");
    trigger.shutdown();

    // A second interrupt means the operator insists — bail without waiting for drain.
    let _ = sigint.recv().await;
    tracing::warn!("second interrupt; forcing exit");
    std::process::exit(130);
}

#[cfg(not(unix))]
async fn listen(trigger: ShutdownTrigger) {
    // Non-Unix targets get SIGINT (Ctrl-C) only; SIGTERM handling is Unix-only.
    if tokio::signal::ctrl_c().await.is_ok() {
        tracing::info!("shutdown signal received; draining");
        trigger.shutdown();
    }
    let _ = tokio::signal::ctrl_c().await;
    tracing::warn!("second interrupt; forcing exit");
    std::process::exit(130);
}
