//! Alert-rule AST (v1).
//!
//! The rule model lives in the domain so both the streaming evaluator (`owt-alerts`)
//! and the API contract (`owt-api-types`) share one definition. No evaluation logic
//! here — that is `owt-alerts` (see `docs/design/alerts.md`). Deferred until v1; this
//! is a designed-ahead placeholder so the type name and dependency edge exist.

use serde::{Deserialize, Serialize};

/// A single alert rule's predicate tree. Fleshed out in the v1 alerts phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum RuleNode {
    /// Placeholder leaf until the v1 grammar is designed.
    Always,
}
