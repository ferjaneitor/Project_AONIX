//! Semantic groups of a circuit (buses, operands, flags, …).
//!
//! A [`SemanticGroup`] is a declarative grouping of ports or signals. It
//! is **not** a logical operation and introduces **no** new primitive: it
//! only labels how a set of one-bit wires is read by the simulator,
//! verifier, visualizer and human/AI translators (see
//! `docs/24-semantic-tag-conventions.md`).
//!
//! The kind of a group is a **closed enum** ([`SemanticGroupKind`]),
//! mirroring the catalog of `docs/24` §6. Free strings are never the
//! internal source of truth.
//!
//! Consistency rules (member existence, `width` correctness,
//! `bit_position` mandatory/unique/contiguous, tag/kind compatibility)
//! remain the responsibility of the document-level validator
//! (`crate::format::aoncir::validate`). This module only models the data.

use crate::circuit_model::error::{AonixError, AonixResult};
use crate::circuit_model::port::validate_snake_case_identifier;

/// Identifier of a semantic group. Same lexical convention as the other
/// AONIX identifiers: snake_case, starts with a lowercase letter,
/// `[a-z][a-z0-9_]*`, non-empty.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SemanticGroupIdentifier(String);

impl SemanticGroupIdentifier {
    /// Constructs a [`SemanticGroupIdentifier`] validating snake_case syntax.
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

/// Name of a member of a semantic group. The member references either a
/// port or an internal signal by name; whether it resolves to a port or a
/// signal, and whether it exists at all, is decided by the document-level
/// validator, not by this type. Same snake_case convention.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SemanticGroupMember(String);

impl SemanticGroupMember {
    /// Constructs a [`SemanticGroupMember`] validating snake_case syntax.
    pub fn new(name: impl Into<String>) -> AonixResult<Self> {
        let name = name.into();
        validate_snake_case_identifier(&name)?;
        Ok(Self(name))
    }

    /// Returns the underlying string representation.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Closed catalog of semantic group kinds, version 1.0.0.
///
/// Mirrors `docs/24-semantic-tag-conventions.md` §6. Adding a kind is an
/// audited change of the catalog. **Not extensible by external code.**
/// This enum carries no logical behavior and is unrelated to
/// [`crate::circuit_model::GateKind`]; a semantic group never introduces
/// a gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticGroupKind {
    Bus,
    AddressBus,
    DataBus,
    SelectBus,
    ControlBus,
    Operand,
    Flags,
}

impl SemanticGroupKind {
    /// Returns the canonical snake_case name used in the `.aoncir` syntax.
    pub fn canonical_name(self) -> &'static str {
        match self {
            SemanticGroupKind::Bus => "bus",
            SemanticGroupKind::AddressBus => "address_bus",
            SemanticGroupKind::DataBus => "data_bus",
            SemanticGroupKind::SelectBus => "select_bus",
            SemanticGroupKind::ControlBus => "control_bus",
            SemanticGroupKind::Operand => "operand",
            SemanticGroupKind::Flags => "flags",
        }
    }

    /// Parses a canonical group-kind name into a [`SemanticGroupKind`].
    ///
    /// Returns [`AonixError::UnknownGroupKind`] for any name outside the
    /// closed catalog. The `group` field of the error is filled by the
    /// caller, which knows the offending group's identifier.
    pub fn from_canonical_name(group: &str, name: &str) -> AonixResult<Self> {
        Ok(match name {
            "bus" => SemanticGroupKind::Bus,
            "address_bus" => SemanticGroupKind::AddressBus,
            "data_bus" => SemanticGroupKind::DataBus,
            "select_bus" => SemanticGroupKind::SelectBus,
            "control_bus" => SemanticGroupKind::ControlBus,
            "operand" => SemanticGroupKind::Operand,
            "flags" => SemanticGroupKind::Flags,
            other => {
                return Err(AonixError::UnknownGroupKind {
                    group: group.to_string(),
                    kind: other.to_string(),
                });
            }
        })
    }

    /// Whether this kind imposes a bit order on its members (in which case
    /// `bit_position` is mandatory, unique, and contiguous from 0,
    /// LSB-first; see `docs/24` §U.7). Mirrors the ORDERED set used by the
    /// document-level validator.
    pub fn is_ordered(self) -> bool {
        matches!(
            self,
            SemanticGroupKind::Bus
                | SemanticGroupKind::AddressBus
                | SemanticGroupKind::DataBus
                | SemanticGroupKind::SelectBus
                | SemanticGroupKind::Operand
        )
    }
}

/// A semantic group declared in `[[semantic_groups]]`.
///
/// Holds the canonical truth of the group: identifier, kind, members (in
/// declared order) and width. Equality and ordering follow the natural
/// field order. The validator guarantees `width as usize ==
/// members.len()` and member existence before a `Circuit` is built; this
/// struct does not re-validate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticGroup {
    pub identifier: SemanticGroupIdentifier,
    pub kind: SemanticGroupKind,
    pub members: Vec<SemanticGroupMember>,
    pub width: u32,
}

impl SemanticGroup {
    /// Convenience constructor with explicit fields.
    pub fn new(
        identifier: SemanticGroupIdentifier,
        kind: SemanticGroupKind,
        members: Vec<SemanticGroupMember>,
        width: u32,
    ) -> Self {
        Self {
            identifier,
            kind,
            members,
            width,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_identifier_accepts_snake_case() {
        let id = SemanticGroupIdentifier::new("operand_bus").expect("valid");
        assert_eq!(id.as_str(), "operand_bus");
    }

    #[test]
    fn group_identifier_rejects_empty() {
        assert!(matches!(
            SemanticGroupIdentifier::new(""),
            Err(AonixError::InvalidIdentifierSyntax { .. })
        ));
    }

    #[test]
    fn group_member_accepts_snake_case() {
        let member = SemanticGroupMember::new("operand_bit_zero").expect("valid");
        assert_eq!(member.as_str(), "operand_bit_zero");
    }

    #[test]
    fn group_member_rejects_dash() {
        assert!(matches!(
            SemanticGroupMember::new("operand-bit"),
            Err(AonixError::InvalidIdentifierSyntax { .. })
        ));
    }

    #[test]
    fn group_kind_round_trips_through_canonical_name() {
        let kinds = [
            SemanticGroupKind::Bus,
            SemanticGroupKind::AddressBus,
            SemanticGroupKind::DataBus,
            SemanticGroupKind::SelectBus,
            SemanticGroupKind::ControlBus,
            SemanticGroupKind::Operand,
            SemanticGroupKind::Flags,
        ];
        for kind in kinds {
            let name = kind.canonical_name();
            let parsed =
                SemanticGroupKind::from_canonical_name("some_group", name).expect("round trip");
            assert_eq!(parsed, kind);
        }
    }

    #[test]
    fn group_kind_rejects_unknown_with_group_in_error() {
        match SemanticGroupKind::from_canonical_name("operand_bus", "megabus") {
            Err(AonixError::UnknownGroupKind { group, kind }) => {
                assert_eq!(group, "operand_bus");
                assert_eq!(kind, "megabus");
            }
            other => panic!("expected UnknownGroupKind, got {other:?}"),
        }
    }

    #[test]
    fn ordered_kinds_match_doc24_set() {
        assert!(SemanticGroupKind::Bus.is_ordered());
        assert!(SemanticGroupKind::AddressBus.is_ordered());
        assert!(SemanticGroupKind::DataBus.is_ordered());
        assert!(SemanticGroupKind::SelectBus.is_ordered());
        assert!(SemanticGroupKind::Operand.is_ordered());
        assert!(!SemanticGroupKind::ControlBus.is_ordered());
        assert!(!SemanticGroupKind::Flags.is_ordered());
    }

    #[test]
    fn semantic_group_constructor_preserves_fields() {
        let group = SemanticGroup::new(
            SemanticGroupIdentifier::new("operand_bus").expect("valid"),
            SemanticGroupKind::Operand,
            vec![
                SemanticGroupMember::new("operand_bit_zero").expect("valid"),
                SemanticGroupMember::new("operand_bit_one").expect("valid"),
            ],
            2,
        );
        assert_eq!(group.identifier.as_str(), "operand_bus");
        assert_eq!(group.kind, SemanticGroupKind::Operand);
        assert_eq!(group.members.len(), 2);
        assert_eq!(group.width, 2);
    }
}
