//! AONIX structural optimizer — Phase 6 (`docs/15-optimization-rules.md`,
//! `docs/23-optimizer-transformations.md`).
//!
//! "Optimizar es mejorar sin mentir": transform a circuit to improve its
//! structural metrics while **preserving behaviour exactly**. Every
//! transformation in the catalog is behaviour-preserving by construction (the
//! *algebraic* guarantee); on top of that, [`optimize()`] re-checks each
//! candidate by **differential equivalence against the original circuit** —
//! the invariant of `docs/15` verbatim, `∀v: simular(original,v) ==
//! simular(transformado,v)` — which is the *verification* guarantee. A
//! candidate that is not equivalent (a transformation bug) is **discarded**
//! (backtracking); only strictly-better, verified candidates are kept, and
//! the loop runs to a fixpoint.
//!
//! Phase-6 MVP catalog ([`TransformId`]): dead-signal elimination (A.1),
//! double-negation elimination (A.3), idempotence (B.1) and common
//! subexpression elimination (E.1). All produce only AND/OR/NOT — the
//! prohibited list `docs/23` P.1–P.7 (collapsing into XOR/NAND/NOR/XNOR or an
//! opaque node, or skipping re-verification) is impossible here: `GateKind`
//! is a closed enum and re-verification is mandatory.

pub mod error;
mod rewrite;
mod equivalence;
pub mod transform;
pub mod optimize;

pub use error::OptError;
pub use optimize::{optimize, optimize_with, OptReport, OptStep, StepOutcome};
pub use transform::{mvp_transforms, Transform, TransformId};
