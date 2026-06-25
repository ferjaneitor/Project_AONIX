//! AONIX scalable testing — layer 8 of `docs/02-architecture.md`,
//! specified in `docs/07-testing-and-verification.md`.
//!
//! Builds the input sets that feed the verifier, scaling coverage by level:
//!
//! - **exhaustive** — every 2ⁿ input when the space is feasible,
//! - **random** — reproducible samples from a seeded deterministic PRNG
//!   ([`prng::SplitMix64`]); same seed ⇒ same vectors,
//! - **edge cases** — all-zero, all-one, single-bit, alternating patterns,
//! - **regression** — an append-only set of inputs that once failed
//!   ([`RegressionSuite`]); a recorded failure reappears in every later run.
//!
//! [`run_suite`] combines the enabled suites, verifies them with
//! `aonix_verify::verify_inputs`, and returns an aggregate PASA/FALLA
//! [`SuiteReport`]. The runner chooses *which* inputs to check; the verifier
//! remains the sole authority on whether they pass.

pub mod prng;
pub mod generators;
pub mod suite;

pub use generators::{edge_cases, exhaustive, random, EXHAUSTIVE_LIMIT};
pub use prng::SplitMix64;
pub use suite::{run_suite, RegressionSuite, SuiteConfig, SuiteReport};
