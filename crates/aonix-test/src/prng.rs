//! Deterministic, dependency-free PRNG for reproducible random testing.
//!
//! SplitMix64 (Steele, Lea & Flood). Same seed ⇒ same sequence, on every
//! machine — the reproducibility guarantee of `docs/07` §"pruebas aleatorias
//! usan semilla explícita".

/// A SplitMix64 generator. Cheap, deterministic, good distribution for test
/// vector generation (not cryptographic).
#[derive(Debug, Clone)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// Seeds the generator. The same seed always yields the same sequence.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Returns the next pseudo-random `u64`.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Returns the next pseudo-random bit.
    pub fn next_bool(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_yields_same_sequence() {
        let mut a = SplitMix64::new(42);
        let mut b = SplitMix64::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = SplitMix64::new(1);
        let mut b = SplitMix64::new(2);
        let sequence_a: Vec<u64> = (0..16).map(|_| a.next_u64()).collect();
        let sequence_b: Vec<u64> = (0..16).map(|_| b.next_u64()).collect();
        assert_ne!(sequence_a, sequence_b);
    }
}
