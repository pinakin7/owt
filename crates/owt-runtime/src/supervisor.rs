//! Async task supervision (ADR-0003 § Decision — "workers are supervised tokio
//! tasks... surfacing panics as structured errors rather than silent aborts";
//! `docs/ops/observability.md`).
//!
//! [`Supervisor`] runs each registered [`Worker`] as its own tokio task and keeps it
//! alive: a worker that returns `Err` or panics is restarted after a jittered
//! [`Backoff`] delay, and one that stays healthy past `healthy_after` resets its
//! restart count. Every worker stops when the shared [`Shutdown`] fires, at which
//! point [`Supervisor::run`] resolves once all have drained.
//!
//! The backoff schedule mirrors `owt-client`'s reconnect state machine
//! (`crates/owt-client/src/reconnect.rs`): the delay math is pure and unit-tested, and
//! jitter is applied via the static [`Backoff::jitter`] so randomness stays out of the
//! deterministic core.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::time::Instant;

use crate::shutdown::Shutdown;

/// Exponential backoff with a cap, used to space worker restarts.
#[derive(Debug, Clone, Copy)]
pub struct Backoff {
    base: Duration,
    cap: Duration,
}

impl Default for Backoff {
    fn default() -> Self {
        Self {
            base: Duration::from_secs(1),
            cap: Duration::from_secs(30),
        }
    }
}

impl Backoff {
    /// A backoff with an explicit base and cap.
    pub fn new(base: Duration, cap: Duration) -> Self {
        Self { base, cap }
    }

    /// The un-jittered delay for a zero-indexed attempt: `min(cap, base * 2^attempt)`.
    /// Saturates rather than overflowing at large attempt counts.
    pub fn delay(&self, attempt: u32) -> Duration {
        let factor = 1u64.checked_shl(attempt).unwrap_or(u64::MAX);
        let scaled = self.base.saturating_mul(factor.min(u32::MAX as u64) as u32);
        scaled.min(self.cap)
    }

    /// Apply full jitter to a delay given a uniform sample in `[0, 1]` (AWS "full
    /// jitter"): returns a duration in `[0, delay]`.
    pub fn jitter(delay: Duration, rand01: f64) -> Duration {
        delay.mul_f64(rand01.clamp(0.0, 1.0))
    }
}

/// The boxed, restartable future a [`Worker`] produces on each (re)start.
type WorkerFuture = std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send>>;

/// A named, long-lived unit of work the [`Supervisor`] keeps running.
///
/// The factory is invoked once per (re)start, so it must produce a fresh future each
/// time — capture *clones* of any handles the worker needs (e.g. a [`Shutdown`]).
pub struct Worker {
    name: &'static str,
    factory: Box<dyn Fn() -> WorkerFuture + Send + Sync>,
}

impl Worker {
    /// Build a worker from a stable name and a future factory.
    pub fn new<F, Fut>(name: &'static str, factory: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        Self {
            name,
            factory: Box::new(move || Box::pin(factory())),
        }
    }

    /// The worker's stable name, used in log fields.
    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl std::fmt::Debug for Worker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The boxed factory is not `Debug`, but the lint requires an impl.
        f.debug_struct("Worker")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

/// Supervises a set of [`Worker`]s: spawns each as a tokio task, restarts it on
/// failure with jittered backoff, and drains them all on shutdown.
#[derive(Debug)]
pub struct Supervisor {
    shutdown: Shutdown,
    backoff: Backoff,
    healthy_after: Duration,
    workers: Vec<Worker>,
}

impl Supervisor {
    /// A supervisor bound to `shutdown`, with default backoff (1s → 30s cap) and a
    /// 60-second healthy-run threshold.
    pub fn new(shutdown: Shutdown) -> Self {
        Self {
            shutdown,
            backoff: Backoff::default(),
            healthy_after: Duration::from_secs(60),
            workers: Vec::new(),
        }
    }

    /// Override the restart backoff schedule.
    pub fn with_backoff(mut self, backoff: Backoff) -> Self {
        self.backoff = backoff;
        self
    }

    /// Override how long a worker must stay up before its restart count resets.
    pub fn with_healthy_after(mut self, healthy_after: Duration) -> Self {
        self.healthy_after = healthy_after;
        self
    }

    /// Register a worker to supervise.
    pub fn register(&mut self, worker: Worker) -> &mut Self {
        self.workers.push(worker);
        self
    }

    /// Spawn one supervising loop per worker and await them all. Resolves once every
    /// worker has drained after a shutdown request.
    pub async fn run(self) -> anyhow::Result<()> {
        let mut handles = Vec::with_capacity(self.workers.len());
        for worker in self.workers {
            handles.push(tokio::spawn(supervise(
                worker,
                self.shutdown.clone(),
                self.backoff,
                self.healthy_after,
            )));
        }
        for handle in handles {
            // The supervising loop returns `()` and only fails to join if tokio is torn
            // down mid-await; nothing actionable either way.
            let _ = handle.await;
        }
        Ok(())
    }
}

/// The per-worker supervising loop: run the worker as its own task, restart on
/// error/panic with jittered backoff, and stop on shutdown.
async fn supervise(
    worker: Worker,
    mut shutdown: Shutdown,
    backoff: Backoff,
    healthy_after: Duration,
) {
    let name = worker.name();
    let mut attempt: u32 = 0;

    loop {
        if shutdown.is_shutdown() {
            return;
        }

        // Spawn the worker as its own task so a panic surfaces as a `JoinError`
        // (structured log) rather than a silent abort.
        let started = Instant::now();
        let handle = tokio::spawn((worker.factory)());
        match handle.await {
            Ok(Ok(())) => {
                if shutdown.is_shutdown() {
                    tracing::info!(worker = name, "worker drained");
                } else {
                    // A long-lived worker should only return `Ok` at shutdown. An early
                    // clean exit is odd — log it and stop, rather than hot-looping.
                    tracing::warn!(
                        worker = name,
                        "worker exited cleanly without a shutdown request; not restarting"
                    );
                }
                return;
            }
            Ok(Err(err)) => {
                tracing::error!(worker = name, error = %err, "worker failed; restarting");
            }
            Err(join_err) if join_err.is_panic() => {
                tracing::error!(worker = name, "worker panicked; restarting");
            }
            Err(_) => {
                // Cancelled (runtime shutting down) — stop quietly.
                return;
            }
        }

        if shutdown.is_shutdown() {
            return;
        }

        attempt = next_attempt(attempt, started.elapsed(), healthy_after);
        let delay = Backoff::jitter(backoff.delay(attempt), jitter_sample());
        tokio::select! {
            _ = tokio::time::sleep(delay) => {}
            _ = shutdown.cancelled() => return,
        }
        attempt = attempt.saturating_add(1);
    }
}

/// Restart-count bookkeeping: a run that lasted at least `healthy_after` is treated as
/// healthy and resets the count to zero; otherwise the count is preserved so backoff
/// keeps climbing. Pure, so the reset rule is unit-testable without a runtime.
fn next_attempt(attempt: u32, ran_for: Duration, healthy_after: Duration) -> u32 {
    if ran_for >= healthy_after { 0 } else { attempt }
}

/// A rough uniform sample in `[0, 1)` from the wall clock's sub-second nanos, so the
/// supervisor needs no RNG dependency (matching `owt-client`'s session driver).
fn jitter_sample() -> f64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    f64::from(nanos) / 1_000_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_doubles_then_caps() {
        let b = Backoff::default();
        assert_eq!(b.delay(0), Duration::from_secs(1));
        assert_eq!(b.delay(1), Duration::from_secs(2));
        assert_eq!(b.delay(2), Duration::from_secs(4));
        assert_eq!(b.delay(4), Duration::from_secs(16));
        assert_eq!(
            b.delay(5),
            Duration::from_secs(30),
            "32s clamps to the 30s cap"
        );
        assert_eq!(
            b.delay(100),
            Duration::from_secs(30),
            "large attempts saturate"
        );
    }

    #[test]
    fn full_jitter_stays_within_bounds() {
        let d = Duration::from_secs(30);
        assert_eq!(Backoff::jitter(d, 0.0), Duration::ZERO);
        assert_eq!(Backoff::jitter(d, 1.0), d);
        assert!(Backoff::jitter(d, 0.5) <= d);
        assert_eq!(Backoff::jitter(d, 2.0), d, "sample is clamped to [0,1]");
    }

    #[test]
    fn next_attempt_resets_after_healthy_run() {
        let healthy = Duration::from_secs(60);
        assert_eq!(next_attempt(5, Duration::from_secs(60), healthy), 0);
        assert_eq!(next_attempt(5, Duration::from_secs(120), healthy), 0);
    }

    #[test]
    fn next_attempt_keeps_climbing_when_unhealthy() {
        let healthy = Duration::from_secs(60);
        assert_eq!(next_attempt(3, Duration::from_secs(1), healthy), 3);
        assert_eq!(next_attempt(0, Duration::from_millis(10), healthy), 0);
    }
}
