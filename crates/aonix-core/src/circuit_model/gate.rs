//! Primitive gates of AONIX.
//!
//! AONIX recognizes **only three primitive gates**: AND, OR, NOT. This is the
//! absolute rule R2 and it is enforced **at the type system level** by
//! making [`GateKind`] a closed enum with exactly three variants. No
//! external code, no parser, no optimizer, and no agent can introduce a
//! fourth primitive without modifying this file — and any such modification
//! is an audited change (severity S0 in `docs/25`: not auditable, the
//! invariants are not negotiable).

use crate::circuit_model::error::{AonixError, AonixResult};
use crate::circuit_model::port::{validate_snake_case_identifier, PortIdentifier};
use crate::circuit_model::signal::SignalIdentifier;

/// Kind of primitive gate. **Closed set**: AND, OR, NOT.
///
/// Adding a fourth variant would violate R2 of AONIX. Do not extend.
///
/// In Phase 1, AONIX uses **strict binary arity** for AND and OR. A higher
/// arity (3-input AND, etc.) must be built by composing several binary
/// gates; this makes the gate-count and depth metrics honest. A future
/// audited change of the catalog may introduce N-ary variants; until then,
/// arity is fixed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GateKind {
    /// Logical AND. Arity at construction: **exactly 2**.
    And,
    /// Logical OR. Arity at construction: **exactly 2**.
    Or,
    /// Logical NOT. Arity at construction: **exactly 1**.
    Not,
}

impl GateKind {
    /// Canonical uppercase name used in the `.aoncir` syntax.
    pub fn canonical_name(self) -> &'static str {
        match self {
            GateKind::And => "AND",
            GateKind::Or => "OR",
            GateKind::Not => "NOT",
        }
    }

    /// Parses an uppercase gate-kind name into a [`GateKind`].
    ///
    /// Returns [`AonixError::UnknownGateKind`] for any other input. This is
    /// the parser-stage filter that enforces R2 against XOR / NAND / NOR /
    /// XNOR and any derived primitive.
    pub fn from_canonical_name(name: &str) -> AonixResult<Self> {
        match name {
            "AND" => Ok(GateKind::And),
            "OR" => Ok(GateKind::Or),
            "NOT" => Ok(GateKind::Not),
            other => Err(AonixError::UnknownGateKind {
                kind: other.to_string(),
            }),
        }
    }
}

/// Identifier of a gate inside the circuit.
///
/// Lexical convention: snake_case (same as port/signal identifiers).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GateIdentifier(String);

impl GateIdentifier {
    /// Constructs a [`GateIdentifier`] validating snake_case syntax.
    pub fn new(identifier: impl Into<String>) -> AonixResult<Self> {
        let identifier = identifier.into();
        validate_snake_case_identifier(&identifier)?;
        Ok(Self(identifier))
    }

    /// Returns the underlying string representation.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Reference to a signal used as a gate input or as the source of an
/// output assignment.
///
/// A signal reference is either:
///
/// - an external input port of the circuit ([`SignalReference::Port`]), or
/// - an internal signal produced by another gate ([`SignalReference::InternalSignal`]).
///
/// **AONIX does not provide logical constants (`0` / `1`) as primitive
/// signal references in Phase 1.** Allowing them would let any circuit
/// pull "free" zeros and ones from nowhere, distorting metrics and giving
/// the agent a magical source of values. Constants — when needed — are
/// the responsibility of upstream specification logic (for example, tasks
/// `constant_zero` and `constant_one` of the level 1 catalog), not of the
/// core model. A future audited change may reintroduce them as an explicit,
/// level-gated exception.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SignalReference {
    Port(PortIdentifier),
    InternalSignal(SignalIdentifier),
}

/// Concrete gate in a circuit.
///
/// Arities are validated **strictly** at construction:
/// - [`GateKind::Not`] requires exactly one input.
/// - [`GateKind::And`] requires exactly two inputs.
/// - [`GateKind::Or`] requires exactly two inputs.
///
/// A 3-input AND must be built as a composition of two 2-input AND
/// gates; the same applies to OR. This keeps the per-gate cost honest in
/// the evaluator metrics and prevents an implicit "more powerful"
/// primitive from sneaking in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gate {
    pub identifier: GateIdentifier,
    pub kind: GateKind,
    pub inputs: Vec<SignalReference>,
    pub output: SignalIdentifier,
}

impl Gate {
    /// Constructs a [`Gate`] enforcing the strict arity rules of its
    /// [`GateKind`].
    pub fn new(
        identifier: GateIdentifier,
        kind: GateKind,
        inputs: Vec<SignalReference>,
        output: SignalIdentifier,
    ) -> AonixResult<Self> {
        match kind {
            GateKind::Not if inputs.len() != 1 => {
                return Err(AonixError::InvalidGateArity {
                    kind: kind.canonical_name(),
                    given: inputs.len(),
                    expected_description: "exactly 1",
                });
            }
            GateKind::And | GateKind::Or if inputs.len() != 2 => {
                return Err(AonixError::InvalidGateArity {
                    kind: kind.canonical_name(),
                    given: inputs.len(),
                    expected_description: "exactly 2",
                });
            }
            _ => {}
        }
        Ok(Self {
            identifier,
            kind,
            inputs,
            output,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_signal(identifier: &str) -> SignalIdentifier {
        SignalIdentifier::new(identifier).expect("valid signal identifier")
    }

    fn make_port_ref(identifier: &str) -> SignalReference {
        SignalReference::Port(PortIdentifier::new(identifier).expect("valid port identifier"))
    }

    #[test]
    fn gate_kind_canonical_names_are_exactly_three() {
        assert_eq!(GateKind::And.canonical_name(), "AND");
        assert_eq!(GateKind::Or.canonical_name(), "OR");
        assert_eq!(GateKind::Not.canonical_name(), "NOT");
    }

    #[test]
    fn gate_kind_from_canonical_name_accepts_only_and_or_not() {
        assert_eq!(
            GateKind::from_canonical_name("AND").expect("AND parses"),
            GateKind::And
        );
        assert_eq!(
            GateKind::from_canonical_name("OR").expect("OR parses"),
            GateKind::Or
        );
        assert_eq!(
            GateKind::from_canonical_name("NOT").expect("NOT parses"),
            GateKind::Not
        );
    }

    #[test]
    fn gate_kind_rejects_xor_with_specific_error() {
        let result = GateKind::from_canonical_name("XOR");
        match result {
            Err(AonixError::UnknownGateKind { kind }) => assert_eq!(kind, "XOR"),
            other => panic!("expected UnknownGateKind, got {other:?}"),
        }
    }

    #[test]
    fn gate_kind_rejects_nand_nor_xnor_with_specific_error() {
        for forbidden in ["NAND", "NOR", "XNOR"] {
            let result = GateKind::from_canonical_name(forbidden);
            match result {
                Err(AonixError::UnknownGateKind { kind }) => assert_eq!(kind, forbidden),
                other => panic!("expected UnknownGateKind for {forbidden}, got {other:?}"),
            }
        }
    }

    #[test]
    fn gate_kind_rejects_lowercase_and_arbitrary_strings() {
        for invalid in ["and", "Or", "buffer", "nor_gate", ""] {
            let result = GateKind::from_canonical_name(invalid);
            assert!(
                matches!(result, Err(AonixError::UnknownGateKind { .. })),
                "input {invalid:?} should be rejected"
            );
        }
    }

    #[test]
    fn gate_not_with_single_input_constructs_successfully() {
        let gate = Gate::new(
            GateIdentifier::new("g_invert_a").expect("valid gate id"),
            GateKind::Not,
            vec![make_port_ref("operand_a")],
            make_signal("operand_a_negated"),
        )
        .expect("valid NOT");
        assert_eq!(gate.kind, GateKind::Not);
        assert_eq!(gate.inputs.len(), 1);
    }

    #[test]
    fn gate_not_with_zero_inputs_fails_arity() {
        let result = Gate::new(
            GateIdentifier::new("g_bad_not").expect("valid gate id"),
            GateKind::Not,
            vec![],
            make_signal("intermediate"),
        );
        assert!(matches!(
            result,
            Err(AonixError::InvalidGateArity { .. })
        ));
    }

    #[test]
    fn gate_not_with_two_inputs_fails_arity() {
        let result = Gate::new(
            GateIdentifier::new("g_bad_not").expect("valid gate id"),
            GateKind::Not,
            vec![make_port_ref("operand_a"), make_port_ref("operand_b")],
            make_signal("intermediate"),
        );
        assert!(matches!(
            result,
            Err(AonixError::InvalidGateArity { .. })
        ));
    }

    #[test]
    fn gate_and_with_one_input_fails_arity() {
        let result = Gate::new(
            GateIdentifier::new("g_bad_and").expect("valid gate id"),
            GateKind::And,
            vec![make_port_ref("operand_a")],
            make_signal("intermediate"),
        );
        assert!(matches!(
            result,
            Err(AonixError::InvalidGateArity { .. })
        ));
    }

    #[test]
    fn gate_or_with_one_input_fails_arity() {
        let result = Gate::new(
            GateIdentifier::new("g_bad_or").expect("valid gate id"),
            GateKind::Or,
            vec![make_port_ref("operand_a")],
            make_signal("intermediate"),
        );
        assert!(matches!(
            result,
            Err(AonixError::InvalidGateArity { .. })
        ));
    }

    #[test]
    fn gate_and_with_two_inputs_is_valid() {
        let gate = Gate::new(
            GateIdentifier::new("g_and_binary").expect("valid gate id"),
            GateKind::And,
            vec![make_port_ref("operand_a"), make_port_ref("operand_b")],
            make_signal("binary_and_output"),
        )
        .expect("AND with exactly 2 inputs constructs successfully");
        assert_eq!(gate.kind, GateKind::And);
        assert_eq!(gate.inputs.len(), 2);
    }

    #[test]
    fn gate_or_with_two_inputs_is_valid() {
        let gate = Gate::new(
            GateIdentifier::new("g_or_binary").expect("valid gate id"),
            GateKind::Or,
            vec![make_port_ref("operand_a"), make_port_ref("operand_b")],
            make_signal("binary_or_output"),
        )
        .expect("OR with exactly 2 inputs constructs successfully");
        assert_eq!(gate.kind, GateKind::Or);
        assert_eq!(gate.inputs.len(), 2);
    }

    #[test]
    fn gate_and_with_three_inputs_fails_arity() {
        let result = Gate::new(
            GateIdentifier::new("g_and3").expect("valid gate id"),
            GateKind::And,
            vec![
                make_port_ref("operand_a"),
                make_port_ref("operand_b"),
                make_port_ref("operand_c"),
            ],
            make_signal("triple_and_output"),
        );
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
    fn gate_or_with_three_inputs_fails_arity() {
        let result = Gate::new(
            GateIdentifier::new("g_or3").expect("valid gate id"),
            GateKind::Or,
            vec![
                make_port_ref("operand_a"),
                make_port_ref("operand_b"),
                make_port_ref("operand_c"),
            ],
            make_signal("triple_or_output"),
        );
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
    fn gate_and_with_four_inputs_fails_arity() {
        let result = Gate::new(
            GateIdentifier::new("g_and4").expect("valid gate id"),
            GateKind::And,
            vec![
                make_port_ref("operand_a"),
                make_port_ref("operand_b"),
                make_port_ref("operand_c"),
                make_port_ref("operand_d"),
            ],
            make_signal("quad_and_output"),
        );
        assert!(matches!(
            result,
            Err(AonixError::InvalidGateArity { .. })
        ));
    }

    #[test]
    fn gate_or_with_four_inputs_fails_arity() {
        let result = Gate::new(
            GateIdentifier::new("g_or4").expect("valid gate id"),
            GateKind::Or,
            vec![
                make_port_ref("operand_a"),
                make_port_ref("operand_b"),
                make_port_ref("operand_c"),
                make_port_ref("operand_d"),
            ],
            make_signal("quad_or_output"),
        );
        assert!(matches!(
            result,
            Err(AonixError::InvalidGateArity { .. })
        ));
    }

    #[test]
    fn gate_and_with_zero_inputs_fails_arity() {
        let result = Gate::new(
            GateIdentifier::new("g_and_empty").expect("valid gate id"),
            GateKind::And,
            vec![],
            make_signal("empty_and_output"),
        );
        assert!(matches!(
            result,
            Err(AonixError::InvalidGateArity { .. })
        ));
    }

    #[test]
    fn gate_or_with_zero_inputs_fails_arity() {
        let result = Gate::new(
            GateIdentifier::new("g_or_empty").expect("valid gate id"),
            GateKind::Or,
            vec![],
            make_signal("empty_or_output"),
        );
        assert!(matches!(
            result,
            Err(AonixError::InvalidGateArity { .. })
        ));
    }

    #[test]
    fn signal_reference_only_admits_port_and_internal_signal() {
        // Sanity check: the SignalReference enum has exactly two variants
        // in Phase 1. Adding a third without an audited change would
        // break this match's exhaustiveness without the wildcard arm.
        let port_ref = make_port_ref("operand_a");
        let signal_ref = SignalReference::InternalSignal(
            SignalIdentifier::new("internal_signal").expect("valid"),
        );
        let variant_label = |reference: &SignalReference| -> &'static str {
            match reference {
                SignalReference::Port(_) => "port",
                SignalReference::InternalSignal(_) => "internal_signal",
            }
        };
        assert_eq!(variant_label(&port_ref), "port");
        assert_eq!(variant_label(&signal_ref), "internal_signal");
    }
}
