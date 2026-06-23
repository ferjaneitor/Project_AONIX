//! File-format input/output for AONIX.
//!
//! Phase 1 only supports reading the canonical [`aoncir`] format (TOML
//! 1.0.0). The writer, the `.aonclg` format, and other formats are
//! scheduled for later sub-phases of the roadmap.

pub mod aoncir;
