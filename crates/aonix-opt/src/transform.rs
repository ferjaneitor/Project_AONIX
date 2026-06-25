//! The closed catalog of behaviour-preserving transformations (Phase-6 MVP).
//!
//! Each [`Transform`] is a pure detector+rewriter over an immutable
//! [`Circuit`]: [`Transform::apply_once`] finds **one** opportunity, applies
//! the algebraically-sound rewrite, and returns the rebuilt circuit (or
//! `None`). It never decides whether the rewrite is *worth* it — that, and the
//! mandatory re-verification, is the [`optimize`](crate::optimize()) driver's job.

use std::collections::HashMap;

use aonix_core::circuit_model::{Circuit, Gate, GateKind, SignalIdentifier, SignalReference};

use crate::error::OptError;
use crate::rewrite::Working;

/// Closed identity of each implemented transformation, mirroring `docs/23`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransformId {
    /// A.1 — remove signals/gates unreachable from any output.
    DeadSignalElimination,
    /// A.3 — `NOT(NOT(x)) → x`.
    DoubleNegationElimination,
    /// B.1 — `AND(x, x) → x`, `OR(x, x) → x`.
    Idempotence,
    /// E.1 — merge two gates computing the same function.
    CommonSubexpressionElimination,
}

impl TransformId {
    /// The catalog id from `docs/23` (e.g. `"A.1"`).
    pub fn canonical_id(self) -> &'static str {
        match self {
            TransformId::DeadSignalElimination => "A.1",
            TransformId::DoubleNegationElimination => "A.3",
            TransformId::Idempotence => "B.1",
            TransformId::CommonSubexpressionElimination => "E.1",
        }
    }
}

/// A behaviour-preserving rewrite over a circuit.
pub trait Transform {
    /// The catalog identity of this transformation.
    fn id(&self) -> TransformId;
    /// Finds one opportunity and returns the rewritten circuit, or `None`.
    fn apply_once(&self, circuit: &Circuit) -> Result<Option<Circuit>, OptError>;
}

/// The MVP transformations, in the recommended pipeline order (cheap
/// structural rewrites first, dead-signal cleanup last each round).
pub fn mvp_transforms() -> Vec<Box<dyn Transform>> {
    vec![
        Box::new(DoubleNegationElimination),
        Box::new(Idempotence),
        Box::new(CommonSubexpressionElimination),
        Box::new(DeadSignalElimination),
    ]
}

/// A.1 — dead-signal elimination.
pub struct DeadSignalElimination;

impl Transform for DeadSignalElimination {
    fn id(&self) -> TransformId {
        TransformId::DeadSignalElimination
    }

    fn apply_once(&self, circuit: &Circuit) -> Result<Option<Circuit>, OptError> {
        let mut working = Working::from_circuit(circuit);
        if working.eliminate_dead() {
            Ok(Some(working.into_circuit()?))
        } else {
            Ok(None)
        }
    }
}

/// A.3 — `NOT(NOT(x)) → x`.
pub struct DoubleNegationElimination;

impl Transform for DoubleNegationElimination {
    fn id(&self) -> TransformId {
        TransformId::DoubleNegationElimination
    }

    fn apply_once(&self, circuit: &Circuit) -> Result<Option<Circuit>, OptError> {
        let mut working = Working::from_circuit(circuit);
        let opportunity = {
            let producers = working.producers();
            let mut found = None;
            for outer in working.gates() {
                if outer.kind != GateKind::Not {
                    continue;
                }
                if let SignalReference::InternalSignal(inner_signal) = &outer.inputs[0] {
                    if let Some(inner) = producers.get(inner_signal.as_str()) {
                        if inner.kind == GateKind::Not {
                            // outer == NOT(NOT(x)); its output is equivalent to x.
                            found = Some((
                                outer.output.as_str().to_string(),
                                inner.inputs[0].clone(),
                                outer.identifier.as_str().to_string(),
                            ));
                            break;
                        }
                    }
                }
            }
            found
        };
        match opportunity {
            Some((output_id, x_reference, gate_id)) => {
                working.redirect(&output_id, &x_reference);
                working.remove_gate(&gate_id);
                Ok(Some(working.into_circuit()?))
            }
            None => Ok(None),
        }
    }
}

/// B.1 — `AND(x, x) → x`, `OR(x, x) → x`.
pub struct Idempotence;

impl Transform for Idempotence {
    fn id(&self) -> TransformId {
        TransformId::Idempotence
    }

    fn apply_once(&self, circuit: &Circuit) -> Result<Option<Circuit>, OptError> {
        let mut working = Working::from_circuit(circuit);
        let opportunity = {
            let mut found = None;
            for gate in working.gates() {
                if matches!(gate.kind, GateKind::And | GateKind::Or)
                    && gate.inputs.len() == 2
                    && gate.inputs[0] == gate.inputs[1]
                {
                    found = Some((
                        gate.output.as_str().to_string(),
                        gate.inputs[0].clone(),
                        gate.identifier.as_str().to_string(),
                    ));
                    break;
                }
            }
            found
        };
        match opportunity {
            Some((output_id, input_reference, gate_id)) => {
                working.redirect(&output_id, &input_reference);
                working.remove_gate(&gate_id);
                Ok(Some(working.into_circuit()?))
            }
            None => Ok(None),
        }
    }
}

/// E.1 — common subexpression elimination: two gates with the same kind and
/// the same inputs (commutatively, for AND/OR) compute the same value, so the
/// later one is redirected to the earlier one and removed.
pub struct CommonSubexpressionElimination;

impl Transform for CommonSubexpressionElimination {
    fn id(&self) -> TransformId {
        TransformId::CommonSubexpressionElimination
    }

    fn apply_once(&self, circuit: &Circuit) -> Result<Option<Circuit>, OptError> {
        let mut working = Working::from_circuit(circuit);
        let opportunity = {
            let mut seen: HashMap<(GateKind, Vec<String>), String> = HashMap::new();
            let mut found = None;
            for gate in working.gates() {
                let key = (gate.kind, normalized_inputs(gate));
                if let Some(keeper_output) = seen.get(&key) {
                    found = Some((
                        gate.output.as_str().to_string(),
                        keeper_output.clone(),
                        gate.identifier.as_str().to_string(),
                    ));
                    break;
                }
                seen.insert(key, gate.output.as_str().to_string());
            }
            found
        };
        match opportunity {
            Some((duplicate_output, keeper_output, duplicate_gate_id)) => {
                let to = SignalReference::InternalSignal(SignalIdentifier::new(keeper_output)?);
                working.redirect(&duplicate_output, &to);
                working.remove_gate(&duplicate_gate_id);
                Ok(Some(working.into_circuit()?))
            }
            None => Ok(None),
        }
    }
}

fn reference_key(reference: &SignalReference) -> String {
    match reference {
        SignalReference::Port(port) => format!("p:{}", port.as_str()),
        SignalReference::InternalSignal(signal) => format!("s:{}", signal.as_str()),
    }
}

/// Inputs as a sorted key, so `AND(a, b)` and `AND(b, a)` match (AND/OR are
/// commutative; NOT has a single input so order is irrelevant).
fn normalized_inputs(gate: &Gate) -> Vec<String> {
    let mut keys: Vec<String> = gate.inputs.iter().map(reference_key).collect();
    keys.sort();
    keys
}
