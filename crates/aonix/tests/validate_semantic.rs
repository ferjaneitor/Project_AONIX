//! Integration tests for the document-level validator (Sub-phase 1.C).
//!
//! Each fixture in `tests/data/invalid/` isolates **one** violated rule
//! and the test asserts the parser surfaces the specific [`AonixError`]
//! variant with the expected payload.

use std::path::Path;

use aonix::circuit_model::AonixError;
use aonix::format::aoncir;

fn try_parse(file_name: &str) -> Result<(), AonixError> {
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
fn rejects_group_duplicate_id() {
    match try_parse("group_duplicate_id.aoncir") {
        Err(AonixError::DuplicateIdentifier { identifier, scope }) => {
            assert_eq!(identifier, "operand_bus");
            assert_eq!(scope, "semantic_groups");
        }
        other => panic!("expected DuplicateIdentifier, got {other:?}"),
    }
}

#[test]
fn rejects_group_unknown_kind() {
    match try_parse("group_unknown_kind.aoncir") {
        Err(AonixError::UnknownGroupKind { group, kind }) => {
            assert_eq!(group, "operand_bus");
            assert_eq!(kind, "megabus");
        }
        other => panic!("expected UnknownGroupKind, got {other:?}"),
    }
}

#[test]
fn rejects_group_member_not_found() {
    match try_parse("group_member_not_found.aoncir") {
        Err(AonixError::GroupMemberNotFound { group, member }) => {
            assert_eq!(group, "operand_bus");
            assert_eq!(member, "operand_bit_ghost");
        }
        other => panic!("expected GroupMemberNotFound, got {other:?}"),
    }
}

#[test]
fn rejects_group_width_mismatch() {
    match try_parse("group_width_mismatch.aoncir") {
        Err(AonixError::GroupWidthMismatch {
            group,
            declared,
            actual,
        }) => {
            assert_eq!(group, "operand_bus");
            assert_eq!(declared, 3);
            assert_eq!(actual, 2);
        }
        other => panic!("expected GroupWidthMismatch, got {other:?}"),
    }
}

#[test]
fn rejects_group_missing_bit_position() {
    match try_parse("group_missing_bit_position.aoncir") {
        Err(AonixError::MissingBitPosition { group, member }) => {
            assert_eq!(group, "operand_bus");
            assert_eq!(member, "operand_bit_one");
        }
        other => panic!("expected MissingBitPosition, got {other:?}"),
    }
}

#[test]
fn rejects_group_duplicate_bit_position() {
    match try_parse("group_duplicate_bit_position.aoncir") {
        Err(AonixError::DuplicateBitPosition { group, position }) => {
            assert_eq!(group, "operand_bus");
            assert_eq!(position, 0);
        }
        other => panic!("expected DuplicateBitPosition, got {other:?}"),
    }
}

#[test]
fn rejects_group_noncontiguous_bit_position() {
    match try_parse("group_noncontiguous_bit_position.aoncir") {
        Err(AonixError::NonContiguousBitPosition { group, detail }) => {
            assert_eq!(group, "operand_bus");
            assert!(
                detail.contains("contiguous"),
                "detail should explain contiguity; got: {detail}"
            );
        }
        other => panic!("expected NonContiguousBitPosition, got {other:?}"),
    }
}

#[test]
fn rejects_group_undeclared_reference() {
    match try_parse("group_undeclared_reference.aoncir") {
        Err(AonixError::UndeclaredGroupReference {
            referenced_by,
            group,
        }) => {
            assert_eq!(referenced_by, "operand_bit_zero");
            assert_eq!(group, "ghost_bus");
        }
        other => panic!("expected UndeclaredGroupReference, got {other:?}"),
    }
}

#[test]
fn rejects_group_tag_inconsistency() {
    match try_parse("group_tag_inconsistency.aoncir") {
        Err(AonixError::GroupTagInconsistency {
            group,
            member,
            tag,
            group_kind,
        }) => {
            assert_eq!(group, "status_flags");
            assert_eq!(member, "zero_indicator");
            assert_eq!(tag, "data_bit");
            assert_eq!(group_kind, "flags");
        }
        other => panic!("expected GroupTagInconsistency, got {other:?}"),
    }
}

#[test]
fn rejects_gate_input_referencing_output_port() {
    match try_parse("gate_input_references_output_port.aoncir") {
        Err(AonixError::GateInputReferencesOutputPort { gate, output_port }) => {
            assert_eq!(gate, "g_bad");
            assert_eq!(output_port, "data_output");
        }
        other => panic!("expected GateInputReferencesOutputPort, got {other:?}"),
    }
}

#[test]
fn rejects_gate_output_colliding_with_port() {
    match try_parse("gate_output_collides_with_port.aoncir") {
        Err(AonixError::GateOutputCollidesWithPort { gate, port }) => {
            assert_eq!(gate, "g_invert");
            assert_eq!(port, "data_output");
        }
        other => panic!("expected GateOutputCollidesWithPort, got {other:?}"),
    }
}
