//! Functional specifications a circuit can be verified against.
//!
//! Phase 2 supports the two strongest self-contained forms of
//! `docs/12-task-specification.md` §`Specification`:
//!
//! - [`TruthTable`] — an explicit (possibly exhaustive) mapping from input
//!   bit-vector to expected output bit-vector.
//! - [`ReferenceFunction`] — a pure Rust function used as the ground truth;
//!   the verifier enumerates every input and compares.
//!
//! `PropertyList`, `ReferenceCircuit` and `TemporalSpec` are later phases.

use std::collections::BTreeMap;

use thiserror::Error;

/// Error building a [`TruthTable`] (inconsistent arity).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SpecError {
    /// A row's input width does not match the table's declared input arity.
    #[error("truth-table row input width {given} does not match declared input arity {expected}")]
    RowInputArity { expected: usize, given: usize },
    /// A row's output width does not match the table's declared output arity.
    #[error("truth-table row output width {given} does not match declared output arity {expected}")]
    RowOutputArity { expected: usize, given: usize },
    /// The same input appears twice with (possibly) different outputs.
    #[error("truth-table has a duplicate row for input {input:?}")]
    DuplicateRow { input: Vec<bool> },
}

/// Functional specification. The verifier picks the strongest available; in
/// Phase 2 each task carries exactly one of these.
pub enum Specification {
    /// An explicit truth table.
    TruthTable(TruthTable),
    /// A pure reference function evaluated exhaustively.
    ReferenceFunction(ReferenceFunction),
}

impl Specification {
    /// Number of input bits the specification expects.
    pub fn input_arity(&self) -> usize {
        match self {
            Specification::TruthTable(table) => table.input_arity(),
            Specification::ReferenceFunction(function) => function.input_arity(),
        }
    }

    /// Number of output bits the specification produces.
    pub fn output_arity(&self) -> usize {
        match self {
            Specification::TruthTable(table) => table.output_arity(),
            Specification::ReferenceFunction(function) => function.output_arity(),
        }
    }

    /// The expected output for `input`, if the specification covers it. A
    /// [`ReferenceFunction`] is total (always `Some`); a [`TruthTable`]
    /// returns `Some` only for declared rows.
    pub fn expected_output(&self, input: &[bool]) -> Option<Vec<bool>> {
        match self {
            Specification::TruthTable(table) => table.expected(input).cloned(),
            Specification::ReferenceFunction(function) => Some(function.evaluate(input)),
        }
    }
}

/// Explicit truth table: input bit-vector → expected output bit-vector.
///
/// Inputs are stored in a `BTreeMap` so iteration is deterministic. A table
/// may be *complete* (all 2^input_arity rows present) or *partial*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruthTable {
    input_arity: usize,
    output_arity: usize,
    rows: BTreeMap<Vec<bool>, Vec<bool>>,
}

impl TruthTable {
    /// Creates an empty truth table with the given input/output arities.
    pub fn new(input_arity: usize, output_arity: usize) -> Self {
        Self {
            input_arity,
            output_arity,
            rows: BTreeMap::new(),
        }
    }

    /// Adds one row, validating its widths. Builder-style.
    pub fn with_row(mut self, input: Vec<bool>, output: Vec<bool>) -> Result<Self, SpecError> {
        self.insert_row(input, output)?;
        Ok(self)
    }

    /// Builds a table from an iterator of rows.
    pub fn from_rows(
        input_arity: usize,
        output_arity: usize,
        rows: impl IntoIterator<Item = (Vec<bool>, Vec<bool>)>,
    ) -> Result<Self, SpecError> {
        let mut table = Self::new(input_arity, output_arity);
        for (input, output) in rows {
            table.insert_row(input, output)?;
        }
        Ok(table)
    }

    fn insert_row(&mut self, input: Vec<bool>, output: Vec<bool>) -> Result<(), SpecError> {
        if input.len() != self.input_arity {
            return Err(SpecError::RowInputArity {
                expected: self.input_arity,
                given: input.len(),
            });
        }
        if output.len() != self.output_arity {
            return Err(SpecError::RowOutputArity {
                expected: self.output_arity,
                given: output.len(),
            });
        }
        if self.rows.contains_key(&input) {
            return Err(SpecError::DuplicateRow { input });
        }
        self.rows.insert(input, output);
        Ok(())
    }

    /// Number of input bits.
    pub fn input_arity(&self) -> usize {
        self.input_arity
    }

    /// Number of output bits.
    pub fn output_arity(&self) -> usize {
        self.output_arity
    }

    /// The expected output for `input`, if the table declares that row.
    pub fn expected(&self, input: &[bool]) -> Option<&Vec<bool>> {
        self.rows.get(input)
    }

    /// Deterministic iterator over `(input, expected_output)` rows.
    pub fn rows(&self) -> impl Iterator<Item = (&Vec<bool>, &Vec<bool>)> {
        self.rows.iter()
    }

    /// Number of declared rows.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Whether the table has no rows.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Whether the table covers all 2^input_arity input combinations.
    pub fn is_complete(&self) -> bool {
        self.rows.len() == 1usize << self.input_arity
    }
}

/// A pure reference function: maps an input bit-vector to the expected
/// output bit-vector. Used as ground truth for exhaustive verification.
pub struct ReferenceFunction {
    input_arity: usize,
    output_arity: usize,
    function: fn(&[bool]) -> Vec<bool>,
}

impl ReferenceFunction {
    /// Builds a reference function of the given arities.
    pub fn new(input_arity: usize, output_arity: usize, function: fn(&[bool]) -> Vec<bool>) -> Self {
        Self {
            input_arity,
            output_arity,
            function,
        }
    }

    /// Number of input bits.
    pub fn input_arity(&self) -> usize {
        self.input_arity
    }

    /// Number of output bits.
    pub fn output_arity(&self) -> usize {
        self.output_arity
    }

    /// Evaluates the reference function on `input`.
    pub fn evaluate(&self, input: &[bool]) -> Vec<bool> {
        (self.function)(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truth_table_rejects_wrong_input_width() {
        let result = TruthTable::new(2, 1).with_row(vec![true], vec![true]);
        assert!(matches!(result, Err(SpecError::RowInputArity { expected: 2, given: 1 })));
    }

    #[test]
    fn truth_table_rejects_wrong_output_width() {
        let result = TruthTable::new(1, 2).with_row(vec![true], vec![true]);
        assert!(matches!(result, Err(SpecError::RowOutputArity { expected: 2, given: 1 })));
    }

    #[test]
    fn truth_table_rejects_duplicate_row() {
        let result = TruthTable::from_rows(
            1,
            1,
            [(vec![true], vec![true]), (vec![true], vec![false])],
        );
        assert!(matches!(result, Err(SpecError::DuplicateRow { .. })));
    }

    #[test]
    fn truth_table_completeness() {
        let partial = TruthTable::new(2, 1).with_row(vec![false, false], vec![false]).unwrap();
        assert!(!partial.is_complete());
        let complete = TruthTable::from_rows(
            1,
            1,
            [(vec![false], vec![true]), (vec![true], vec![false])],
        )
        .unwrap();
        assert!(complete.is_complete());
    }

    #[test]
    fn reference_function_evaluates() {
        let and = ReferenceFunction::new(2, 1, |input| vec![input[0] && input[1]]);
        assert_eq!(and.evaluate(&[true, true]), vec![true]);
        assert_eq!(and.evaluate(&[true, false]), vec![false]);
    }
}
