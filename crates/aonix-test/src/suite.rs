//! Combining the suites into one aggregate PASA/FALLA run.

use std::collections::BTreeSet;

use aonix_core::circuit_model::Circuit;
use aonix_verify::{verify_inputs, Decision, FailingCase, Specification, VerifyError};

use crate::generators::{edge_cases, exhaustive, random, EXHAUSTIVE_LIMIT};

/// Which suites to run and with what parameters.
#[derive(Debug, Clone)]
pub struct SuiteConfig {
    /// Run exhaustively when the input space is feasible (≤ [`EXHAUSTIVE_LIMIT`]).
    pub exhaustive_if_feasible: bool,
    /// Number of random samples when exhaustive is not used.
    pub random_samples: usize,
    /// Seed for the random samples (explicit for reproducibility).
    pub seed: u64,
    /// Include the catalogued edge cases.
    pub edge_cases: bool,
    /// Explicit regression inputs to always re-check.
    pub regression: Vec<Vec<bool>>,
}

impl Default for SuiteConfig {
    /// Exhaustive when feasible (else 1000 random, seed 0), edge cases on, no
    /// regression.
    fn default() -> Self {
        Self {
            exhaustive_if_feasible: true,
            random_samples: 1000,
            seed: 0,
            edge_cases: true,
            regression: Vec::new(),
        }
    }
}

/// Aggregate result of a suite run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuiteReport {
    /// Aggregate binary decision (PASA iff every evaluated case passed).
    pub decision: Decision,
    /// Distinct cases actually evaluated (after de-duplication).
    pub total_cases: usize,
    /// Every case that failed.
    pub failing_cases: Vec<FailingCase>,
    /// How many cases each enabled suite contributed (before de-dup).
    pub breakdown: Vec<(&'static str, usize)>,
}

impl SuiteReport {
    /// Whether the aggregate decision is [`Decision::Pass`].
    pub fn passed(&self) -> bool {
        matches!(self.decision, Decision::Pass)
    }
}

/// Runs the configured suites for `circuit` against `spec` and returns an
/// aggregate report. Inputs are de-duplicated deterministically before
/// verification; the verifier decides PASA/FALLA.
pub fn run_suite(
    circuit: &Circuit,
    spec: &Specification,
    config: &SuiteConfig,
) -> Result<SuiteReport, VerifyError> {
    let input_arity = spec.input_arity();
    let mut breakdown: Vec<(&'static str, usize)> = Vec::new();
    let mut inputs: Vec<Vec<bool>> = Vec::new();

    let exhaustive_used = config.exhaustive_if_feasible && input_arity <= EXHAUSTIVE_LIMIT;
    if exhaustive_used {
        let cases = exhaustive(input_arity);
        breakdown.push(("exhaustive", cases.len()));
        inputs.extend(cases);
    } else if config.random_samples > 0 {
        let cases = random(input_arity, config.random_samples, config.seed);
        breakdown.push(("random", cases.len()));
        inputs.extend(cases);
    }
    if config.edge_cases {
        let cases = edge_cases(input_arity);
        breakdown.push(("edge", cases.len()));
        inputs.extend(cases);
    }
    if !config.regression.is_empty() {
        breakdown.push(("regression", config.regression.len()));
        inputs.extend(config.regression.iter().cloned());
    }

    // Deterministic de-duplication.
    let unique: BTreeSet<Vec<bool>> = inputs.into_iter().collect();
    let inputs: Vec<Vec<bool>> = unique.into_iter().collect();

    let report = verify_inputs(circuit, spec, &inputs)?;
    Ok(SuiteReport {
        decision: report.decision,
        total_cases: report.cases_evaluated,
        failing_cases: report.failing_cases,
        breakdown,
    })
}

/// Append-only set of inputs that once failed. A recorded case is re-checked
/// in every future run, so a regression "reappears automatically"
/// (`docs/07` §"Regresión automática"). Backed by a `BTreeSet`, so it is
/// de-duplicated and iterated deterministically; it never shrinks on its own.
#[derive(Debug, Clone, Default)]
pub struct RegressionSuite {
    cases: BTreeSet<Vec<bool>>,
}

impl RegressionSuite {
    /// An empty regression suite.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records one failing input. Returns `true` if it was new.
    pub fn record(&mut self, input: Vec<bool>) -> bool {
        self.cases.insert(input)
    }

    /// Records every failing case from a report.
    pub fn record_failures(&mut self, report: &SuiteReport) {
        for case in &report.failing_cases {
            self.cases.insert(case.input.clone());
        }
    }

    /// The recorded cases, deterministically ordered (ready for
    /// [`SuiteConfig::regression`]).
    pub fn cases(&self) -> Vec<Vec<bool>> {
        self.cases.iter().cloned().collect()
    }

    /// Number of recorded cases.
    pub fn len(&self) -> usize {
        self.cases.len()
    }

    /// Whether the suite is empty.
    pub fn is_empty(&self) -> bool {
        self.cases.is_empty()
    }

    /// Whether `input` is already recorded.
    pub fn contains(&self, input: &[bool]) -> bool {
        self.cases.contains(input)
    }
}
