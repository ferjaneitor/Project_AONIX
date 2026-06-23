//! Integration tests for the `.aoncir` parser on **valid** inputs.
//!
//! Each test loads a `.aoncir` fixture from `tests/data/` and asserts a
//! specific property of the resulting [`aonix::circuit_model::Circuit`].

use std::path::Path;

use aonix::circuit_model::{Circuit, PortRole, SemanticTag};
use aonix::format::aoncir;

fn load_aoncir(file_name: &str) -> Circuit {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join(file_name);
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
    aoncir::parse(&raw).unwrap_or_else(|error| panic!("parse failed for {file_name}: {error}"))
}

#[test]
fn parses_inverter() {
    let circuit = load_aoncir("inverter.aoncir");
    assert_eq!(circuit.input_count(), 1);
    assert_eq!(circuit.output_count(), 1);
    assert_eq!(circuit.gate_count(), 1);
    assert_eq!(circuit.signal_count(), 1);
}

#[test]
fn parses_two_input_and() {
    let circuit = load_aoncir("two_input_and.aoncir");
    assert_eq!(circuit.input_count(), 2);
    assert_eq!(circuit.output_count(), 1);
    assert_eq!(circuit.gate_count(), 1);
    let gate = circuit.gates().next().expect("one gate present");
    assert_eq!(gate.kind.canonical_name(), "AND");
    assert_eq!(gate.inputs.len(), 2);
}

#[test]
fn parses_two_input_or() {
    let circuit = load_aoncir("two_input_or.aoncir");
    let gate = circuit.gates().next().expect("one gate present");
    assert_eq!(gate.kind.canonical_name(), "OR");
    assert_eq!(gate.inputs.len(), 2);
}

#[test]
fn parses_multiplexer_2_to_1() {
    let circuit = load_aoncir("multiplexer_2_to_1.aoncir");
    assert_eq!(circuit.input_count(), 3);
    assert_eq!(circuit.output_count(), 1);
    assert_eq!(circuit.gate_count(), 4);
    assert_eq!(circuit.signal_count(), 4);
}

#[test]
fn preserves_input_port_declaration_order_in_multiplexer() {
    let circuit = load_aoncir("multiplexer_2_to_1.aoncir");
    let names: Vec<&str> = circuit
        .inputs_in_order()
        .iter()
        .map(|port| port.identifier.as_str())
        .collect();
    assert_eq!(names, ["data_input_zero", "data_input_one", "select_input"]);
}

#[test]
fn preserves_output_port_declaration_order_in_multiplexer() {
    let circuit = load_aoncir("multiplexer_2_to_1.aoncir");
    let names: Vec<&str> = circuit
        .outputs_in_order()
        .iter()
        .map(|port| port.identifier.as_str())
        .collect();
    assert_eq!(names, ["data_output"]);
}

#[test]
fn input_ports_carry_correct_semantic_tags_in_multiplexer() {
    let circuit = load_aoncir("multiplexer_2_to_1.aoncir");
    let inputs = circuit.inputs_in_order();
    assert_eq!(inputs[0].semantic_tag, Some(SemanticTag::DataBit));
    assert_eq!(inputs[1].semantic_tag, Some(SemanticTag::DataBit));
    assert_eq!(inputs[2].semantic_tag, Some(SemanticTag::Select));
}

#[test]
fn ports_have_expected_role() {
    let circuit = load_aoncir("multiplexer_2_to_1.aoncir");
    for port in circuit.inputs_in_order() {
        assert_eq!(port.role, PortRole::Input);
    }
    for port in circuit.outputs_in_order() {
        assert_eq!(port.role, PortRole::Output);
    }
}

#[test]
fn inverter_uses_only_not_gate() {
    let circuit = load_aoncir("inverter.aoncir");
    for gate in circuit.gates() {
        assert_eq!(gate.kind.canonical_name(), "NOT");
    }
}

#[test]
fn multiplexer_uses_only_and_or_not() {
    let circuit = load_aoncir("multiplexer_2_to_1.aoncir");
    for gate in circuit.gates() {
        let name = gate.kind.canonical_name();
        assert!(
            matches!(name, "AND" | "OR" | "NOT"),
            "unexpected gate kind {name}"
        );
    }
}

#[test]
fn parses_bus_passthrough_with_semantic_groups() {
    // The validator (Sub-phase 1.C) must accept a well-formed document
    // with two ordered semantic groups whose bit_positions are unique
    // and contiguous from 0 (LSB-first).
    let circuit = load_aoncir("bus_passthrough_two_bit.aoncir");
    assert_eq!(circuit.input_count(), 2);
    assert_eq!(circuit.output_count(), 2);
    assert_eq!(circuit.gate_count(), 2);
    for gate in circuit.gates() {
        assert_eq!(gate.kind.canonical_name(), "NOT");
    }
}

#[test]
fn bus_passthrough_preserves_bit_position_lsb_first() {
    let circuit = load_aoncir("bus_passthrough_two_bit.aoncir");
    let inputs = circuit.inputs_in_order();
    assert_eq!(inputs[0].identifier.as_str(), "operand_bit_zero");
    assert_eq!(inputs[0].bit_position, Some(0)); // LSB
    assert_eq!(inputs[1].identifier.as_str(), "operand_bit_one");
    assert_eq!(inputs[1].bit_position, Some(1)); // MSB for width 2
}

#[test]
fn opaque_auxiliary_sections_do_not_alter_technical_truth() {
    // inverter.aoncir has no [verification]/[metrics]/[layout]/[history];
    // inverter_with_opaque_sections.aoncir has the same graph but those
    // sections filled with arbitrary, even contradictory, content. The
    // resulting Circuit must be identical: the opaque sections cannot
    // change the technical truth.
    let plain = load_aoncir("inverter.aoncir");
    let with_opaque = load_aoncir("inverter_with_opaque_sections.aoncir");
    assert_eq!(plain, with_opaque);
}
