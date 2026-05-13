//! Circuit model — foundational data types of AONIX.
//!
//! This module groups the core data structures: identifiers, semantic tags,
//! signals, gates, ports, logical values, and the circuit itself.
//!
//! The absolute rule R2 of AONIX (only AND, OR and NOT as primitive gates)
//! is enforced **at the type system level**: [`gate::GateKind`] is a closed
//! enum with exactly three variants. No external code, and no parser, can
//! introduce a new primitive without modifying this module.

pub mod error;
pub mod value;
pub mod port;
pub mod signal;
pub mod gate;
pub mod circuit;

pub use error::{AonixError, AonixResult};
pub use value::{Bit, InputVector, OutputVector};
pub use port::{GroupIdentifier, Port, PortIdentifier, PortRole, SemanticTag};
pub use signal::{Signal, SignalIdentifier};
pub use gate::{Gate, GateIdentifier, GateKind, SignalReference};
pub use circuit::{Circuit, CircuitBuilder};
