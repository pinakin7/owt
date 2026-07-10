//! `owt-domain` — the contract every pipeline honors.
//!
//! Canonical entities, ID newtypes, the ingest envelope, the error taxonomy, and
//! (v1) the alert-rule AST. Pure types + serde with **zero I/O dependencies** so the
//! TUI build stays lean and the contract is portable
//! (see `docs/architecture/cargo-workspace.md`, layering rule 1).
//!
//! This crate is the *normative* encoding of `docs/architecture/data-model.md`; the
//! JSON in that document is illustration, these types are the source of truth.
//!
//! Skeleton note: entity fields are intentionally minimal in this first scaffold.
//! Full modeling of every field lands in the data-model implementation phase.

pub mod alert;
pub mod entities;
pub mod envelope;
pub mod error;
pub mod ids;

pub use error::{DomainError, Result};
