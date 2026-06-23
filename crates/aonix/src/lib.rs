//! AONIX — umbrella crate re-exporting the workspace's public API.
//!
//! The implementation lives in focused crates following the layered
//! architecture of `docs/02-architecture.md`:
//!
//! - `aonix-core` — circuit model and canonical `.aoncir` format (layers 1–3).
//! - `aonix-sim` — deterministic combinational simulator (layer 5).
//! - `aonix-validate` — action validator (layer 4).
//! - `aonix-verify` — exhaustive verifier (layer 6).
//! - `aonix-eval` — structural evaluator (layer 7).
//!
//! This facade preserves the flat, stable paths `aonix::circuit_model`,
//! `aonix::format`, `aonix::simulation`, `aonix::validate`, `aonix::verify`
//! and `aonix::eval` for downstream code (CLI, tests and future crates)
//! regardless of how the implementation crates evolve.

pub use aonix_core::{circuit_model, format};
pub use aonix_sim::simulation;

/// Action validator (layer 4) — see `aonix_validate`.
pub mod validate {
    pub use aonix_validate::*;
}

/// Exhaustive verifier (layer 6) — see `aonix_verify`.
pub mod verify {
    pub use aonix_verify::*;
}

/// Structural evaluator (layer 7) — see `aonix_eval`.
pub mod eval {
    pub use aonix_eval::*;
}
