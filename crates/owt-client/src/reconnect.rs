//! The reconnect state machine (`docs/design/tui-client.md` § Connection lifecycle).
//!
//! [`ConnState`] is the status-bar model the TUI renders; [`Backoff`] produces the
//! `1 → 2 → 4 … → 60 s` capped delay schedule for reconnect attempts. The delay
//! computation is pure and unit-tested; jitter is applied by the caller (which owns
//! the RNG) via [`Backoff::jitter`] so this module stays deterministic.

use std::time::Duration;

/// The connection health the status bar reflects.
///
/// `Connected` (WS + REST healthy) → `Degraded` (WS down, REST ok; panes show age
/// badges) → `Reconnecting` (backoff in progress) → `Offline` (REST failing too;
/// cached data + banner).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnState {
    /// WebSocket and REST both healthy.
    #[default]
    Connected,
    /// WebSocket down, REST still serving — live panes go stale with age badges.
    Degraded,
    /// A reconnect attempt is in flight (backoff between tries).
    Reconnecting,
    /// REST is failing too; only cached data is shown behind a banner.
    Offline,
}

impl ConnState {
    /// A short human label for the status bar.
    pub fn label(self) -> &'static str {
        match self {
            ConnState::Connected => "connected",
            ConnState::Degraded => "degraded",
            ConnState::Reconnecting => "reconnecting",
            ConnState::Offline => "offline",
        }
    }

    /// Whether live updates are currently flowing.
    pub fn is_live(self) -> bool {
        matches!(self, ConnState::Connected)
    }
}

/// Exponential backoff with a cap, per the reconnect schedule.
#[derive(Debug, Clone, Copy)]
pub struct Backoff {
    base: Duration,
    cap: Duration,
}

impl Default for Backoff {
    fn default() -> Self {
        Self {
            base: Duration::from_secs(1),
            cap: Duration::from_secs(60),
        }
    }
}

impl Backoff {
    /// A backoff with an explicit base and cap.
    pub fn new(base: Duration, cap: Duration) -> Self {
        Self { base, cap }
    }

    /// The un-jittered delay for a zero-indexed attempt: `min(cap, base * 2^attempt)`.
    ///
    /// Saturates rather than overflowing at large attempt counts.
    pub fn delay(&self, attempt: u32) -> Duration {
        let factor = 1u64.checked_shl(attempt).unwrap_or(u64::MAX);
        let scaled = self.base.saturating_mul(factor.min(u32::MAX as u64) as u32);
        scaled.min(self.cap)
    }

    /// Apply full jitter to a delay given a caller-supplied uniform sample in `[0, 1]`.
    /// Returns a duration in `[0, delay]` (AWS "full jitter").
    pub fn jitter(delay: Duration, rand01: f64) -> Duration {
        let f = rand01.clamp(0.0, 1.0);
        delay.mul_f64(f)
    }
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
        assert_eq!(b.delay(5), Duration::from_secs(32));
        assert_eq!(
            b.delay(6),
            Duration::from_secs(60),
            "64s clamps to the 60s cap"
        );
        assert_eq!(
            b.delay(100),
            Duration::from_secs(60),
            "large attempts saturate"
        );
    }

    #[test]
    fn full_jitter_stays_within_bounds() {
        let d = Duration::from_secs(60);
        assert_eq!(Backoff::jitter(d, 0.0), Duration::ZERO);
        assert_eq!(Backoff::jitter(d, 1.0), d);
        assert!(Backoff::jitter(d, 0.5) <= d);
        assert_eq!(Backoff::jitter(d, 2.0), d, "sample is clamped to [0,1]");
    }

    #[test]
    fn conn_state_helpers() {
        assert!(ConnState::Connected.is_live());
        assert!(!ConnState::Reconnecting.is_live());
        assert_eq!(ConnState::Offline.label(), "offline");
        assert_eq!(ConnState::default(), ConnState::Connected);
    }
}
