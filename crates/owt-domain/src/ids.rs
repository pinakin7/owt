//! Identifier newtypes — `docs/architecture/data-model.md` § Identifier grammar.
//!
//! Source-native IDs are never rewritten; owt-assigned IDs are deterministic where
//! possible (replay yields identical IDs). Newtypes keep the grammar honest at the
//! type level — a `MarketId` can never be passed where a `TokenId` is expected.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Defines a transparent string-backed identifier newtype with `Display` and
/// `From<String>`/`From<&str>` conveniences.
macro_rules! string_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_owned())
            }
        }
    };
}

string_id!(
    /// Polymarket-native event id (verbatim, Gamma).
    EventId
);
string_id!(
    /// Polymarket-native market id (verbatim, Gamma).
    MarketId
);
string_id!(
    /// Polymarket-native decimal token id (verbatim, CLOB).
    TokenId
);
string_id!(
    /// `0x`-hex condition id (verbatim, Gamma/CLOB).
    ConditionId
);
string_id!(
    /// `0x`-hex question id (verbatim, Gamma/CLOB).
    QuestionId
);
string_id!(
    /// `ent_{kebab-name}` owt-assigned, stable (entity dictionary).
    EntityId
);
string_id!(
    /// `sha256(canonical_url)` hex, assigned by the normalizer.
    NewsId
);
string_id!(
    /// Source-native trade id (verbatim, Data API).
    TradeId
);
string_id!(
    /// `{tx_hash}:{log_index}` (Goldsky, v1).
    FillId
);
string_id!(
    /// Source-native comment id (verbatim, RTDS, v1).
    CommentId
);
string_id!(
    /// `event:{event_id}:{ts_rfc3339}:{kind}:{hash8}` (timeline builder).
    TimelineId
);
string_id!(
    /// `{name}.v{major}` owt-assigned (forecast engine, v1).
    ModelId
);

/// UUIDv7, minted once at receipt: the JetStream dedupe key (`Nats-Msg-Id`) and the
/// raw-archive object name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EnvelopeId(pub Uuid);

impl std::fmt::Display for EnvelopeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}
