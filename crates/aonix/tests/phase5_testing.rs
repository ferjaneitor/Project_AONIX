//! Phase 5 integration: scalable test suites and automatic regression.
//!
//! Acceptance (`docs/11-roadmap.md` Phase 5 / `docs/07`): combined suites give
//! an aggregate PASA/FALLA; a failing case recorded once **reappears
//! automatically** in every later run.

use std::path::Path;

use aonix::circuit_model::Circuit;
use aonix::format::aoncir;
use aonix::testing::{run_suite, RegressionSuite, SuiteConfig};
use aonix::verify::{Decision, ReferenceFunction, Specification};

fn load_aoncir(file_name: &str) -> Circuit {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join(file_name);
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {path:?}: {error}"));
    aoncir::parse(&raw).unwrap_or_else(|error| panic!("parse {file_name}: {error}"))
}

fn and_spec() -> Specification {
    Specification::ReferenceFunction(ReferenceFunction::new(2, 1, |input| vec![input[0] && input[1]]))
}

fn or_spec() -> Specification {
    Specification::ReferenceFunction(ReferenceFunction::new(2, 1, |input| vec![input[0] || input[1]]))
}

#[test]
fn correct_circuit_passes_combined_suite() {
    let circuit = load_aoncir("two_input_and.aoncir");
    let report = run_suite(&circuit, &and_spec(), &SuiteConfig::default()).unwrap();
    assert_eq!(report.decision, Decision::Pass);
    // 2 inputs ⇒ exhaustive feasible; the run includes the exhaustive suite.
    assert!(report.breakdown.iter().any(|(name, _)| *name == "exhaustive"));
    assert_eq!(report.total_cases, 4);
}

#[test]
fn full_adder_passes_combined_suite() {
    let circuit = load_aoncir("one_bit_full_adder.aoncir");
    let spec = Specification::ReferenceFunction(ReferenceFunction::new(3, 2, |input| {
        let ones = input[0] as u8 + input[1] as u8 + input[2] as u8;
        vec![ones % 2 == 1, ones >= 2]
    }));
    let report = run_suite(&circuit, &spec, &SuiteConfig::default()).unwrap();
    assert!(report.passed());
    assert_eq!(report.total_cases, 8);
}

#[test]
fn wrong_circuit_fails_with_specific_cases() {
    let circuit = load_aoncir("two_input_and.aoncir");
    let report = run_suite(&circuit, &or_spec(), &SuiteConfig::default()).unwrap();
    assert_eq!(report.decision, Decision::Fail);
    // AND differs from OR exactly on (0,1) and (1,0).
    assert!(report.failing_cases.iter().any(|c| c.input == vec![false, true]));
    assert!(report.failing_cases.iter().any(|c| c.input == vec![true, false]));
}

#[test]
fn recorded_regression_reappears_in_later_runs() {
    let circuit = load_aoncir("two_input_and.aoncir");

    // First run against the OR spec discovers failing cases.
    let first = run_suite(&circuit, &or_spec(), &SuiteConfig::default()).unwrap();
    assert_eq!(first.decision, Decision::Fail);
    let mut regression = RegressionSuite::new();
    regression.record_failures(&first);
    assert!(regression.contains(&[false, true]));
    assert!(regression.contains(&[true, false]));

    // A later run with ONLY the regression suite re-checks those exact cases
    // automatically — no exhaustive, no edge, no random.
    let regression_only = SuiteConfig {
        exhaustive_if_feasible: false,
        random_samples: 0,
        seed: 0,
        edge_cases: false,
        regression: regression.cases(),
    };
    let second = run_suite(&circuit, &or_spec(), &regression_only).unwrap();
    assert_eq!(second.decision, Decision::Fail);
    assert_eq!(second.total_cases, regression.len());
    assert!(second.breakdown.iter().any(|(name, _)| *name == "regression"));
}

#[test]
fn random_suite_is_reproducible_with_same_seed() {
    let circuit = load_aoncir("two_input_and.aoncir");
    let config = SuiteConfig {
        exhaustive_if_feasible: false,
        random_samples: 200,
        seed: 5,
        edge_cases: false,
        regression: Vec::new(),
    };
    let first = run_suite(&circuit, &and_spec(), &config).unwrap();
    let second = run_suite(&circuit, &and_spec(), &config).unwrap();
    assert_eq!(first, second);
}
