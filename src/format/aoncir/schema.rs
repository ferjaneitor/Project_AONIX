//! Serde schema mirroring the `.aoncir` TOML 1.0.0 grammar of
//! `docs/21-aoncir-syntax.md`.
//!
//! These types are intermediate: the parser deserializes a TOML document
//! into [`AoncirDocument`] and then converts it into the strongly typed
//! [`crate::circuit_model::Circuit`]. Strict structural validations (port
//! role, gate kind, arity, cycles, identifier uniqueness, output
//! assignment) belong to the conversion stage, not to this schema.
//!
//! All structs use `#[serde(deny_unknown_fields)]` so that any field
//! outside the documented grammar produces a deserialization error,
//! aligned with the spirit of the parser strictness defined in
//! `docs/21-aoncir-syntax.md`. Optional auxiliary sections
//! (`[verification]`, `[metrics]`, `[layout]`, `[history]`) are accepted
//! but kept as raw `toml::Value`s; Phase 1.B does not validate their
//! contents.

use std::collections::BTreeMap;

use serde::Deserialize;

/// Root document of a `.aoncir` file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AoncirDocument {
    pub format: FormatSection,
    pub meta: MetaSection,
    pub ports: PortsSection,

    #[serde(default, rename = "semantic_groups")]
    pub semantic_groups: Vec<SemanticGroupEntry>,

    #[serde(default)]
    pub signals: Vec<SignalEntry>,

    #[serde(default)]
    pub gates: Vec<GateEntry>,

    #[serde(default)]
    pub outputs: Vec<OutputAssignmentEntry>,

    /// Reserved for the verifier's seal. Not validated in Phase 1.B.
    #[serde(default)]
    pub verification: Option<toml::Value>,

    /// Reserved for the evaluator's metrics. Not validated in Phase 1.B.
    #[serde(default)]
    pub metrics: Option<toml::Value>,

    /// Reserved for the 2D layout. Not validated in Phase 1.B.
    #[serde(default)]
    pub layout: Option<toml::Value>,

    /// Reserved for the ancestor chain. Not validated in Phase 1.B.
    #[serde(default)]
    pub history: Option<toml::Value>,
}

/// `[format]` section. Identifies the physical syntax version.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormatSection {
    pub format_version: String,
    #[serde(default = "default_encoding")]
    pub encoding: String,
}

fn default_encoding() -> String {
    "utf-8".to_string()
}

/// `[meta]` section. Identifies the circuit.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetaSection {
    pub name: String,
    pub version: String,

    /// Structural parameters of the circuit (for example `{ width = 1 }`).
    /// Phase 1.B accepts any object content.
    #[serde(default)]
    pub parameters: BTreeMap<String, toml::Value>,

    pub level: u32,

    #[serde(default)]
    pub task_id: Option<String>,

    #[serde(default)]
    pub hash_canonical: Option<String>,

    #[serde(default)]
    pub predecessor: Option<String>,

    #[serde(default)]
    pub author: Option<String>,

    #[serde(default)]
    pub created_at: Option<String>,

    #[serde(default)]
    pub status: Option<String>,
}

/// `[ports]` section. Each subsection is an array of tables; the order of
/// appearance is the formal contract of the input / output vector.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortsSection {
    #[serde(default)]
    pub inputs: Vec<PortEntry>,

    #[serde(default)]
    pub outputs: Vec<PortEntry>,
}

/// One `[[ports.inputs]]` or `[[ports.outputs]]` entry.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortEntry {
    pub name: String,

    /// Empty string means "no semantic tag". Otherwise must belong to the
    /// closed catalog of `docs/24-semantic-tag-conventions.md`.
    #[serde(default)]
    pub semantic_tag: String,

    /// Empty string means "no group".
    #[serde(default)]
    pub group: String,

    /// Optional bit position inside the group. `bit_position = 0` is the
    /// LSB by canonical convention (see `docs/24` §U.7).
    #[serde(default)]
    pub bit_position: Option<u32>,
}

/// One `[[semantic_groups]]` entry.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticGroupEntry {
    pub id: String,
    pub kind: String,
    pub members: Vec<String>,
    pub width: u32,
}

/// One `[[signals]]` entry.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignalEntry {
    pub id: String,

    #[serde(default)]
    pub semantic_tag: String,

    #[serde(default)]
    pub group: String,
}

/// One `[[gates]]` entry.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateEntry {
    pub id: String,
    pub kind: String,
    pub inputs: Vec<String>,
    pub output: String,
}

/// One `[[outputs]]` entry — assignment of a circuit output port to a
/// signal source.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutputAssignmentEntry {
    pub port: String,
    pub source: String,
}
