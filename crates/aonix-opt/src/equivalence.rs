//! Differential behavioural equivalence — the verification guarantee.
//!
//! Implements the `docs/15` invariant directly: two circuits are equivalent
//! iff they produce the same output on every input. Exhaustive when the input
//! space is feasible; otherwise a deterministic seeded sample (a transformed
//! circuit is never accepted on the strength of an unchecked equivalence).

use aonix_core::circuit_model::{Bit, Circuit, InputVector};
use aonix_sim::simulation::simulate;

use crate::error::OptError;

/// Exhaustive comparison is used up to this input width.
const MAX_EXHAUSTIVE_INPUT_BITS: usize = 16;
/// Random samples used above the exhaustive threshold.
const RANDOM_SAMPLES: u64 = 4096;

/// Returns `(equivalent, counterexample)`: whether `a` and `b` are
/// behaviourally equivalent and, if not, one input on which they differ.
pub(crate) fn behaviourally_equivalent(
    a: &Circuit,
    b: &Circuit,
) -> Result<(bool, Option<Vec<bool>>), OptError> {
    if a.input_count() != b.input_count() || a.output_count() != b.output_count() {
        return Ok((false, None));
    }
    let bits = a.input_count();

    if bits <= MAX_EXHAUSTIVE_INPUT_BITS {
        let total: u64 = 1u64 << bits;
        for combination in 0..total {
            let input: Vec<bool> = (0..bits).map(|index| (combination >> index) & 1 == 1).collect();
            if !agree(a, b, &input)? {
                return Ok((false, Some(input)));
            }
        }
        return Ok((true, None));
    }

    // Deterministic seeded fallback for very wide circuits.
    let mut state: u64 = 0x1234_5678_9ABC_DEF0;
    for _ in 0..RANDOM_SAMPLES {
        let mut input = Vec::with_capacity(bits);
        let mut produced = 0;
        while produced < bits {
            state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            let mut word = z ^ (z >> 31);
            for _ in 0..64 {
                if produced == bits {
                    break;
                }
                input.push(word & 1 == 1);
                word >>= 1;
                produced += 1;
            }
        }
        if !agree(a, b, &input)? {
            return Ok((false, Some(input)));
        }
    }
    Ok((true, None))
}

fn agree(a: &Circuit, b: &Circuit, input: &[bool]) -> Result<bool, OptError> {
    let vector = InputVector::new(input.iter().map(|&bit| Bit(bit)).collect());
    let output_a = simulate(a, &vector)?;
    let output_b = simulate(b, &vector)?;
    Ok(output_a.as_slice() == output_b.as_slice())
}
