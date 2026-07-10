//! Small display formatters shared by the panes (prices in cents, compact volumes).

/// Format a probability/price in `[0, 1]` as cents, e.g. `0.42` → `42.0¢`.
pub fn cents(price: f64) -> String {
    format!("{:.1}¢", price * 100.0)
}

/// Format a signed price delta as cents with a leading sign, e.g. `+3.0¢`.
pub fn signed_cents(delta: f64) -> String {
    format!("{:+.1}¢", delta * 100.0)
}

/// Format a quote-currency amount compactly, e.g. `3_200_000.0` → `$3.2M`.
pub fn usd_compact(amount: f64) -> String {
    let a = amount.abs();
    let (scaled, suffix) = if a >= 1e9 {
        (amount / 1e9, "B")
    } else if a >= 1e6 {
        (amount / 1e6, "M")
    } else if a >= 1e3 {
        (amount / 1e3, "K")
    } else {
        (amount, "")
    };
    if suffix.is_empty() {
        format!("${scaled:.0}")
    } else {
        format!("${scaled:.1}{suffix}")
    }
}

/// Format an `OffsetDateTime` as `HH:MM:SS` for tape/timeline rows.
pub fn hms(ts: time::OffsetDateTime) -> String {
    format!("{:02}:{:02}:{:02}", ts.hour(), ts.minute(), ts.second())
}

/// Format an `OffsetDateTime` as `HH:MM`.
pub fn hm(ts: time::OffsetDateTime) -> String {
    format!("{:02}:{:02}", ts.hour(), ts.minute())
}
