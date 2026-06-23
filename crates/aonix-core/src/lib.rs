//! AONIX core — foundational circuit model and canonical `.aoncir` format.
//!
//! Covers architecture layers 1–3 of `docs/02-architecture.md`:
//!
//! - [`circuit_model`] — the logical world: logical values, ports, signals,
//!   primitive gates, semantic groups and the immutable circuit DAG.
//! - [`format`](mod@format) — the canonical `.aoncir` representation
//!   (parser, writer, document validator and canonical hash).
//!
//! AONIX enforces the absolute rule R2 ("only AND, OR and NOT as primitive
//! gates") at the type system level: see [`circuit_model::GateKind`].

pub mod circuit_model;
pub mod format;
