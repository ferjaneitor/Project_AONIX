//! Integration tests for Sub-phase 1.C.1 — semantic groups preserved in
//! the canonical `Circuit`.
//!
//! These tests confirm the parser propagates `[[semantic_groups]]` into
//! the model (so the Sub-phase 1.D writer can regenerate a valid
//! `.aoncir`) and that the document-level validator remains the authority
//! for consistency (invalid groups still fail).

use std::path::Path;

use aonix::circuit_model::{AonixError, Circuit, SemanticGroupKind};
use aonix::format::aoncir;

fn load_valid(file_name: &str) -> Circuit {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join(file_name);
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
    aoncir::parse(&raw).unwrap_or_else(|error| panic!("parse failed for {file_name}: {error}"))
}

fn try_parse_invalid(file_name: &str) -> Result<(), AonixError> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("invalid")
        .join(file_name);
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
    aoncir::parse(&raw).map(|_circuit| ())
}

#[test]
fn parser_preserves_semantic_groups_in_circuit() {
    let circuit = load_valid("bus_passthrough_two_bit.aoncir");
    assert_eq!(circuit.semantic_group_count(), 2);
}

#[test]
fn circuit_exposes_groups_in_deterministic_declared_order() {
    let circuit = load_valid("bus_passthrough_two_bit.aoncir");
    let ids: Vec<&str> = circuit
        .semantic_groups()
        .iter()
        .map(|group| group.identifier.as_str())
        .collect();
    // Declared order in the .aoncir: operand_bus first, then result_bus.
    assert_eq!(ids, ["operand_bus", "result_bus"]);
}

#[test]
fn bus_group_preserves_width() {
    let circuit = load_valid("bus_passthrough_two_bit.aoncir");
    let result_bus = circuit
        .semantic_groups()
        .iter()
        .find(|group| group.identifier.as_str() == "result_bus")
        .expect("result_bus present");
    assert_eq!(result_bus.kind, SemanticGroupKind::Bus);
    assert_eq!(result_bus.width, 2);
    assert_eq!(result_bus.members.len(), 2);
}

#[test]
fn operand_group_preserves_members_in_order() {
    let circuit = load_valid("bus_passthrough_two_bit.aoncir");
    let operand_bus = circuit
        .semantic_groups()
        .iter()
        .find(|group| group.identifier.as_str() == "operand_bus")
        .expect("operand_bus present");
    assert_eq!(operand_bus.kind, SemanticGroupKind::Operand);
    let members: Vec<&str> = operand_bus
        .members
        .iter()
        .map(|member| member.as_str())
        .collect();
    assert_eq!(members, ["operand_bit_zero", "operand_bit_one"]);
}

#[test]
fn circuit_without_groups_has_zero_groups() {
    let circuit = load_valid("multiplexer_2_to_1.aoncir");
    assert_eq!(circuit.semantic_group_count(), 0);
    assert!(circuit.semantic_groups().is_empty());
}

// The validator remains the authority: invalid groups must still fail,
// exactly as in Sub-phase 1.C (no regression introduced by storing them).

#[test]
fn group_with_missing_member_still_fails() {
    assert!(matches!(
        try_parse_invalid("group_member_not_found.aoncir"),
        Err(AonixError::GroupMemberNotFound { .. })
    ));
}

#[test]
fn group_with_duplicate_bit_position_still_fails() {
    assert!(matches!(
        try_parse_invalid("group_duplicate_bit_position.aoncir"),
        Err(AonixError::DuplicateBitPosition { .. })
    ));
}

#[test]
fn group_with_noncontiguous_bit_position_still_fails() {
    assert!(matches!(
        try_parse_invalid("group_noncontiguous_bit_position.aoncir"),
        Err(AonixError::NonContiguousBitPosition { .. })
    ));
}

#[test]
fn group_with_unknown_kind_still_fails() {
    assert!(matches!(
        try_parse_invalid("group_unknown_kind.aoncir"),
        Err(AonixError::UnknownGroupKind { .. })
    ));
}
