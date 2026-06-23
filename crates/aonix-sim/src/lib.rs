//! AONIX simulation — deterministic combinational simulator (layer 5).
//!
//! Depends on `aonix-core`. Exposes [`simulation::simulate`] (single
//! vector), [`simulation::simulate_batch`] (many vectors) and
//! [`simulation::simulate_exhaustive`] (full 2^n truth table), plus the
//! deterministic [`simulation::compute_topological_order`].
//!
//! **Determinism guarantee.** Same circuit + same input ⇒ same output, on
//! every machine. **R2 compliance.** Gate evaluation matches exhaustively
//! on the closed enum `aonix_core::circuit_model::GateKind`.

pub mod simulation;
