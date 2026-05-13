//! AONIX — AND-OR-NOT Integrated eXploration.
//!
//! Public library entry point. See [`circuit_model`] for the foundational
//! data types of the digital circuit model.
//!
//! AONIX enforces the absolute rule R2 ("only AND, OR and NOT as primitive
//! gates") at the type system level: see [`circuit_model::GateKind`].

pub mod circuit_model;
