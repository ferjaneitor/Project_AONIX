//! The verifier's binary decision and structured failure report
//! (`docs/07-testing-and-verification.md` §"Decisión binaria del verificador").

use thiserror::Error;

/// Binary verification decision. There are no shades: a circuit passes or it
/// does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// The circuit matched the specification on every evaluated case.
    Pass,
    /// At least one case mismatched.
    Fail,
}

/// One input where the produced output diverged from the specification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailingCase {
    /// Input bit-vector (declared input-port order).
    pub input: Vec<bool>,
    /// Output the specification requires.
    pub expected: Vec<bool>,
    /// Output the circuit actually produced.
    pub produced: Vec<bool>,
}

/// Structured result of a verification run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationReport {
    /// The binary decision.
    pub decision: Decision,
    /// How many cases were evaluated.
    pub cases_evaluated: usize,
    /// Every case that failed (empty iff `decision == Pass`).
    pub failing_cases: Vec<FailingCase>,
}

impl VerificationReport {
    /// Convenience: whether the decision is [`Decision::Pass`].
    pub fn passed(&self) -> bool {
        matches!(self.decision, Decision::Pass)
    }
}

/// A precondition error that prevents verification from even running. These
/// are distinct from a `Fail` decision: the spec does not match the
/// circuit's interface, or the input space is too large for exhaustive
/// verification.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum VerifyError {
    /// The circuit's input-port count differs from the specification's.
    #[error("input arity mismatch: circuit has {circuit}, spec expects {spec}")]
    InputArityMismatch { circuit: usize, spec: usize },
    /// The circuit's output-port count differs from the specification's.
    #[error("output arity mismatch: circuit has {circuit}, spec expects {spec}")]
    OutputArityMismatch { circuit: usize, spec: usize },
    /// The circuit has more input ports than exhaustive verification allows.
    #[error("circuit has {inputs} input ports; exhaustive verification is capped at {max}")]
    NotExhaustivelyVerifiable { inputs: usize, max: usize },
    /// The simulator returned an error while evaluating a case (defensive —
    /// a well-built circuit never triggers this).
    #[error("simulation error during verification: {0}")]
    Simulation(aonix_core::circuit_model::AonixError),
}
