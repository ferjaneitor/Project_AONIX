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

    /// Syntactic error from the TOML parser, with detail message forwarded
    /// from the underlying library.
    #[error("invalid TOML syntax in .aoncir: {detail}")]
    TomlSyntax { detail: String },

    /// The `[format].format_version` of a `.aoncir` does not match a version
    /// supported by this parser.
    #[error(
        "unsupported .aoncir format_version: got {found:?}, expected {expected:?}"
    )]
    UnsupportedFormatVersion {
        found: String,
        expected: &'static str,
    },

    /// Semantic group declared with a `kind` outside the closed catalog
    /// of `docs/24-semantic-tag-conventions.md`.
    #[error("unknown semantic group kind {kind:?} in group {group:?}")]
    UnknownGroupKind { group: String, kind: String },

    /// A semantic group lists a member that is not a declared port or
    /// internal signal.
    #[error("semantic group {group:?} references unknown member {member:?}")]
    GroupMemberNotFound { group: String, member: String },

    /// `width` of a semantic group does not equal the number of members.
    #[error(
        "semantic group {group:?} width mismatch: declared {declared}, has {actual} members"
    )]
    GroupWidthMismatch {
        group: String,
        declared: u32,
        actual: usize,
    },

    /// A member of an order-bearing group lacks the mandatory
    /// `bit_position` field.
    #[error(
        "member {member:?} of ordered group {group:?} is missing the mandatory bit_position"
    )]
    MissingBitPosition { group: String, member: String },

    /// Two members of the same group declare the same `bit_position`.
    #[error("semantic group {group:?} has duplicate bit_position {position}")]
    DuplicateBitPosition { group: String, position: u32 },

    /// The set of `bit_position` values of a group is not the contiguous
    /// range `0..width` (LSB-first convention, see `docs/24` §U.7).
    #[error("semantic group {group:?} has non-contiguous bit_position set: {detail}")]
    NonContiguousBitPosition { group: String, detail: String },

    /// A port or signal references a `group` that is not declared in
    /// `[[semantic_groups]]`.
    #[error("{referenced_by:?} references undeclared semantic group {group:?}")]
    UndeclaredGroupReference {
        referenced_by: String,
        group: String,
    },

    /// A member's `semantic_tag` is incompatible with the `kind` of the
    /// group it belongs to (see compatibility table in `docs/24` §C.3).
    #[error(
        "member {member:?} with tag {tag:?} is incompatible with group {group:?} of kind {group_kind:?}"
    )]
    GroupTagInconsistency {
        group: String,
        member: String,
        tag: String,
        group_kind: String,
    },

    /// A gate input references an output port of the circuit. Gate inputs
    /// may only come from input ports or internal signals.
    #[error(
        "gate {gate:?} input references output port {output_port:?}; gate inputs cannot read output ports"
    )]
    GateInputReferencesOutputPort {
        gate: String,
        output_port: String,
    },

    /// A gate output identifier collides with a declared port name. Gate
    /// outputs must be internal signals, never ports.
    #[error(
        "gate {gate:?} writes to {port:?}, which is a declared port; gate outputs must be internal signals"
    )]
    GateOutputCollidesWithPort { gate: String, port: String },

    /// The [`crate::circuit_model::InputVector`] passed to the simulator
    /// does not match the input arity of the circuit (the number of
    /// `[[ports.inputs]]` declared).
    #[error(
        "input vector length mismatch: expected {expected} bits, got {given}"
    )]
    InputVectorLengthMismatch { expected: usize, given: usize },

    /// The circuit has too many input ports for exhaustive truth-table
    /// enumeration: 2^inputs rows would be impractical. Emitted by
    /// [`crate::simulation::simulate_exhaustive`].
    #[error(
        "circuit has {inputs} input ports; exhaustive enumeration is capped at {max} input bits"
    )]
    ExhaustiveInputTooLarge { inputs: usize, max: usize },
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
