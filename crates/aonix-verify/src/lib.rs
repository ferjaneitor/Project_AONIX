//! AONIX verifier — layer 6 of `docs/02-architecture.md`.
//!
//! The verifier answers the single question "does this circuit satisfy its
//! specification?" with a **binary** decision (PASA/FALLA — no shades), per
//! `docs/07-testing-and-verification.md`. It is the *only* authority on
//! correctness; the simulator computes and the evaluator measures, but
//! neither decides.
//!
//! Phase 2 delivers **exhaustive** verification: for a circuit with `n`
//! input ports (n ≤ [`aonix_sim::simulation::MAX_EXHAUSTIVE_INPUT_BITS`]),
//! every one of the 2ⁿ input combinations is simulated and compared against
//! the specification. Random sampling, property checks and reference-circuit
//! comparison are later phases.
//!
//! A specification ([`Specification`]) is either an explicit [`TruthTable`]
//! or a pure Rust [`ReferenceFunction`]. The result is a structured
//! [`VerificationReport`] listing every failing case.

pub mod spec;
pub mod report;
pub mod verify;

pub use report::{Decision, FailingCase, VerificationReport, VerifyError};
pub use spec::{ReferenceFunction, Specification, SpecError, TruthTable};
pub use verify::{verify, verify_inputs};
