//! Integration tests for Sub-phase 1.F — batch and exhaustive simulation.
//!
//! Loads `.aoncir` fixtures and exercises
//! [`aonix::simulation::simulate_batch`] and
//! [`aonix::simulation::simulate_exhaustive`], cross-checking them against
//! the single-vector [`aonix::simulation::simulate`].

use std::path::Path;

use aonix::circuit_model::{Bit, Circuit, InputVector};
use aonix::format::aoncir;
use aonix::simulation::{simulate, simulate_batch, simulate_exhaustive};

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

fn output_bools(circuit: &Circuit, input: &InputVector) -> Vec<bool> {
    simulate(circuit, input)
        .expect("simulation should succeed")
        .as_slice()
        .iter()
        .map(|bit| bit.is_one())
        .collect()
}

#[test]
fn batch_matches_individual_simulation() {
    let circuit = load_aoncir("two_input_and.aoncir");
    let inputs = vec![
        input_from_bools(&[false, false]),
        input_from_bools(&[false, true]),
        input_from_bools(&[true, false]),
        input_from_bools(&[true, true]),
    ];
    let batch = simulate_batch(&circuit, &inputs).expect("batch succeeds");
    assert_eq!(batch.len(), inputs.len());
    for (input, produced) in inputs.iter().zip(&batch) {
        let expected = output_bools(&circuit, input);
        let produced_bools: Vec<bool> = produced.as_slice().iter().map(|bit| bit.is_one()).collect();
        assert_eq!(produced_bools, expected);
    }
}

#[test]
fn batch_propagates_first_error() {
    let circuit = load_aoncir("two_input_and.aoncir");
    // Second vector has the wrong length: the whole batch must fail.
    let inputs = vec![
        input_from_bools(&[true, true]),
        input_from_bools(&[true]),
    ];
    assert!(simulate_batch(&circuit, &inputs).is_err());
}

#[test]
fn exhaustive_two_input_and_is_four_rows() {
    let circuit = load_aoncir("two_input_and.aoncir");
    let table = simulate_exhaustive(&circuit).expect("exhaustive succeeds");
    assert_eq!(table.len(), 4, "2 inputs ⇒ 2^2 = 4 rows");
    // Only the all-ones row produces a one.
    for (input, output) in &table {
        let in_bits: Vec<bool> = input.as_slice().iter().map(|b| b.is_one()).collect();
        let out_bit = output.as_slice()[0].is_one();
        assert_eq!(out_bit, in_bits[0] && in_bits[1]);
    }
}

#[test]
fn exhaustive_full_adder_matches_arithmetic() {
    // one_bit_full_adder.aoncir declares inputs [operand_a, operand_b,
    // carry_input] and outputs [sum_output, carry_output].
    let circuit = load_aoncir("one_bit_full_adder.aoncir");
    let table = simulate_exhaustive(&circuit).expect("exhaustive succeeds");
    assert_eq!(table.len(), 8, "3 inputs ⇒ 2^3 = 8 rows");
    for (input, output) in &table {
        let bits: Vec<bool> = input.as_slice().iter().map(|b| b.is_one()).collect();
        let (a, b, carry_in) = (bits[0], bits[1], bits[2]);
        let total = a as u8 + b as u8 + carry_in as u8;
        let expected_sum = total & 1 == 1;
        let expected_carry = total >= 2;
        let sum = output.as_slice()[0].is_one();
        let carry = output.as_slice()[1].is_one();
        assert_eq!(sum, expected_sum, "sum mismatch for {bits:?}");
        assert_eq!(carry, expected_carry, "carry mismatch for {bits:?}");
    }
}

#[test]
fn exhaustive_is_deterministic_across_runs() {
    let circuit = load_aoncir("multiplexer_2_to_1.aoncir");
    let first = simulate_exhaustive(&circuit).expect("first run");
    let second = simulate_exhaustive(&circuit).expect("second run");
    assert_eq!(first, second);
}
