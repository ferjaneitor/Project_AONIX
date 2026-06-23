//! AONIX structural evaluator — layer 7 of `docs/02-architecture.md`.
//!
//! The evaluator **measures quality, it does not decide correctness**
//! (`docs/02` §"Capa 7", `docs/15-optimization-rules.md`). Correctness is the
//! verifier's job; the evaluator only computes structural [`Metrics`] and a
//! deterministic [`compare`](fn@compare) used to rank competing versions of a
//! circuit.
//!
//! Metrics (per `docs/15` §"Qué se optimiza"): gate count by kind, logical
//! depth (critical-path length in gates), dead signals (unreachable from any
//! output), fan-in / fan-out, subexpression sharing (reuse), and a
//! configurable weighted aggregate cost.
//!
//! The default ranking ([`DEFAULT_RANKING`]) is the lexicographic order of
//! `docs/13` §28: gate count, then depth, then dead signals, then reuse.
//! Ties favour the incumbent (`docs/19`): see [`is_strictly_better`].

pub mod metrics;
pub mod compare;

pub use compare::{compare, default_compare, is_strictly_better, Criterion, DEFAULT_RANKING};
pub use metrics::{evaluate, evaluate_with_weights, CostWeights, Metrics};
