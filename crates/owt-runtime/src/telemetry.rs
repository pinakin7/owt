//! Telemetry initialization (`docs/ops/observability.md`).
//!
//! Today this wires structured JSON logging via `tracing-subscriber` with an
//! env-filter. OpenTelemetry traces and the Prometheus recorder are deferred behind
//! the `TODO` seam below (ADR-0005 lands the bus and the metrics that ride with it);
//! [`TelemetryGuard`] already exists as the return type so adding the OTLP flush guard
//! later is a non-breaking change.

/// Held by `main` for the process lifetime. Empty today; becomes the OTLP flush guard
/// when OpenTelemetry lands, so [`init`]'s signature stays stable.
#[derive(Debug)]
#[must_use = "telemetry is torn down when this guard is dropped; hold it for the process lifetime"]
pub struct TelemetryGuard(());

/// Initialize structured JSON logging. Idempotent-safe: if a global subscriber is
/// already installed (e.g. a second call, or a test harness), this is a no-op.
///
/// Filter precedence: `OWT_LOG` → `RUST_LOG` → `info`.
pub fn init() -> TelemetryGuard {
    // `try_init` errors only if a global subscriber is already set; ignore for
    // idempotency.
    let _ = tracing_subscriber::fmt()
        .json()
        .with_env_filter(env_filter())
        .with_target(true)
        .try_init();

    // TODO(ADR-0005 / observability): install the tracing-opentelemetry OTLP layer and
    // the Prometheus recorder here, returning the OTLP flush guard inside
    // `TelemetryGuard` so traces flush on drop.
    TelemetryGuard(())
}

/// Resolve the log filter from the environment, falling back to `info`.
fn env_filter() -> tracing_subscriber::EnvFilter {
    for var in ["OWT_LOG", "RUST_LOG"] {
        if let Ok(spec) = std::env::var(var)
            && let Ok(filter) = tracing_subscriber::EnvFilter::try_new(&spec)
        {
            return filter;
        }
    }
    tracing_subscriber::EnvFilter::new("info")
}
