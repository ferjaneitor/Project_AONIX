//! Parser of `.aoncir` TOML documents into [`Circuit`].
//!
//! The parser is **strict**:
//!
//! - The `format_version` must match [`SUPPORTED_FORMAT_VERSION`].
//! - Gate `kind` is restricted to `"AND"`, `"OR"`, `"NOT"`; any other value
//!   is rejected at conversion time by [`crate::circuit_model::GateKind::from_canonical_name`].
//! - Arity is strict (`exactly 1` for `NOT`, `exactly 2` for `AND` and
//!   `OR`); enforced by [`crate::circuit_model::Gate::new`].
//! - The order of appearance of `[[ports.inputs]]` and `[[ports.outputs]]`
//!   is preserved; it becomes the formal contract of the input / output
//!   vectors.
//! - Unknown TOML fields cause a syntax error (via
//!   `#[serde(deny_unknown_fields)]` on every section in the schema).
//! - Every signal reference inside a gate input or output assignment must
//!   resolve to either an input port of the circuit or a declared
//!   internal signal; constants are not valid references in Phase 1.

use std::collections::HashSet;

use crate::circuit_model::{
    AonixError, AonixResult, Circuit, CircuitBuilder, Gate, GateIdentifier, GateKind,
    GroupIdentifier, Port, PortIdentifier, PortRole, SemanticTag, Signal, SignalIdentifier,
    SignalReference,
};

use super::schema::{
    AoncirDocument, GateEntry, OutputAssignmentEntry, PortEntry, SignalEntry,
};

/// The only `format_version` accepted by the Phase 1 parser.
pub const SUPPORTED_FORMAT_VERSION: &str = "1.0.0";

/// Parses a `.aoncir` TOML document into a fully validated [`Circuit`].
///
/// Returns an [`AonixError`] for syntactic, semantic, structural, or
/// arity violations. The returned [`Circuit`] is guaranteed to satisfy
/// all the invariants of [`CircuitBuilder::finish`].
pub fn parse(input: &str) -> AonixResult<Circuit> {
    let document: AoncirDocument =
        toml::from_str(input).map_err(|error| AonixError::TomlSyntax {
            detail: error.to_string(),
        })?;

    validate_format_version(&document.format.format_version)?;

    // Document-level validation (semantic groups, bit_position ordering,
    // tag/kind consistency, gate-vs-port reference rules). Runs before the
    // Circuit is built so that document-specific errors surface with
    // precise messages; structural graph validation happens later in
    // CircuitBuilder::finish.
    super::validate::validate_document(&document)?;

    let mut builder = CircuitBuilder::new();

    // Build a lookup of declared input port names and declared internal
    // signal names; gate inputs and output assignments must reference one
    // of these (or fail with UndefinedIdentifier at builder.finish() time,
    // but we anticipate the check here for a clearer error message).
    let known_input_port_names: HashSet<&str> = document
        .ports
        .inputs
        .iter()
        .map(|entry| entry.name.as_str())
        .collect();
    let known_signal_ids: HashSet<&str> = document
        .signals
        .iter()
        .map(|entry| entry.id.as_str())
        .collect();

    for entry in &document.ports.inputs {
        let port = build_port(entry, PortRole::Input)?;
        builder.add_input_port(port)?;
    }

    for entry in &document.ports.outputs {
        let port = build_port(entry, PortRole::Output)?;
        builder.add_output_port(port)?;
    }

    for entry in &document.signals {
        let signal = build_signal(entry)?;
        builder.add_signal(signal)?;
    }

    for entry in &document.gates {
        let gate = build_gate(entry, &known_input_port_names, &known_signal_ids)?;
        builder.add_gate(gate)?;
    }

    for entry in &document.outputs {
        let (port_identifier, source_reference) =
            build_output_assignment(entry, &known_input_port_names, &known_signal_ids)?;
        builder.assign_output(port_identifier, source_reference)?;
    }

    builder.finish()
}

fn validate_format_version(version: &str) -> AonixResult<()> {
    if version != SUPPORTED_FORMAT_VERSION {
        return Err(AonixError::UnsupportedFormatVersion {
            found: version.to_string(),
            expected: SUPPORTED_FORMAT_VERSION,
        });
    }
    Ok(())
}

fn build_port(entry: &PortEntry, role: PortRole) -> AonixResult<Port> {
    let identifier = PortIdentifier::new(entry.name.clone())?;
    let semantic_tag = parse_optional_semantic_tag(&entry.semantic_tag)?;
    let group = parse_optional_group(&entry.group)?;
    Ok(Port::new(
        identifier,
        role,
        semantic_tag,
        group,
        entry.bit_position,
    ))
}

fn build_signal(entry: &SignalEntry) -> AonixResult<Signal> {
    let identifier = SignalIdentifier::new(entry.id.clone())?;
    let semantic_tag = parse_optional_semantic_tag(&entry.semantic_tag)?;
    let group = parse_optional_group(&entry.group)?;
    Ok(Signal::new(identifier, semantic_tag, group))
}

fn build_gate(
    entry: &GateEntry,
    known_input_port_names: &HashSet<&str>,
    known_signal_ids: &HashSet<&str>,
) -> AonixResult<Gate> {
    let kind = GateKind::from_canonical_name(&entry.kind)?;
    let identifier = GateIdentifier::new(entry.id.clone())?;
    let output = SignalIdentifier::new(entry.output.clone())?;
    let inputs: Vec<SignalReference> = entry
        .inputs
        .iter()
        .map(|name| resolve_signal_reference(name, known_input_port_names, known_signal_ids))
        .collect::<AonixResult<Vec<_>>>()?;
    Gate::new(identifier, kind, inputs, output)
}

fn build_output_assignment(
    entry: &OutputAssignmentEntry,
    known_input_port_names: &HashSet<&str>,
    known_signal_ids: &HashSet<&str>,
) -> AonixResult<(PortIdentifier, SignalReference)> {
    let port = PortIdentifier::new(entry.port.clone())?;
    let source =
        resolve_signal_reference(&entry.source, known_input_port_names, known_signal_ids)?;
    Ok((port, source))
}

fn resolve_signal_reference(
    name: &str,
    known_input_port_names: &HashSet<&str>,
    known_signal_ids: &HashSet<&str>,
) -> AonixResult<SignalReference> {
    if known_input_port_names.contains(name) {
        Ok(SignalReference::Port(PortIdentifier::new(name.to_string())?))
    } else if known_signal_ids.contains(name) {
        Ok(SignalReference::InternalSignal(SignalIdentifier::new(
            name.to_string(),
        )?))
    } else {
        Err(AonixError::UndefinedIdentifier {
            identifier: name.to_string(),
        })
    }
}

fn parse_optional_semantic_tag(raw: &str) -> AonixResult<Option<SemanticTag>> {
    if raw.is_empty() {
        Ok(None)
    } else {
        SemanticTag::from_canonical_name(raw).map(Some)
    }
}

fn parse_optional_group(raw: &str) -> AonixResult<Option<GroupIdentifier>> {
    if raw.is_empty() {
        Ok(None)
    } else {
        GroupIdentifier::new(raw.to_string()).map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_format_version_accepts_supported() {
        assert!(validate_format_version(SUPPORTED_FORMAT_VERSION).is_ok());
    }

    #[test]
    fn validate_format_version_rejects_other_versions() {
        for unsupported in ["0.9.0", "1.0.1", "2.0.0", ""] {
            let result = validate_format_version(unsupported);
            match result {
                Err(AonixError::UnsupportedFormatVersion { found, expected }) => {
                    assert_eq!(found, unsupported);
                    assert_eq!(expected, SUPPORTED_FORMAT_VERSION);
                }
                other => panic!("expected UnsupportedFormatVersion, got {other:?}"),
            }
        }
    }

    #[test]
    fn parse_optional_semantic_tag_returns_none_for_empty() {
        let result = parse_optional_semantic_tag("").expect("empty is valid");
        assert!(result.is_none());
    }

    #[test]
    fn parse_optional_semantic_tag_returns_some_for_known_tag() {
        let result = parse_optional_semantic_tag("carry").expect("known tag");
        assert_eq!(result, Some(SemanticTag::Carry));
    }

    #[test]
    fn parse_optional_semantic_tag_rejects_unknown() {
        let result = parse_optional_semantic_tag("ghost_tag");
        assert!(matches!(
            result,
            Err(AonixError::UnknownSemanticTag { .. })
        ));
    }

    #[test]
    fn parse_optional_group_returns_none_for_empty() {
        let result = parse_optional_group("").expect("empty is valid");
        assert!(result.is_none());
    }

    #[test]
    fn parse_optional_group_returns_some_for_valid_identifier() {
        let result =
            parse_optional_group("operand_a_bus").expect("valid identifier");
        assert!(result.is_some());
    }
}
