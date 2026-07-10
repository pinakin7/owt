//! Request/response DTOs. These are the wire projections of `owt-domain` entities —
//! deliberately separate types so the storage model can evolve without breaking the
//! contract. Fleshed out alongside `docs/design/query-api.md`.

use owt_domain::ids::MarketId;
use serde::{Deserialize, Serialize};

/// Summary projection of a market for list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSummary {
    /// Market identifier.
    pub market_id: MarketId,
    /// Human-readable question.
    pub title: String,
}
