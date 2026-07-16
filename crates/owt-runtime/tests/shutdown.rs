//! Contract tests for the graceful-shutdown coordinator (`docs/ops/deployment.md`
//! § SIGTERM drain). Exercises the `watch`-backed, level-triggered semantics that let
//! a handle cloned *after* shutdown still observe it.

use std::time::Duration;

use owt_runtime::shutdown::Shutdown;
use tokio::time::timeout;

#[tokio::test]
async fn cancelled_resolves_after_trigger() {
    let (trigger, mut shutdown) = Shutdown::channel();
    let waiter = tokio::spawn(async move { shutdown.cancelled().await });

    trigger.shutdown();

    timeout(Duration::from_secs(1), waiter)
        .await
        .expect("cancelled resolved within 1s")
        .expect("waiter task joined");
}

#[tokio::test]
async fn is_shutdown_reflects_state_transition() {
    let (trigger, shutdown) = Shutdown::channel();
    assert!(!shutdown.is_shutdown());
    trigger.shutdown();
    assert!(shutdown.is_shutdown());
}

#[tokio::test]
async fn late_cloned_handle_still_observes_prior_shutdown() {
    let (trigger, shutdown) = Shutdown::channel();
    trigger.shutdown();

    // Clone *after* the trigger fired: the level-triggered watch must still report it.
    let mut late = shutdown.clone();
    assert!(late.is_shutdown());
    timeout(Duration::from_secs(1), late.cancelled())
        .await
        .expect("late handle observed the prior shutdown");
}

#[tokio::test]
async fn all_clones_observe_a_single_trigger() {
    let (trigger, shutdown) = Shutdown::channel();
    let mut a = shutdown.clone();
    let mut b = shutdown.clone();
    let ha = tokio::spawn(async move { a.cancelled().await });
    let hb = tokio::spawn(async move { b.cancelled().await });

    trigger.shutdown();

    timeout(Duration::from_secs(1), ha).await.unwrap().unwrap();
    timeout(Duration::from_secs(1), hb).await.unwrap().unwrap();
}

#[tokio::test]
async fn cancelled_resolves_when_all_triggers_dropped() {
    let (trigger, mut shutdown) = Shutdown::channel();
    drop(trigger);

    // No trigger can ever fire; a worker must still not hang.
    timeout(Duration::from_secs(1), shutdown.cancelled())
        .await
        .expect("cancelled resolved after all triggers dropped");
}
