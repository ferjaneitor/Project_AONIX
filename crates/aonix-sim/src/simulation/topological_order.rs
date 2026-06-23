//! Deterministic topological ordering of the gates of a [`Circuit`].
//!
//! Algorithm: Kahn's algorithm with a `BTreeSet<&GateIdentifier>` as the
//! ready queue, which guarantees that ties are broken by lexicographic
//! order of [`GateIdentifier`]. This makes the output **strictly
//! deterministic**: same circuit ⇒ same order, every time.
//!
//! Cycle handling: [`aonix_core::circuit_model::CircuitBuilder::finish`]
//! already rejects circuits with cycles. This function is defensive
//! anyway: if it fails to drain every gate it returns
//! [`AonixError::CycleDetected`].
//!
//! [`Circuit`]: aonix_core::circuit_model::Circuit
//! [`GateIdentifier`]: aonix_core::circuit_model::GateIdentifier

use std::collections::{BTreeMap, BTreeSet};

use aonix_core::circuit_model::{
    AonixError, AonixResult, Circuit, GateIdentifier, SignalIdentifier, SignalReference,
};

/// Computes a deterministic topological order of the gates of `circuit`.
///
/// The returned `Vec<GateIdentifier>` contains every gate of the circuit
/// exactly once. A gate `B` appears after every gate `A` such that the
/// output signal of `A` is an input of `B`. Ties between gates that
/// become ready simultaneously are broken lexicographically by
/// [`GateIdentifier`].
///
/// # Errors
///
/// Returns [`AonixError::CycleDetected`] if the circuit graph contains a
/// cycle. In practice this should not happen for circuits built via
/// [`aonix_core::circuit_model::CircuitBuilder::finish`] or loaded through
/// the `.aoncir` parser, both of which already reject cycles; the check
/// is kept here as a defensive invariant.
pub fn compute_topological_order(circuit: &Circuit) -> AonixResult<Vec<GateIdentifier>> {
    // 1. producer map: signal -> gate that produces it.
    let mut producer: BTreeMap<&SignalIdentifier, &GateIdentifier> = BTreeMap::new();
    for gate in circuit.gates() {
        producer.insert(&gate.output, &gate.identifier);
    }

    // 2. Build the set of unique (producer_gate, consumer_gate) edges.
    //    Using a BTreeSet deduplicates the case where a gate consumes the
    //    same producer's output more than once (for example AND(s, s)).
    let mut edges: BTreeSet<(&GateIdentifier, &GateIdentifier)> = BTreeSet::new();
    for gate in circuit.gates() {
        for input in &gate.inputs {
            if let SignalReference::InternalSignal(signal) = input {
                if let Some(producer_gate) = producer.get(signal) {
                    edges.insert((*producer_gate, &gate.identifier));
                }
            }
        }
    }

    // 3. in_degree: number of unique producer dependencies per gate.
    let mut in_degree: BTreeMap<&GateIdentifier, usize> =
        circuit.gates().map(|gate| (&gate.identifier, 0)).collect();
    for (_, consumer) in &edges {
        if let Some(count) = in_degree.get_mut(consumer) {
            *count += 1;
        }
    }

    // 4. dependencies: for each producer, the list of consumers it
    //    unlocks when removed. Built from the same edges set so order is
    //    deterministic.
    let mut dependencies: BTreeMap<&GateIdentifier, Vec<&GateIdentifier>> = BTreeMap::new();
    for (producer_gate, consumer) in &edges {
        dependencies
            .entry(*producer_gate)
            .or_default()
            .push(*consumer);
    }

    // 5. ready set: gates with in_degree 0, ordered lexicographically.
    let mut ready: BTreeSet<&GateIdentifier> = in_degree
        .iter()
        .filter_map(|(id, &count)| if count == 0 { Some(*id) } else { None })
        .collect();

    let mut order: Vec<GateIdentifier> = Vec::with_capacity(circuit.gate_count());

    while let Some(next) = ready.iter().next().copied() {
        ready.remove(&next);
        order.push(next.clone());
        if let Some(consumers) = dependencies.get(&next) {
            for &consumer in consumers {
                if let Some(count) = in_degree.get_mut(&consumer) {
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        ready.insert(consumer);
                    }
                }
            }
        }
    }

    if order.len() != circuit.gate_count() {
        // Defensive: CircuitBuilder::finish should have caught this.
        let stuck = in_degree
            .iter()
            .find(|&(_, &count)| count > 0)
            .map(|(id, _)| id.as_str().to_string())
            .unwrap_or_else(|| "<unknown>".to_string());
        return Err(AonixError::CycleDetected { gate: stuck });
    }

    Ok(order)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aonix_core::circuit_model::{
        CircuitBuilder, Gate, GateKind, Port, PortIdentifier, PortRole, Signal, SignalIdentifier,
        SignalReference,
    };

    fn input_port(name: &str) -> Port {
        Port::new(
            PortIdentifier::new(name).expect("valid port id"),
            PortRole::Input,
            None,
            None,
            None,
        )
    }

    fn output_port(name: &str) -> Port {
        Port::new(
            PortIdentifier::new(name).expect("valid port id"),
            PortRole::Output,
            None,
            None,
            None,
        )
    }

    fn signal(name: &str) -> Signal {
        Signal::new(
            SignalIdentifier::new(name).expect("valid signal id"),
            None,
            None,
        )
    }

    fn port_ref(name: &str) -> SignalReference {
        SignalReference::Port(PortIdentifier::new(name).expect("valid port id"))
    }

    fn signal_ref(name: &str) -> SignalReference {
        SignalReference::InternalSignal(SignalIdentifier::new(name).expect("valid signal id"))
    }

    #[test]
    fn empty_gate_set_returns_empty_order() {
        let mut builder = CircuitBuilder::new();
        builder.add_input_port(input_port("data_input")).expect("ok");
        builder.add_output_port(output_port("data_output")).expect("ok");
        builder
            .assign_output(
                PortIdentifier::new("data_output").expect("ok"),
                port_ref("data_input"),
            )
            .expect("ok");
        let circuit = builder.finish().expect("ok");
        let order = compute_topological_order(&circuit).expect("ok");
        assert!(order.is_empty());
    }

    #[test]
    fn order_respects_signal_dependency() {
        // g_first writes intermediate; g_second reads intermediate.
        // g_first must come before g_second even though "g_second" sorts
        // after "g_first" lexicographically (consistent here, but the
        // dependency is what matters).
        let mut builder = CircuitBuilder::new();
        builder
            .add_input_port(input_port("operand_a"))
            .expect("ok");
        builder
            .add_input_port(input_port("operand_b"))
            .expect("ok");
        builder.add_output_port(output_port("result")).expect("ok");
        builder.add_signal(signal("intermediate")).expect("ok");
        builder.add_signal(signal("final_signal")).expect("ok");
        let g_first = Gate::new(
            aonix_core::circuit_model::GateIdentifier::new("g_first").expect("ok"),
            GateKind::And,
            vec![port_ref("operand_a"), port_ref("operand_b")],
            SignalIdentifier::new("intermediate").expect("ok"),
        )
        .expect("ok");
        let g_second = Gate::new(
            aonix_core::circuit_model::GateIdentifier::new("g_second").expect("ok"),
            GateKind::Not,
            vec![signal_ref("intermediate")],
            SignalIdentifier::new("final_signal").expect("ok"),
        )
        .expect("ok");
        builder.add_gate(g_first).expect("ok");
        builder.add_gate(g_second).expect("ok");
        builder
            .assign_output(
                PortIdentifier::new("result").expect("ok"),
                signal_ref("final_signal"),
            )
            .expect("ok");
        let circuit = builder.finish().expect("ok");
        let order = compute_topological_order(&circuit).expect("ok");
        let names: Vec<&str> = order.iter().map(|gate_id| gate_id.as_str()).collect();
        let pos_first = names.iter().position(|name| *name == "g_first").unwrap();
        let pos_second = names.iter().position(|name| *name == "g_second").unwrap();
        assert!(pos_first < pos_second, "g_first must come before g_second");
    }

    #[test]
    fn independent_gates_appear_in_lexicographic_order() {
        // Three independent NOT gates with names that, sorted lexically,
        // give: g_alpha, g_beta, g_gamma. Each consumes a separate input.
        // Topological order must follow lex order because there are no
        // dependencies between them.
        let mut builder = CircuitBuilder::new();
        builder.add_input_port(input_port("input_a")).expect("ok");
        builder.add_input_port(input_port("input_b")).expect("ok");
        builder.add_input_port(input_port("input_c")).expect("ok");
        builder.add_output_port(output_port("out_a")).expect("ok");
        builder.add_output_port(output_port("out_b")).expect("ok");
        builder.add_output_port(output_port("out_c")).expect("ok");
        builder.add_signal(signal("signal_a")).expect("ok");
        builder.add_signal(signal("signal_b")).expect("ok");
        builder.add_signal(signal("signal_c")).expect("ok");
        for (gate_name, input_port_name, signal_name) in [
            ("g_gamma", "input_c", "signal_c"),
            ("g_alpha", "input_a", "signal_a"),
            ("g_beta", "input_b", "signal_b"),
        ] {
            let gate = Gate::new(
                aonix_core::circuit_model::GateIdentifier::new(gate_name).expect("ok"),
                GateKind::Not,
                vec![port_ref(input_port_name)],
                SignalIdentifier::new(signal_name).expect("ok"),
            )
            .expect("ok");
            builder.add_gate(gate).expect("ok");
        }
        for (port_name, source) in
            [("out_a", "signal_a"), ("out_b", "signal_b"), ("out_c", "signal_c")]
        {
            builder
                .assign_output(
                    PortIdentifier::new(port_name).expect("ok"),
                    signal_ref(source),
                )
                .expect("ok");
        }
        let circuit = builder.finish().expect("ok");
        let order = compute_topological_order(&circuit).expect("ok");
        let names: Vec<&str> = order.iter().map(|id| id.as_str()).collect();
        assert_eq!(names, ["g_alpha", "g_beta", "g_gamma"]);
    }

    #[test]
    fn order_is_stable_across_runs() {
        // Build the same circuit twice and verify the order is identical.
        let build_circuit = || {
            let mut builder = CircuitBuilder::new();
            builder.add_input_port(input_port("input_a")).expect("ok");
            builder.add_input_port(input_port("input_b")).expect("ok");
            builder.add_output_port(output_port("result")).expect("ok");
            builder.add_signal(signal("intermediate")).expect("ok");
            builder.add_signal(signal("final_signal")).expect("ok");
            builder
                .add_gate(
                    Gate::new(
                        aonix_core::circuit_model::GateIdentifier::new("g_and").expect("ok"),
                        GateKind::And,
                        vec![port_ref("input_a"), port_ref("input_b")],
                        SignalIdentifier::new("intermediate").expect("ok"),
                    )
                    .expect("ok"),
                )
                .expect("ok");
            builder
                .add_gate(
                    Gate::new(
                        aonix_core::circuit_model::GateIdentifier::new("g_not").expect("ok"),
                        GateKind::Not,
                        vec![signal_ref("intermediate")],
                        SignalIdentifier::new("final_signal").expect("ok"),
                    )
                    .expect("ok"),
                )
                .expect("ok");
            builder
                .assign_output(
                    PortIdentifier::new("result").expect("ok"),
                    signal_ref("final_signal"),
                )
                .expect("ok");
            builder.finish().expect("ok")
        };
        let first_order = compute_topological_order(&build_circuit()).expect("ok");
        let second_order = compute_topological_order(&build_circuit()).expect("ok");
        assert_eq!(first_order, second_order);
    }
}
