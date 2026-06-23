//! Canonical hashing of a [`Circuit`] (sub-phase 1.J).
//!
//! Implements `hash_canonical` per `docs/03-format-aoncir.md` §"Hash
//! canónico" and `docs/21-aoncir-syntax.md`. The hash is computed over a
//! deterministic serialization of the **logical model only**:
//!
//! - input and output ports in their declared order (that order is the
//!   formal contract of the I/O vectors, so reordering ports yields a
//!   different circuit and a different hash),
//! - internal signals and gates in canonical identifier order (the
//!   [`Circuit`] stores them in `BTreeMap`s, so iteration is already
//!   lexicographic and independent of the textual order of `[[gates]]`),
//! - semantic groups sorted by identifier (group order is not semantic),
//! - output assignments in declared output-port order.
//!
//! It deliberately **excludes** non-canonical data: `[meta]`
//! (author/date/name/version), `[layout]`, `[verification]`, `[metrics]`
//! and `[history]`. It is therefore invariant to the textual order of
//! `[[gates]]`, comments and whitespace (docs/21 rule on hash determinism),
//! but sensitive to port order and semantic tags.
//!
//! This is a *structural* hash: it includes the internal identifiers, which
//! `docs/03` rule 10 declares part of the canonical truth. Full
//! rename/isomorphism invariance (recognising two circuits that differ only
//! in signal names as equal) is an open canonicalization problem deferred
//! in `docs/21` and is **not** attempted here. The `predecessor` lineage
//! field of `[meta]` is likewise not part of the structural hash; version
//! chaining is a memory/versioning concern (Phase 4), not a property of the
//! circuit graph.

use std::fmt::Write as _;

use crate::circuit_model::{Circuit, GateKind, Port, SignalReference};

/// Algorithm prefix of every canonical hash string, per `docs/21`
/// (`<algorithm>:<hex>`).
pub const HASH_ALGORITHM_PREFIX: &str = "blake3";

/// Computes the canonical hash of `circuit` as the string
/// `"blake3:<64-hex-chars>"`.
///
/// Same logical circuit ⇒ same hash, every time and on every machine
/// (this is the Phase 1 acceptance criterion "mismo `.aoncir` ⇒ mismo hash
/// canónico"). See the module documentation for exactly what the hash does
/// and does not depend on.
pub fn hash_canonical(circuit: &Circuit) -> String {
    let serialization = canonical_serialization(circuit);
    let digest = blake3::hash(serialization.as_bytes());
    format!("{HASH_ALGORITHM_PREFIX}:{}", digest.to_hex())
}

/// Builds the deterministic canonical byte serialization that
/// [`hash_canonical`] hashes. Exposed within the crate so tests can assert
/// on the pre-image and so debugging tools can show what was hashed.
pub(crate) fn canonical_serialization(circuit: &Circuit) -> String {
    let mut out = String::new();
    out.push_str("AONIX-CANON-v1\n");

    // 1. Input ports, in declared order (formal contract of InputVector).
    let _ = writeln!(out, "INPUTS:{}", circuit.input_count());
    for port in circuit.inputs_in_order() {
        write_port_line(&mut out, port);
    }

    // 2. Output ports, in declared order (formal contract of OutputVector).
    let _ = writeln!(out, "OUTPUTS:{}", circuit.output_count());
    for port in circuit.outputs_in_order() {
        write_port_line(&mut out, port);
    }

    // 3. Semantic groups, sorted by identifier (the order in which groups
    //    are declared is not semantically meaningful).
    let mut groups: Vec<_> = circuit.semantic_groups().iter().collect();
    groups.sort_by(|left, right| left.identifier.as_str().cmp(right.identifier.as_str()));
    let _ = writeln!(out, "GROUPS:{}", groups.len());
    for group in groups {
        let members: Vec<&str> = group.members.iter().map(|member| member.as_str()).collect();
        let _ = writeln!(
            out,
            "G|{}|{}|{}|{}",
            group.identifier.as_str(),
            group.kind.canonical_name(),
            group.width,
            members.join(",")
        );
    }

    // 4. Internal signals, in canonical identifier order (BTreeMap order).
    let _ = writeln!(out, "SIGNALS:{}", circuit.signal_count());
    for signal in circuit.signals() {
        let _ = writeln!(
            out,
            "S|{}|{}|{}",
            signal.identifier.as_str(),
            signal.semantic_tag.map(|tag| tag.canonical_name()).unwrap_or(""),
            signal.group.as_ref().map(|group| group.as_str()).unwrap_or("")
        );
    }

    // 5. Gates, in canonical identifier order (BTreeMap order). This is what
    //    makes the hash invariant to the textual order of `[[gates]]`.
    let _ = writeln!(out, "GATES:{}", circuit.gate_count());
    for gate in circuit.gates() {
        let inputs: Vec<&str> = gate.inputs.iter().map(signal_reference_name).collect();
        let _ = writeln!(
            out,
            "X|{}|{}|{}|{}",
            gate.identifier.as_str(),
            gate_kind_name(gate.kind),
            inputs.join(","),
            gate.output.as_str()
        );
    }

    // 6. Output assignments, in declared output-port order.
    let _ = writeln!(out, "ASSIGN:{}", circuit.output_count());
    for port in circuit.outputs_in_order() {
        if let Some(source) = circuit.output_assignment(&port.identifier) {
            let _ = writeln!(
                out,
                "A|{}|{}",
                port.identifier.as_str(),
                signal_reference_name(source)
            );
        }
    }

    out
}

fn write_port_line(out: &mut String, port: &Port) {
    let _ = writeln!(
        out,
        "P|{}|{}|{}|{}",
        port.identifier.as_str(),
        port.semantic_tag.map(|tag| tag.canonical_name()).unwrap_or(""),
        port.group.as_ref().map(|group| group.as_str()).unwrap_or(""),
        port.bit_position.map(|position| position.to_string()).unwrap_or_default()
    );
}

fn gate_kind_name(kind: GateKind) -> &'static str {
    match kind {
        GateKind::And => "AND",
        GateKind::Or => "OR",
        GateKind::Not => "NOT",
    }
}

fn signal_reference_name(reference: &SignalReference) -> &str {
    match reference {
        SignalReference::Port(port) => port.as_str(),
        SignalReference::InternalSignal(signal) => signal.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::aoncir::{parse, write};

    const INVERTER: &str = r#"
[format]
format_version = "1.0.0"

[meta]
name = "inverter"
version = "1.0.0"
level = 1

[[ports.inputs]]
name = "data_input"
semantic_tag = "data_bit"

[[ports.outputs]]
name = "data_output"
semantic_tag = "data_bit"

[[signals]]
id = "data_output_internal"

[[gates]]
id = "g_invert"
kind = "NOT"
inputs = ["data_input"]
output = "data_output_internal"

[[outputs]]
port = "data_output"
source = "data_output_internal"
"#;

    /// Same `[[meta]]` differs (name/level), gates listed in a different
    /// order, extra whitespace — but the logical circuit is identical, so
    /// the canonical hash must match.
    const INVERTER_REORDERED: &str = r#"
[format]
format_version = "1.0.0"

[meta]
name = "a_completely_different_name"
version = "9.9.9"
level = 4

[[ports.inputs]]
name = "data_input"
semantic_tag = "data_bit"

[[ports.outputs]]
name = "data_output"
semantic_tag = "data_bit"

[[signals]]
id = "data_output_internal"

[[gates]]
id = "g_invert"
kind = "NOT"
inputs = ["data_input"]
output = "data_output_internal"

[[outputs]]
port = "data_output"
source = "data_output_internal"

[verification]
result = "PASS"

[layout]
note = "ignored for hashing"
"#;

    fn hash_of(source: &str) -> String {
        hash_canonical(&parse(source).expect("valid fixture"))
    }

    #[test]
    fn hash_has_blake3_prefix_and_hex_body() {
        let hash = hash_of(INVERTER);
        let (prefix, body) = hash.split_once(':').expect("prefix present");
        assert_eq!(prefix, "blake3");
        assert_eq!(body.len(), 64, "blake3 hex digest is 64 chars");
        assert!(body.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn same_circuit_yields_same_hash() {
        assert_eq!(hash_of(INVERTER), hash_of(INVERTER));
    }

    #[test]
    fn hash_ignores_meta_layout_and_verification_sections() {
        // Different meta/verification/layout, identical graph ⇒ same hash.
        assert_eq!(hash_of(INVERTER), hash_of(INVERTER_REORDERED));
    }

    #[test]
    fn hash_survives_write_round_trip() {
        let circuit = parse(INVERTER).expect("valid");
        let reparsed = parse(&write(&circuit)).expect("written output reparses");
        assert_eq!(hash_canonical(&circuit), hash_canonical(&reparsed));
    }

    #[test]
    fn different_gate_kind_yields_different_hash() {
        // Same interface, but the gate is wired differently: feed the input
        // straight through instead of inverting it. Different graph ⇒
        // different hash.
        let passthrough = r#"
[format]
format_version = "1.0.0"

[meta]
name = "inverter"
version = "1.0.0"
level = 1

[[ports.inputs]]
name = "data_input"
semantic_tag = "data_bit"

[[ports.outputs]]
name = "data_output"
semantic_tag = "data_bit"

[[outputs]]
port = "data_output"
source = "data_input"
"#;
        assert_ne!(hash_of(INVERTER), hash_of(passthrough));
    }

    #[test]
    fn port_order_changes_hash() {
        let a_then_b = r#"
[format]
format_version = "1.0.0"
[meta]
name = "two_in"
version = "1.0.0"
level = 1
[[ports.inputs]]
name = "operand_a"
[[ports.inputs]]
name = "operand_b"
[[ports.outputs]]
name = "result"
[[outputs]]
port = "result"
source = "operand_a"
"#;
        let b_then_a = r#"
[format]
format_version = "1.0.0"
[meta]
name = "two_in"
version = "1.0.0"
level = 1
[[ports.inputs]]
name = "operand_b"
[[ports.inputs]]
name = "operand_a"
[[ports.outputs]]
name = "result"
[[outputs]]
port = "result"
source = "operand_a"
"#;
        assert_ne!(hash_of(a_then_b), hash_of(b_then_a));
    }
}
