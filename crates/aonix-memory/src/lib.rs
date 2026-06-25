//! AONIX canonical and historical memory — layer 9 of
//! `docs/02-architecture.md`, governed by `docs/19-versioning-policy.md`.
//!
//! Principle: **one official-active truth per circuit + size, complete
//! history**. For each `(name, parameters)` ([`CircuitKey`]) there is at most
//! one official-active `.aoncir`; superseded versions live in an append-only
//! history; verified-but-not-promoted candidates live in experimental memory.
//!
//! Promotion ([`MemoryStore::promote`]) is **atomic** and only replaces the
//! incumbent on a **strict** structural improvement (ties favour the
//! incumbent — `docs/19` V.4), reusing `aonix_eval::is_strictly_better`.
//! Structural identity is the canonical hash, so the same circuit is never
//! stored twice (`docs/19` §"Hashing y deduplicación").
//!
//! Phase 4 uses an auditable flat-file layout (the `.aoncir` is canonical
//! text); a database backend remains an open decision in `docs/11`.

pub mod store;

pub use store::{CircuitKey, MemoryError, MemoryStore, PromotionOutcome};
