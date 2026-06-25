//! Input-vector generators for the test suites.

use crate::prng::SplitMix64;

/// Above this input width, exhaustive enumeration is refused (2ⁿ too large);
/// suites fall back to random + edge + regression. Mirrors the verifier and
/// `docs/07` (≤ ~2²⁰ feasible).
pub const EXHAUSTIVE_LIMIT: usize = 20;

/// Every one of the 2ⁿ input combinations, ascending, LSB-first (index 0 is
/// the least-significant bit).
pub fn exhaustive(input_arity: usize) -> Vec<Vec<bool>> {
    let total: u64 = 1u64 << input_arity;
    (0..total)
        .map(|combination| {
            (0..input_arity)
                .map(|index| (combination >> index) & 1 == 1)
                .collect()
        })
        .collect()
}

/// `count` reproducible random input vectors from `seed`.
pub fn random(input_arity: usize, count: usize, seed: u64) -> Vec<Vec<bool>> {
    let mut rng = SplitMix64::new(seed);
    (0..count)
        .map(|_| (0..input_arity).map(|_| rng.next_bool()).collect())
        .collect()
}

/// The catalogued edge cases of `docs/07`: all-zero, all-one, each
/// single-bit-on, each single-bit-off, and the two alternating patterns.
pub fn edge_cases(input_arity: usize) -> Vec<Vec<bool>> {
    if input_arity == 0 {
        return vec![vec![]];
    }
    let mut cases = Vec::new();
    cases.push(vec![false; input_arity]); // all zero
    cases.push(vec![true; input_arity]); // all one
    for index in 0..input_arity {
        let mut single_on = vec![false; input_arity];
        single_on[index] = true;
        cases.push(single_on);
    }
    for index in 0..input_arity {
        let mut single_off = vec![true; input_arity];
        single_off[index] = false;
        cases.push(single_off);
    }
    cases.push((0..input_arity).map(|index| index % 2 == 0).collect()); // 1010…
    cases.push((0..input_arity).map(|index| index % 2 == 1).collect()); // 0101…
    cases
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exhaustive_has_2_to_the_n_rows() {
        assert_eq!(exhaustive(0).len(), 1);
        assert_eq!(exhaustive(3).len(), 8);
        assert_eq!(exhaustive(4).len(), 16);
    }

    #[test]
    fn random_is_reproducible_with_same_seed() {
        assert_eq!(random(4, 50, 7), random(4, 50, 7));
        assert_ne!(random(4, 50, 7), random(4, 50, 8));
    }

    #[test]
    fn edge_cases_include_all_zero_and_all_one() {
        let cases = edge_cases(3);
        assert!(cases.contains(&vec![false, false, false]));
        assert!(cases.contains(&vec![true, true, true]));
        assert!(cases.contains(&vec![true, false, false])); // single-bit-on
        assert!(cases.contains(&vec![false, true, true])); // single-bit-off
    }
}
