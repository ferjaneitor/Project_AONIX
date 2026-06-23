//! Structural metrics of a [`Circuit`] and how they are computed.

use std::collections::{BTreeMap, BTreeSet};

use aonix_core::circuit_model::{Circuit, Gate, GateKind, SignalReference};

/// Configurable weights for the aggregate cost. All "lower is better".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CostWeights {
    /// Weight on the total gate count.
    pub gate: u64,
    /// Weight on the logical depth.
    pub depth: u64,
    /// Weight (penalty) on dead signals.
    pub dead_signal: u64,
    /// Weight (penalty) on the maximum fan-out.
    pub fan_out: u64,
}

impl Default for CostWeights {
    /// Gate count and depth weigh 1 each, dead signals are penalised heavily
    /// (they must be 0 in an official-active circuit), fan-out is not
    /// penalised by default.
    fn default() -> Self {
        Self {
            gate: 1,
            depth: 1,
            dead_signal: 10,
            fan_out: 0,
        }
    }
}

/// Structural quality metrics of a circuit. Pure function of the circuit's
/// topology, hence deterministic and reproducible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Metrics {
    /// Total number of gates.
    pub gate_count_total: usize,
    /// Number of AND gates.
    pub and_count: usize,
    /// Number of OR gates.
    pub or_count: usize,
    /// Number of NOT gates.
    pub not_count: usize,
    /// Number of declared internal signals.
    pub signal_count: usize,
    /// Logical depth: the longest path, in gates, from an input port to an
    /// output port (0 if every output is a direct pass-through of an input).
    pub depth: usize,
    /// Declared internal signals not reachable backward from any output.
    pub dead_signals: usize,
    /// Maximum gate fan-in (1 for NOT, 2 for AND/OR in Phase 1).
    pub max_fan_in: usize,
    /// Maximum fan-out of any input port or internal signal (number of
    /// consumers: gate inputs plus output assignments).
    pub max_fan_out: usize,
    /// Number of internal signals consumed by two or more consumers — a
    /// proxy for subexpression sharing / reuse. Higher is better.
    pub shared_signal_count: usize,
    /// Weighted aggregate cost (see [`CostWeights`]). Lower is better.
    pub aggregate_cost: u64,
}

/// Evaluates `circuit` with the [default weights](CostWeights::default).
pub fn evaluate(circuit: &Circuit) -> Metrics {
    evaluate_with_weights(circuit, &CostWeights::default())
}

/// Evaluates `circuit`, using `weights` for the aggregate cost only (every
/// other metric is independent of the weights).
pub fn evaluate_with_weights(circuit: &Circuit, weights: &CostWeights) -> Metrics {
    let mut and_count = 0;
    let mut or_count = 0;
    let mut not_count = 0;
    for gate in circuit.gates() {
        match gate.kind {
            GateKind::And => and_count += 1,
            GateKind::Or => or_count += 1,
            GateKind::Not => not_count += 1,
        }
    }
    let gate_count_total = and_count + or_count + not_count;

    // signal name -> producing gate.
    let mut producer: BTreeMap<&str, &Gate> = BTreeMap::new();
    for gate in circuit.gates() {
        producer.insert(gate.output.as_str(), gate);
    }

    // Memoised gate levels for the depth (critical path).
    let mut levels: BTreeMap<&str, usize> = BTreeMap::new();
    for gate in circuit.gates() {
        gate_level(gate, &producer, &mut levels);
    }
    let mut depth = 0;
    for port in circuit.outputs_in_order() {
        if let Some(SignalReference::InternalSignal(signal)) =
            circuit.output_assignment(&port.identifier)
        {
            if let Some(&level) = levels.get(signal.as_str()) {
                depth = depth.max(level);
            }
        }
    }

    // Dead signals: declared signals not reachable backward from any output.
    let reachable = reachable_signals(circuit, &producer);
    let dead_signals = circuit
        .signals()
        .filter(|signal| !reachable.contains(signal.identifier.as_str()))
        .count();

    // Fan-out per referenceable name (gate inputs + output assignments).
    let mut fan_out: BTreeMap<&str, usize> = BTreeMap::new();
    for gate in circuit.gates() {
        for input in &gate.inputs {
            *fan_out.entry(reference_name(input)).or_insert(0) += 1;
        }
    }
    for port in circuit.outputs_in_order() {
        if let Some(source) = circuit.output_assignment(&port.identifier) {
            *fan_out.entry(reference_name(source)).or_insert(0) += 1;
        }
    }
    let max_fan_out = fan_out.values().copied().max().unwrap_or(0);
    let shared_signal_count = circuit
        .signals()
        .filter(|signal| fan_out.get(signal.identifier.as_str()).copied().unwrap_or(0) >= 2)
        .count();

    let max_fan_in = circuit.gates().map(|gate| gate.inputs.len()).max().unwrap_or(0);

    let aggregate_cost = weights.gate * gate_count_total as u64
        + weights.depth * depth as u64
        + weights.dead_signal * dead_signals as u64
        + weights.fan_out * max_fan_out as u64;

    Metrics {
        gate_count_total,
        and_count,
        or_count,
        not_count,
        signal_count: circuit.signal_count(),
        depth,
        dead_signals,
        max_fan_in,
        max_fan_out,
        shared_signal_count,
        aggregate_cost,
    }
}

fn gate_level<'a>(
    gate: &'a Gate,
    producer: &BTreeMap<&'a str, &'a Gate>,
    levels: &mut BTreeMap<&'a str, usize>,
) -> usize {
    if let Some(&level) = levels.get(gate.output.as_str()) {
        return level;
    }
    let mut max_input_level = 0;
    for input in &gate.inputs {
        if let SignalReference::InternalSignal(signal) = input {
            if let Some(producing) = producer.get(signal.as_str()) {
                max_input_level = max_input_level.max(gate_level(producing, producer, levels));
            }
        }
    }
    let level = 1 + max_input_level;
    levels.insert(gate.output.as_str(), level);
    level
}

fn reachable_signals<'a>(
    circuit: &'a Circuit,
    producer: &BTreeMap<&'a str, &'a Gate>,
) -> BTreeSet<&'a str> {
    let mut reachable: BTreeSet<&str> = BTreeSet::new();
    let mut stack: Vec<&str> = Vec::new();
    for port in circuit.outputs_in_order() {
        if let Some(SignalReference::InternalSignal(signal)) =
            circuit.output_assignment(&port.identifier)
        {
            if reachable.insert(signal.as_str()) {
                stack.push(signal.as_str());
            }
        }
    }
    while let Some(signal) = stack.pop() {
        if let Some(gate) = producer.get(signal) {
            for input in &gate.inputs {
                if let SignalReference::InternalSignal(source) = input {
                    if reachable.insert(source.as_str()) {
                        stack.push(source.as_str());
                    }
                }
            }
        }
    }
    reachable
}

fn reference_name(reference: &SignalReference) -> &str {
    match reference {
        SignalReference::Port(port) => port.as_str(),
        SignalReference::InternalSignal(signal) => signal.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aonix_core::circuit_model::{
        CircuitBuilder, Gate, GateIdentifier, GateKind, Port, PortIdentifier, PortRole, Signal,
        SignalIdentifier, SignalReference,
    };

    fn inverter() -> aonix_core::circuit_model::Circuit {
        let mut builder = CircuitBuilder::new();
        builder
            .add_input_port(Port::new(
                PortIdentifier::new("data_input").unwrap(),
                PortRole::Input,
                None,
                None,
                None,
            ))
            .unwrap();
        builder
            .add_output_port(Port::new(
                PortIdentifier::new("data_output").unwrap(),
                PortRole::Output,
                None,
                None,
                None,
            ))
            .unwrap();
        builder
            .add_signal(Signal::new(SignalIdentifier::new("inner").unwrap(), None, None))
            .unwrap();
        builder
            .add_gate(
                Gate::new(
                    GateIdentifier::new("g_not").unwrap(),
                    GateKind::Not,
                    vec![SignalReference::Port(PortIdentifier::new("data_input").unwrap())],
                    SignalIdentifier::new("inner").unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
        builder
            .assign_output(
                PortIdentifier::new("data_output").unwrap(),
                SignalReference::InternalSignal(SignalIdentifier::new("inner").unwrap()),
            )
            .unwrap();
        builder.finish().unwrap()
    }

    #[test]
    fn inverter_metrics_are_minimal() {
        let metrics = evaluate(&inverter());
        assert_eq!(metrics.gate_count_total, 1);
        assert_eq!(metrics.not_count, 1);
        assert_eq!(metrics.and_count, 0);
        assert_eq!(metrics.or_count, 0);
        assert_eq!(metrics.depth, 1);
        assert_eq!(metrics.dead_signals, 0);
        assert_eq!(metrics.max_fan_in, 1);
        assert_eq!(metrics.signal_count, 1);
    }

    #[test]
    fn evaluation_is_reproducible() {
        let circuit = inverter();
        assert_eq!(evaluate(&circuit), evaluate(&circuit));
    }
}
