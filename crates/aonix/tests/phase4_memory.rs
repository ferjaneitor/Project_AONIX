//! Phase 4 integration: canonical/historical memory with atomic promotion.
//!
//! Acceptance criteria (`docs/11-roadmap.md` Phase 4): promotion is
//! transactional; a strictly-better version replaces the active one and the
//! previous one is archived; the archived version is recoverable without loss.

use std::path::Path;

use aonix::format::aoncir::{hash_canonical, parse};
use aonix::memory::{CircuitKey, MemoryStore, PromotionOutcome};

fn load_text(file_name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join(file_name);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"))
}

fn hex_of(hash: &str) -> String {
    hash.split_once(':').map(|(_, hex)| hex.to_string()).unwrap_or_else(|| hash.to_string())
}

#[test]
fn full_promotion_lifecycle() {
    let canonical = parse(&load_text("one_bit_full_adder.aoncir")).expect("canonical parses");
    let redundant = parse(&load_text("one_bit_full_adder_redundant.aoncir")).expect("redundant parses");
    let canonical_hash = hash_canonical(&canonical);
    let redundant_hash = hash_canonical(&redundant);

    let root = std::env::temp_dir().join("aonix_phase4_memory_integration");
    let _ = std::fs::remove_dir_all(&root);
    let store = MemoryStore::open(&root);
    let key = CircuitKey::new("one_bit_full_adder", "w1").expect("valid key");

    // 1. First promotion installs the (worse) redundant version as active.
    assert!(matches!(
        store.promote(&key, &redundant).unwrap(),
        PromotionOutcome::InstalledFirst { .. }
    ));

    // 2. The leaner canonical version is strictly better: it replaces the
    //    incumbent, which is archived to history.
    match store.promote(&key, &canonical).unwrap() {
        PromotionOutcome::Replaced { new_hash, archived_hash } => {
            assert_eq!(new_hash, canonical_hash);
            assert_eq!(archived_hash, redundant_hash);
        }
        other => panic!("expected Replaced, got {other:?}"),
    }
    assert_eq!(store.active_hash(&key).unwrap().unwrap(), canonical_hash);

    // 3. Re-promoting the same circuit is a no-op (dedup by canonical hash).
    assert!(matches!(
        store.promote(&key, &canonical).unwrap(),
        PromotionOutcome::AlreadyActive { .. }
    ));

    // 4. The redundant version is now worse than the active one: rejected to
    //    experimental, active unchanged.
    assert!(matches!(
        store.promote(&key, &redundant).unwrap(),
        PromotionOutcome::RejectedNotBetter { .. }
    ));
    assert_eq!(store.active_hash(&key).unwrap().unwrap(), canonical_hash);

    // 5. History holds exactly the archived redundant version, recoverable
    //    without loss.
    let redundant_hex = hex_of(&redundant_hash);
    assert_eq!(store.history_hexes(&key).unwrap(), vec![redundant_hex.clone()]);
    let recovered = store.historical(&key, &redundant_hex).unwrap().expect("archived version present");
    assert_eq!(hash_canonical(&recovered), redundant_hash);

    // 6. The store lists the key.
    let keys = store.list().unwrap();
    assert!(keys.iter().any(|k| k.name() == "one_bit_full_adder" && k.parameters() == "w1"));

    let _ = std::fs::remove_dir_all(&root);
}
