//! The multiplexed WebSocket frame protocol (ADR-0010). One WS per client carries
//! many logical subscriptions, distinguished by a subscription topic string.

use serde::{Deserialize, Serialize};

/// A client → server frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
#[non_exhaustive]
pub enum ClientFrame {
    /// Begin receiving updates for a topic.
    Subscribe {
        /// The subscription topic, e.g. `market:will-fed-cut…:state`.
        topic: String,
    },
    /// Stop receiving updates for a topic.
    Unsubscribe {
        /// The subscription topic.
        topic: String,
    },
}

/// A server → client frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
#[non_exhaustive]
pub enum ServerFrame {
    /// A data update for a subscribed topic.
    Update {
        /// The topic this update belongs to.
        topic: String,
        /// The update payload (a serialized DTO).
        data: serde_json::Value,
    },
    /// An out-of-band error frame.
    Error {
        /// The topic the error relates to, if any.
        topic: Option<String>,
        /// Human-readable message.
        message: String,
    },
}
