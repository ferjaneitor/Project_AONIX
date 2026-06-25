//! Exhaustive verification of a [`Circuit`] against a [`Specification`].

use aonix_core::circuit_model::{Bit, Circuit, InputVector};
use aonix_sim::simulation::{simulate, MAX_EXHAUSTIVE_INPUT_BITS};

use crate::report::{Decision, FailingCase, VerificationReport, VerifyError};
use crate::spec::{ReferenceFunction, Specification, TruthTable};

/// Verifies `circuit` against `spec`, returning a binary [`VerificationReport`].
///
/// - Against a [`TruthTable`], every declared row is checked (so a partial
///   table verifies exactly the cases it lists).
/// - Against a [`ReferenceFunction`], every one of the 2ⁿ inputs is
///   enumerated and compared.
///
/// # Errors
///
/// Returns a [`VerifyError`] precondition failure when the spec's arity does
/// not match the circuit, when an exhaustive run would exceed
/// [`MAX_EXHAUSTIVE_INPUT_BITS`], or when the simulator itself errors.
pub fn verify(
    circuit: &Circuit,
    spec: &Specification,
) -> Result<VerificationReport, VerifyError> {
    match spec {
        Specification::TruthTable(table) => verify_truth_table(circuit, table),
        Specification::ReferenceFunction(function) => verify_reference_function(circuit, function),
    }
}

/// Verifies `circuit` against `spec` on an **explicit set of inputs** (used
/// by the scalable test suites of `aonix-test`: edge cases, random samples,
/// regression). Inputs the spec does not cover are skipped (not counted).
///
/// # Errors
///
/// Same precondition failures as [`verify`] (arity mismatch, simulation
/// error). Does not check for exhaustiveness — the caller chooses the inputs.
pub fn verify_inputs(
    circuit: &Circuit,
    spec: &Specification,
    inputs: &[Vec<bool>],
) -> Result<VerificationReport, VerifyError> {
    check_arity(circuit, spec.input_arity(), spec.output_arity())?;

    let mut failing_cases = Vec::new();
    let mut cases_evaluated = 0usize;
    for input in inputs {
        let Some(expected) = spec.expected_output(input) else {
            continue;
        };
        let produced = run_case(circuit, input)?;
        cases_evaluated += 1;
        if produced != expected {
            failing_cases.push(FailingCase {
                input: input.clone(),
                expected,
                produced,
            });
        }
    }

    Ok(finish(cases_evaluated, failing_cases))
}

fn verify_truth_table(
    circuit: &Circuit,
    table: &TruthTable,
) -> Result<VerificationReport, VerifyError> {
    check_arity(circuit, table.input_arity(), table.output_arity())?;

    let mut failing_cases = Vec::new();
    let mut cases_evaluated = 0usize;
    for (input, expected) in table.rows() {
        let produced = run_case(circuit, input)?;
        cases_evaluated += 1;
        if &produced != expected {
            failing_cases.push(FailingCase {
                input: input.clone(),
                expected: expected.clone(),
                produced,
            });
        }
    }

    Ok(finish(cases_evaluated, failing_cases))
}

fn verify_reference_function(
    circuit: &Circuit,
    function: &ReferenceFunction,
) -> Result<VerificationReport, VerifyError> {
    check_arity(circuit, function.input_arity(), function.output_arity())?;

    let input_count = circuit.input_count();
    if input_count > MAX_EXHAUSTIVE_INPUT_BITS {
        return Err(VerifyError::NotExhaustivelyVerifiable {
            inputs: input_count,
            max: MAX_EXHAUSTIVE_INPUT_BITS,
        });
    }

    let total: u64 = 1u64 << input_count;
    let mut failing_cases = Vec::new();
    for combination in 0..total {
        let input: Vec<bool> = (0..input_count)
            .map(|index| ((combination >> index) & 1) == 1)
            .collect();
        let expected = function.evaluate(&input);
        let produced = run_case(circuit, &input)?;
        if produced != expected {
            failing_cases.push(FailingCase {
                input,
                expected,
                produced,
            });
        }
    }

    Ok(finish(total as usize, failing_cases))
}

fn check_arity(circuit: &Circuit, input_arity: usize, output_arity: usize) -> Result<(), VerifyError> {
    if circuit.input_count() != input_arity {
        return Err(VerifyError::InputArityMismatch {
            circuit: circuit.input_count(),
            spec: input_arity,
        });
    }
    if circuit.output_count() != output_arity {
        return Err(VerifyError::OutputArityMismatch {
            circuit: circuit.output_count(),
            spec: output_arity,
        });
    }
    Ok(())
}

fn run_case(circuit: &Circuit, input: &[bool]) -> Result<Vec<bool>, VerifyError> {
    let vector = InputVector::new(input.iter().map(|&bit| Bit(bit)).collect());
    let output = simulate(circuit, &vector).map_err(VerifyError::Simulation)?;
    Ok(output.as_slice().iter().map(|bit| bit.is_one()).collect())
}

fn finish(cases_evaluated: usize, failing_cases: Vec<FailingCase>) -> VerificationReport {
    let decision = if failing_cases.is_empty() {
        Decision::Pass
    } else {
        Decision::Fail
    };
    VerificationReport {
        decision,
        cases_evaluated,
        failing_cases,
    }
}
