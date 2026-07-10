//! Opaque cursor pagination shared by every list endpoint.

use serde::{Deserialize, Serialize};

/// An opaque, base64url-encoded pagination cursor. Its internal shape is a server
/// implementation detail; clients treat it as an opaque token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Cursor(pub String);

/// A single page of results plus the cursor to fetch the next page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    /// The items on this page.
    pub items: Vec<T>,
    /// Cursor for the next page, or `None` at the end.
    pub next: Option<Cursor>,
}
