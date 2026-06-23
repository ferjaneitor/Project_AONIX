//! Document-level validation of a parsed `.aoncir` (`AoncirDocument`).
//!
//! This module enforces the rules of `docs/21-aoncir-syntax.md` and
//! `docs/24-semantic-tag-conventions.md` that operate on the **whole
//! document** and that the strongly typed [`crate::circuit_model::Circuit`]
//! cannot express on its own (semantic groups, `bit_position` ordering,
//! tag/kind consistency, gate-vs-port reference rules).
//!
//! Validations strictly structural to the gate graph (identifier
//! uniqueness across ports/signals/gates, undefined references, missing
//! output assignments, cycles) are enforced separately by
//! [`crate::circuit_model::CircuitBuilder::finish`] and are **not**
//! duplicated here.
//!
//! ## Scope notes (deliberately deferred)
//!
//! - **Dead-signal detection** (a declared internal signal that does not
//!   reach any output) is *not* validated here. It requires a
//!   reachability analysis from the output cones and the `docs/21` rule 11
//!   conditions it on `status = "official_active"` with a warning regime
//!   for `experimental`. There is no warning channel yet. It is therefore
//!   deferred to the metrics sub-phase (1.I) and tracked there.
//! - The auxiliary sections `[verification]`, `[metrics]`, `[layout]`,
//!   `[history]` are accepted as opaque `toml::Value`s by the schema and
//!   are never read to build the `Circuit`; they cannot alter the
//!   technical truth. This module performs no validation on their content
//!   on purpose (the spec only requires that they exist optionally).

use std::collections::BTreeMap;

use crate::circuit_model::{AonixError, AonixResult};

use super::schema::AoncirDocument;

/// Group kinds that impose a bit order on their members. For these,
/// `bit_position` is mandatory, unique, and contiguous from 0.
const ORDERED_GROUP_KINDS: &[&str] = &[
    "bus",
    "address_bus",
    "data_bus",
    "select_bus",
    "operand",
];

/// Group kinds that do not impose a strict bit order. `bit_position` is
/// optional for their members in Phase 1.
const UNORDERED_GROUP_KINDS: &[&str] = &["control_bus", "flags"];

/// Runs every document-level validation. Called by the parser after the
/// TOML deserialization and `format_version` check, and before the
/// `Circuit` is built, so that document-specific errors surface with
/// precise messages.
pub fn validate_document(document: &AoncirDocument) -> AonixResult<()> {
    validate_semantic_groups(document)?;
    validate_group_references_and_tag_consistency(document)?;
    validate_gate_references(document)?;
    Ok(())
}

/// Information about a group member resolved from the document.
struct MemberInfo<'a> {
    semantic_tag: &'a str,
    bit_position: Option<u32>,
    is_port: bool,
}

fn find_member<'a>(document: &'a AoncirDocument, name: &str) -> Option<MemberInfo<'a>> {
    for port in &document.ports.inputs {
        if port.name == name {
            return Some(MemberInfo {
                semantic_tag: &port.semantic_tag,
                bit_position: port.bit_position,
                is_port: true,
            });
        }
    }
    for port in &document.ports.outputs {
        if port.name == name {
            return Some(MemberInfo {
                semantic_tag: &port.semantic_tag,
                bit_position: port.bit_position,
                is_port: true,
            });
        }
    }
    for signal in &document.signals {
        if signal.id == name {
            return Some(MemberInfo {
                semantic_tag: &signal.semantic_tag,
                bit_position: None,
                is_port: false,
            });
        }
    }
    None
}

/// A — `[[semantic_groups]]` validation.
fn validate_semantic_groups(document: &AoncirDocument) -> AonixResult<()> {
    let mut seen_group_ids: BTreeMap<&str, ()> = BTreeMap::new();

    for group in &document.semantic_groups {
        // A1 — unique group identifier.
        if seen_group_ids.insert(group.id.as_str(), ()).is_some() {
            return Err(AonixError::DuplicateIdentifier {
                identifier: group.id.clone(),
                scope: "semantic_groups",
            });
        }

        // A2 — kind in the closed catalog.
        let kind_is_ordered = ORDERED_GROUP_KINDS.contains(&group.kind.as_str());
        let kind_is_unordered = UNORDERED_GROUP_KINDS.contains(&group.kind.as_str());
        if !kind_is_ordered && !kind_is_unordered {
            return Err(AonixError::UnknownGroupKind {
                group: group.id.clone(),
                kind: group.kind.clone(),
            });
        }

        // A4 — width equals the number of members.
        if group.width as usize != group.members.len() {
            return Err(AonixError::GroupWidthMismatch {
                group: group.id.clone(),
                declared: group.width,
                actual: group.members.len(),
            });
        }

        // A3 — every member exists; collect bit positions for A5..A7.
        let mut bit_positions: Vec<u32> = Vec::with_capacity(group.members.len());
        for member_name in &group.members {
            let Some(member) = find_member(document, member_name) else {
                return Err(AonixError::GroupMemberNotFound {
                    group: group.id.clone(),
                    member: member_name.clone(),
                });
            };

            if kind_is_ordered {
                // A5 — bit_position mandatory for ordered groups. Only
                // ports carry a bit_position; an internal signal cannot
                // be ordered, so it counts as missing.
                let Some(position) = member.bit_position else {
                    return Err(AonixError::MissingBitPosition {
                        group: group.id.clone(),
                        member: member_name.clone(),
                    });
                };
                let _ = member.is_port;
                bit_positions.push(position);
            }
        }

        if kind_is_ordered {
            // A6 — bit positions unique within the group.
            let mut sorted = bit_positions.clone();
            sorted.sort_unstable();
            for window in sorted.windows(2) {
                if window[0] == window[1] {
                    return Err(AonixError::DuplicateBitPosition {
                        group: group.id.clone(),
                        position: window[0],
                    });
                }
            }

            // A7 — bit positions are exactly the contiguous range
            // 0..width (LSB-first canonical convention, docs/24 §U.7:
            // bit_position 0 = LSB, width-1 = MSB).
            for (expected, actual) in sorted.iter().enumerate() {
                if *actual != expected as u32 {
                    return Err(AonixError::NonContiguousBitPosition {
                        group: group.id.clone(),
                        detail: format!(
                            "expected contiguous 0..{}, found set {sorted:?}",
                            group.width
                        ),
                    });
                }
            }
        }
    }

    Ok(())
}

/// B — every `group` referenced by a port/signal must be declared, and
/// member `semantic_tag` must be compatible with the group `kind`.
fn validate_group_references_and_tag_consistency(
    document: &AoncirDocument,
) -> AonixResult<()> {
    let group_kind_by_id: BTreeMap<&str, &str> = document
        .semantic_groups
        .iter()
        .map(|group| (group.id.as_str(), group.kind.as_str()))
        .collect();

    // B1 — referenced groups must be declared.
    let check_reference = |referenced_by: &str, group: &str| -> AonixResult<()> {
        if !group.is_empty() && !group_kind_by_id.contains_key(group) {
            return Err(AonixError::UndeclaredGroupReference {
                referenced_by: referenced_by.to_string(),
                group: group.to_string(),
            });
        }
        Ok(())
    };

    for port in document.ports.inputs.iter().chain(&document.ports.outputs) {
        check_reference(&port.name, &port.group)?;
    }
    for signal in &document.signals {
        check_reference(&signal.id, &signal.group)?;
    }

    // B2 — member tag must be compatible with the group kind, when a tag
    // is present. An empty tag is permitted (generic role); the stricter
    // "flags members must carry a Category-3 tag" rule is deferred and
    // documented below.
    for group in &document.semantic_groups {
        for member_name in &group.members {
            let Some(member) = find_member(document, member_name) else {
                // Already reported by A3; defensive continue.
                continue;
            };
            if member.semantic_tag.is_empty() {
                continue;
            }
            if !tag_compatible_with_group_kind(member.semantic_tag, &group.kind) {
                return Err(AonixError::GroupTagInconsistency {
                    group: group.id.clone(),
                    member: member_name.clone(),
                    tag: member.semantic_tag.to_string(),
                    group_kind: group.kind.clone(),
                });
            }
        }
    }

    Ok(())
}

/// Compatibility table of `docs/24-semantic-tag-conventions.md` §C.3.
fn tag_compatible_with_group_kind(tag: &str, group_kind: &str) -> bool {
    match group_kind {
        "bus" => matches!(
            tag,
            "data_bit" | "operand_bit" | "sum_bit" | "difference_bit"
        ),
        "address_bus" | "data_bus" => tag == "data_bit",
        "select_bus" => tag == "select",
        "operand" => tag == "operand_bit",
        "control_bus" => matches!(
            tag,
            "control_signal" | "clock" | "reset" | "enable" | "write_enable" | "read_enable"
        ),
        "flags" => matches!(
            tag,
            "zero_flag"
                | "carry_flag"
                | "overflow_flag"
                | "negative_flag"
                | "parity_flag"
                | "equal_flag"
                | "greater_flag"
                | "less_flag"
        ),
        // Unknown kinds are rejected earlier by A2; be permissive here.
        _ => true,
    }
}

/// C — gate input/output reference rules.
fn validate_gate_references(document: &AoncirDocument) -> AonixResult<()> {
    let output_port_names: BTreeMap<&str, ()> = document
        .ports
        .outputs
        .iter()
        .map(|port| (port.name.as_str(), ()))
        .collect();
    let any_port_names: BTreeMap<&str, ()> = document
        .ports
        .inputs
        .iter()
        .chain(&document.ports.outputs)
        .map(|port| (port.name.as_str(), ()))
        .collect();

    for gate in &document.gates {
        // C1 — no gate input may read an output port.
        for input_name in &gate.inputs {
            if output_port_names.contains_key(input_name.as_str()) {
                return Err(AonixError::GateInputReferencesOutputPort {
                    gate: gate.id.clone(),
                    output_port: input_name.clone(),
                });
            }
        }
        // C2 — a gate output must be an internal signal, never a port.
        if any_port_names.contains_key(gate.output.as_str()) {
            return Err(AonixError::GateOutputCollidesWithPort {
                gate: gate.id.clone(),
                port: gate.output.clone(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_compatibility_bus_accepts_data_and_operand() {
        assert!(tag_compatible_with_group_kind("data_bit", "bus"));
        assert!(tag_compatible_with_group_kind("operand_bit", "bus"));
        assert!(tag_compatible_with_group_kind("sum_bit", "bus"));
        assert!(!tag_compatible_with_group_kind("select", "bus"));
    }

    #[test]
    fn tag_compatibility_select_bus_only_select() {
        assert!(tag_compatible_with_group_kind("select", "select_bus"));
        assert!(!tag_compatible_with_group_kind("data_bit", "select_bus"));
    }

    #[test]
    fn tag_compatibility_flags_only_category_three() {
        assert!(tag_compatible_with_group_kind("zero_flag", "flags"));
        assert!(tag_compatible_with_group_kind("carry_flag", "flags"));
        assert!(!tag_compatible_with_group_kind("data_bit", "flags"));
    }

    #[test]
    fn ordered_and_unordered_kind_sets_are_disjoint() {
        for ordered in ORDERED_GROUP_KINDS {
            assert!(!UNORDERED_GROUP_KINDS.contains(ordered));
        }
    }
}
