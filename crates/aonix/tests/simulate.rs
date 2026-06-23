//! Integration tests for Sub-phase 1.E — deterministic combinational
//! simulator.
//!
//! Each test loads an `.aoncir` fixture, parses it into a `Circuit`, and
//! exercises [`aonix::simulation::simulate`] on either the full truth
//! table or a property (length mismatch, determinism, output order…).

use std::path::Path;

use aonix::circuit_model::{AonixError, Bit, Circuit, InputVector, OutputVector};
use aonix::format::aoncir;
use aonix::simulation::simulate;

fn load_aoncir(file_name: &str) -> Circuit {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join(file_name);
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
    aoncir::parse(&raw).unwrap_or_else(|error| panic!("parse failed for {file_name}: {error}"))
}

fn input_from_bools(bools: &[bool]) -> InputVector {
    InputVector::new(bools.iter().copied().map(Bit).collect())
}

fn expect_output(circuit: &Circuit, input: &InputVector) -> OutputVector {
    simulate(circuit, input).expect("simulation should succeed")
}

fn run_truth_table(file_name: &str, expected: &[(Vec<bool>, Vec<bool>)]) {
    let circuit = load_aoncir(file_name);
    for (input_bits, expected_output_bits) in expected {
        let input = input_from_bools(input_bits);
        let produced = expect_output(&circuit, &input);
        let produced_bools: Vec<bool> = produced.as_slice().iter().map(|bit| bit.is_one()).collect();
        assert_eq!(
            &produced_bools, expected_output_bits,
            "{file_name}: for input {input_bits:?} expected {expected_output_bits:?} got {produced_bools:?}"
        );
    }
}

#[test]
fn simulate_pass_through_correctly() {
    run_truth_table(
        "pass_through.aoncir",
        &[
            (vec![false], vec![false]),
            (vec![true], vec![true]),
        ],
    );
}

#[test]
fn simulate_inverter_correctly() {
    run_truth_table(
        "inverter.aoncir",
        &[
            (vec![false], vec![true]),
            (vec![true], vec![false]),
        ],
    );
}

#[test]
fn simulate_two_input_and_truth_table() {
    run_truth_table(
        "two_input_and.aoncir",
        &[
            (vec![false, false], vec![false]),
            (vec![false, true], vec![false]),
            (vec![true, false], vec![false]),
            (vec![true, true], vec![true]),
        ],
    );
}

#[test]
fn simulate_two_input_or_truth_table() {
    run_truth_table(
        "two_input_or.aoncir",
        &[
            (vec![false, false], vec![false]),
            (vec![false, true], vec![true]),
            (vec![true, false], vec![true]),
            (vec![true, true], vec![true]),
        ],
    );
}

#[test]
fn simulate_multiplexer_2_to_1_full_truth_table() {
    // multiplexer_2_to_1.aoncir declares inputs in order:
    //   data_input_zero, data_input_one, select_input
    // When select = 0 ⇒ output = data_input_zero
    // When select = 1 ⇒ output = data_input_one
    let mut expected: Vec<(Vec<bool>, Vec<bool>)> = Vec::with_capacity(8);
    for index in 0..8u8 {
        let d0 = (index & 0b001) != 0;
        let d1 = (index & 0b010) != 0;
        let select = (index & 0b100) != 0;
        let output = if select { d1 } else { d0 };
        expected.push((vec![d0, d1, select], vec![output]));
    }
    run_truth_table("multiplexer_2_to_1.aoncir", &expected);
}

#[test]
fn simulate_one_bit_full_adder_full_truth_table() {
    // one_bit_full_adder.aoncir declares inputs in order:
    //   operand_a, operand_b, carry_input
    // and outputs in order:
    //   sum_output, carry_output
    // Reference: arithmetic sum a + b + carry_input.
    let mut expected: Vec<(Vec<bool>, Vec<bool>)> = Vec::with_capacity(8);
    for index in 0..8u8 {
        let operand_a = (index & 0b001) != 0;
        let operand_b = (index & 0b010) != 0;
        let carry_input = (index & 0b100) != 0;
        let sum_total =
            (operand_a as u8) + (operand_b as u8) + (carry_input as u8);
        let sum_bit = (sum_total & 0b01) != 0;
        let carry_bit = (sum_total & 0b10) != 0;
        expected.push((
            vec![operand_a, operand_b, carry_input],
            vec![sum_bit, carry_bit],
        ));
    }
    run_truth_table("one_bit_full_adder.aoncir", &expected);
}

#[test]
fn simulate_is_deterministic_over_one_hundred_runs() {
    let circuit = load_aoncir("one_bit_full_adder.aoncir");
    // Pick a specific non-trivial vector and run it many times.
    let input = input_from_bools(&[true, true, false]);
    let reference = expect_output(&circuit, &input);
    for _ in 0..100 {
        let produced = expect_output(&circuit, &input);
        assert_eq!(produced, reference);
    }
}

#[test]
fn simulate_rejects_input_vector_with_wrong_length() {
    let circuit = load_aoncir("one_bit_full_adder.aoncir"); // expects 3 inputs

    // Too short.
    let short_input = input_from_bools(&[true, false]);
    match simulate(&circuit, &short_input) {
        Err(AonixError::InputVectorLengthMismatch { expected, given }) => {
            assert_eq!(expected, 3);
            assert_eq!(given, 2);
        }
        other => panic!("expected InputVectorLengthMismatch, got {other:?}"),
    }

    // Too long.
    let long_input = input_from_bools(&[true, false, true, false]);
    match simulate(&circuit, &long_input) {
        Err(AonixError::InputVectorLengthMismatch { expected, given }) => {
            assert_eq!(expected, 3);
            assert_eq!(given, 4);
        }
        other => panic!("expected InputVectorLengthMismatch, got {other:?}"),
    }
}

#[test]
fn simulate_outputs_in_declared_order() {
    // For the full adder, the declared output order is
    // [sum_output, carry_output]. Pick an input where sum and carry
    // differ to detect any swap: (1, 1, 0) gives sum=0, carry=1.
    let circuit = load_aoncir("one_bit_full_adder.aoncir");
    let input = input_from_bools(&[true, true, false]);
    let produced = expect_output(&circuit, &input);
    assert_eq!(produced.len(), 2);
    assert_eq!(produced.get(0), Some(Bit::ZERO), "sum_output should be 0 at position 0");
    assert_eq!(produced.get(1), Some(Bit::ONE), "carry_output should be 1 at position 1");
}

#[test]
fn simulate_allows_output_directly_from_input_port() {
    // pass_through.aoncir has no gates and assigns the output port
    // directly to an input port. The simulator must resolve the output
    // by reading the InputVector through the port reference.
    let circuit = load_aoncir("pass_through.aoncir");
    assert_eq!(circuit.gate_count(), 0, "pass_through has no gates by construction");
    let zero_output = expect_output(&circuit, &input_from_bools(&[false]));
    let one_output = expect_output(&circuit, &input_from_bools(&[true]));
    assert_eq!(zero_output.get(0), Some(Bit::ZERO));
    assert_eq!(one_output.get(0), Some(Bit::ONE));
}

#[test]
fn simulate_never_uses_forbidden_gate_kinds() {
    // Run the simulator on several canonical fixtures and assert every
    // gate that exists in them is one of AND/OR/NOT. This documents at
    // the integration-test level that no forbidden kind can ever reach
    // the evaluator (the closed enum makes this true by construction;
    // the test pins it down for any future regression).
    for fixture in [
        "inverter.aoncir",
        "two_input_and.aoncir",
        "two_input_or.aoncir",
        "multiplexer_2_to_1.aoncir",
        "one_bit_full_adder.aoncir",
    ] {
        let circuit = load_aoncir(fixture);
        for gate in circuit.gates() {
            let name = gate.kind.canonical_name();
            assert!(
                matches!(name, "AND" | "OR" | "NOT"),
                "{fixture} contains unexpected gate kind {name}"
            );
        }
        // Trigger one simulation so the evaluator's match runs at least
        // once on real input.
        let zero_input =
            InputVector::new(vec![Bit::ZERO; circuit.input_count()]);
        let _ = simulate(&circuit, &zero_input).expect("simulation must succeed");
    }
}

#[test]
fn simulate_has_no_primitive_constants() {
    // For every fixture, walk every SignalReference appearing in gate
    // inputs and in output assignments, and assert it matches one of the
    // two legal variants. If the SignalReference enum ever gained a
    // ConstantZero/ConstantOne variant, the match below would fail to
    // compile or one of these asserts would fire — either way the test
    // catches the regression at the integration level.
    use aonix::circuit_model::SignalReference;
    for fixture in [
        "pass_through.aoncir",
        "inverter.aoncir",
        "two_input_and.aoncir",
        "two_input_or.aoncir",
        "multiplexer_2_to_1.aoncir",
        "one_bit_full_adder.aoncir",
    ] {
        let circuit = load_aoncir(fixture);
        for gate in circuit.gates() {
            for input_reference in &gate.inputs {
                match input_reference {
                    SignalReference::Port(_) | SignalReference::InternalSignal(_) => {}
                }
            }
        }
        for port in circuit.outputs_in_order() {
            if let Some(reference) = circuit.output_assignment(&port.identifier) {
                match reference {
                    SignalReference::Port(_) | SignalReference::InternalSignal(_) => {}
                }
            }
        }
        // Smoke-test that simulation still works on this fixture.
        let zero_input =
            InputVector::new(vec![Bit::ZERO; circuit.input_count()]);
        let _ = simulate(&circuit, &zero_input).expect("simulation must succeed");
    }
}
