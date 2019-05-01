use std::fmt::{self, Debug, Display, Formatter};
use std::mem::replace;

use fixedbitset::FixedBitSet;

use crate::cell::value::SudokuValue;

pub mod value;

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
            value: Self::import_value(value, max),
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
        let new_value = Self::import_value(value, max);

        replace(&mut self.value, new_value)
    }

    fn set_candidates<I>(&mut self, candidates: I, max: Value)
    where
        I: IntoIterator<Item = Value>,
    {
        self.candidates = Self::import_candidates(candidates, max)
    }
}

/// Conversion Helpers
impl<Value> Cell<Value>
where
    Value: SudokuValue,
    Value::Error: std::error::Error,
{
    fn import_value(value: Value, max: Value) -> Value {
        Self::assert_value(value, max)
    }
    fn import_candidates<I: IntoIterator<Item = Value>>(candidates: I, max: Value) -> FixedBitSet {
        candidates
            .into_iter()
            .map(|candidate| {
                let candidate: Value = Self::assert_candidate(candidate, max);
                candidate.as_usize()
            })
            .collect()
    }
    fn export_candidates(candidates: &FixedBitSet) -> Vec<Value> {
        candidates
            .ones()
            .map(|candidate| Value::try_from(candidate).unwrap())
            .collect()
    }

    fn assert_value(value: Value, max: Value) -> Value {
        assert!(value <= max);

        value
    }

    fn assert_candidate(candidate: Value, max: Value) -> Value {
        assert!(candidate != Value::zero() && candidate <= max);

        candidate
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
