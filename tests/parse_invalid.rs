//! Integration tests for the `.aoncir` parser on **invalid** inputs.
//!
//! Each test loads a `.aoncir` fixture from `tests/data/invalid/` and
//! asserts the parser returns a specific [`AonixError`] variant.

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
fn rejects_kind_xor_with_unknown_gate_kind_error() {
    let result = try_parse("kind_xor.aoncir");
    match result {
        Err(AonixError::UnknownGateKind { kind }) => assert_eq!(kind, "XOR"),
        other => panic!("expected UnknownGateKind, got {other:?}"),
    }
}

#[test]
fn rejects_kind_nand_with_unknown_gate_kind_error() {
    let result = try_parse("kind_nand.aoncir");
    match result {
        Err(AonixError::UnknownGateKind { kind }) => assert_eq!(kind, "NAND"),
        other => panic!("expected UnknownGateKind, got {other:?}"),
    }
}

#[test]
fn rejects_kind_nor_with_unknown_gate_kind_error() {
    let result = try_parse("kind_nor.aoncir");
    match result {
        Err(AonixError::UnknownGateKind { kind }) => assert_eq!(kind, "NOR"),
        other => panic!("expected UnknownGateKind, got {other:?}"),
    }
}

#[test]
fn rejects_kind_xnor_with_unknown_gate_kind_error() {
    let result = try_parse("kind_xnor.aoncir");
    match result {
        Err(AonixError::UnknownGateKind { kind }) => assert_eq!(kind, "XNOR"),
        other => panic!("expected UnknownGateKind, got {other:?}"),
    }
}

#[test]
fn rejects_and_with_three_inputs_with_invalid_arity_error() {
    let result = try_parse("arity_and_three_inputs.aoncir");
    match result {
        Err(AonixError::InvalidGateArity {
            kind,
            given,
            expected_description,
        }) => {
            assert_eq!(kind, "AND");
            assert_eq!(given, 3);
            assert_eq!(expected_description, "exactly 2");
        }
        other => panic!("expected InvalidGateArity, got {other:?}"),
    }
}

#[test]
fn rejects_or_with_three_inputs_with_invalid_arity_error() {
    let result = try_parse("arity_or_three_inputs.aoncir");
    match result {
        Err(AonixError::InvalidGateArity {
            kind,
            given,
            expected_description,
        }) => {
            assert_eq!(kind, "OR");
            assert_eq!(given, 3);
            assert_eq!(expected_description, "exactly 2");
        }
        other => panic!("expected InvalidGateArity, got {other:?}"),
    }
}

#[test]
fn rejects_not_with_two_inputs_with_invalid_arity_error() {
    let result = try_parse("arity_not_two_inputs.aoncir");
    match result {
        Err(AonixError::InvalidGateArity {
            kind,
            given,
            expected_description,
        }) => {
            assert_eq!(kind, "NOT");
            assert_eq!(given, 2);
            assert_eq!(expected_description, "exactly 1");
        }
        other => panic!("expected InvalidGateArity, got {other:?}"),
    }
}

#[test]
fn rejects_unsupported_format_version_with_specific_error() {
    let result = try_parse("unsupported_format_version.aoncir");
    match result {
        Err(AonixError::UnsupportedFormatVersion { found, expected }) => {
            assert_eq!(found, "0.9.0");
            assert_eq!(expected, "1.0.0");
        }
        other => panic!("expected UnsupportedFormatVersion, got {other:?}"),
    }
}

#[test]
fn rejects_dangling_signal_reference_with_undefined_identifier_error() {
    let result = try_parse("dangling_signal.aoncir");
    match result {
        Err(AonixError::UndefinedIdentifier { identifier }) => {
            assert_eq!(identifier, "data_input_typo");
        }
        other => panic!("expected UndefinedIdentifier, got {other:?}"),
    }
}

#[test]
fn rejects_duplicate_identifier_with_specific_error() {
    let result = try_parse("duplicate_identifier.aoncir");
    match result {
        Err(AonixError::DuplicateIdentifier { identifier, scope }) => {
            assert_eq!(identifier, "duplicated_signal");
            assert_eq!(scope, "signals");
        }
        other => panic!("expected DuplicateIdentifier, got {other:?}"),
    }
}

#[test]
fn rejects_unknown_field_with_toml_syntax_error() {
    // `#[serde(deny_unknown_fields)]` causes deserialization to fail
    // when [meta].ghost_field is present.
    let result = try_parse("unknown_field.aoncir");
    match result {
        Err(AonixError::TomlSyntax { detail }) => {
            assert!(
                detail.contains("ghost_field") || detail.contains("unknown field"),
                "TOML error should mention the unknown field; got: {detail}"
            );
        }
        other => panic!("expected TomlSyntax, got {other:?}"),
    }
}
