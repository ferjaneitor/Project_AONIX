//! AONIX action validator — layer 4 of `docs/02-architecture.md`.
//!
//! The validator is the gate every agent action passes through before the
//! simulator, verifier, evaluator or coordinator ever see it
//! (`docs/08-actions-and-rewards.md`). It models the **incremental
//! construction** of a circuit:
//!
//! - [`Action`] — the closed set of construction actions. Only AND/OR/NOT
//!   gate creation exists, so a derived primitive cannot even be
//!   represented; [`Action::create_gate`] additionally rejects forbidden
//!   gate-kind names at the action layer.
//! - [`BuildState`] — the partial circuit. [`BuildState::validate`] applies
//!   the 10 rules of `docs/08`; [`BuildState::apply`] mutates after a
//!   successful validation; [`BuildState::finalize`] hands a finished,
//!   structurally re-checked [`aonix_core::circuit_model::Circuit`] to the
//!   verifier.
//! - [`ValidationError`] — typed rejection causes (rejection level L0 of
//!   `docs/14-circuit-rejection.md`).

pub mod action;
pub mod state;
pub mod validate;

pub use action::{Action, ActionKind};
pub use state::BuildState;
pub use validate::ValidationError;
