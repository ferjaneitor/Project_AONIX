//! Phase 3 integration: the structural evaluator and its deterministic
//! comparator.
//!
//! Acceptance criterion (`docs/11-roadmap.md` Phase 3): given two versions of
//! `one_bit_full_adder`, the evaluator returns a stable, reproducible order.

use std::cmp::Ordering;
use std::path::Path;

use aonix::circuit_model::Circuit;
use aonix::eval::{default_compare, evaluate, is_strictly_better, DEFAULT_RANKING};
use aonix::format::aoncir;

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
fn canonical_full_adder_metrics_are_as_expected() {
    let metrics = evaluate(&load_aoncir("one_bit_full_adder.aoncir"));
    assert_eq!(metrics.gate_count_total, 13);
    assert_eq!(metrics.and_count, 6);
    assert_eq!(metrics.or_count, 3);
    assert_eq!(metrics.not_count, 4);
    assert_eq!(metrics.signal_count, 13);
    assert_eq!(metrics.depth, 6, "critical path is 6 gates deep");
    assert_eq!(metrics.dead_signals, 0);
    assert_eq!(metrics.max_fan_in, 2);
}

#[test]
fn redundant_version_has_one_more_gate_and_a_dead_signal() {
    let metrics = evaluate(&load_aoncir("one_bit_full_adder_redundant.aoncir"));
    assert_eq!(metrics.gate_count_total, 14);
    assert_eq!(metrics.dead_signals, 1);
}

#[test]
fn canonical_ranks_strictly_better_than_redundant() {
    let canonical = evaluate(&load_aoncir("one_bit_full_adder.aoncir"));
    let redundant = evaluate(&load_aoncir("one_bit_full_adder_redundant.aoncir"));

    assert_eq!(default_compare(&canonical, &redundant), Ordering::Less);
    assert!(is_strictly_better(&canonical, &redundant, DEFAULT_RANKING));
    assert!(!is_strictly_better(&redundant, &canonical, DEFAULT_RANKING));
    // A version is never strictly better than itself (ties favour incumbent).
    assert!(!is_strictly_better(&canonical, &canonical, DEFAULT_RANKING));
}

#[test]
fn order_is_stable_and_reproducible_across_runs() {
    let first_canonical = evaluate(&load_aoncir("one_bit_full_adder.aoncir"));
    let first_redundant = evaluate(&load_aoncir("one_bit_full_adder_redundant.aoncir"));
    let second_canonical = evaluate(&load_aoncir("one_bit_full_adder.aoncir"));
    let second_redundant = evaluate(&load_aoncir("one_bit_full_adder_redundant.aoncir"));

    // Same circuit ⇒ identical metrics.
    assert_eq!(first_canonical, second_canonical);
    assert_eq!(first_redundant, second_redundant);
    // Same pair ⇒ identical ordering, every run.
    assert_eq!(
        default_compare(&first_canonical, &first_redundant),
        default_compare(&second_canonical, &second_redundant),
    );
}
