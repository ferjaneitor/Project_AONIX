//! AONIX — AND-OR-NOT Integrated eXploration.
//!
//! Public library entry point.
//!
//! - [`circuit_model`] holds the foundational data types of the digital
//!   circuit model.
//! - [`format`] holds the file-format readers and writer (currently only
//!   the `.aoncir` format; see [`format::aoncir`]).
//! - [`simulation`] holds the deterministic combinational simulator
//!   (Sub-phase 1.E).
//!
//! AONIX enforces the absolute rule R2 ("only AND, OR and NOT as primitive
//! gates") at the type system level: see [`circuit_model::GateKind`].

pub mod circuit_model;
pub mod format;
pub mod simulation;
