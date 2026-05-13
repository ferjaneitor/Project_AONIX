//! Logical values of AONIX: [`Bit`], [`InputVector`], [`OutputVector`].
//!
//! AONIX is deterministic and bit-level: every logical value is either
//! `false` or `true`, with no floating point, no tri-state, no implicit
//! "undefined" value. The two vector types are thin newtypes around an
//! ordered list of bits whose order is the formal contract of the
//! `.aoncir` (the order of appearance of `[[ports.inputs]]` and
//! `[[ports.outputs]]`).

/// Logical bit. Only two states: zero and one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bit(pub bool);

impl Bit {
    /// The logical zero (`false`).
    pub const ZERO: Bit = Bit(false);
    /// The logical one (`true`).
    pub const ONE: Bit = Bit(true);

    /// Returns `true` if this bit is the logical one.
    pub fn is_one(self) -> bool {
        self.0
    }

    /// Returns `true` if this bit is the logical zero.
    pub fn is_zero(self) -> bool {
        !self.0
    }
}

impl From<bool> for Bit {
    fn from(value: bool) -> Self {
        Bit(value)
    }
}

impl From<Bit> for bool {
    fn from(value: Bit) -> Self {
        value.0
    }
}

/// Input vector of a circuit.
///
/// The order of bits follows exactly the order of appearance of
/// `[[ports.inputs]]` in the `.aoncir` file. This order is the formal
/// contract of the circuit; reordering it changes the canonical hash.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputVector {
    bits: Vec<Bit>,
}

impl InputVector {
    /// Constructs an input vector from the given ordered list of bits.
    pub fn new(bits: Vec<Bit>) -> Self {
        Self { bits }
    }

    /// Length in bits.
    pub fn len(&self) -> usize {
        self.bits.len()
    }

    /// Whether the vector has zero bits.
    pub fn is_empty(&self) -> bool {
        self.bits.is_empty()
    }

    /// Returns the bit at the given index, or `None` if out of range.
    pub fn get(&self, index: usize) -> Option<Bit> {
        self.bits.get(index).copied()
    }

    /// Returns a borrow of the underlying ordered slice.
    pub fn as_slice(&self) -> &[Bit] {
        &self.bits
    }
}

/// Output vector of a circuit.
///
/// The order of bits follows exactly the order of appearance of
/// `[[ports.outputs]]` in the `.aoncir` file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OutputVector {
    bits: Vec<Bit>,
}

impl OutputVector {
    /// Constructs an output vector from the given ordered list of bits.
    pub fn new(bits: Vec<Bit>) -> Self {
        Self { bits }
    }

    /// Length in bits.
    pub fn len(&self) -> usize {
        self.bits.len()
    }

    /// Whether the vector has zero bits.
    pub fn is_empty(&self) -> bool {
        self.bits.is_empty()
    }

    /// Returns the bit at the given index, or `None` if out of range.
    pub fn get(&self, index: usize) -> Option<Bit> {
        self.bits.get(index).copied()
    }

    /// Returns a borrow of the underlying ordered slice.
    pub fn as_slice(&self) -> &[Bit] {
        &self.bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_zero_and_one_constants_match_underlying_bool() {
        assert_eq!(Bit::ZERO, Bit(false));
        assert_eq!(Bit::ONE, Bit(true));
        assert!(Bit::ONE.is_one());
        assert!(Bit::ZERO.is_zero());
        assert!(!Bit::ZERO.is_one());
        assert!(!Bit::ONE.is_zero());
    }

    #[test]
    fn bit_round_trips_through_bool() {
        let bit_zero: Bit = false.into();
        let bit_one: Bit = true.into();
        assert_eq!(bit_zero, Bit::ZERO);
        assert_eq!(bit_one, Bit::ONE);
        let back_zero: bool = bit_zero.into();
        let back_one: bool = bit_one.into();
        assert!(!back_zero);
        assert!(back_one);
    }

    #[test]
    fn input_vector_length_matches_constructor_input() {
        let vector = InputVector::new(vec![Bit::ZERO, Bit::ONE, Bit::ONE]);
        assert_eq!(vector.len(), 3);
        assert!(!vector.is_empty());
    }

    #[test]
    fn input_vector_empty_when_constructed_empty() {
        let vector = InputVector::new(Vec::new());
        assert_eq!(vector.len(), 0);
        assert!(vector.is_empty());
    }

    #[test]
    fn output_vector_get_returns_bit_at_index_or_none() {
        let vector = OutputVector::new(vec![Bit::ZERO, Bit::ONE]);
        assert_eq!(vector.get(0), Some(Bit::ZERO));
        assert_eq!(vector.get(1), Some(Bit::ONE));
        assert_eq!(vector.get(2), None);
    }

    #[test]
    fn input_vector_as_slice_returns_underlying_order() {
        let bits = vec![Bit::ONE, Bit::ZERO, Bit::ONE, Bit::ZERO];
        let vector = InputVector::new(bits.clone());
        assert_eq!(vector.as_slice(), bits.as_slice());
    }
}
