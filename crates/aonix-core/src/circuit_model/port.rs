//! External ports of a circuit, plus semantic tags and group identifiers.
//!
//! A [`Port`] models an input or output of the circuit. The order of
//! appearance of input ports defines the contract of the [`super::InputVector`];
//! same for outputs.
//!
//! Semantic tags belong to a closed catalog defined in
//! `docs/24-semantic-tag-conventions.md`. The [`SemanticTag`] enum mirrors
//! that catalog. Extension via namespace is out of scope for Sub-phase 1.A.

use crate::circuit_model::error::{AonixError, AonixResult};

/// Identifier of an external port of the circuit.
///
/// Lexical convention: snake_case, starts with a lowercase letter, contains
/// only `[a-z0-9_]`, non-empty.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PortIdentifier(String);

impl PortIdentifier {
    /// Constructs a [`PortIdentifier`] validating snake_case syntax.
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

/// Identifier of a semantic group (bus, flags, operand, control_bus, ...).
///
/// Same lexical convention as [`PortIdentifier`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GroupIdentifier(String);

impl GroupIdentifier {
    /// Constructs a [`GroupIdentifier`] validating snake_case syntax.
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

/// Closed catalog of semantic tags recognized by AONIX, version 1.0.0.
///
/// Mirrors `docs/24-semantic-tag-conventions.md`. Adding a tag is an
/// audited change of the catalog (severity S3 in `docs/25`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticTag {
    // Category 1 — generic data
    DataBit,
    OperandBit,
    SumBit,
    DifferenceBit,
    ParityBit,
    PatternMatch,
    // Category 2 — carry / sign
    Carry,
    Borrow,
    SignBit,
    // Category 3 — flags
    ZeroFlag,
    CarryFlag,
    OverflowFlag,
    NegativeFlag,
    ParityFlag,
    EqualFlag,
    GreaterFlag,
    LessFlag,
    // Category 4 — selectors / mux
    Select,
    SelectOutput,
    Comparison,
    // Category 5 — temporality (levels ≥ 11)
    Clock,
    Reset,
    Enable,
    WriteEnable,
    ReadEnable,
    // Category 7 — generic control
    ControlSignal,
}

impl SemanticTag {
    /// Returns the canonical snake_case name used in the `.aoncir` syntax.
    pub fn canonical_name(self) -> &'static str {
        match self {
            SemanticTag::DataBit => "data_bit",
            SemanticTag::OperandBit => "operand_bit",
            SemanticTag::SumBit => "sum_bit",
            SemanticTag::DifferenceBit => "difference_bit",
            SemanticTag::ParityBit => "parity_bit",
            SemanticTag::PatternMatch => "pattern_match",
            SemanticTag::Carry => "carry",
            SemanticTag::Borrow => "borrow",
            SemanticTag::SignBit => "sign_bit",
            SemanticTag::ZeroFlag => "zero_flag",
            SemanticTag::CarryFlag => "carry_flag",
            SemanticTag::OverflowFlag => "overflow_flag",
            SemanticTag::NegativeFlag => "negative_flag",
            SemanticTag::ParityFlag => "parity_flag",
            SemanticTag::EqualFlag => "equal_flag",
            SemanticTag::GreaterFlag => "greater_flag",
            SemanticTag::LessFlag => "less_flag",
            SemanticTag::Select => "select",
            SemanticTag::SelectOutput => "select_output",
            SemanticTag::Comparison => "comparison",
            SemanticTag::Clock => "clock",
            SemanticTag::Reset => "reset",
            SemanticTag::Enable => "enable",
            SemanticTag::WriteEnable => "write_enable",
            SemanticTag::ReadEnable => "read_enable",
            SemanticTag::ControlSignal => "control_signal",
        }
    }

    /// Parses a canonical snake_case tag name into a [`SemanticTag`].
    ///
    /// Returns [`AonixError::UnknownSemanticTag`] for any name not in the
    /// closed catalog (a parser-stage rejection that aligns with L3 in
    /// `docs/14-circuit-rejection.md`).
    pub fn from_canonical_name(name: &str) -> AonixResult<Self> {
        Ok(match name {
            "data_bit" => SemanticTag::DataBit,
            "operand_bit" => SemanticTag::OperandBit,
            "sum_bit" => SemanticTag::SumBit,
            "difference_bit" => SemanticTag::DifferenceBit,
            "parity_bit" => SemanticTag::ParityBit,
            "pattern_match" => SemanticTag::PatternMatch,
            "carry" => SemanticTag::Carry,
            "borrow" => SemanticTag::Borrow,
            "sign_bit" => SemanticTag::SignBit,
            "zero_flag" => SemanticTag::ZeroFlag,
            "carry_flag" => SemanticTag::CarryFlag,
            "overflow_flag" => SemanticTag::OverflowFlag,
            "negative_flag" => SemanticTag::NegativeFlag,
            "parity_flag" => SemanticTag::ParityFlag,
            "equal_flag" => SemanticTag::EqualFlag,
            "greater_flag" => SemanticTag::GreaterFlag,
            "less_flag" => SemanticTag::LessFlag,
            "select" => SemanticTag::Select,
            "select_output" => SemanticTag::SelectOutput,
            "comparison" => SemanticTag::Comparison,
            "clock" => SemanticTag::Clock,
            "reset" => SemanticTag::Reset,
            "enable" => SemanticTag::Enable,
            "write_enable" => SemanticTag::WriteEnable,
            "read_enable" => SemanticTag::ReadEnable,
            "control_signal" => SemanticTag::ControlSignal,
            other => {
                return Err(AonixError::UnknownSemanticTag {
                    tag: other.to_string(),
                });
            }
        })
    }
}

/// Role of a port: external input of the circuit, or external output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortRole {
    Input,
    Output,
}

impl PortRole {
    /// Returns the canonical text representation for diagnostics.
    pub fn canonical_name(self) -> &'static str {
        match self {
            PortRole::Input => "Input",
            PortRole::Output => "Output",
        }
    }
}

/// External port of the circuit.
///
/// Order of appearance in the source `.aoncir` defines the formal contract
/// of the [`super::InputVector`] / [`super::OutputVector`]. This struct
/// holds the per-port data; the order is preserved by the [`super::circuit::CircuitBuilder`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Port {
    pub identifier: PortIdentifier,
    pub role: PortRole,
    pub semantic_tag: Option<SemanticTag>,
    pub group: Option<GroupIdentifier>,
    pub bit_position: Option<u32>,
}

impl Port {
    /// Convenience constructor with explicit fields.
    pub fn new(
        identifier: PortIdentifier,
        role: PortRole,
        semantic_tag: Option<SemanticTag>,
        group: Option<GroupIdentifier>,
        bit_position: Option<u32>,
    ) -> Self {
        Self {
            identifier,
            role,
            semantic_tag,
            group,
            bit_position,
        }
    }
}

/// Validates that an identifier follows the AONIX snake_case rule:
/// non-empty, starts with `[a-z]`, then `[a-z0-9_]*`.
pub(crate) fn validate_snake_case_identifier(identifier: &str) -> AonixResult<()> {
    if identifier.is_empty() {
        return Err(AonixError::InvalidIdentifierSyntax {
            identifier: identifier.to_string(),
        });
    }
    let mut characters = identifier.chars();
    let first = characters.next().expect("non-empty by prior check");
    if !first.is_ascii_lowercase() {
        return Err(AonixError::InvalidIdentifierSyntax {
            identifier: identifier.to_string(),
        });
    }
    for character in characters {
        let allowed =
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_';
        if !allowed {
            return Err(AonixError::InvalidIdentifierSyntax {
                identifier: identifier.to_string(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_identifier_accepts_snake_case() {
        let identifier = PortIdentifier::new("operand_a").expect("valid identifier");
        assert_eq!(identifier.as_str(), "operand_a");
    }

    #[test]
    fn port_identifier_rejects_empty_string() {
        let result = PortIdentifier::new("");
        assert!(matches!(
            result,
            Err(AonixError::InvalidIdentifierSyntax { .. })
        ));
    }

    #[test]
    fn port_identifier_rejects_uppercase_start() {
        let result = PortIdentifier::new("OperandA");
        assert!(matches!(
            result,
            Err(AonixError::InvalidIdentifierSyntax { .. })
        ));
    }

    #[test]
    fn port_identifier_rejects_invalid_characters() {
        let result = PortIdentifier::new("operand-a");
        assert!(matches!(
            result,
            Err(AonixError::InvalidIdentifierSyntax { .. })
        ));
    }

    #[test]
    fn group_identifier_accepts_snake_case() {
        let group = GroupIdentifier::new("operand_a_bus").expect("valid group");
        assert_eq!(group.as_str(), "operand_a_bus");
    }

    #[test]
    fn semantic_tag_round_trips_through_canonical_name() {
        let cases = [
            SemanticTag::DataBit,
            SemanticTag::OperandBit,
            SemanticTag::Carry,
            SemanticTag::Borrow,
            SemanticTag::ZeroFlag,
            SemanticTag::Select,
            SemanticTag::Clock,
            SemanticTag::ControlSignal,
        ];
        for tag in cases {
            let name = tag.canonical_name();
            let parsed = SemanticTag::from_canonical_name(name).expect("round trip");
            assert_eq!(parsed, tag);
        }
    }

    #[test]
    fn semantic_tag_rejects_unknown_name() {
        let result = SemanticTag::from_canonical_name("not_a_real_tag");
        assert!(matches!(
            result,
            Err(AonixError::UnknownSemanticTag { .. })
        ));
    }

    #[test]
    fn port_role_canonical_names_are_stable() {
        assert_eq!(PortRole::Input.canonical_name(), "Input");
        assert_eq!(PortRole::Output.canonical_name(), "Output");
    }

    #[test]
    fn port_constructor_preserves_fields() {
        let identifier = PortIdentifier::new("carry_input").expect("valid");
        let group = Some(GroupIdentifier::new("operand_a_bus").expect("valid"));
        let port = Port::new(
            identifier.clone(),
            PortRole::Input,
            Some(SemanticTag::Carry),
            group.clone(),
            Some(0),
        );
        assert_eq!(port.identifier, identifier);
        assert_eq!(port.role, PortRole::Input);
        assert_eq!(port.semantic_tag, Some(SemanticTag::Carry));
        assert_eq!(port.group, group);
        assert_eq!(port.bit_position, Some(0));
    }
}
