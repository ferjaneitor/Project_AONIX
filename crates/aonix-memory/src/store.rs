//! Flat-file canonical/historical memory store.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use aonix_core::circuit_model::{AonixError, Circuit};
use aonix_core::format::aoncir::{hash_canonical, parse, write};
use aonix_eval::{evaluate, is_strictly_better, Criterion, DEFAULT_RANKING};
use thiserror::Error;

/// Identity of a circuit family member: a canonical `name` plus a
/// `parameters` token (e.g. `width`). Each `(name, parameters)` keeps its own
/// official-active version and its own history (`docs/19` V.7).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CircuitKey {
    name: String,
    parameters: String,
}

impl CircuitKey {
    /// Builds a key, validating that both tokens are filesystem-safe
    /// (`[a-z0-9_]`, non-empty) so they cannot escape the store root.
    pub fn new(name: impl Into<String>, parameters: impl Into<String>) -> Result<Self, MemoryError> {
        let name = name.into();
        let parameters = parameters.into();
        validate_token(&name)?;
        validate_token(&parameters)?;
        Ok(Self { name, parameters })
    }

    /// Key with the default `"base"` parameters slot (non-parametric circuit).
    pub fn simple(name: impl Into<String>) -> Result<Self, MemoryError> {
        Self::new(name, "base")
    }

    /// The circuit's canonical name.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// The parameters token.
    pub fn parameters(&self) -> &str {
        &self.parameters
    }
}

/// Result of a [`MemoryStore::promote`] call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromotionOutcome {
    /// No prior active version existed; the candidate became active.
    InstalledFirst { hash: String },
    /// The candidate strictly improved on the incumbent, which was archived.
    Replaced { new_hash: String, archived_hash: String },
    /// The candidate is structurally identical to the active version
    /// (same canonical hash); nothing changed (dedup).
    AlreadyActive { hash: String },
    /// The candidate did not strictly improve on the active version; it was
    /// recorded in experimental memory and the active version kept.
    RejectedNotBetter { candidate_hash: String, active_hash: String },
}

/// Why a memory operation failed.
#[derive(Debug, Error)]
pub enum MemoryError {
    /// Filesystem error.
    #[error("memory I/O error: {0}")]
    Io(#[from] io::Error),
    /// A stored `.aoncir` failed to parse (corruption or version skew).
    #[error("stored .aoncir failed to parse: {0}")]
    Parse(#[from] AonixError),
    /// A key token was empty or contained unsafe characters.
    #[error("invalid memory key: {detail}")]
    InvalidKey { detail: String },
}

/// A flat-file memory store rooted at a directory.
///
/// Layout per key:
///
/// ```text
/// <root>/<name>/<parameters>/active.aoncir
/// <root>/<name>/<parameters>/history/<hash_hex>.aoncir
/// <root>/<name>/<parameters>/experimental/<hash_hex>.aoncir
/// ```
#[derive(Debug, Clone)]
pub struct MemoryStore {
    root: PathBuf,
}

impl MemoryStore {
    /// Opens (or lazily creates on first write) a store at `root`.
    pub fn open(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Promotes `circuit` for `key` using the [`DEFAULT_RANKING`].
    pub fn promote(&self, key: &CircuitKey, circuit: &Circuit) -> Result<PromotionOutcome, MemoryError> {
        self.promote_with_ranking(key, circuit, DEFAULT_RANKING)
    }

    /// Promotes `circuit` for `key`, comparing against the incumbent under
    /// `ranking`. The official-active version only changes on a **strict**
    /// improvement; the whole operation is atomic (the incumbent is archived
    /// before the active file is replaced, and the active file is swapped in
    /// by an atomic rename).
    pub fn promote_with_ranking(
        &self,
        key: &CircuitKey,
        circuit: &Circuit,
        ranking: &[Criterion],
    ) -> Result<PromotionOutcome, MemoryError> {
        let candidate_text = write(circuit);
        let candidate_hash = hash_canonical(circuit);
        let candidate_hex = hex_part(&candidate_hash).to_string();

        match self.read_optional(&self.active_path(key))? {
            None => {
                write_atomic(&self.active_path(key), &candidate_text)?;
                Ok(PromotionOutcome::InstalledFirst {
                    hash: candidate_hash,
                })
            }
            Some(active_text) => {
                let active_circuit = parse(&active_text)?;
                let active_hash = hash_canonical(&active_circuit);
                if active_hash == candidate_hash {
                    return Ok(PromotionOutcome::AlreadyActive {
                        hash: candidate_hash,
                    });
                }

                let candidate_metrics = evaluate(circuit);
                let active_metrics = evaluate(&active_circuit);
                if is_strictly_better(&candidate_metrics, &active_metrics, ranking) {
                    // Archive the incumbent first (append-only); only then
                    // swap the active file in atomically.
                    let active_hex = hex_part(&active_hash).to_string();
                    write_if_absent(&self.history_dir(key).join(format!("{active_hex}.aoncir")), &active_text)?;
                    write_atomic(&self.active_path(key), &candidate_text)?;
                    Ok(PromotionOutcome::Replaced {
                        new_hash: candidate_hash,
                        archived_hash: active_hash,
                    })
                } else {
                    write_if_absent(
                        &self.experimental_dir(key).join(format!("{candidate_hex}.aoncir")),
                        &candidate_text,
                    )?;
                    Ok(PromotionOutcome::RejectedNotBetter {
                        candidate_hash,
                        active_hash,
                    })
                }
            }
        }
    }

    /// The official-active circuit for `key`, if any.
    pub fn active(&self, key: &CircuitKey) -> Result<Option<Circuit>, MemoryError> {
        match self.read_optional(&self.active_path(key))? {
            Some(text) => Ok(Some(parse(&text)?)),
            None => Ok(None),
        }
    }

    /// The canonical hash of the active version for `key`, if any.
    pub fn active_hash(&self, key: &CircuitKey) -> Result<Option<String>, MemoryError> {
        Ok(self.active(key)?.as_ref().map(hash_canonical))
    }

    /// The hash-hex names of every archived (historical) version for `key`,
    /// sorted deterministically.
    pub fn history_hexes(&self, key: &CircuitKey) -> Result<Vec<String>, MemoryError> {
        list_hexes(&self.history_dir(key))
    }

    /// Recovers an archived circuit by its hash-hex, if present (`docs/19`
    /// §"Cualquier versión histórica puede consultarse").
    pub fn historical(&self, key: &CircuitKey, hash_hex: &str) -> Result<Option<Circuit>, MemoryError> {
        match self.read_optional(&self.history_dir(key).join(format!("{hash_hex}.aoncir")))? {
            Some(text) => Ok(Some(parse(&text)?)),
            None => Ok(None),
        }
    }

    /// The hash-hex names of every experimental (rejected) version for `key`.
    pub fn experimental_hexes(&self, key: &CircuitKey) -> Result<Vec<String>, MemoryError> {
        list_hexes(&self.experimental_dir(key))
    }

    /// Every `(name, parameters)` key present in the store, sorted.
    pub fn list(&self) -> Result<Vec<CircuitKey>, MemoryError> {
        let mut keys = Vec::new();
        if !self.root.exists() {
            return Ok(keys);
        }
        for name_entry in fs::read_dir(&self.root)? {
            let name_entry = name_entry?;
            if !name_entry.file_type()?.is_dir() {
                continue;
            }
            let name = name_entry.file_name().to_string_lossy().into_owned();
            for param_entry in fs::read_dir(name_entry.path())? {
                let param_entry = param_entry?;
                if !param_entry.file_type()?.is_dir() {
                    continue;
                }
                let parameters = param_entry.file_name().to_string_lossy().into_owned();
                keys.push(CircuitKey { name: name.clone(), parameters });
            }
        }
        keys.sort();
        Ok(keys)
    }

    fn key_dir(&self, key: &CircuitKey) -> PathBuf {
        self.root.join(&key.name).join(&key.parameters)
    }
    fn active_path(&self, key: &CircuitKey) -> PathBuf {
        self.key_dir(key).join("active.aoncir")
    }
    fn history_dir(&self, key: &CircuitKey) -> PathBuf {
        self.key_dir(key).join("history")
    }
    fn experimental_dir(&self, key: &CircuitKey) -> PathBuf {
        self.key_dir(key).join("experimental")
    }

    fn read_optional(&self, path: &Path) -> Result<Option<String>, MemoryError> {
        match fs::read_to_string(path) {
            Ok(text) => Ok(Some(text)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(MemoryError::Io(error)),
        }
    }
}

fn validate_token(token: &str) -> Result<(), MemoryError> {
    if token.is_empty()
        || !token
            .chars()
            .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_')
    {
        return Err(MemoryError::InvalidKey {
            detail: format!("token {token:?} must be non-empty and match [a-z0-9_]"),
        });
    }
    Ok(())
}

fn hex_part(hash: &str) -> &str {
    hash.split_once(':').map(|(_, hex)| hex).unwrap_or(hash)
}

fn write_atomic(path: &Path, content: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("tmp");
    fs::write(&temp, content)?;
    fs::rename(&temp, path)?;
    Ok(())
}

fn write_if_absent(path: &Path, content: &str) -> io::Result<()> {
    if path.exists() {
        return Ok(());
    }
    write_atomic(path, content)
}

fn list_hexes(dir: &Path) -> Result<Vec<String>, MemoryError> {
    let mut hexes = Vec::new();
    if !dir.exists() {
        return Ok(hexes);
    }
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("aoncir") {
            if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
                hexes.push(stem.to_string());
            }
        }
    }
    hexes.sort();
    Ok(hexes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aonix_core::circuit_model::{
        CircuitBuilder, Gate, GateIdentifier, GateKind, Port, PortIdentifier, PortRole, Signal,
        SignalIdentifier, SignalReference,
    };

    fn temp_root(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("aonix_memory_test_{label}"));
        let _ = fs::remove_dir_all(&root);
        root
    }

    fn input_port(name: &str) -> Port {
        Port::new(PortIdentifier::new(name).unwrap(), PortRole::Input, None, None, None)
    }
    fn output_port(name: &str) -> Port {
        Port::new(PortIdentifier::new(name).unwrap(), PortRole::Output, None, None, None)
    }

    /// Lean circuit: out = NOT a (1 gate).
    fn inverter() -> Circuit {
        let mut builder = CircuitBuilder::new();
        builder.add_input_port(input_port("operand_a")).unwrap();
        builder.add_output_port(output_port("result")).unwrap();
        builder.add_signal(Signal::new(SignalIdentifier::new("inner").unwrap(), None, None)).unwrap();
        builder
            .add_gate(
                Gate::new(
                    GateIdentifier::new("g_not").unwrap(),
                    GateKind::Not,
                    vec![SignalReference::Port(PortIdentifier::new("operand_a").unwrap())],
                    SignalIdentifier::new("inner").unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
        builder
            .assign_output(
                PortIdentifier::new("result").unwrap(),
                SignalReference::InternalSignal(SignalIdentifier::new("inner").unwrap()),
            )
            .unwrap();
        builder.finish().unwrap()
    }

    /// Same interface, worse structure: out = NOT a, plus a dead extra NOT.
    fn inverter_with_dead_gate() -> Circuit {
        let mut builder = CircuitBuilder::new();
        builder.add_input_port(input_port("operand_a")).unwrap();
        builder.add_output_port(output_port("result")).unwrap();
        builder.add_signal(Signal::new(SignalIdentifier::new("inner").unwrap(), None, None)).unwrap();
        builder.add_signal(Signal::new(SignalIdentifier::new("dead").unwrap(), None, None)).unwrap();
        builder
            .add_gate(
                Gate::new(
                    GateIdentifier::new("g_not").unwrap(),
                    GateKind::Not,
                    vec![SignalReference::Port(PortIdentifier::new("operand_a").unwrap())],
                    SignalIdentifier::new("inner").unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
        builder
            .add_gate(
                Gate::new(
                    GateIdentifier::new("g_dead").unwrap(),
                    GateKind::Not,
                    vec![SignalReference::Port(PortIdentifier::new("operand_a").unwrap())],
                    SignalIdentifier::new("dead").unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
        builder
            .assign_output(
                PortIdentifier::new("result").unwrap(),
                SignalReference::InternalSignal(SignalIdentifier::new("inner").unwrap()),
            )
            .unwrap();
        builder.finish().unwrap()
    }

    #[test]
    fn first_promotion_installs_active() {
        let store = MemoryStore::open(temp_root("first_install"));
        let key = CircuitKey::simple("inverter").unwrap();
        let outcome = store.promote(&key, &inverter()).unwrap();
        assert!(matches!(outcome, PromotionOutcome::InstalledFirst { .. }));
        assert!(store.active(&key).unwrap().is_some());
    }

    #[test]
    fn identical_circuit_is_deduplicated() {
        let store = MemoryStore::open(temp_root("dedup"));
        let key = CircuitKey::simple("inverter").unwrap();
        store.promote(&key, &inverter()).unwrap();
        let outcome = store.promote(&key, &inverter()).unwrap();
        assert!(matches!(outcome, PromotionOutcome::AlreadyActive { .. }));
        assert!(store.history_hexes(&key).unwrap().is_empty());
    }

    #[test]
    fn worse_candidate_is_rejected_to_experimental() {
        let store = MemoryStore::open(temp_root("reject"));
        let key = CircuitKey::simple("inverter").unwrap();
        store.promote(&key, &inverter()).unwrap();
        let outcome = store.promote(&key, &inverter_with_dead_gate()).unwrap();
        assert!(matches!(outcome, PromotionOutcome::RejectedNotBetter { .. }));
        // Active is unchanged (still the lean inverter).
        let active_hash = store.active_hash(&key).unwrap().unwrap();
        assert_eq!(active_hash, hash_canonical(&inverter()));
        assert_eq!(store.experimental_hexes(&key).unwrap().len(), 1);
        assert!(store.history_hexes(&key).unwrap().is_empty());
    }

    #[test]
    fn better_candidate_replaces_and_archives_incumbent() {
        let store = MemoryStore::open(temp_root("replace"));
        let key = CircuitKey::simple("inverter").unwrap();
        // Start with the worse version as active.
        store.promote(&key, &inverter_with_dead_gate()).unwrap();
        let outcome = store.promote(&key, &inverter()).unwrap();
        assert!(matches!(outcome, PromotionOutcome::Replaced { .. }));
        // Active is now the lean inverter; the worse one is in history.
        assert_eq!(store.active_hash(&key).unwrap().unwrap(), hash_canonical(&inverter()));
        let archived_hex = hex_part(&hash_canonical(&inverter_with_dead_gate())).to_string();
        assert_eq!(store.history_hexes(&key).unwrap(), vec![archived_hex.clone()]);
        // Historical recovery without loss.
        let recovered = store.historical(&key, &archived_hex).unwrap().unwrap();
        assert_eq!(hash_canonical(&recovered), hash_canonical(&inverter_with_dead_gate()));
    }

    #[test]
    fn list_reports_keys() {
        let root = temp_root("list");
        let store = MemoryStore::open(&root);
        store.promote(&CircuitKey::new("inverter", "base").unwrap(), &inverter()).unwrap();
        store.promote(&CircuitKey::new("adder", "w1").unwrap(), &inverter()).unwrap();
        let keys = store.list().unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.iter().any(|k| k.name() == "adder" && k.parameters() == "w1"));
    }

    #[test]
    fn invalid_key_is_rejected() {
        assert!(matches!(CircuitKey::new("Bad Name", "base"), Err(MemoryError::InvalidKey { .. })));
        assert!(matches!(CircuitKey::new("ok", "../escape"), Err(MemoryError::InvalidKey { .. })));
    }
}
