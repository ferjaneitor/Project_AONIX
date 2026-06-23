//! Internal signals of a circuit.
//!
//! A [`Signal`] is an internal wire produced by some gate (or, in future
//! sub-phases, declared explicitly). It carries an optional semantic tag
//! and may belong to a semantic group, with the same conventions as
//! external ports (see [`super::port`]).

use crate::circuit_model::error::AonixResult;
use crate::circuit_model::port::{
    validate_snake_case_identifier, GroupIdentifier, SemanticTag,
};

/// Identifier of an internal signal of the circuit.
///
/// Lexical convention: snake_case, starts with a lowercase letter, contains
/// only `[a-z0-9_]`, non-empty.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SignalIdentifier(String);

impl SignalIdentifier {
    /// Constructs a [`SignalIdentifier`] validating snake_case syntax.
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

/// Internal signal of the circuit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signal {
    pub identifier: SignalIdentifier,
    pub semantic_tag: Option<SemanticTag>,
    pub group: Option<GroupIdentifier>,
}

impl Signal {
    /// Convenience constructor with explicit fields.
    pub fn new(
        identifier: SignalIdentifier,
        semantic_tag: Option<SemanticTag>,
        group: Option<GroupIdentifier>,
    ) -> Self {
        Self {
            identifier,
            semantic_tag,
            group,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit_model::error::AonixError;

    #[test]
    fn signal_identifier_accepts_snake_case() {
        let identifier =
            SignalIdentifier::new("operand_b_negated").expect("valid identifier");
        assert_eq!(identifier.as_str(), "operand_b_negated");
    }

    #[test]
    fn signal_identifier_accepts_digits_after_first_letter() {
        let identifier =
            SignalIdentifier::new("intermediate_signal_42").expect("valid identifier");
        assert_eq!(identifier.as_str(), "intermediate_signal_42");
    }

    #[test]
    fn signal_identifier_rejects_empty_string() {
        let result = SignalIdentifier::new("");
        assert!(matches!(
            result,
            Err(AonixError::InvalidIdentifierSyntax { .. })
        ));
    }

    #[test]
    fn signal_identifier_rejects_starting_digit() {
        let result = SignalIdentifier::new("9signal");
        assert!(matches!(
            result,
            Err(AonixError::InvalidIdentifierSyntax { .. })
        ));
    }

    #[test]
    fn signal_identifier_rejects_dash_character() {
        let result = SignalIdentifier::new("operand-negated");
        assert!(matches!(
            result,
            Err(AonixError::InvalidIdentifierSyntax { .. })
        ));
    }

    #[test]
    fn signal_constructor_preserves_fields() {
        let identifier = SignalIdentifier::new("carry_partial").expect("valid");
        let group = Some(GroupIdentifier::new("operand_a_bus").expect("valid"));
        let signal = Signal::new(identifier.clone(), Some(SemanticTag::Carry), group.clone());
        assert_eq!(signal.identifier, identifier);
        assert_eq!(signal.semantic_tag, Some(SemanticTag::Carry));
        assert_eq!(signal.group, group);
    }
}
