//! Typed error catalog of AONIX.
//!
//! Every operation in the circuit model returns [`AonixResult<T>`] and
//! emits a specific [`AonixError`] variant describing the cause. The
//! variants are aligned with the rejection levels of `docs/14-circuit-rejection.md`.

use thiserror::Error;

/// Canonical result type for every fallible operation in AONIX.
pub type AonixResult<T> = Result<T, AonixError>;

/// Closed catalog of typed errors of AONIX.
///
/// New variants are added through audited changes (see `docs/25`). No
/// variant is silently removed. Each variant carries enough information
/// for downstream tooling to map it to the documented `cause_code`.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AonixError {
    /// Identifier with invalid syntax (empty or contains characters outside
    /// `[a-z0-9_]`, or does not start with a lowercase letter).
    #[error("invalid identifier syntax: {identifier:?}")]
    InvalidIdentifierSyntax { identifier: String },

    /// Identifier duplicated inside the same scope (signals, gates, ports).
    #[error("duplicate identifier: {identifier:?} in scope {scope}")]
    DuplicateIdentifier { identifier: String, scope: &'static str },

    /// Identifier referenced from a gate or output assignment but never
    /// declared as a port, signal, or constant.
    #[error("undefined identifier referenced: {identifier:?}")]
    UndefinedIdentifier { identifier: String },

    /// Wrong arity for a primitive gate. NOT requires exactly 1 input;
    /// AND and OR require exactly 2 (strict binary arity in Phase 1).
    #[error(
        "invalid gate arity for kind {kind:?}: given {given}, expected {expected_description}"
    )]
    InvalidGateArity {
        kind: &'static str,
        given: usize,
        expected_description: &'static str,
    },

    /// Gate kind string outside the closed set {"AND", "OR", "NOT"}.
    ///
    /// Emitted by any code path that converts a textual gate-kind name to a
    /// [`super::gate::GateKind`]. Catches attempts to introduce XOR, NAND,
    /// NOR, XNOR or any other derived primitive.
    #[error("unknown gate kind: {kind:?} (only AND, OR, NOT are allowed)")]
    UnknownGateKind { kind: String },

    /// Semantic tag not present in the closed catalog of `docs/24`.
    #[error("unknown semantic tag: {tag:?}")]
    UnknownSemanticTag { tag: String },

    /// Cycle detected in the circuit DAG. Includes the identifier of one of
    /// the gates involved in the cycle for diagnosis.
    #[error("cycle detected in circuit graph involving gate {gate:?}")]
    CycleDetected { gate: String },

    /// Output port of the circuit has no source assignment.
    #[error("circuit output port has no source assignment: {port:?}")]
    UnassignedOutputPort { port: String },

    /// Output port has more than one source assignment.
    #[error("circuit output port has duplicate assignment: {port:?}")]
    DuplicateOutputAssignment { port: String },

    /// `bit_position` is invalid inside the group (duplicate, out of range,
    /// or non-contiguous from 0 up to width-1).
    #[error("invalid bit_position in group {group:?}: {detail}")]
    InvalidBitPosition { group: String, detail: String },

    /// A required list is empty (for example, `[[ports.outputs]]`).
    #[error("required list is empty: {what}")]
    RequiredListEmpty { what: &'static str },

    /// Port declared with role inconsistent with the section that registers it
    /// (for example, an `Output` port being added through an `add_input_port`
    /// call).
    #[error("port role mismatch for port {port:?}: expected {expected:?}, got {actual:?}")]
    PortRoleMismatch {
        port: String,
        expected: &'static str,
        actual: &'static str,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_for_invalid_identifier_syntax() {
        let error = AonixError::InvalidIdentifierSyntax {
            identifier: "BadName".to_string(),
        };
        let rendered = format!("{error}");
        assert!(rendered.contains("invalid identifier syntax"));
        assert!(rendered.contains("BadName"));
    }

    #[test]
    fn error_display_for_unknown_gate_kind_mentions_allowed_set() {
        let error = AonixError::UnknownGateKind { kind: "XOR".to_string() };
        let rendered = format!("{error}");
        assert!(rendered.contains("XOR"));
        assert!(rendered.contains("AND, OR, NOT"));
    }

    #[test]
    fn error_equality_for_same_variant_and_payload() {
        let first = AonixError::DuplicateIdentifier {
            identifier: "operand_a".to_string(),
            scope: "ports",
        };
        let second = AonixError::DuplicateIdentifier {
            identifier: "operand_a".to_string(),
            scope: "ports",
        };
        assert_eq!(first, second);
    }
}
