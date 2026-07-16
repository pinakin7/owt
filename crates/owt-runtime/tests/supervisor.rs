//! Contract tests for task supervision (ADR-0003 § Decision — supervised tokio tasks;
//! `docs/ops/observability.md`). Runs on a paused clock so backoff sleeps resolve
//! instantly in wall-clock time while logical time advances.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::time::Duration;

use owt_runtime::shutdown::Shutdown;
use owt_runtime::supervisor::{Supervisor, Worker};

#[tokio::test(start_paused = true)]
async fn restarts_failing_worker_with_backoff_until_success() {
    let (trigger, shutdown) = Shutdown::channel();
    let calls = Arc::new(AtomicUsize::new(0));

    let c = calls.clone();
    let worker = Worker::new("flaky", move || {
        let c = c.clone();
        async move {
            let n = c.fetch_add(1, SeqCst);
            if n < 3 {
                anyhow::bail!("boom {n}");
            }
            Ok(())
        }
    });

    let mut supervisor = Supervisor::new(shutdown);
    supervisor.register(worker);

    // Keep the trigger alive so `cancelled()` never fires as "all senders dropped".
    let _trigger = trigger;
    supervisor.run().await.expect("supervisor drains");

    // 3 failures (attempts 0,1,2) then success on the 4th invocation.
    assert_eq!(calls.load(SeqCst), 4);
}

#[tokio::test(start_paused = true)]
async fn panicking_worker_is_restarted_not_aborted() {
    let (trigger, shutdown) = Shutdown::channel();
    let calls = Arc::new(AtomicUsize::new(0));

    let c = calls.clone();
    let worker = Worker::new("panicky", move || {
        let c = c.clone();
        async move {
            let n = c.fetch_add(1, SeqCst);
            if n == 0 {
                panic!("deliberate panic on first run");
            }
            Ok(())
        }
    });

    let mut supervisor = Supervisor::new(shutdown);
    supervisor.register(worker);

    let _trigger = trigger;
    supervisor.run().await.expect("supervisor drains");

    // Panic on run 0 is caught (JoinError::is_panic) and the worker is restarted.
    assert_eq!(calls.load(SeqCst), 2);
}

#[tokio::test(start_paused = true)]
async fn backoff_resets_after_healthy_run() {
    let (trigger, shutdown) = Shutdown::channel();
    let calls = Arc::new(AtomicUsize::new(0));

    let c = calls.clone();
    let worker = Worker::new("healthy-then-fail", move || {
        let c = c.clone();
        async move {
            let n = c.fetch_add(1, SeqCst) + 1;
            // Stay up past `healthy_after` so each run counts as healthy (resetting the
            // restart count), then fail — until we finally succeed.
            tokio::time::sleep(Duration::from_secs(10)).await;
            if n < 4 {
                anyhow::bail!("cycle {n}");
            }
            Ok(())
        }
    });

    let mut supervisor = Supervisor::new(shutdown).with_healthy_after(Duration::from_secs(5));
    supervisor.register(worker);

    let _trigger = trigger;
    supervisor.run().await.expect("supervisor drains");

    // Each of the first three runs is healthy-but-failed (backoff resets each time),
    // then the fourth succeeds.
    assert_eq!(calls.load(SeqCst), 4);
}

#[tokio::test]
async fn shutdown_stops_all_workers_and_run_resolves() {
    let (trigger, shutdown) = Shutdown::channel();
    let started = Arc::new(AtomicUsize::new(0));
    let stopped = Arc::new(AtomicUsize::new(0));

    let mut supervisor = Supervisor::new(shutdown.clone());
    for _ in 0..3 {
        let sd = shutdown.clone();
        let started = started.clone();
        let stopped = stopped.clone();
        supervisor.register(Worker::new("idle", move || {
            let mut sd = sd.clone();
            let started = started.clone();
            let stopped = stopped.clone();
            async move {
                started.fetch_add(1, SeqCst);
                sd.cancelled().await;
                stopped.fetch_add(1, SeqCst);
                Ok(())
            }
        }));
    }

    let run = tokio::spawn(supervisor.run());

    // Let every worker reach `cancelled()` before requesting shutdown — otherwise the
    // supervisor legitimately skips starting a worker whose shutdown is already set.
    while started.load(SeqCst) < 3 {
        tokio::task::yield_now().await;
    }
    trigger.shutdown();

    tokio::time::timeout(Duration::from_secs(5), run)
        .await
        .expect("run resolves within 5s")
        .expect("run task joins")
        .expect("supervisor drains");
    assert_eq!(stopped.load(SeqCst), 3);
}

#[tokio::test(start_paused = true)]
async fn clean_early_exit_is_not_hot_looped() {
    let (trigger, shutdown) = Shutdown::channel();
    let calls = Arc::new(AtomicUsize::new(0));

    let c = calls.clone();
    let worker = Worker::new("quick", move || {
        let c = c.clone();
        async move {
            c.fetch_add(1, SeqCst);
            Ok(())
        }
    });

    let mut supervisor = Supervisor::new(shutdown);
    supervisor.register(worker);

    let _trigger = trigger;
    supervisor.run().await.expect("supervisor drains");

    // A clean return without a shutdown request stops supervision — no restart storm.
    assert_eq!(calls.load(SeqCst), 1);
}
