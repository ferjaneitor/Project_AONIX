//! AONIX — AND-OR-NOT Integrated eXploration.
//!
//! Public library entry point.
//!
//! - [`circuit_model`] holds the foundational data types of the digital
//!   circuit model.
//! - [`format`] holds the file-format readers (currently only the
//!   `.aoncir` parser; see [`format::aoncir`]).
//!
//! AONIX enforces the absolute rule R2 ("only AND, OR and NOT as primitive
//! gates") at the type system level: see [`circuit_model::GateKind`].

pub mod circuit_model;
pub mod format;
