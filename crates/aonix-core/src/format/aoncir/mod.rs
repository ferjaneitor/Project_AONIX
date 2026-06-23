//! `.aoncir` format — canonical circuit representation of AONIX.
//!
//! Phase 1.B exposes the **parser** (TOML → [`Circuit`]) and the
//! corresponding **schema** types. Writing back to TOML and computing the
//! canonical hash are scheduled for sub-phases 1.D and 1.H respectively.
//!
//! See `docs/21-aoncir-syntax.md` for the normative specification.
//!
//! [`Circuit`]: crate::circuit_model::Circuit

pub mod schema;
pub mod validate;
pub mod parse;
pub mod write;
pub mod hash;

pub use parse::{parse, SUPPORTED_FORMAT_VERSION};
pub use write::write;
pub use hash::hash_canonical;
