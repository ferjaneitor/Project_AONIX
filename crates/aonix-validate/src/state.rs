//! The partial circuit under construction and the validation rules over it.
//!
//! [`BuildState`] mirrors what [`aonix_core::circuit_model::CircuitBuilder`]
//! will eventually accumulate, but is mutable and queryable so that each
//! agent action can be validated *before* it is applied (incremental
//! construction). The 10 validator rules of `docs/08-actions-and-rewards.md`
//! are enforced by [`BuildState::validate`].

use std::collections::{BTreeMap, BTreeSet};

use aonix_core::circuit_model::{
    AonixResult, Circuit, CircuitBuilder, Gate, GateIdentifier, GateKind, GroupIdentifier, Port,
    PortIdentifier, SemanticTag, Signal, SignalIdentifier, SignalReference,
};

use crate::action::{Action, ActionKind};
use crate::validate::ValidationError;

#[derive(Debug, Clone)]
struct SignalSpec {
    id: String,
    semantic_tag: Option<SemanticTag>,
    group: Option<String>,
}

#[derive(Debug, Clone)]
struct GateRecord {
    kind: GateKind,
    inputs: Vec<String>,
    output: String,
}

/// The partial circuit being built. Seeded with the task's input and output
/// ports; the agent then declares signals, creates gates and assigns outputs.
#[derive(Debug, Clone)]
pub struct BuildState {
    input_ports: Vec<Port>,
    output_ports: Vec<Port>,
    input_names: BTreeSet<String>,
    output_names: BTreeSet<String>,
    signals: Vec<SignalSpec>,
    signal_names: BTreeSet<String>,
    gates: BTreeMap<String, GateRecord>,
    assignments: BTreeMap<String, String>,
    used_ids: BTreeSet<String>,
}

impl BuildState {
    /// Creates a build state for a task whose interface is the given input
    /// and output ports (their declaration order is preserved as the formal
    /// I/O-vector contract).
    pub fn new(input_ports: Vec<Port>, output_ports: Vec<Port>) -> Self {
        let mut input_names = BTreeSet::new();
        let mut output_names = BTreeSet::new();
        let mut used_ids = BTreeSet::new();
        for port in &input_ports {
            let name = port.identifier.as_str().to_string();
            input_names.insert(name.clone());
            used_ids.insert(name);
        }
        for port in &output_ports {
            let name = port.identifier.as_str().to_string();
            output_names.insert(name.clone());
            used_ids.insert(name);
        }
        Self {
            input_ports,
            output_ports,
            input_names,
            output_names,
            signals: Vec::new(),
            signal_names: BTreeSet::new(),
            gates: BTreeMap::new(),
            assignments: BTreeMap::new(),
            used_ids,
        }
    }

    // --- counts (read-only introspection) ---

    /// Number of declared internal signals.
    pub fn signal_count(&self) -> usize {
        self.signals.len()
    }
    /// Number of gates created so far.
    pub fn gate_count(&self) -> usize {
        self.gates.len()
    }
    /// Number of output ports already assigned.
    pub fn assignment_count(&self) -> usize {
        self.assignments.len()
    }

    // --- queries ---

    fn is_input_port(&self, name: &str) -> bool {
        self.input_names.contains(name)
    }
    fn is_output_port(&self, name: &str) -> bool {
        self.output_names.contains(name)
    }
    fn is_signal(&self, name: &str) -> bool {
        self.signal_names.contains(name)
    }
    fn id_in_use(&self, id: &str) -> bool {
        self.used_ids.contains(id)
    }
    fn referenceable_source(&self, name: &str) -> bool {
        self.is_input_port(name) || self.is_signal(name)
    }
    fn signal_is_produced(&self, signal: &str) -> bool {
        self.gates.values().any(|gate| gate.output.as_str() == signal)
    }
    fn signal_is_consumed(&self, signal: &str) -> bool {
        self.gates
            .values()
            .any(|gate| gate.inputs.iter().any(|input| input.as_str() == signal))
            || self.assignments.values().any(|source| source.as_str() == signal)
    }

    fn producer_of(&self, signal: &str) -> Option<&GateRecord> {
        self.gates.values().find(|gate| gate.output.as_str() == signal)
    }
    fn would_create_cycle(&self, inputs: &[String], output: &str) -> bool {
        inputs
            .iter()
            .any(|input| self.reaches(input, output, &mut BTreeSet::new()))
    }
    fn reaches(&self, from: &str, target: &str, visited: &mut BTreeSet<String>) -> bool {
        if from == target {
            return true;
        }
        if !visited.insert(from.to_string()) {
            return false;
        }
        match self.producer_of(from) {
            Some(gate) => gate
                .inputs
                .iter()
                .any(|input| self.reaches(input, target, visited)),
            None => false,
        }
    }

    // --- validation (the 10 rules of docs/08) ---

    /// Checks whether `action` is legal in the current state, returning the
    /// precise [`ValidationError`] otherwise. Does not mutate.
    pub fn validate(&self, action: &Action) -> Result<(), ValidationError> {
        match action {
            Action::DeclareSignal { id, .. } => {
                validate_identifier(id)?;
                if self.id_in_use(id) {
                    return Err(ValidationError::DuplicateIdentifier { id: id.clone() });
                }
                Ok(())
            }
            Action::CreateGate {
                id,
                kind,
                inputs,
                output,
            } => {
                validate_identifier(id)?;
                if self.id_in_use(id) {
                    return Err(ValidationError::DuplicateIdentifier { id: id.clone() });
                }
                let expected = match kind {
                    GateKind::Not => 1,
                    GateKind::And | GateKind::Or => 2,
                };
                if inputs.len() != expected {
                    return Err(ValidationError::InvalidArity {
                        kind: kind.canonical_name(),
                        given: inputs.len(),
                        expected: arity_description(*kind),
                    });
                }
                if !self.is_signal(output) {
                    return Err(ValidationError::GateOutputNotDeclaredSignal {
                        signal: output.clone(),
                    });
                }
                if self.signal_is_produced(output) {
                    return Err(ValidationError::SignalAlreadyProduced {
                        signal: output.clone(),
                    });
                }
                for input in inputs {
                    if self.is_output_port(input) {
                        return Err(ValidationError::GateInputReferencesOutputPort {
                            port: input.clone(),
                        });
                    }
                    if !self.referenceable_source(input) {
                        return Err(ValidationError::UndefinedReference {
                            reference: input.clone(),
                        });
                    }
                }
                if inputs.iter().any(|input| input == output) {
                    return Err(ValidationError::SelfLoop { gate: id.clone() });
                }
                if self.would_create_cycle(inputs, output) {
                    return Err(ValidationError::CycleIntroduced { gate: id.clone() });
                }
                Ok(())
            }
            Action::AssignOutput { port, source } => {
                if !self.is_output_port(port) {
                    return Err(ValidationError::UnknownOutputPort { port: port.clone() });
                }
                if self.assignments.contains_key(port) {
                    return Err(ValidationError::DuplicateOutputAssignment { port: port.clone() });
                }
                if !self.referenceable_source(source) {
                    return Err(ValidationError::UndefinedReference {
                        reference: source.clone(),
                    });
                }
                Ok(())
            }
            Action::DeleteSignal { id } => {
                if !self.is_signal(id) {
                    return Err(ValidationError::UndefinedReference { reference: id.clone() });
                }
                if self.signal_is_produced(id) || self.signal_is_consumed(id) {
                    return Err(ValidationError::SignalNotDead { id: id.clone() });
                }
                Ok(())
            }
            Action::DeleteGate { id } => {
                let gate = self
                    .gates
                    .get(id)
                    .ok_or_else(|| ValidationError::UnknownGate { id: id.clone() })?;
                if self.signal_is_consumed(&gate.output) {
                    return Err(ValidationError::GateOutputStillConsumed {
                        gate: id.clone(),
                        signal: gate.output.clone(),
                    });
                }
                Ok(())
            }
            Action::StopConstruction => Ok(()),
        }
    }

    /// Validates `action` and, if legal, applies it to the state.
    pub fn apply(&mut self, action: &Action) -> Result<(), ValidationError> {
        self.validate(action)?;
        match action {
            Action::DeclareSignal {
                id,
                semantic_tag,
                group,
            } => {
                self.signals.push(SignalSpec {
                    id: id.clone(),
                    semantic_tag: *semantic_tag,
                    group: group.clone(),
                });
                self.signal_names.insert(id.clone());
                self.used_ids.insert(id.clone());
            }
            Action::CreateGate {
                id,
                kind,
                inputs,
                output,
            } => {
                self.gates.insert(
                    id.clone(),
                    GateRecord {
                        kind: *kind,
                        inputs: inputs.clone(),
                        output: output.clone(),
                    },
                );
                self.used_ids.insert(id.clone());
            }
            Action::AssignOutput { port, source } => {
                self.assignments.insert(port.clone(), source.clone());
            }
            Action::DeleteSignal { id } => {
                self.signals.retain(|signal| &signal.id != id);
                self.signal_names.remove(id);
                self.used_ids.remove(id);
            }
            Action::DeleteGate { id } => {
                self.gates.remove(id);
                self.used_ids.remove(id);
            }
            Action::StopConstruction => {}
        }
        Ok(())
    }

    /// Coarse enumeration of which action categories are currently legal.
    /// Concrete actions are unbounded (the agent chooses identifiers), so
    /// this reports the *kinds* available rather than every instance.
    pub fn legal_action_kinds(&self) -> Vec<ActionKind> {
        let mut kinds = vec![ActionKind::DeclareSignal, ActionKind::StopConstruction];
        let has_source = !self.input_names.is_empty() || !self.signal_names.is_empty();
        let has_free_signal = self
            .signals
            .iter()
            .any(|signal| !self.signal_is_produced(&signal.id));
        if has_source && has_free_signal {
            kinds.push(ActionKind::CreateGate);
        }
        let has_unassigned_output = self
            .output_ports
            .iter()
            .any(|port| !self.assignments.contains_key(port.identifier.as_str()));
        if has_source && has_unassigned_output {
            kinds.push(ActionKind::AssignOutput);
        }
        if self
            .signals
            .iter()
            .any(|signal| !self.signal_is_produced(&signal.id) && !self.signal_is_consumed(&signal.id))
        {
            kinds.push(ActionKind::DeleteSignal);
        }
        if self
            .gates
            .values()
            .any(|gate| !self.signal_is_consumed(&gate.output))
        {
            kinds.push(ActionKind::DeleteGate);
        }
        kinds
    }

    /// Builds the finished, immutable circuit, running the full structural
    /// validation of [`CircuitBuilder::finish`] as defense in depth. The
    /// result is what the verifier consumes.
    pub fn finalize(&self) -> AonixResult<Circuit> {
        let mut builder = CircuitBuilder::new();
        for port in &self.input_ports {
            builder.add_input_port(port.clone())?;
        }
        for port in &self.output_ports {
            builder.add_output_port(port.clone())?;
        }
        for signal in &self.signals {
            let group = match &signal.group {
                Some(group) => Some(GroupIdentifier::new(group.clone())?),
                None => None,
            };
            builder.add_signal(Signal::new(
                SignalIdentifier::new(signal.id.clone())?,
                signal.semantic_tag,
                group,
            ))?;
        }
        for (id, record) in &self.gates {
            let inputs = record
                .inputs
                .iter()
                .map(|name| self.resolve_reference(name))
                .collect::<AonixResult<Vec<SignalReference>>>()?;
            let gate = Gate::new(
                GateIdentifier::new(id.clone())?,
                record.kind,
                inputs,
                SignalIdentifier::new(record.output.clone())?,
            )?;
            builder.add_gate(gate)?;
        }
        for (port, source) in &self.assignments {
            builder.assign_output(PortIdentifier::new(port.clone())?, self.resolve_reference(source)?)?;
        }
        builder.finish()
    }

    fn resolve_reference(&self, name: &str) -> AonixResult<SignalReference> {
        if self.is_input_port(name) {
            Ok(SignalReference::Port(PortIdentifier::new(name.to_string())?))
        } else {
            Ok(SignalReference::InternalSignal(SignalIdentifier::new(
                name.to_string(),
            )?))
        }
    }
}

fn validate_identifier(id: &str) -> Result<(), ValidationError> {
    SignalIdentifier::new(id)
        .map(|_| ())
        .map_err(|_| ValidationError::InvalidIdentifier { id: id.to_string() })
}

fn arity_description(kind: GateKind) -> &'static str {
    match kind {
        GateKind::Not => "exactly 1",
        GateKind::And | GateKind::Or => "exactly 2",
    }
}

#[cfg(test)]
mod tests {
    use super::BuildState;
    use crate::action::{Action, ActionKind};
    use crate::validate::ValidationError;
    use aonix_core::circuit_model::{GateKind, Port, PortIdentifier, PortRole};

    fn input_port(name: &str) -> Port {
        Port::new(PortIdentifier::new(name).expect("id"), PortRole::Input, None, None, None)
    }
    fn output_port(name: &str) -> Port {
        Port::new(PortIdentifier::new(name).expect("id"), PortRole::Output, None, None, None)
    }
    fn declare(id: &str) -> Action {
        Action::DeclareSignal { id: id.to_string(), semantic_tag: None, group: None }
    }
    fn state_one_in_one_out() -> BuildState {
        BuildState::new(vec![input_port("operand_a")], vec![output_port("result")])
    }

    #[test]
    fn duplicate_signal_identifier_is_rejected() {
        let mut state = state_one_in_one_out();
        state.apply(&declare("intermediate")).expect("first ok");
        let result = state.validate(&declare("intermediate"));
        assert!(matches!(result, Err(ValidationError::DuplicateIdentifier { id }) if id == "intermediate"));
    }

    #[test]
    fn signal_id_colliding_with_port_is_rejected() {
        let state = state_one_in_one_out();
        let result = state.validate(&declare("operand_a"));
        assert!(matches!(result, Err(ValidationError::DuplicateIdentifier { .. })));
    }

    #[test]
    fn wrong_gate_arity_is_rejected() {
        let mut state = state_one_in_one_out();
        state.apply(&declare("out_signal")).unwrap();
        // AND with a single input.
        let gate = Action::CreateGate {
            id: "g_bad".into(),
            kind: GateKind::And,
            inputs: vec!["operand_a".into()],
            output: "out_signal".into(),
        };
        let result = state.validate(&gate);
        assert!(matches!(
            result,
            Err(ValidationError::InvalidArity { kind: "AND", given: 1, expected: "exactly 2" })
        ));
    }

    #[test]
    fn gate_with_undefined_input_is_rejected() {
        let mut state = state_one_in_one_out();
        state.apply(&declare("out_signal")).unwrap();
        let gate = Action::create_gate("g_not", "NOT", vec!["ghost".into()], "out_signal").unwrap();
        let result = state.validate(&gate);
        assert!(matches!(result, Err(ValidationError::UndefinedReference { reference }) if reference == "ghost"));
    }

    #[test]
    fn gate_writing_to_undeclared_signal_is_rejected() {
        let state = state_one_in_one_out();
        let gate = Action::create_gate("g_not", "NOT", vec!["operand_a".into()], "undeclared").unwrap();
        let result = state.validate(&gate);
        assert!(matches!(result, Err(ValidationError::GateOutputNotDeclaredSignal { signal }) if signal == "undeclared"));
    }

    #[test]
    fn gate_input_reading_output_port_is_rejected() {
        let mut state = state_one_in_one_out();
        state.apply(&declare("out_signal")).unwrap();
        let gate = Action::create_gate("g_not", "NOT", vec!["result".into()], "out_signal").unwrap();
        let result = state.validate(&gate);
        assert!(matches!(result, Err(ValidationError::GateInputReferencesOutputPort { port }) if port == "result"));
    }

    #[test]
    fn self_loop_is_rejected() {
        let mut state = state_one_in_one_out();
        state.apply(&declare("loop_signal")).unwrap();
        let gate = Action::create_gate("g_not", "NOT", vec!["loop_signal".into()], "loop_signal").unwrap();
        let result = state.validate(&gate);
        assert!(matches!(result, Err(ValidationError::SelfLoop { gate }) if gate == "g_not"));
    }

    #[test]
    fn assignment_to_unknown_port_is_rejected() {
        let mut state = state_one_in_one_out();
        state.apply(&declare("out_signal")).unwrap();
        let result = state.validate(&Action::AssignOutput {
            port: "ghost_port".into(),
            source: "out_signal".into(),
        });
        assert!(matches!(result, Err(ValidationError::UnknownOutputPort { port }) if port == "ghost_port"));
    }

    #[test]
    fn duplicate_output_assignment_is_rejected() {
        let mut state = state_one_in_one_out();
        state.apply(&declare("out_signal")).unwrap();
        state.apply(&declare("other_signal")).unwrap();
        state
            .apply(&Action::AssignOutput { port: "result".into(), source: "out_signal".into() })
            .expect("first assignment ok");
        let result = state.validate(&Action::AssignOutput {
            port: "result".into(),
            source: "other_signal".into(),
        });
        assert!(matches!(result, Err(ValidationError::DuplicateOutputAssignment { .. })));
    }

    #[test]
    fn legal_action_kinds_includes_stop_and_declare_initially() {
        let state = state_one_in_one_out();
        let kinds = state.legal_action_kinds();
        assert!(kinds.contains(&ActionKind::DeclareSignal));
        assert!(kinds.contains(&ActionKind::StopConstruction));
    }
}

