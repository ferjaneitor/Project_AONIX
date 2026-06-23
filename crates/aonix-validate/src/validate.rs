//! Typed validation errors (rejection level L0 of
//! `docs/14-circuit-rejection.md`). The actual rule logic lives on
//! [`crate::BuildState`]; this module is the closed error catalog.

use thiserror::Error;

/// Why the validator rejected a proposed action.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ValidationError {
    /// Identifier does not satisfy the AONIX snake_case rule.
    #[error("invalid identifier syntax: {id:?}")]
    InvalidIdentifier { id: String },

    /// An attempt to create a gate whose kind is not AND, OR or NOT
    /// (XOR/NAND/NOR/XNOR or anything else). Absolute rejection (R2).
    #[error("forbidden gate kind {kind:?}: only AND, OR and NOT are allowed")]
    ForbiddenGateKind { kind: String },

    /// Identifier already used by a port, signal or gate.
    #[error("duplicate identifier: {id:?}")]
    DuplicateIdentifier { id: String },

    /// Wrong number of inputs for the gate kind.
    #[error("invalid arity for {kind}: given {given}, expected {expected}")]
    InvalidArity { kind: &'static str, given: usize, expected: &'static str },

    /// A gate input or output assignment references something that is
    /// neither an input port nor a declared signal.
    #[error("reference to undefined signal or input port: {reference:?}")]
    UndefinedReference { reference: String },

    /// A gate input reads an output port. Gate inputs may only come from
    /// input ports or internal signals.
    #[error("gate input reads output port {port:?}; gate inputs cannot read output ports")]
    GateInputReferencesOutputPort { port: String },

    /// A gate writes to a name that is not a declared internal signal.
    #[error("gate output {signal:?} is not a declared internal signal")]
    GateOutputNotDeclaredSignal { signal: String },

    /// Two gates would write the same internal signal.
    #[error("signal {signal:?} is already produced by another gate")]
    SignalAlreadyProduced { signal: String },

    /// A gate uses its own output as one of its inputs.
    #[error("gate {gate:?} has a direct self-loop (output used as its own input)")]
    SelfLoop { gate: String },

    /// Adding the gate would close a combinational cycle.
    #[error("gate {gate:?} would introduce a cycle")]
    CycleIntroduced { gate: String },

    /// Assignment targets a name that is not a declared output port.
    #[error("unknown output port: {port:?}")]
    UnknownOutputPort { port: String },

    /// An output port already has a source assignment.
    #[error("output port {port:?} already has an assignment")]
    DuplicateOutputAssignment { port: String },

    /// `delete_dead_signal` on a signal that is still produced or consumed.
    #[error("signal {id:?} is not dead (still produced or consumed); cannot delete")]
    SignalNotDead { id: String },

    /// `delete_gate` on a gate that does not exist.
    #[error("unknown gate: {id:?}")]
    UnknownGate { id: String },

    /// `delete_gate` on a gate whose output is still consumed by another
    /// gate or an output assignment (deleting it would dangle references).
    #[error("gate {gate:?} output {signal:?} is still consumed; cannot delete")]
    GateOutputStillConsumed { gate: String, signal: String },
}
