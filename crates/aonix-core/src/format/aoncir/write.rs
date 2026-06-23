//! Writer of [`Circuit`] into a `.aoncir` TOML 1.0.0 document.
//!
//! The writer is **deterministic** and emits **only information derived
//! from the `Circuit`**:
//!
//! - `[format]` with the fixed `format_version = "1.0.0"`.
//! - `[meta]` with neutral, deterministic placeholders. The `Circuit`
//!   model does not transport `[meta]` (name/version/level are not part
//!   of the technical truth of the graph), but the physical format
//!   requires the section to re-parse. The round-trip therefore proves
//!   **structural equivalence of the circuit**, not byte equality — which
//!   is exactly the contract agreed for Sub-phase 1.D. This is not
//!   "inventing" an opaque section: `[meta]` is mandatory format
//!   scaffolding, unlike `[verification]`/`[metrics]`/`[layout]`/
//!   `[history]`, which the writer never emits.
//! - `[[ports.inputs]]` / `[[ports.outputs]]` in the exact declared
//!   order (the formal contract of the input/output vectors).
//! - `[[semantic_groups]]` with identifier, kind, members, width.
//! - `[[signals]]`, `[[gates]]`, `[[outputs]]`.
//!
//! Gates are emitted in canonical deterministic order: ascending by gate
//! identifier (the `Circuit` stores them in a `BTreeMap`, so iteration is
//! already lexicographic).
//!
//! Guarantees by construction:
//!
//! - No primitive constants: [`SignalReference`] has only `Port` and
//!   `InternalSignal`; there is no constant variant to emit.
//! - No forbidden gate kinds: [`GateKind::canonical_name`] yields only
//!   `"AND"`, `"OR"`, `"NOT"`.
//! - No unknown fields and no opaque auxiliary sections.

use std::fmt::Write as _;

use crate::circuit_model::{Circuit, GateKind, Port, SignalReference};

/// The format version every writer output declares.
const WRITTEN_FORMAT_VERSION: &str = "1.0.0";

/// Serializes a [`Circuit`] into a `.aoncir` TOML 1.0.0 string that the
/// parser accepts and that re-parses to a structurally equivalent
/// circuit.
pub fn write(circuit: &Circuit) -> String {
    let mut out = String::new();
    write_format(&mut out);
    write_meta(&mut out);
    write_input_ports(&mut out, circuit);
    write_output_ports(&mut out, circuit);
    write_semantic_groups(&mut out, circuit);
    write_signals(&mut out, circuit);
    write_gates(&mut out, circuit);
    write_output_assignments(&mut out, circuit);
    out
}

fn write_format(out: &mut String) {
    out.push_str("[format]\n");
    let _ = writeln!(out, "format_version = \"{WRITTEN_FORMAT_VERSION}\"");
    out.push_str("encoding = \"utf-8\"\n\n");
}

fn write_meta(out: &mut String) {
    // Neutral, deterministic placeholders. The Circuit does not carry
    // meta; these satisfy the mandatory format scaffolding only.
    out.push_str("[meta]\n");
    out.push_str("name = \"unnamed_circuit\"\n");
    out.push_str("version = \"1.0.0\"\n");
    out.push_str("level = 0\n\n");
}

fn write_port(out: &mut String, header: &str, port: &Port) {
    out.push_str(header);
    let _ = writeln!(out, "name = \"{}\"", port.identifier.as_str());
    let tag = port
        .semantic_tag
        .map(|tag| tag.canonical_name())
        .unwrap_or("");
    let _ = writeln!(out, "semantic_tag = \"{tag}\"");
    let group = port
        .group
        .as_ref()
        .map(|group| group.as_str())
        .unwrap_or("");
    let _ = writeln!(out, "group = \"{group}\"");
    if let Some(position) = port.bit_position {
        let _ = writeln!(out, "bit_position = {position}");
    }
    out.push('\n');
}

fn write_input_ports(out: &mut String, circuit: &Circuit) {
    for port in circuit.inputs_in_order() {
        write_port(out, "[[ports.inputs]]\n", port);
    }
}

fn write_output_ports(out: &mut String, circuit: &Circuit) {
    for port in circuit.outputs_in_order() {
        write_port(out, "[[ports.outputs]]\n", port);
    }
}

fn write_semantic_groups(out: &mut String, circuit: &Circuit) {
    for group in circuit.semantic_groups() {
        out.push_str("[[semantic_groups]]\n");
        let _ = writeln!(out, "id = \"{}\"", group.identifier.as_str());
        let _ = writeln!(out, "kind = \"{}\"", group.kind.canonical_name());
        out.push_str("members = [");
        for (index, member) in group.members.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            let _ = write!(out, "\"{}\"", member.as_str());
        }
        out.push_str("]\n");
        let _ = writeln!(out, "width = {}", group.width);
        out.push('\n');
    }
}

fn write_signals(out: &mut String, circuit: &Circuit) {
    for signal in circuit.signals() {
        out.push_str("[[signals]]\n");
        let _ = writeln!(out, "id = \"{}\"", signal.identifier.as_str());
        let tag = signal
            .semantic_tag
            .map(|tag| tag.canonical_name())
            .unwrap_or("");
        let _ = writeln!(out, "semantic_tag = \"{tag}\"");
        let group = signal
            .group
            .as_ref()
            .map(|group| group.as_str())
            .unwrap_or("");
        let _ = writeln!(out, "group = \"{group}\"");
        out.push('\n');
    }
}

fn signal_reference_name(reference: &SignalReference) -> &str {
    // Exhaustive over the closed two-variant enum. There is no constant
    // variant, so the writer can never emit a primitive constant.
    match reference {
        SignalReference::Port(port) => port.as_str(),
        SignalReference::InternalSignal(signal) => signal.as_str(),
    }
}

fn write_gates(out: &mut String, circuit: &Circuit) {
    // circuit.gates() iterates the BTreeMap in ascending GateIdentifier
    // order: canonical and deterministic.
    for gate in circuit.gates() {
        out.push_str("[[gates]]\n");
        let _ = writeln!(out, "id = \"{}\"", gate.identifier.as_str());
        // canonical_name yields exactly "AND" | "OR" | "NOT".
        let kind_name: &str = match gate.kind {
            GateKind::And => GateKind::And.canonical_name(),
            GateKind::Or => GateKind::Or.canonical_name(),
            GateKind::Not => GateKind::Not.canonical_name(),
        };
        let _ = writeln!(out, "kind = \"{kind_name}\"");
        out.push_str("inputs = [");
        for (index, input) in gate.inputs.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            let _ = write!(out, "\"{}\"", signal_reference_name(input));
        }
        out.push_str("]\n");
        let _ = writeln!(out, "output = \"{}\"", gate.output.as_str());
        out.push('\n');
    }
}

fn write_output_assignments(out: &mut String, circuit: &Circuit) {
    // Emit assignments in the declared order of output ports.
    for port in circuit.outputs_in_order() {
        if let Some(source) = circuit.output_assignment(&port.identifier) {
            out.push_str("[[outputs]]\n");
            let _ = writeln!(out, "port = \"{}\"", port.identifier.as_str());
            let _ = writeln!(out, "source = \"{}\"", signal_reference_name(source));
            out.push('\n');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::aoncir::parse;

    const INVERTER: &str = r#"
[format]
format_version = "1.0.0"
encoding = "utf-8"

[meta]
name = "inverter"
version = "1.0.0"
level = 1

[[ports.inputs]]
name = "data_input"
semantic_tag = "data_bit"
group = ""

[[ports.outputs]]
name = "data_output"
semantic_tag = "data_bit"
group = ""

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

    #[test]
    fn writer_emits_fixed_format_version() {
        let circuit = parse::parse(INVERTER).expect("inverter parses");
        let text = write(&circuit);
        assert!(text.contains("format_version = \"1.0.0\""));
    }

    #[test]
    fn writer_output_reparses_to_equivalent_circuit() {
        let original = parse::parse(INVERTER).expect("inverter parses");
        let text = write(&original);
        let reparsed = parse::parse(&text).expect("written output reparses");
        assert_eq!(original, reparsed);
    }

    #[test]
    fn writer_is_deterministic() {
        let circuit = parse::parse(INVERTER).expect("inverter parses");
        assert_eq!(write(&circuit), write(&circuit));
    }

    #[test]
    fn writer_emits_no_opaque_sections() {
        let circuit = parse::parse(INVERTER).expect("inverter parses");
        let text = write(&circuit);
        assert!(!text.contains("[verification]"));
        assert!(!text.contains("[metrics]"));
        assert!(!text.contains("[layout]"));
        assert!(!text.contains("[history]"));
    }
}
