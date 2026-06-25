//! Deterministic ranking of circuits by their structural [`Metrics`].
//!
//! The comparator never decides correctness (that is the verifier's job); it
//! only orders two *already-correct* circuits so the coordinator can choose
//! which to keep as the official-active version (`docs/13` §28, `docs/19`).

use std::cmp::Ordering;

use crate::metrics::Metrics;

/// A single ranking dimension. "Better" always maps to [`Ordering::Less`] so
/// that sorting ascending puts the better circuit first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Criterion {
    /// Fewer gates is better.
    GateCount,
    /// Shallower logical depth is better.
    Depth,
    /// Fewer dead signals is better.
    DeadSignals,
    /// Lower maximum fan-out is better (controls hot-spot signals,
    /// `docs/19` default `ranking_order`).
    FanOutMax,
    /// More subexpression sharing (reuse) is better.
    Reuse,
    /// Lower weighted aggregate cost is better.
    AggregateCost,
}

/// The default lexicographic ranking of `docs/13` §28: gate count, then
/// depth, then dead signals, then reuse.
pub const DEFAULT_RANKING: &[Criterion] = &[
    Criterion::GateCount,
    Criterion::Depth,
    Criterion::DeadSignals,
    Criterion::Reuse,
];

/// Lexicographically compares `a` and `b` along `order`. Returns
/// [`Ordering::Less`] when `a` is the better circuit. A pure function of the
/// metrics, hence stable and reproducible.
pub fn compare(a: &Metrics, b: &Metrics, order: &[Criterion]) -> Ordering {
    for criterion in order {
        let ordering = match criterion {
            Criterion::GateCount => a.gate_count_total.cmp(&b.gate_count_total),
            Criterion::Depth => a.depth.cmp(&b.depth),
            Criterion::DeadSignals => a.dead_signals.cmp(&b.dead_signals),
            Criterion::FanOutMax => a.max_fan_out.cmp(&b.max_fan_out),
            // Higher reuse is better, so reverse the comparison.
            Criterion::Reuse => b.shared_signal_count.cmp(&a.shared_signal_count),
            Criterion::AggregateCost => a.aggregate_cost.cmp(&b.aggregate_cost),
        };
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    Ordering::Equal
}

/// Compares with the [`DEFAULT_RANKING`].
pub fn default_compare(a: &Metrics, b: &Metrics) -> Ordering {
    compare(a, b, DEFAULT_RANKING)
}

/// Whether `candidate` is a **strict** improvement over `incumbent` under
/// `order`. A tie favours the incumbent (`docs/19` §"mejora estricta"), so an
/// equal ranking returns `false`.
pub fn is_strictly_better(candidate: &Metrics, incumbent: &Metrics, order: &[Criterion]) -> bool {
    compare(candidate, incumbent, order) == Ordering::Less
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics(gates: usize, depth: usize, dead: usize, reuse: usize) -> Metrics {
        Metrics {
            gate_count_total: gates,
            and_count: 0,
            or_count: 0,
            not_count: 0,
            signal_count: 0,
            depth,
            dead_signals: dead,
            max_fan_in: 0,
            max_fan_out: 0,
            shared_signal_count: reuse,
            aggregate_cost: 0,
        }
    }

    #[test]
    fn fewer_gates_ranks_first() {
        let lean = metrics(5, 3, 0, 2);
        let bulky = metrics(7, 3, 0, 2);
        assert_eq!(default_compare(&lean, &bulky), Ordering::Less);
        assert!(is_strictly_better(&lean, &bulky, DEFAULT_RANKING));
        assert!(!is_strictly_better(&bulky, &lean, DEFAULT_RANKING));
    }

    #[test]
    fn depth_breaks_gate_count_ties() {
        let shallow = metrics(5, 2, 0, 2);
        let deep = metrics(5, 4, 0, 2);
        assert_eq!(default_compare(&shallow, &deep), Ordering::Less);
    }

    #[test]
    fn more_reuse_wins_when_else_equal() {
        let more_reuse = metrics(5, 3, 0, 4);
        let less_reuse = metrics(5, 3, 0, 1);
        assert_eq!(default_compare(&more_reuse, &less_reuse), Ordering::Less);
    }

    #[test]
    fn fan_out_max_lower_is_better() {
        let mut low = metrics(5, 3, 0, 2);
        low.max_fan_out = 2;
        let mut high = metrics(5, 3, 0, 2);
        high.max_fan_out = 5;
        assert_eq!(compare(&low, &high, &[Criterion::FanOutMax]), Ordering::Less);
        assert_eq!(compare(&high, &low, &[Criterion::FanOutMax]), Ordering::Greater);
    }

    #[test]
    fn identical_metrics_are_a_tie_and_not_strictly_better() {
        let a = metrics(5, 3, 0, 2);
        let b = metrics(5, 3, 0, 2);
        assert_eq!(default_compare(&a, &b), Ordering::Equal);
        assert!(!is_strictly_better(&a, &b, DEFAULT_RANKING));
    }

    #[test]
    fn dead_signals_penalised_before_reuse() {
        let clean = metrics(5, 3, 0, 1);
        let with_dead = metrics(5, 3, 2, 9);
        // Even with much more reuse, dead signals rank it worse.
        assert_eq!(default_compare(&clean, &with_dead), Ordering::Less);
    }
}
