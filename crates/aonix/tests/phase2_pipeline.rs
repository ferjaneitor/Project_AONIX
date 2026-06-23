//! Phase 2 integration: parser/validator ⇒ simulator ⇒ verifier.
//!
//! Demonstrates the acceptance criteria of `docs/11-roadmap.md` Phase 2:
//!
//! - a circuit can be built step by step (action by action) through the
//!   validator without any legitimate action being rejected,
//! - exhaustive verification confirms correctness or pinpoints a failing
//!   case,
//! - any action introducing a derived primitive (XOR/NAND/NOR/XNOR) is
//!   rejected.

use std::path::Path;

use aonix::circuit_model::{Circuit, Port, PortIdentifier, PortRole};
use aonix::format::aoncir;
use aonix::validate::{Action, BuildState, ValidationError};
use aonix::verify::{verify, Decision, ReferenceFunction, Specification, TruthTable, VerifyError};

fn input_port(name: &str) -> Port {
    Port::new(PortIdentifier::new(name).expect("valid id"), PortRole::Input, None, None, None)
}

fn output_port(name: &str) -> Port {
    Port::new(PortIdentifier::new(name).expect("valid id"), PortRole::Output, None, None, None)
}

fn declare(id: &str) -> Action {
    Action::DeclareSignal { id: id.to_string(), semantic_tag: None, group: None }
}

fn assign(port: &str, source: &str) -> Action {
    Action::AssignOutput { port: port.to_string(), source: source.to_string() }
}

fn load_aoncir(file_name: &str) -> Circuit {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join(file_name);
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
    aoncir::parse(&raw).unwrap_or_else(|error| panic!("parse failed for {file_name}: {error}"))
}

/// Half adder built gate by gate: sum = (a AND NOT b) OR (NOT a AND b),
/// carry = a AND b. Every action must be accepted, finalize must succeed,
/// and exhaustive verification against the arithmetic reference must PASS.
#[test]
fn validator_builds_half_adder_and_verifier_passes() {
    let mut state = BuildState::new(
        vec![input_port("operand_a"), input_port("operand_b")],
        vec![output_port("sum_output"), output_port("carry_output")],
    );

    let actions = [
        declare("operand_a_negated"),
        declare("operand_b_negated"),
        declare("a_and_not_b"),
        declare("not_a_and_b"),
        declare("sum_internal"),
        declare("carry_internal"),
        Action::create_gate("g_not_a", "NOT", vec!["operand_a".into()], "operand_a_negated").unwrap(),
        Action::create_gate("g_not_b", "NOT", vec!["operand_b".into()], "operand_b_negated").unwrap(),
        Action::create_gate("g_a_and_not_b", "AND", vec!["operand_a".into(), "operand_b_negated".into()], "a_and_not_b").unwrap(),
        Action::create_gate("g_not_a_and_b", "AND", vec!["operand_a_negated".into(), "operand_b".into()], "not_a_and_b").unwrap(),
        Action::create_gate("g_sum", "OR", vec!["a_and_not_b".into(), "not_a_and_b".into()], "sum_internal").unwrap(),
        Action::create_gate("g_carry", "AND", vec!["operand_a".into(), "operand_b".into()], "carry_internal").unwrap(),
        assign("sum_output", "sum_internal"),
        assign("carry_output", "carry_internal"),
    ];

    for action in &actions {
        state
            .apply(action)
            .unwrap_or_else(|error| panic!("legitimate action rejected: {action:?} -> {error}"));
    }
    assert_eq!(state.gate_count(), 6);
    assert_eq!(state.assignment_count(), 2);

    let circuit = state.finalize().expect("half adder finalizes");

    // sum = a XOR b, carry = a AND b; outputs in declared order [sum, carry].
    let reference = ReferenceFunction::new(2, 2, |input| {
        vec![input[0] ^ input[1], input[0] && input[1]]
    });
    let report = verify(&circuit, &Specification::ReferenceFunction(reference)).expect("verifiable");
    assert_eq!(report.decision, Decision::Pass);
    assert_eq!(report.cases_evaluated, 4);
    assert!(report.failing_cases.is_empty());
}

#[test]
fn validator_rejects_forbidden_gate_kind() {
    let result = Action::create_gate("g_xor", "XOR", vec!["a".into(), "b".into()], "out");
    assert!(matches!(result, Err(ValidationError::ForbiddenGateKind { kind }) if kind == "XOR"));
}

#[test]
fn validator_rejects_cycle_introducing_gate() {
    let mut state = BuildState::new(vec![input_port("a")], vec![output_port("out")]);
    state.apply(&declare("s_one")).unwrap();
    state.apply(&declare("s_two")).unwrap();
    // g_one: s_one = a AND s_two  (s_one depends on s_two)
    state
        .apply(&Action::create_gate("g_one", "AND", vec!["a".into(), "s_two".into()], "s_one").unwrap())
        .expect("g_one is legal");
    // g_two: s_two = a AND s_one  would make s_two depend on s_one -> cycle.
    let result = state.apply(
        &Action::create_gate("g_two", "AND", vec!["a".into(), "s_one".into()], "s_two").unwrap(),
    );
    assert!(matches!(result, Err(ValidationError::CycleIntroduced { gate }) if gate == "g_two"));
}

#[test]
fn one_bit_full_adder_fixture_verifies_against_reference() {
    let circuit = load_aoncir("one_bit_full_adder.aoncir");
    // inputs [operand_a, operand_b, carry_input]; outputs [sum_output, carry_output].
    let reference = ReferenceFunction::new(3, 2, |input| {
        let ones = input[0] as u8 + input[1] as u8 + input[2] as u8;
        vec![ones % 2 == 1, ones >= 2]
    });
    let report = verify(&circuit, &Specification::ReferenceFunction(reference)).expect("verifiable");
    assert!(report.passed());
    assert_eq!(report.cases_evaluated, 8);
}

#[test]
fn verifier_detects_wrong_circuit_with_failing_case() {
    // two_input_and computes AND; verifying it against an OR spec must FAIL.
    let circuit = load_aoncir("two_input_and.aoncir");
    let table = TruthTable::from_rows(
        2,
        1,
        [
            (vec![false, false], vec![false]),
            (vec![false, true], vec![true]),  // AND gives false here
            (vec![true, false], vec![true]),  // AND gives false here
            (vec![true, true], vec![true]),
        ],
    )
    .unwrap();
    let report = verify(&circuit, &Specification::TruthTable(table)).expect("verifiable");
    assert_eq!(report.decision, Decision::Fail);
    assert_eq!(report.cases_evaluated, 4);
    assert_eq!(report.failing_cases.len(), 2);
    assert!(report
        .failing_cases
        .iter()
        .any(|case| case.input == vec![false, true] && case.produced == vec![false]));
}

#[test]
fn verifier_rejects_arity_mismatch() {
    let circuit = load_aoncir("two_input_and.aoncir"); // 2 inputs, 1 output
    let table = TruthTable::new(3, 1); // expects 3 inputs
    let result = verify(&circuit, &Specification::TruthTable(table));
    assert!(matches!(result, Err(VerifyError::InputArityMismatch { circuit: 2, spec: 3 })));
}
