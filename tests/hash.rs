//! Integration tests for Sub-phase 1.J — canonical hashing.
//!
//! Validates the Phase 1 acceptance criterion "mismo `.aoncir` ⇒ mismo
//! hash canónico" against real fixtures, plus invariance to the optional
//! opaque sections and to a write round-trip.

use std::path::Path;

use aonix::circuit_model::Circuit;
use aonix::format::aoncir::{self, hash_canonical, write};

fn load_aoncir(file_name: &str) -> Circuit {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join(file_name);
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {path:?}: {error}"));
    aoncir::parse(&raw).unwrap_or_else(|error| panic!("parse failed for {file_name}: {error}"))
}

#[test]
fn hash_is_blake3_prefixed() {
    let hash = hash_canonical(&load_aoncir("one_bit_full_adder.aoncir"));
    let (prefix, body) = hash.split_once(':').expect("prefix present");
    assert_eq!(prefix, "blake3");
    assert_eq!(body.len(), 64);
    assert!(body.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn same_fixture_hashes_stably() {
    let first = hash_canonical(&load_aoncir("one_bit_full_adder.aoncir"));
    let second = hash_canonical(&load_aoncir("one_bit_full_adder.aoncir"));
    assert_eq!(first, second);
}

#[test]
fn opaque_sections_do_not_affect_the_hash() {
    // inverter.aoncir and inverter_with_opaque_sections.aoncir describe the
    // same graph; the auxiliary [verification]/[metrics]/[layout]/[history]
    // sections are not part of the canonical truth.
    let plain = hash_canonical(&load_aoncir("inverter.aoncir"));
    let with_opaque = hash_canonical(&load_aoncir("inverter_with_opaque_sections.aoncir"));
    assert_eq!(plain, with_opaque);
}

#[test]
fn hash_survives_canonical_write_round_trip() {
    for fixture in [
        "inverter.aoncir",
        "two_input_and.aoncir",
        "multiplexer_2_to_1.aoncir",
        "one_bit_full_adder.aoncir",
        "bus_passthrough_two_bit.aoncir",
    ] {
        let circuit = load_aoncir(fixture);
        let reparsed = aoncir::parse(&write(&circuit))
            .unwrap_or_else(|error| panic!("re-parse failed for {fixture}: {error}"));
        assert_eq!(
            hash_canonical(&circuit),
            hash_canonical(&reparsed),
            "hash changed across write round-trip for {fixture}"
        );
    }
}

#[test]
fn distinct_circuits_have_distinct_hashes() {
    let inverter = hash_canonical(&load_aoncir("inverter.aoncir"));
    let pass_through = hash_canonical(&load_aoncir("pass_through.aoncir"));
    let two_input_and = hash_canonical(&load_aoncir("two_input_and.aoncir"));
    assert_ne!(inverter, pass_through);
    assert_ne!(inverter, two_input_and);
    assert_ne!(pass_through, two_input_and);
}
