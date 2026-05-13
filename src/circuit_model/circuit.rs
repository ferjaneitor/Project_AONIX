//! Whole circuit: ordered ports, internal signals, gates and output assignments.
//!
//! A [`Circuit`] is immutable. To construct one, use [`CircuitBuilder`] and
//! call [`CircuitBuilder::finish`]; the builder performs all structural
//! validations at that point:
//!
//! - identifiers unique across ports, signals and gates,
//! - no signal references to undeclared identifiers,
//! - every output port has exactly one source assignment,
//! - the gate graph is a DAG (no cycles),
//! - the list of output ports is non-empty.
//!
//! Per-gate kind and arity validation is enforced earlier, at
//! [`super::gate::Gate::new`], so by the time a gate reaches the builder
//! it is already valid in isolation. The builder only adds cross-cutting
//! checks.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::circuit_model::error::{AonixError, AonixResult};
use crate::circuit_model::gate::{Gate, GateIdentifier, SignalReference};
use crate::circuit_model::port::{Port, PortIdentifier, PortRole};
use crate::circuit_model::signal::{Signal, SignalIdentifier};

/// Immutable circuit ready for simulation, verification, evaluation,
/// visualization or hashing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Circuit {
    inputs_in_order: Vec<Port>,
    outputs_in_order: Vec<Port>,
    signals: BTreeMap<SignalIdentifier, Signal>,
    gates: BTreeMap<GateIdentifier, Gate>,
    output_assignments: BTreeMap<PortIdentifier, SignalReference>,
}

impl Circuit {
    /// Returns the input ports in the order declared in the source file.
    /// This order is the formal contract of the input vector.
    pub fn inputs_in_order(&self) -> &[Port] {
        &self.inputs_in_order
    }

    /// Returns the output ports in the order declared in the source file.
    /// This order is the formal contract of the output vector.
    pub fn outputs_in_order(&self) -> &[Port] {
        &self.outputs_in_order
    }

    /// Iterator over the internal signals declared in the circuit.
    pub fn signals(&self) -> impl Iterator<Item = &Signal> {
        self.signals.values()
    }

    /// Iterator over the gates of the circuit, sorted by gate identifier.
    /// The sort is deterministic; do not rely on it being topological.
    pub fn gates(&self) -> impl Iterator<Item = &Gate> {
        self.gates.values()
    }

    /// Looks up an internal signal by identifier.
    pub fn signal(&self, identifier: &SignalIdentifier) -> Option<&Signal> {
        self.signals.get(identifier)
    }

    /// Looks up a gate by identifier.
    pub fn gate(&self, identifier: &GateIdentifier) -> Option<&Gate> {
        self.gates.get(identifier)
    }

    /// Returns the source signal reference assigned to the given output port.
    pub fn output_assignment(&self, port: &PortIdentifier) -> Option<&SignalReference> {
        self.output_assignments.get(port)
    }

    /// Number of input ports.
    pub fn input_count(&self) -> usize {
        self.inputs_in_order.len()
    }

    /// Number of output ports.
    pub fn output_count(&self) -> usize {
        self.outputs_in_order.len()
    }

    /// Number of internal signals declared.
    pub fn signal_count(&self) -> usize {
        self.signals.len()
    }

    /// Number of gates.
    pub fn gate_count(&self) -> usize {
        self.gates.len()
    }
}

/// Builder for [`Circuit`]. Accumulates ports, signals, gates and output
/// assignments. Most local errors (duplicate identifier, wrong port role)
/// are reported eagerly; cross-cutting errors (cycles, undefined
/// references, missing output assignments) are reported by
/// [`CircuitBuilder::finish`].
#[derive(Debug, Default)]
pub struct CircuitBuilder {
    inputs_in_order: Vec<Port>,
    outputs_in_order: Vec<Port>,
    signals: BTreeMap<SignalIdentifier, Signal>,
    gates: BTreeMap<GateIdentifier, Gate>,
    output_assignments: BTreeMap<PortIdentifier, SignalReference>,
    all_identifiers: HashSet<String>,
}

impl CircuitBuilder {
    /// Creates an empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an input port. The port must have [`PortRole::Input`].
    /// Input ports are appended in order; that order is the contract of
    /// the future [`super::InputVector`].
    pub fn add_input_port(&mut self, port: Port) -> AonixResult<&mut Self> {
        if port.role != PortRole::Input {
            return Err(AonixError::PortRoleMismatch {
                port: port.identifier.as_str().to_string(),
                expected: PortRole::Input.canonical_name(),
                actual: port.role.canonical_name(),
            });
        }
        self.register_unique_identifier(port.identifier.as_str(), "ports")?;
        self.inputs_in_order.push(port);
        Ok(self)
    }

    /// Registers an output port. The port must have [`PortRole::Output`].
    /// Output ports are appended in order; that order is the contract of
    /// the future [`super::OutputVector`].
    pub fn add_output_port(&mut self, port: Port) -> AonixResult<&mut Self> {
        if port.role != PortRole::Output {
            return Err(AonixError::PortRoleMismatch {
                port: port.identifier.as_str().to_string(),
                expected: PortRole::Output.canonical_name(),
                actual: port.role.canonical_name(),
            });
        }
        self.register_unique_identifier(port.identifier.as_str(), "ports")?;
        self.outputs_in_order.push(port);
        Ok(self)
    }

    /// Registers an internal signal.
    pub fn add_signal(&mut self, signal: Signal) -> AonixResult<&mut Self> {
        self.register_unique_identifier(signal.identifier.as_str(), "signals")?;
        self.signals.insert(signal.identifier.clone(), signal);
        Ok(self)
    }

    /// Registers a gate. Gate arity has already been validated by
    /// [`Gate::new`]; here we only enforce identifier uniqueness.
    pub fn add_gate(&mut self, gate: Gate) -> AonixResult<&mut Self> {
        self.register_unique_identifier(gate.identifier.as_str(), "gates")?;
        self.gates.insert(gate.identifier.clone(), gate);
        Ok(self)
    }

    /// Assigns a source signal reference to an output port. Each output
    /// port may receive at most one assignment.
    pub fn assign_output(
        &mut self,
        port: PortIdentifier,
        source: SignalReference,
    ) -> AonixResult<&mut Self> {
        if self.output_assignments.contains_key(&port) {
            return Err(AonixError::DuplicateOutputAssignment {
                port: port.as_str().to_string(),
            });
        }
        self.output_assignments.insert(port, source);
        Ok(self)
    }

    /// Finalizes the builder, running all cross-cutting structural
    /// validations and returning the immutable [`Circuit`].
    pub fn finish(self) -> AonixResult<Circuit> {
        if self.outputs_in_order.is_empty() {
            return Err(AonixError::RequiredListEmpty {
                what: "ports.outputs",
            });
        }

        let known_ports: BTreeSet<&PortIdentifier> = self
            .inputs_in_order
            .iter()
            .map(|port| &port.identifier)
            .collect();
        let known_signals: BTreeSet<&SignalIdentifier> = self.signals.keys().collect();

        // 1. Every signal referenced by a gate must exist as port input,
        //    declared internal signal, or constant.
        for gate in self.gates.values() {
            for input in &gate.inputs {
                ensure_reference_known(input, &known_ports, &known_signals)?;
            }
        }

        // 2. Every output port has an assignment.
        for output_port in &self.outputs_in_order {
            let assignment = self.output_assignments.get(&output_port.identifier);
            let Some(reference) = assignment else {
                return Err(AonixError::UnassignedOutputPort {
                    port: output_port.identifier.as_str().to_string(),
                });
            };
            ensure_reference_known(reference, &known_ports, &known_signals)?;
        }

        // 3. Every output port declared must have been assigned (covered
        //    by 2). We also reject assignments to ports that are not
        //    declared as outputs.
        let declared_output_ports: BTreeSet<&PortIdentifier> = self
            .outputs_in_order
            .iter()
            .map(|port| &port.identifier)
            .collect();
        for assigned_port in self.output_assignments.keys() {
            if !declared_output_ports.contains(assigned_port) {
                return Err(AonixError::UndefinedIdentifier {
                    identifier: assigned_port.as_str().to_string(),
                });
            }
        }

        // 4. Gate output signals must be declared as internal signals.
        for gate in self.gates.values() {
            if !known_signals.contains(&gate.output) {
                return Err(AonixError::UndefinedIdentifier {
                    identifier: gate.output.as_str().to_string(),
                });
            }
        }

        // 5. No cycles in the gate graph. The graph edges go from each
        //    input signal reference of a gate to that gate's output
        //    signal. Constants and external input ports cannot start a
        //    cycle by definition.
        detect_cycle_or_pass(&self.gates)?;

        Ok(Circuit {
            inputs_in_order: self.inputs_in_order,
            outputs_in_order: self.outputs_in_order,
            signals: self.signals,
            gates: self.gates,
            output_assignments: self.output_assignments,
        })
    }

    fn register_unique_identifier(
        &mut self,
        identifier: &str,
        scope: &'static str,
    ) -> AonixResult<()> {
        if !self.all_identifiers.insert(identifier.to_string()) {
            return Err(AonixError::DuplicateIdentifier {
                identifier: identifier.to_string(),
                scope,
            });
        }
        Ok(())
    }
}

fn ensure_reference_known(
    reference: &SignalReference,
    known_ports: &BTreeSet<&PortIdentifier>,
    known_signals: &BTreeSet<&SignalIdentifier>,
) -> AonixResult<()> {
    match reference {
        SignalReference::Port(port) => {
            if !known_ports.contains(port) {
                return Err(AonixError::UndefinedIdentifier {
                    identifier: port.as_str().to_string(),
                });
            }
        }
        SignalReference::InternalSignal(signal) => {
            if !known_signals.contains(signal) {
                return Err(AonixError::UndefinedIdentifier {
                    identifier: signal.as_str().to_string(),
                });
            }
        }
    }
    Ok(())
}

/// Detects cycles in the gate graph using DFS with three colors
/// (white / gray / black). Constant and port-input references are leaves.
fn detect_cycle_or_pass(gates: &BTreeMap<GateIdentifier, Gate>) -> AonixResult<()> {
    let mut signal_to_producing_gate: BTreeMap<&SignalIdentifier, &GateIdentifier> = BTreeMap::new();
    for (gate_identifier, gate) in gates {
        signal_to_producing_gate.insert(&gate.output, gate_identifier);
    }

    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Color {
        White,
        Gray,
        Black,
    }
    let mut color: BTreeMap<&GateIdentifier, Color> = gates.keys().map(|k| (k, Color::White)).collect();

    fn visit<'a>(
        gate_identifier: &'a GateIdentifier,
        gates: &'a BTreeMap<GateIdentifier, Gate>,
        signal_to_producing_gate: &'a BTreeMap<&'a SignalIdentifier, &'a GateIdentifier>,
        color: &mut BTreeMap<&'a GateIdentifier, Color>,
    ) -> AonixResult<()> {
        match color.get(gate_identifier).copied().unwrap_or(Color::White) {
            Color::Black => return Ok(()),
            Color::Gray => {
                return Err(AonixError::CycleDetected {
                    gate: gate_identifier.as_str().to_string(),
                });
            }
            Color::White => {}
        }
        color.insert(gate_identifier, Color::Gray);
        let gate = gates
            .get(gate_identifier)
            .expect("gate_identifier comes from gates map");
        for input in &gate.inputs {
            if let SignalReference::InternalSignal(signal) = input {
                if let Some(producing_gate) = signal_to_producing_gate.get(signal) {
                    visit(*producing_gate, gates, signal_to_producing_gate, color)?;
                }
            }
        }
        color.insert(gate_identifier, Color::Black);
        Ok(())
    }

    for gate_identifier in gates.keys() {
        if matches!(
            color.get(gate_identifier).copied().unwrap_or(Color::White),
            Color::White
        ) {
            visit(gate_identifier, gates, &signal_to_producing_gate, &mut color)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit_model::gate::{Gate, GateIdentifier, GateKind, SignalReference};
    use crate::circuit_model::port::{Port, PortIdentifier, PortRole, SemanticTag};
    use crate::circuit_model::signal::{Signal, SignalIdentifier};

    fn make_input_port(name: &str, tag: Option<SemanticTag>) -> Port {
        Port::new(
            PortIdentifier::new(name).expect("valid port id"),
            PortRole::Input,
            tag,
            None,
            None,
        )
    }

    fn make_output_port(name: &str, tag: Option<SemanticTag>) -> Port {
        Port::new(
            PortIdentifier::new(name).expect("valid port id"),
            PortRole::Output,
            tag,
            None,
            None,
        )
    }

    fn make_signal(name: &str) -> Signal {
        Signal::new(SignalIdentifier::new(name).expect("valid signal id"), None, None)
    }

    fn make_signal_reference_for_port(name: &str) -> SignalReference {
        SignalReference::Port(PortIdentifier::new(name).expect("valid port id"))
    }

    fn make_signal_reference_for_signal(name: &str) -> SignalReference {
        SignalReference::InternalSignal(SignalIdentifier::new(name).expect("valid signal id"))
    }

    /// Builds the canonical `inverter` circuit: one input, one output, one NOT gate.
    fn build_inverter_circuit() -> AonixResult<Circuit> {
        let mut builder = CircuitBuilder::new();
        builder.add_input_port(make_input_port("data_input", Some(SemanticTag::DataBit)))?;
        builder.add_output_port(make_output_port("data_output", Some(SemanticTag::DataBit)))?;
        builder.add_signal(make_signal("data_output_internal"))?;
        let gate = Gate::new(
            GateIdentifier::new("g_invert").expect("valid"),
            GateKind::Not,
            vec![make_signal_reference_for_port("data_input")],
            SignalIdentifier::new("data_output_internal").expect("valid"),
        )?;
        builder.add_gate(gate)?;
        builder.assign_output(
            PortIdentifier::new("data_output").expect("valid"),
            make_signal_reference_for_signal("data_output_internal"),
        )?;
        builder.finish()
    }

    #[test]
    fn inverter_builds_and_exposes_ports_in_declared_order() {
        let circuit = build_inverter_circuit().expect("inverter is valid");
        assert_eq!(circuit.input_count(), 1);
        assert_eq!(circuit.output_count(), 1);
        assert_eq!(circuit.signal_count(), 1);
        assert_eq!(circuit.gate_count(), 1);
        assert_eq!(circuit.inputs_in_order()[0].identifier.as_str(), "data_input");
        assert_eq!(circuit.outputs_in_order()[0].identifier.as_str(), "data_output");
    }

    #[test]
    fn finish_fails_when_outputs_list_is_empty() {
        let mut builder = CircuitBuilder::new();
        builder
            .add_input_port(make_input_port("data_input", None))
            .expect("valid");
        let result = builder.finish();
        assert!(matches!(
            result,
            Err(AonixError::RequiredListEmpty { .. })
        ));
    }

    #[test]
    fn finish_fails_when_output_port_has_no_assignment() {
        let mut builder = CircuitBuilder::new();
        builder
            .add_input_port(make_input_port("data_input", None))
            .expect("valid");
        builder
            .add_output_port(make_output_port("data_output", None))
            .expect("valid");
        let result = builder.finish();
        assert!(matches!(
            result,
            Err(AonixError::UnassignedOutputPort { .. })
        ));
    }

    #[test]
    fn finish_fails_when_signal_reference_is_undefined() {
        let mut builder = CircuitBuilder::new();
        builder
            .add_input_port(make_input_port("data_input", None))
            .expect("valid");
        builder
            .add_output_port(make_output_port("data_output", None))
            .expect("valid");
        // The signal referenced as the gate input does not exist as a port
        // (typo: `data_input_typo`).
        builder
            .add_signal(make_signal("data_output_internal"))
            .expect("valid");
        let gate = Gate::new(
            GateIdentifier::new("g_invert").expect("valid"),
            GateKind::Not,
            vec![make_signal_reference_for_port("data_input_typo")],
            SignalIdentifier::new("data_output_internal").expect("valid"),
        )
        .expect("Gate::new only checks arity, reference validity is checked later");
        builder.add_gate(gate).expect("valid");
        builder
            .assign_output(
                PortIdentifier::new("data_output").expect("valid"),
                make_signal_reference_for_signal("data_output_internal"),
            )
            .expect("valid");
        let result = builder.finish();
        match result {
            Err(AonixError::UndefinedIdentifier { identifier }) => {
                assert_eq!(identifier, "data_input_typo");
            }
            other => panic!("expected UndefinedIdentifier, got {other:?}"),
        }
    }

    #[test]
    fn add_input_port_rejects_output_role() {
        let mut builder = CircuitBuilder::new();
        let wrong_role_port = Port::new(
            PortIdentifier::new("data_input").expect("valid"),
            PortRole::Output,
            None,
            None,
            None,
        );
        let result = builder.add_input_port(wrong_role_port);
        assert!(matches!(result, Err(AonixError::PortRoleMismatch { .. })));
    }

    #[test]
    fn add_output_port_rejects_input_role() {
        let mut builder = CircuitBuilder::new();
        let wrong_role_port = Port::new(
            PortIdentifier::new("data_output").expect("valid"),
            PortRole::Input,
            None,
            None,
            None,
        );
        let result = builder.add_output_port(wrong_role_port);
        assert!(matches!(result, Err(AonixError::PortRoleMismatch { .. })));
    }

    #[test]
    fn duplicate_identifier_across_scopes_is_rejected() {
        // Reusing `data_signal` as both a port and a signal identifier
        // must fail: identifiers are unique across the whole .aoncir.
        let mut builder = CircuitBuilder::new();
        builder
            .add_input_port(make_input_port("data_signal", None))
            .expect("first occurrence is valid");
        let result = builder.add_signal(make_signal("data_signal"));
        match result {
            Err(AonixError::DuplicateIdentifier { identifier, .. }) => {
                assert_eq!(identifier, "data_signal");
            }
            other => panic!("expected DuplicateIdentifier, got {other:?}"),
        }
    }

    #[test]
    fn duplicate_output_assignment_is_rejected() {
        let mut builder = CircuitBuilder::new();
        builder
            .add_output_port(make_output_port("data_output", None))
            .expect("valid");
        builder
            .add_signal(make_signal("source_signal"))
            .expect("valid");
        builder
            .add_signal(make_signal("alternative_source_signal"))
            .expect("valid");
        builder
            .assign_output(
                PortIdentifier::new("data_output").expect("valid"),
                make_signal_reference_for_signal("source_signal"),
            )
            .expect("first assignment is valid");
        let result = builder.assign_output(
            PortIdentifier::new("data_output").expect("valid"),
            make_signal_reference_for_signal("alternative_source_signal"),
        );
        assert!(matches!(
            result,
            Err(AonixError::DuplicateOutputAssignment { .. })
        ));
    }

    #[test]
    fn cycle_in_gate_graph_is_detected() {
        // Two gates mutually referencing each other through internal
        // signals: g_alpha consumes signal `beta_out`, g_beta consumes
        // signal `alpha_out`. Both gates produce signals that feed each
        // other — a clear combinational cycle.
        let mut builder = CircuitBuilder::new();
        builder
            .add_input_port(make_input_port("dummy_input", None))
            .expect("valid");
        builder
            .add_output_port(make_output_port("dummy_output", None))
            .expect("valid");
        builder
            .add_signal(make_signal("alpha_out"))
            .expect("valid");
        builder
            .add_signal(make_signal("beta_out"))
            .expect("valid");
        let gate_alpha = Gate::new(
            GateIdentifier::new("g_alpha").expect("valid"),
            GateKind::And,
            vec![
                make_signal_reference_for_signal("beta_out"),
                make_signal_reference_for_port("dummy_input"),
            ],
            SignalIdentifier::new("alpha_out").expect("valid"),
        )
        .expect("valid AND arity");
        let gate_beta = Gate::new(
            GateIdentifier::new("g_beta").expect("valid"),
            GateKind::And,
            vec![
                make_signal_reference_for_signal("alpha_out"),
                make_signal_reference_for_port("dummy_input"),
            ],
            SignalIdentifier::new("beta_out").expect("valid"),
        )
        .expect("valid AND arity");
        builder.add_gate(gate_alpha).expect("valid");
        builder.add_gate(gate_beta).expect("valid");
        builder
            .assign_output(
                PortIdentifier::new("dummy_output").expect("valid"),
                make_signal_reference_for_signal("alpha_out"),
            )
            .expect("valid");
        let result = builder.finish();
        assert!(matches!(result, Err(AonixError::CycleDetected { .. })));
    }

    #[test]
    fn gate_output_signal_must_be_declared() {
        let mut builder = CircuitBuilder::new();
        builder
            .add_input_port(make_input_port("data_input", None))
            .expect("valid");
        builder
            .add_output_port(make_output_port("data_output", None))
            .expect("valid");
        builder
            .add_signal(make_signal("declared_signal"))
            .expect("valid");
        // The gate writes to `undeclared_signal`, which was never added.
        let gate = Gate::new(
            GateIdentifier::new("g_invert").expect("valid"),
            GateKind::Not,
            vec![make_signal_reference_for_port("data_input")],
            SignalIdentifier::new("undeclared_signal").expect("valid"),
        )
        .expect("Gate::new only checks arity");
        builder.add_gate(gate).expect("valid");
        builder
            .assign_output(
                PortIdentifier::new("data_output").expect("valid"),
                make_signal_reference_for_signal("declared_signal"),
            )
            .expect("valid");
        let result = builder.finish();
        match result {
            Err(AonixError::UndefinedIdentifier { identifier }) => {
                assert_eq!(identifier, "undeclared_signal");
            }
            other => panic!("expected UndefinedIdentifier, got {other:?}"),
        }
    }

    #[test]
    fn assign_output_to_undeclared_port_is_rejected_on_finish() {
        let mut builder = CircuitBuilder::new();
        builder
            .add_output_port(make_output_port("data_output", None))
            .expect("valid");
        builder
            .add_signal(make_signal("source_signal"))
            .expect("valid");
        // The assignment targets `ghost_output`, which was never declared.
        builder
            .assign_output(
                PortIdentifier::new("ghost_output").expect("valid"),
                make_signal_reference_for_signal("source_signal"),
            )
            .expect("assignment accepted, validated at finish");
        // Even though there is an assignment, `data_output` does not have one,
        // so finish reports UnassignedOutputPort first.
        let result = builder.finish();
        assert!(matches!(
            result,
            Err(AonixError::UnassignedOutputPort { .. })
        ));
    }
}
