//! Deterministic combinational simulator for AONIX circuits.
//!
//! Sub-phase 1.E exposes a **single-vector evaluator**: given a
//! [`crate::circuit_model::Circuit`] and an
//! [`crate::circuit_model::InputVector`] of the right length, it computes
//! the [`crate::circuit_model::OutputVector`] whose order is the formal
//! contract of `[[ports.outputs]]`.
//!
//! Batch simulation (multiple input vectors at once), exhaustive
//! verification, canonical hashing, and metrics are scheduled for later
//! sub-phases (1.F, 1.G, 1.H, 1.I).
//!
//! **Determinism guarantee.** Same circuit + same input ⇒ same output,
//! every time, on every machine. The gate evaluation order is the
//! topological sort produced by
//! [`topological_order::compute_topological_order`], whose ready set is a
//! `BTreeSet<GateIdentifier>` (lexicographic tie-breaking).
//!
//! **R2 compliance.** Gate evaluation matches exhaustively on the closed
//! enum [`crate::circuit_model::GateKind`] — only `AND`, `OR`, `NOT`. No
//! free strings are interpreted; no primitive constants are introduced.

pub mod topological_order;
pub mod evaluation;

pub use evaluation::simulate;
pub use topological_order::compute_topological_order;
