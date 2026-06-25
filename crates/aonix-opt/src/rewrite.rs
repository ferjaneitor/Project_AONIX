//! Mutable working copy of a circuit's components, plus the graph-rewrite
//! primitives shared by the transformations.
//!
//! `Circuit` is immutable, so a transformation extracts its parts into a
//! [`Working`], mutates them, and rebuilds via `CircuitBuilder` (which re-runs
//! all structural validation — a free safety net).

use std::collections::{BTreeMap, BTreeSet};

use aonix_core::circuit_model::{
    Circuit, CircuitBuilder, Gate, Port, PortIdentifier, Signal, SignalReference,
};

use crate::error::OptError;

pub(crate) struct Working {
    inputs: Vec<Port>,
    outputs: Vec<Port>,
    signals: Vec<Signal>,
    gates: Vec<Gate>,
    assignments: Vec<(PortIdentifier, SignalReference)>,
}

impl Working {
    pub(crate) fn from_circuit(circuit: &Circuit) -> Self {
        let assignments = circuit
            .outputs_in_order()
            .iter()
            .filter_map(|port| {
                circuit
                    .output_assignment(&port.identifier)
                    .map(|source| (port.identifier.clone(), source.clone()))
            })
            .collect();
        Self {
            inputs: circuit.inputs_in_order().to_vec(),
            outputs: circuit.outputs_in_order().to_vec(),
            signals: circuit.signals().cloned().collect(),
            gates: circuit.gates().cloned().collect(),
            assignments,
        }
    }

    pub(crate) fn into_circuit(self) -> Result<Circuit, OptError> {
        let mut builder = CircuitBuilder::new();
        for port in self.inputs {
            builder.add_input_port(port)?;
        }
        for port in self.outputs {
            builder.add_output_port(port)?;
        }
        for signal in self.signals {
            builder.add_signal(signal)?;
        }
        for gate in self.gates {
            builder.add_gate(gate)?;
        }
        for (port, source) in self.assignments {
            builder.assign_output(port, source)?;
        }
        Ok(builder.finish()?)
    }

    /// Read-only view of the gates, in canonical (identifier) order.
    pub(crate) fn gates(&self) -> &[Gate] {
        &self.gates
    }

    /// Map from an internal signal id to the gate that produces it.
    pub(crate) fn producers(&self) -> BTreeMap<&str, &Gate> {
        self.gates
            .iter()
            .map(|gate| (gate.output.as_str(), gate))
            .collect()
    }

    /// The set of internal-signal ids reachable backward from any output.
    pub(crate) fn reachable_signal_ids(&self) -> BTreeSet<String> {
        let producers = self.producers();
        let mut reachable: BTreeSet<String> = BTreeSet::new();
        let mut stack: Vec<String> = Vec::new();
        for (_, source) in &self.assignments {
            if let SignalReference::InternalSignal(signal) = source {
                if reachable.insert(signal.as_str().to_string()) {
                    stack.push(signal.as_str().to_string());
                }
            }
        }
        while let Some(signal_id) = stack.pop() {
            if let Some(gate) = producers.get(signal_id.as_str()) {
                for input in &gate.inputs {
                    if let SignalReference::InternalSignal(source) = input {
                        if reachable.insert(source.as_str().to_string()) {
                            stack.push(source.as_str().to_string());
                        }
                    }
                }
            }
        }
        reachable
    }

    /// Replaces every reference to the internal signal `from_signal_id` (in
    /// gate inputs and output assignments) with `to`.
    pub(crate) fn redirect(&mut self, from_signal_id: &str, to: &SignalReference) {
        let is_from = |reference: &SignalReference| {
            matches!(reference, SignalReference::InternalSignal(signal) if signal.as_str() == from_signal_id)
        };
        for gate in &mut self.gates {
            for input in &mut gate.inputs {
                if is_from(input) {
                    *input = to.clone();
                }
            }
        }
        for (_, source) in &mut self.assignments {
            if is_from(source) {
                *source = to.clone();
            }
        }
    }

    /// Removes the gate with the given identifier.
    pub(crate) fn remove_gate(&mut self, gate_id: &str) {
        self.gates.retain(|gate| gate.identifier.as_str() != gate_id);
    }

    /// Keeps only the gates and signals whose ids are in `reachable`.
    pub(crate) fn retain_reachable(&mut self, reachable: &BTreeSet<String>) {
        self.gates.retain(|gate| reachable.contains(gate.output.as_str()));
        self.signals.retain(|signal| reachable.contains(signal.identifier.as_str()));
    }

    /// Removes every signal and gate unreachable from any output. Returns
    /// `true` if anything was removed.
    pub(crate) fn eliminate_dead(&mut self) -> bool {
        let reachable = self.reachable_signal_ids();
        let before = (self.gates.len(), self.signals.len());
        self.retain_reachable(&reachable);
        before != (self.gates.len(), self.signals.len())
    }
}
