//! The optimizer driver: fixpoint loop + double guarantee.

use aonix_core::circuit_model::Circuit;
use aonix_core::format::aoncir::hash_canonical;
use aonix_eval::{evaluate, is_strictly_better, Criterion, Metrics, DEFAULT_RANKING};

use crate::equivalence::behaviourally_equivalent;
use crate::error::OptError;
use crate::transform::{mvp_transforms, Transform, TransformId};

/// Hard cap on optimization rounds (a safety net; the loop normally reaches a
/// fixpoint far sooner, since every accepted step strictly reduces metrics).
const MAX_ITERATIONS: usize = 1000;

/// What happened to one attempted transformation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepOutcome {
    /// Equivalent and strictly better: kept.
    Accepted,
    /// Equivalent but not strictly better: discarded.
    RejectedNoImprovement,
    /// Not behaviourally equivalent (a transformation bug): discarded, with a
    /// counterexample input if one was found.
    RejectedRegression { counterexample: Option<Vec<bool>> },
}

/// One entry of the (append-only) optimization log — the "optimization
/// memory" of `docs/15`.
#[derive(Debug, Clone)]
pub struct OptStep {
    pub transform: TransformId,
    pub hash_before: String,
    pub hash_after: String,
    pub metrics_before: Metrics,
    pub metrics_after: Metrics,
    pub outcome: StepOutcome,
}

/// The result of optimizing a circuit.
#[derive(Debug, Clone)]
pub struct OptReport {
    /// The optimized circuit (or the original intact if nothing improved).
    pub circuit: Circuit,
    /// Whether the result is strictly better than the input.
    pub improved: bool,
    /// Rounds executed before reaching the fixpoint.
    pub iterations: usize,
    pub metrics_initial: Metrics,
    pub metrics_final: Metrics,
    /// Every transformation attempt, in order.
    pub log: Vec<OptStep>,
}

/// Optimizes `original` with the MVP catalog and default ranking.
///
/// Precondition (`docs/15` pipeline step 1): `original` is already a valid,
/// verified circuit. The result is guaranteed behaviourally equivalent to
/// `original`.
pub fn optimize(original: &Circuit) -> Result<OptReport, OptError> {
    optimize_with(original, &mvp_transforms(), DEFAULT_RANKING)
}

/// Optimizes `original` with an explicit transformation set and ranking.
///
/// Each candidate is accepted only if it is **behaviourally equivalent to the
/// original** (the verification guarantee) **and** strictly better under
/// `ranking` (the evaluator's judgment). Non-equivalent candidates — i.e.
/// transformation bugs — are discarded (backtracking). Iterates to a fixpoint.
pub fn optimize_with(
    original: &Circuit,
    transforms: &[Box<dyn Transform>],
    ranking: &[Criterion],
) -> Result<OptReport, OptError> {
    let metrics_initial = evaluate(original);
    let mut current = original.clone();
    let mut current_metrics = metrics_initial;
    let mut log = Vec::new();
    let mut iterations = 0;

    while iterations < MAX_ITERATIONS {
        let mut changed = false;
        for transform in transforms {
            let Some(candidate) = transform.apply_once(&current)? else {
                continue;
            };
            let metrics_before = current_metrics;
            let metrics_after = evaluate(&candidate);
            let hash_before = hash_canonical(&current);
            let hash_after = hash_canonical(&candidate);

            // Verification guarantee: differential equivalence to the ORIGINAL.
            let (equivalent, counterexample) = behaviourally_equivalent(original, &candidate)?;
            let outcome = if !equivalent {
                StepOutcome::RejectedRegression { counterexample }
            } else if is_strictly_better(&metrics_after, &metrics_before, ranking) {
                current = candidate;
                current_metrics = metrics_after;
                changed = true;
                StepOutcome::Accepted
            } else {
                StepOutcome::RejectedNoImprovement
            };

            log.push(OptStep {
                transform: transform.id(),
                hash_before,
                hash_after,
                metrics_before,
                metrics_after,
                outcome,
            });
        }
        iterations += 1;
        if !changed {
            break;
        }
    }

    let improved = is_strictly_better(&current_metrics, &metrics_initial, ranking);
    Ok(OptReport {
        circuit: current,
        improved,
        iterations,
        metrics_initial,
        metrics_final: current_metrics,
        log,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use aonix_core::circuit_model::{
        CircuitBuilder, Gate, GateIdentifier, GateKind, Port, PortIdentifier, PortRole, Signal,
        SignalIdentifier, SignalReference,
    };
    use crate::transform::Transform;

    fn input_port(name: &str) -> Port {
        Port::new(PortIdentifier::new(name).unwrap(), PortRole::Input, None, None, None)
    }
    fn output_port(name: &str) -> Port {
        Port::new(PortIdentifier::new(name).unwrap(), PortRole::Output, None, None, None)
    }
    fn signal(name: &str) -> Signal {
        Signal::new(SignalIdentifier::new(name).unwrap(), None, None)
    }
    fn port_ref(name: &str) -> SignalReference {
        SignalReference::Port(PortIdentifier::new(name).unwrap())
    }
    fn signal_ref(name: &str) -> SignalReference {
        SignalReference::InternalSignal(SignalIdentifier::new(name).unwrap())
    }
    fn gate(id: &str, kind: GateKind, inputs: Vec<SignalReference>, output: &str) -> Gate {
        Gate::new(GateIdentifier::new(id).unwrap(), kind, inputs, SignalIdentifier::new(output).unwrap()).unwrap()
    }

    /// out = NOT(NOT(a)); should optimize to out = a (0 gates).
    fn double_negation_circuit() -> Circuit {
        let mut b = CircuitBuilder::new();
        b.add_input_port(input_port("a")).unwrap();
        b.add_output_port(output_port("out")).unwrap();
        b.add_signal(signal("not_a")).unwrap();
        b.add_signal(signal("not_not_a")).unwrap();
        b.add_gate(gate("g1", GateKind::Not, vec![port_ref("a")], "not_a")).unwrap();
        b.add_gate(gate("g2", GateKind::Not, vec![signal_ref("not_a")], "not_not_a")).unwrap();
        b.assign_output(PortIdentifier::new("out").unwrap(), signal_ref("not_not_a")).unwrap();
        b.finish().unwrap()
    }

    /// out = AND(a, a); should optimize to out = a (0 gates).
    fn idempotent_circuit() -> Circuit {
        let mut b = CircuitBuilder::new();
        b.add_input_port(input_port("a")).unwrap();
        b.add_output_port(output_port("out")).unwrap();
        b.add_signal(signal("a_and_a")).unwrap();
        b.add_gate(gate("g_and", GateKind::And, vec![port_ref("a"), port_ref("a")], "a_and_a")).unwrap();
        b.assign_output(PortIdentifier::new("out").unwrap(), signal_ref("a_and_a")).unwrap();
        b.finish().unwrap()
    }

    #[test]
    fn double_negation_is_eliminated_and_equivalent() {
        let circuit = double_negation_circuit();
        let report = optimize(&circuit).unwrap();
        assert!(report.improved);
        assert_eq!(report.metrics_final.gate_count_total, 0);
        // out is now driven directly by input a.
        assert!(report.log.iter().any(|s| s.outcome == StepOutcome::Accepted));
    }

    #[test]
    fn idempotent_gate_is_eliminated() {
        let report = optimize(&idempotent_circuit()).unwrap();
        assert!(report.improved);
        assert_eq!(report.metrics_final.gate_count_total, 0);
    }

    #[test]
    fn optimize_is_idempotent() {
        let once = optimize(&double_negation_circuit()).unwrap();
        let twice = optimize(&once.circuit).unwrap();
        assert_eq!(hash_canonical(&once.circuit), hash_canonical(&twice.circuit));
        assert!(!twice.improved); // already at fixpoint
    }

    /// A deliberately buggy transformation that returns a behaviourally
    /// different circuit must be rejected, leaving the original untouched.
    struct BuggyTransform;
    impl Transform for BuggyTransform {
        fn id(&self) -> TransformId {
            TransformId::Idempotence
        }
        fn apply_once(&self, _circuit: &Circuit) -> Result<Option<Circuit>, OptError> {
            // Same interface (1 in, 1 out) but the opposite behaviour: out = NOT a.
            let mut b = CircuitBuilder::new();
            b.add_input_port(input_port("a")).unwrap();
            b.add_output_port(output_port("out")).unwrap();
            b.add_signal(signal("not_a")).unwrap();
            b.add_gate(gate("g_not", GateKind::Not, vec![port_ref("a")], "not_a")).unwrap();
            b.assign_output(PortIdentifier::new("out").unwrap(), signal_ref("not_a")).unwrap();
            Ok(Some(b.finish().unwrap()))
        }
    }

    #[test]
    fn buggy_non_equivalent_transform_is_rejected() {
        // Original: out = AND(a, a) (behaves as identity on a).
        let original = idempotent_circuit();
        let original_hash = hash_canonical(&original);
        let transforms: Vec<Box<dyn Transform>> = vec![Box::new(BuggyTransform)];
        let report = optimize_with(&original, &transforms, DEFAULT_RANKING).unwrap();
        // The buggy rewrite (out = NOT a) is not equivalent, so it is rejected
        // and the result is the original, unchanged.
        assert!(!report.improved);
        assert_eq!(hash_canonical(&report.circuit), original_hash);
        assert!(report
            .log
            .iter()
            .any(|step| matches!(step.outcome, StepOutcome::RejectedRegression { .. })));
    }
}
