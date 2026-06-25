//! Phase 6 integration: the structural optimizer on real circuits.
//!
//! Acceptance (`docs/11-roadmap.md` Phase 6 / `docs/15`): a circuit with known
//! redundancy is reduced to a measurable improvement without breaking
//! behaviour, and the result contains only AND/OR/NOT (R2, `docs/23` P.1–P.7).

use std::path::Path;

use aonix::circuit_model::{
    Circuit, CircuitBuilder, Gate, GateIdentifier, GateKind, Port, PortIdentifier, PortRole, Signal,
    SignalIdentifier, SignalReference,
};
use aonix::format::aoncir::{hash_canonical, parse};
use aonix::opt::optimize;

fn load_aoncir(file_name: &str) -> Circuit {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join(file_name);
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
    parse(&raw).unwrap_or_else(|e| panic!("parse {file_name}: {e}"))
}

#[test]
fn redundant_full_adder_optimizes_to_canonical() {
    let redundant = load_aoncir("one_bit_full_adder_redundant.aoncir");
    let canonical = load_aoncir("one_bit_full_adder.aoncir");

    let report = optimize(&redundant).expect("optimize succeeds");

    assert!(report.improved, "removing the dead gate is a strict improvement");
    assert_eq!(report.metrics_final.gate_count_total, 13);
    assert_eq!(report.metrics_final.dead_signals, 0);
    // Dropping the dead gate + signal yields exactly the canonical adder.
    assert_eq!(
        hash_canonical(&report.circuit),
        hash_canonical(&canonical),
        "optimized redundant adder is structurally the canonical adder",
    );
}

#[test]
fn optimization_produces_only_and_or_not() {
    let report = optimize(&load_aoncir("one_bit_full_adder_redundant.aoncir")).unwrap();
    for gate in report.circuit.gates() {
        assert!(
            matches!(gate.kind, GateKind::And | GateKind::Or | GateKind::Not),
            "R2: every gate stays AND/OR/NOT",
        );
    }
}

#[test]
fn common_subexpression_is_merged() {
    // out1 = AND(a,b); out2 = AND(a,b) — two identical subexpressions.
    let mut builder = CircuitBuilder::new();
    let input = |name: &str| Port::new(PortIdentifier::new(name).unwrap(), PortRole::Input, None, None, None);
    let output = |name: &str| Port::new(PortIdentifier::new(name).unwrap(), PortRole::Output, None, None, None);
    builder.add_input_port(input("a")).unwrap();
    builder.add_input_port(input("b")).unwrap();
    builder.add_output_port(output("out_one")).unwrap();
    builder.add_output_port(output("out_two")).unwrap();
    builder.add_signal(Signal::new(SignalIdentifier::new("first").unwrap(), None, None)).unwrap();
    builder.add_signal(Signal::new(SignalIdentifier::new("second").unwrap(), None, None)).unwrap();
    let and = |id: &str, out: &str| {
        Gate::new(
            GateIdentifier::new(id).unwrap(),
            GateKind::And,
            vec![
                SignalReference::Port(PortIdentifier::new("a").unwrap()),
                SignalReference::Port(PortIdentifier::new("b").unwrap()),
            ],
            SignalIdentifier::new(out).unwrap(),
        )
        .unwrap()
    };
    builder.add_gate(and("g_first", "first")).unwrap();
    builder.add_gate(and("g_second", "second")).unwrap();
    builder
        .assign_output(PortIdentifier::new("out_one").unwrap(), SignalReference::InternalSignal(SignalIdentifier::new("first").unwrap()))
        .unwrap();
    builder
        .assign_output(PortIdentifier::new("out_two").unwrap(), SignalReference::InternalSignal(SignalIdentifier::new("second").unwrap()))
        .unwrap();
    let circuit = builder.finish().unwrap();

    let report = optimize(&circuit).expect("optimize succeeds");
    assert!(report.improved);
    assert_eq!(report.metrics_initial.gate_count_total, 2);
    assert_eq!(report.metrics_final.gate_count_total, 1, "the duplicate AND is merged by CSE");
}
