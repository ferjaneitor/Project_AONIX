//! Construction actions an agent proposes
//! (`docs/08-actions-and-rewards.md` §"Acciones permitidas").

use aonix_core::circuit_model::{GateKind, SemanticTag};

use crate::validate::ValidationError;

/// The category of an [`Action`], used to enumerate which kinds of action
/// are currently legal without committing to a concrete identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind {
    DeclareSignal,
    CreateGate,
    AssignOutput,
    DeleteSignal,
    DeleteGate,
    StopConstruction,
}

/// A single action over the partial circuit. **Closed set**: the only
/// gate-creating variant carries a [`GateKind`], which is itself a closed
/// AND/OR/NOT enum, so no derived primitive can be expressed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Declare an internal signal (a wire a gate will later drive).
    DeclareSignal {
        id: String,
        semantic_tag: Option<SemanticTag>,
        group: Option<String>,
    },
    /// Create an AND/OR/NOT gate writing to a declared internal signal.
    CreateGate {
        id: String,
        kind: GateKind,
        inputs: Vec<String>,
        output: String,
    },
    /// Tie a circuit output port to a signal source (an input port or a
    /// declared internal signal).
    AssignOutput { port: String, source: String },
    /// Remove a dead internal signal.
    DeleteSignal { id: String },
    /// Remove a gate whose output is no longer consumed.
    DeleteGate { id: String },
    /// Declare the construction finished (the coordinator then verifies).
    StopConstruction,
}

impl Action {
    /// The category of this action.
    pub fn kind(&self) -> ActionKind {
        match self {
            Action::DeclareSignal { .. } => ActionKind::DeclareSignal,
            Action::CreateGate { .. } => ActionKind::CreateGate,
            Action::AssignOutput { .. } => ActionKind::AssignOutput,
            Action::DeleteSignal { .. } => ActionKind::DeleteSignal,
            Action::DeleteGate { .. } => ActionKind::DeleteGate,
            Action::StopConstruction => ActionKind::StopConstruction,
        }
    }

    /// Agent-facing gate-creation constructor. The gate kind is parsed
    /// through the closed AND/OR/NOT catalog and **any derived primitive**
    /// (XOR/NAND/NOR/XNOR, or any unknown name) is rejected at the action
    /// layer with [`ValidationError::ForbiddenGateKind`] — the direct
    /// enforcement of R2 on the action interface
    /// (`docs/08` §"Acciones prohibidas").
    pub fn create_gate(
        id: impl Into<String>,
        kind_name: &str,
        inputs: Vec<String>,
        output: impl Into<String>,
    ) -> Result<Action, ValidationError> {
        let kind = GateKind::from_canonical_name(kind_name)
            .map_err(|_| ValidationError::ForbiddenGateKind {
                kind: kind_name.to_string(),
            })?;
        Ok(Action::CreateGate {
            id: id.into(),
            kind,
            inputs,
            output: output.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_gate_accepts_and_or_not() {
        for (name, expected) in [
            ("AND", GateKind::And),
            ("OR", GateKind::Or),
            ("NOT", GateKind::Not),
        ] {
            let action = Action::create_gate("g", name, vec!["a".into(), "b".into()], "out")
                .expect("AND/OR/NOT accepted");
            match action {
                Action::CreateGate { kind, .. } => assert_eq!(kind, expected),
                other => panic!("expected CreateGate, got {other:?}"),
            }
        }
    }

    #[test]
    fn create_gate_rejects_derived_primitives() {
        for forbidden in ["XOR", "XNOR", "NAND", "NOR", "buffer", "and"] {
            let result = Action::create_gate("g", forbidden, vec!["a".into(), "b".into()], "out");
            assert!(
                matches!(result, Err(ValidationError::ForbiddenGateKind { ref kind }) if kind == forbidden),
                "{forbidden:?} must be rejected as a forbidden gate kind",
            );
        }
    }
}
