use std::fmt::{self, Debug, Display, Formatter};
use std::mem::replace;

use fixedbitset::FixedBitSet;

use crate::cell::value::SudokuValue;

pub mod value;

// TODO: cell should be an enum
//  a cell can only hold either a value or candidates
//  value() and candidates() should return a option
//  legal range of value is 1..=n
//  rethink non zero * situation

// TODO: set or toggle value

// TODO: is_editable cell

// TODO: are default value parameters wise?
pub trait SudokuCell<Value = usize>: Default + Clone + Display + Debug + Ord + Eq + Send
where
    Value: SudokuValue,
    Value::Error: std::error::Error,
{
    fn new<I>(value: Value, candidates: I, max: Value) -> Self
    where
        I: IntoIterator<Item = Value>;
    fn new_with_value(value: Value, max: Value) -> Self;

    fn value(&self) -> Value;
    fn candidates(&self) -> Vec<Value>;

    fn set_value(&mut self, value: Value, max: Value) -> Value;
    fn set_candidates<I>(&mut self, candidates: I, max: Value)
    where
        I: IntoIterator<Item = Value>;

    fn toggle_candidate(&mut self, candidate: Value, max: Value) -> bool;
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Default, Debug)]
pub struct Cell<Value = usize>
where
    Value: SudokuValue,
    Value::Error: std::error::Error,
{
    value: Value,
    candidates: FixedBitSet,
}

impl<Value> SudokuCell<Value> for Cell<Value>
where
    Value: SudokuValue,
    Value::Error: std::error::Error,
{
    fn new<I>(value: Value, candidates: I, max: Value) -> Self
    where
        I: IntoIterator<Item = Value>,
    {
        Self {
            value: Self::assert_value(value, max),
            candidates: Self::import_candidates(candidates, max),
        }
    }

    fn new_with_value(value: Value, max: Value) -> Self {
        Self::new(value, vec![], max)
    }

    fn value(&self) -> Value {
        self.value
    }

    fn candidates(&self) -> Vec<Value> {
        Self::export_candidates(&self.candidates)
    }

    fn set_value(&mut self, value: Value, max: Value) -> Value {
        let new_value = Self::assert_value(value, max);

        replace(&mut self.value, new_value)
    }

    fn set_candidates<I>(&mut self, candidates: I, max: Value)
    where
        I: IntoIterator<Item = Value>,
    {
        self.candidates = Self::import_candidates(candidates, max)
    }

    fn toggle_candidate(&mut self, candidate: Value, max: Value) -> bool {
        let candidate = Self::assert_candidate(candidate, max).as_usize();

        let new_state = !self.candidates[candidate];

        self.candidates.set(candidate, new_state);

        new_state
    }
}

/// Conversion Helpers
impl<Value> Cell<Value>
where
    Value: SudokuValue,
    Value::Error: std::error::Error,
{
    fn import_candidates<I: IntoIterator<Item = Value>>(candidates: I, max: Value) -> FixedBitSet {
        let mut candidates: FixedBitSet = candidates
            .into_iter()
            .map(|candidate| {
                let candidate: Value = Self::assert_candidate(candidate, max);
                candidate.as_usize()
            })
            .collect();

        candidates.grow(max.as_usize() + 1);

        candidates
    }
    fn export_candidates(candidates: &FixedBitSet) -> Vec<Value> {
        candidates.ones().map(Self::export_candidate).collect()
    }

    fn export_candidate(candidate: usize) -> Value {
        Value::try_from(candidate).unwrap() + Value::one()
    }

    fn assert_value(value: Value, max: Value) -> Value {
        assert!(value <= max);

        value
    }

    fn assert_candidate(candidate: Value, max: Value) -> Value {
        assert!(candidate != Value::zero() && candidate <= max);

        candidate - Value::one()
    }
}

impl<Value> Display for Cell<Value>
where
    Value: SudokuValue,
    Value::Error: std::error::Error,
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_str(&self.value.grid_string())
    }
}
