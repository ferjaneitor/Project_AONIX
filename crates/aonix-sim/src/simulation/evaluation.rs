//! Deterministic single-vector evaluator for a [`Circuit`].
//!
//! Sub-phase 1.E exposes [`simulate`]: given a circuit and an input
//! vector of the right length, compute the output vector. Gate
//! evaluation matches exhaustively on the closed enum [`GateKind`], so
//! it is **impossible** to evaluate a kind outside `{And, Or, Not}`.
//! Primitive constants do not exist as [`SignalReference`] variants in
//! Phase 1, so no constant value can sneak in.
//!
//! [`Circuit`]: aonix_core::circuit_model::Circuit
//! [`GateKind`]: aonix_core::circuit_model::GateKind
//! [`SignalReference`]: aonix_core::circuit_model::SignalReference

use std::collections::BTreeMap;

use aonix_core::circuit_model::{
    AonixError, AonixResult, Bit, Circuit, Gate, GateKind, InputVector, OutputVector,
    PortIdentifier, SignalIdentifier, SignalReference,
};

use super::topological_order::compute_topological_order;

/// Simulates `circuit` on `input` and returns the [`OutputVector`].
///
/// The order of bits in the input vector follows the declaration order
/// of `[[ports.inputs]]`; the order of bits in the output vector follows
/// the declaration order of `[[ports.outputs]]`. Same circuit + same
/// input ⇒ same output, every time.
///
/// # Errors
///
/// - [`AonixError::InputVectorLengthMismatch`] if `input.len()` differs
///   from the number of declared input ports.
/// - [`AonixError::UnassignedOutputPort`] if a declared output port is
///   missing its source assignment (defensive — the builder rejects this
///   at finish time).
/// - [`AonixError::UndefinedIdentifier`] if a signal reference does not
///   resolve at evaluation time (defensive — the builder also rejects).
/// - [`AonixError::CycleDetected`] if the topological sort fails
///   (defensive).
pub fn simulate(circuit: &Circuit, input: &InputVector) -> AonixResult<OutputVector> {
    let expected = circuit.input_count();
    let given = input.len();
    if given != expected {
        return Err(AonixError::InputVectorLengthMismatch { expected, given });
    }

    let mut port_values: BTreeMap<PortIdentifier, Bit> = BTreeMap::new();
    for (index, port) in circuit.inputs_in_order().iter().enumerate() {
        let bit = input.get(index).expect("length checked above");
        port_values.insert(port.identifier.clone(), bit);
    }

    let order = compute_topological_order(circuit)?;
    let mut signal_values: BTreeMap<SignalIdentifier, Bit> = BTreeMap::new();

    for gate_identifier in &order {
        let gate = circuit
            .gate(gate_identifier)
            .expect("gate identifier comes from circuit's topological sort");
        let input_bits = collect_gate_input_bits(gate, &port_values, &signal_values)?;
        let output_bit = evaluate_primitive_gate(gate.kind, &input_bits);
        signal_values.insert(gate.output.clone(), output_bit);
    }

    let mut output_bits: Vec<Bit> = Vec::with_capacity(circuit.output_count());
    for port in circuit.outputs_in_order() {
        let reference =
            circuit
                .output_assignment(&port.identifier)
                .ok_or_else(|| AonixError::UnassignedOutputPort {
                    port: port.identifier.as_str().to_string(),
                })?;
        let bit = resolve_signal_reference(reference, &port_values, &signal_values)?;
        output_bits.push(bit);
    }

    Ok(OutputVector::new(output_bits))
}

/// Simulates `circuit` over every input vector in `inputs`, returning one
/// [`OutputVector`] per input in the same order.
///
/// This is sub-phase 1.F batch mode: it is a thin, deterministic wrapper
/// over [`simulate`]. The first input that fails (for example a wrong
/// length) aborts the whole batch with that error.
pub fn simulate_batch(
    circuit: &Circuit,
    inputs: &[InputVector],
) -> AonixResult<Vec<OutputVector>> {
    inputs.iter().map(|input| simulate(circuit, input)).collect()
}

/// Upper bound on the number of input ports for which
/// [`simulate_exhaustive`] will enumerate the full 2^n truth table. Beyond
/// this, exhaustive enumeration is refused to avoid accidental blow-up;
/// targeted or random testing is the job of later phases.
pub const MAX_EXHAUSTIVE_INPUT_BITS: usize = 20;

/// Computes the **full truth table** of `circuit`: each of the 2^n input
/// combinations (n = number of input ports) paired with its output vector.
///
/// Combinations are generated in ascending binary order, where the input
/// port at index `i` (in declared order) takes bit `i` of the counter
/// (LSB-first: index 0 is the least-significant bit, matching the AONIX
/// `bit_position` convention of `docs/24` §U.7). The result is therefore
/// deterministic and stable across runs.
///
/// # Errors
///
/// Returns [`AonixError::ExhaustiveInputTooLarge`] when the circuit has more
/// than [`MAX_EXHAUSTIVE_INPUT_BITS`] input ports. Propagates any error from
/// [`simulate`] (defensive: a well-built circuit never triggers them).
pub fn simulate_exhaustive(
    circuit: &Circuit,
) -> AonixResult<Vec<(InputVector, OutputVector)>> {
    let input_count = circuit.input_count();
    if input_count > MAX_EXHAUSTIVE_INPUT_BITS {
        return Err(AonixError::ExhaustiveInputTooLarge {
            inputs: input_count,
            max: MAX_EXHAUSTIVE_INPUT_BITS,
        });
    }

    let total: u64 = 1u64 << input_count;
    let mut table: Vec<(InputVector, OutputVector)> = Vec::with_capacity(total as usize);
    for combination in 0..total {
        let bits: Vec<Bit> = (0..input_count)
            .map(|index| Bit(((combination >> index) & 1) == 1))
            .collect();
        let input = InputVector::new(bits);
        let output = simulate(circuit, &input)?;
        table.push((input, output));
    }
    Ok(table)
}

fn collect_gate_input_bits(
    gate: &Gate,
    port_values: &BTreeMap<PortIdentifier, Bit>,
    signal_values: &BTreeMap<SignalIdentifier, Bit>,
) -> AonixResult<Vec<Bit>> {
    gate.inputs
        .iter()
        .map(|reference| resolve_signal_reference(reference, port_values, signal_values))
        .collect()
}

fn resolve_signal_reference(
    reference: &SignalReference,
    port_values: &BTreeMap<PortIdentifier, Bit>,
    signal_values: &BTreeMap<SignalIdentifier, Bit>,
) -> AonixResult<Bit> {
    match reference {
        SignalReference::Port(identifier) => {
            port_values
                .get(identifier)
                .copied()
                .ok_or_else(|| AonixError::UndefinedIdentifier {
                    identifier: identifier.as_str().to_string(),
                })
        }
        SignalReference::InternalSignal(identifier) => {
            signal_values.get(identifier).copied().ok_or_else(|| {
                AonixError::UndefinedIdentifier {
                    identifier: identifier.as_str().to_string(),
                }
            })
        }
    }
}

/// Evaluates a single primitive gate over its already-resolved input bits.
///
/// The exhaustive match on [`GateKind`] is the type-level enforcement of
/// R2: no other gate kind exists, no other gate kind can be evaluated.
/// Arity is trusted (already enforced by [`aonix_core::circuit_model::Gate::new`]).
fn evaluate_primitive_gate(kind: GateKind, inputs: &[Bit]) -> Bit {
    match kind {
        GateKind::Not => Bit(!inputs[0].is_one()),
        GateKind::And => Bit(inputs[0].is_one() && inputs[1].is_one()),
        GateKind::Or => Bit(inputs[0].is_one() || inputs[1].is_one()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_not_inverts_bit() {
        assert_eq!(evaluate_primitive_gate(GateKind::Not, &[Bit::ZERO]), Bit::ONE);
        assert_eq!(evaluate_primitive_gate(GateKind::Not, &[Bit::ONE]), Bit::ZERO);
    }

    #[test]
    fn evaluate_and_returns_one_only_when_both_inputs_are_one() {
        assert_eq!(
            evaluate_primitive_gate(GateKind::And, &[Bit::ZERO, Bit::ZERO]),
            Bit::ZERO
        );
        assert_eq!(
            evaluate_primitive_gate(GateKind::And, &[Bit::ZERO, Bit::ONE]),
            Bit::ZERO
        );
        assert_eq!(
            evaluate_primitive_gate(GateKind::And, &[Bit::ONE, Bit::ZERO]),
            Bit::ZERO
        );
        assert_eq!(
            evaluate_primitive_gate(GateKind::And, &[Bit::ONE, Bit::ONE]),
            Bit::ONE
        );
    }

    #[test]
    fn evaluate_or_returns_zero_only_when_both_inputs_are_zero() {
        assert_eq!(
            evaluate_primitive_gate(GateKind::Or, &[Bit::ZERO, Bit::ZERO]),
            Bit::ZERO
        );
        assert_eq!(
            evaluate_primitive_gate(GateKind::Or, &[Bit::ZERO, Bit::ONE]),
            Bit::ONE
        );
        assert_eq!(
            evaluate_primitive_gate(GateKind::Or, &[Bit::ONE, Bit::ZERO]),
            Bit::ONE
        );
        assert_eq!(
            evaluate_primitive_gate(GateKind::Or, &[Bit::ONE, Bit::ONE]),
            Bit::ONE
        );
    }
}
