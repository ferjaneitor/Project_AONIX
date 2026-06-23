//! AONIX — umbrella crate re-exporting the workspace's public API.
//!
//! The implementation lives in focused crates following the layered
//! architecture of `docs/02-architecture.md`:
//!
//! - `aonix-core` — circuit model and canonical `.aoncir` format (layers 1–3).
//! - `aonix-sim` — deterministic combinational simulator (layer 5).
//!
//! This facade preserves the flat, stable paths `aonix::circuit_model`,
//! `aonix::format` and `aonix::simulation` for downstream code (CLI, tests
//! and future crates) regardless of how the implementation crates evolve.

pub use aonix_core::{circuit_model, format};
pub use aonix_sim::simulation;
