//! Integration tests for Sub-phase 1.D — the `.aoncir` writer and its
//! round-trip equivalence with the parser.
//!
//! The contract is **structural equivalence of the circuit**, not byte
//! equality: `parse(write(parse(file))) == parse(file)`. The `[meta]`
//! section is neutral scaffolding not stored in `Circuit`, so it is not
//! compared.

use std::path::Path;

use aonix::circuit_model::Circuit;
use aonix::format::aoncir;

fn load(file_name: &str) -> Circuit {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join(file_name);
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
    aoncir::parse(&raw).unwrap_or_else(|error| panic!("parse failed for {file_name}: {error}"))
}

/// parse -> write -> parse, asserting structural equivalence.
fn assert_roundtrip_stable(file_name: &str) {
    let original = load(file_name);
    let written = aoncir::write(&original);
    let reparsed = aoncir::parse(&written).unwrap_or_else(|error| {
        panic!("writer output for {file_name} failed to reparse: {error}\n---\n{written}")
    });
    assert_eq!(
        original, reparsed,
        "round-trip changed the circuit for {file_name}"
    );
}

#[test]
fn roundtrip_pass_through_stable() {
    assert_roundtrip_stable("pass_through.aoncir");
}

#[test]
fn roundtrip_inverter_stable() {
    assert_roundtrip_stable("inverter.aoncir");
}

#[test]
fn roundtrip_two_input_and_stable() {
    assert_roundtrip_stable("two_input_and.aoncir");
}

#[test]
fn roundtrip_two_input_or_stable() {
    assert_roundtrip_stable("two_input_or.aoncir");
}

#[test]
fn roundtrip_multiplexer_2_to_1_stable() {
    assert_roundtrip_stable("multiplexer_2_to_1.aoncir");
}

#[test]
fn roundtrip_one_bit_full_adder_stable() {
    assert_roundtrip_stable("one_bit_full_adder.aoncir");
}

#[test]
fn roundtrip_preserves_port_declaration_order() {
    let original = load("multiplexer_2_to_1.aoncir");
    let reparsed = aoncir::parse(&aoncir::write(&original)).expect("reparses");
    let original_inputs: Vec<&str> = original
        .inputs_in_order()
        .iter()
        .map(|p| p.identifier.as_str())
        .collect();
    let reparsed_inputs: Vec<&str> = reparsed
        .inputs_in_order()
        .iter()
        .map(|p| p.identifier.as_str())
        .collect();
    assert_eq!(
        original_inputs,
        ["data_input_zero", "data_input_one", "select_input"]
    );
    assert_eq!(original_inputs, reparsed_inputs);

    let original_outputs: Vec<&str> = original
        .outputs_in_order()
        .iter()
        .map(|p| p.identifier.as_str())
        .collect();
    let reparsed_outputs: Vec<&str> = reparsed
        .outputs_in_order()
        .iter()
        .map(|p| p.identifier.as_str())
        .collect();
    assert_eq!(original_outputs, reparsed_outputs);
}

#[test]
fn roundtrip_preserves_semantic_tags() {
    let original = load("multiplexer_2_to_1.aoncir");
    let reparsed = aoncir::parse(&aoncir::write(&original)).expect("reparses");
    for (a, b) in original
        .inputs_in_order()
        .iter()
        .zip(reparsed.inputs_in_order())
    {
        assert_eq!(a.semantic_tag, b.semantic_tag);
    }
    // select_input must keep its Select tag through the round-trip.
    let select = reparsed
        .inputs_in_order()
        .iter()
        .find(|p| p.identifier.as_str() == "select_input")
        .expect("select_input present");
    assert_eq!(
        select.semantic_tag,
        Some(aonix::circuit_model::SemanticTag::Select)
    );
}

#[test]
fn roundtrip_preserves_groups() {
    let original = load("bus_passthrough_two_bit.aoncir");
    let reparsed = aoncir::parse(&aoncir::write(&original)).expect("reparses");
    assert_eq!(
        original.semantic_groups(),
        reparsed.semantic_groups(),
        "semantic groups must survive the round-trip"
    );
    // And the whole circuit is equivalent.
    assert_eq!(original, reparsed);
}

#[test]
fn roundtrip_preserves_bit_position() {
    let original = load("bus_passthrough_two_bit.aoncir");
    let reparsed = aoncir::parse(&aoncir::write(&original)).expect("reparses");
    for (a, b) in original
        .inputs_in_order()
        .iter()
        .zip(reparsed.inputs_in_order())
    {
        assert_eq!(a.bit_position, b.bit_position);
    }
    let bit_zero = reparsed
        .inputs_in_order()
        .iter()
        .find(|p| p.identifier.as_str() == "operand_bit_zero")
        .expect("operand_bit_zero present");
    assert_eq!(bit_zero.bit_position, Some(0)); // LSB survives
}

#[test]
fn roundtrip_normalizes_gate_order_deterministically() {
    let original = load("multiplexer_2_to_1.aoncir");
    let written_once = aoncir::write(&original);
    let written_twice = aoncir::write(&original);
    // Deterministic output.
    assert_eq!(written_once, written_twice);

    // Gate ids appear in ascending lexicographic order in the output.
    let mut gate_ids_in_output: Vec<&str> = Vec::new();
    for line in written_once.lines() {
        if let Some(rest) = line.strip_prefix("id = \"") {
            if let Some(id) = rest.strip_suffix('"') {
                // Only collect gate ids: they live right after a
                // "[[gates]]" header. We approximate by collecting every
                // id and then filtering to those that are gate ids.
                gate_ids_in_output.push(id);
            }
        }
    }
    // Extract just the gate identifiers from the circuit and check the
    // output order equals their sorted order.
    let mut expected: Vec<&str> =
        original.gates().map(|g| g.identifier.as_str()).collect();
    expected.sort_unstable();
    let observed_gate_ids: Vec<&str> = gate_ids_in_output
        .into_iter()
        .filter(|id| expected.contains(id))
        .collect();
    assert_eq!(observed_gate_ids, expected);

    // And reparsing still yields an equivalent circuit.
    let reparsed = aoncir::parse(&written_once).expect("reparses");
    assert_eq!(original, reparsed);
}

#[test]
fn writer_output_rejects_no_rules_when_reparsed() {
    // Every fixture's writer output must reparse without any error.
    for file in [
        "pass_through.aoncir",
        "inverter.aoncir",
        "two_input_and.aoncir",
        "two_input_or.aoncir",
        "multiplexer_2_to_1.aoncir",
        "one_bit_full_adder.aoncir",
        "bus_passthrough_two_bit.aoncir",
    ] {
        let original = load(file);
        let written = aoncir::write(&original);
        let reparsed = aoncir::parse(&written);
        assert!(
            reparsed.is_ok(),
            "writer output for {file} was rejected on reparse: {:?}",
            reparsed.err()
        );
    }
}

#[test]
fn writer_does_not_emit_auxiliary_opaque_sections() {
    for file in [
        "multiplexer_2_to_1.aoncir",
        "one_bit_full_adder.aoncir",
        "bus_passthrough_two_bit.aoncir",
    ] {
        let written = aoncir::write(&load(file));
        assert!(!written.contains("[verification]"), "{file}");
        assert!(!written.contains("[metrics]"), "{file}");
        assert!(!written.contains("[layout]"), "{file}");
        assert!(!written.contains("[history]"), "{file}");
    }
}

#[test]
fn writer_never_emits_forbidden_gate_kinds() {
    // bus_passthrough has NOT gates; full_adder has AND/OR/NOT; mux too.
    for file in [
        "multiplexer_2_to_1.aoncir",
        "one_bit_full_adder.aoncir",
        "bus_passthrough_two_bit.aoncir",
    ] {
        let original = load(file);
        let written = aoncir::write(&original);
        // Every `kind = "..."` that appears *inside a [[gates]] block*
        // must be AND, OR or NOT. We track the current section so we do
        // not confuse a gate kind with a [[semantic_groups]] kind (which
        // is legitimately "operand", "bus", etc.).
        let mut in_gates_block = false;
        for line in written.lines() {
            let trimmed = line.trim();
            if trimmed == "[[gates]]" {
                in_gates_block = true;
                continue;
            }
            if trimmed.starts_with("[[") || trimmed.starts_with('[') {
                in_gates_block = false;
                continue;
            }
            if in_gates_block {
                if let Some(rest) = trimmed.strip_prefix("kind = \"") {
                    if let Some(kind) = rest.strip_suffix('"') {
                        assert!(
                            matches!(kind, "AND" | "OR" | "NOT"),
                            "forbidden gate kind {kind:?} emitted for {file}"
                        );
                    }
                }
            }
        }
        // Structural confirmation: reparse and check every gate kind.
        let reparsed = aoncir::parse(&written).expect("reparses");
        for gate in reparsed.gates() {
            assert!(matches!(
                gate.kind.canonical_name(),
                "AND" | "OR" | "NOT"
            ));
        }
    }
}

#[test]
fn writer_never_emits_primitive_constants() {
    // SignalReference is a closed two-variant enum (Port, InternalSignal);
    // there is no constant variant, so the writer structurally cannot
    // emit "0"/"1" as a gate input or output source. We assert it on the
    // reparsed circuit: every gate input and every output source resolves
    // to an identifier that starts with a lowercase letter (snake_case),
    // never the literal "0" or "1".
    for file in [
        "pass_through.aoncir",
        "inverter.aoncir",
        "multiplexer_2_to_1.aoncir",
        "one_bit_full_adder.aoncir",
        "bus_passthrough_two_bit.aoncir",
    ] {
        let original = load(file);
        let reparsed = aoncir::parse(&aoncir::write(&original)).expect("reparses");
        for gate in reparsed.gates() {
            for input in &gate.inputs {
                let name = match input {
                    aonix::circuit_model::SignalReference::Port(p) => p.as_str(),
                    aonix::circuit_model::SignalReference::InternalSignal(s) => s.as_str(),
                };
                assert_ne!(name, "0", "{file}");
                assert_ne!(name, "1", "{file}");
                assert!(
                    name.chars().next().is_some_and(|c| c.is_ascii_lowercase()),
                    "input {name:?} is not a snake_case identifier in {file}"
                );
            }
        }
    }
}
