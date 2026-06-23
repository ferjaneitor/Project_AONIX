//! Deterministic combinational simulator for AONIX circuits.
//!
//! Given a [`aonix_core::circuit_model::Circuit`] and an
//! [`aonix_core::circuit_model::InputVector`] of the right length, it
//! computes the [`aonix_core::circuit_model::OutputVector`] whose order is
//! the formal contract of `[[ports.outputs]]`.
//!
//! Sub-phase 1.E delivered the single-vector [`simulate`]; sub-phase 1.F
//! adds [`simulate_batch`] (many vectors) and [`simulate_exhaustive`] (the
//! full 2^n truth table). Canonical hashing lives in `aonix-core`
//! (`format::aoncir::hash_canonical`); metrics are a later phase.
//!
//! **Determinism guarantee.** Same circuit + same input ⇒ same output,
//! every time, on every machine. The gate evaluation order is the
//! topological sort produced by
//! [`topological_order::compute_topological_order`], whose ready set is a
//! `BTreeSet<GateIdentifier>` (lexicographic tie-breaking).
//!
//! **R2 compliance.** Gate evaluation matches exhaustively on the closed
//! enum [`aonix_core::circuit_model::GateKind`] — only `AND`, `OR`, `NOT`. No
//! free strings are interpreted; no primitive constants are introduced.

pub mod topological_order;
pub mod evaluation;

pub use evaluation::{
    simulate, simulate_batch, simulate_exhaustive, MAX_EXHAUSTIVE_INPUT_BITS,
};
pub use topological_order::compute_topological_order;
