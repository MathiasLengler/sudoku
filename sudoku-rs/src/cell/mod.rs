use std::fmt::{self, Debug, Display, Formatter};
use std::mem::replace;
use std::num::NonZeroUsize;

use fixedbitset::FixedBitSet;
use num::{NumCast, One, PrimInt, Unsigned};
use serde::{Deserialize, Serialize};

use crate::cell::value::SudokuValue;

pub mod value;

// TODO: set or toggle value

// TODO: is_editable cell

// TODO: remove Value::Primitive from public API
pub trait SudokuCell<Value = NonZeroUsize>: Clone + Display + Debug + Ord + Eq + Send
where
    Value: SudokuValue,
    Value::Primitive: PrimInt + Unsigned,
{
    /// Constructs new empty cell (empty candidates and no value)
    fn new(max: Value::Primitive) -> Self;
    /// Constructs a new cell with a set value
    fn new_with_value(value: Value::Primitive, max: Value::Primitive) -> Self;
    /// Constructs a new cell with the provided candidates
    fn new_with_candidates<I>(candidates: I, max: Value::Primitive) -> Self
    where
        I: IntoIterator<Item = Value::Primitive>;

    fn view(&self) -> CellView;

    /// Value if any.
    fn value(&self) -> Option<Value::Primitive>;

    /// Value as a usize if any.
    fn value_as_usize(&self) -> Option<usize>;

    /// Candidates if any
    fn candidates(&self) -> Option<Vec<Value::Primitive>>;

    fn delete(&mut self, max: Value::Primitive);

    fn set_value(&mut self, value: Value::Primitive, max: Value::Primitive);
    fn set_or_toggle_value(&mut self, value: Value::Primitive, max: Value::Primitive);
    fn set_candidates<I>(&mut self, candidates: I, max: Value::Primitive)
    where
        I: IntoIterator<Item = Value::Primitive>;

    fn toggle_candidate(&mut self, candidate: Value::Primitive, max: Value::Primitive);
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Debug)]
pub enum Cell<Value = NonZeroUsize>
where
    Value: SudokuValue,
    Value::Primitive: PrimInt + Unsigned,
{
    Value(Value),
    Candidates(FixedBitSet),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind")]
pub enum CellView {
    Value { value: usize },
    Candidates { candidates: Vec<usize> },
}

impl<Value> SudokuCell<Value> for Cell<Value>
where
    Value: SudokuValue,
    Value::Primitive: PrimInt + Unsigned,
{
    fn new(max: Value::Primitive) -> Self {
        Self::new_with_candidates(std::iter::empty(), max)
    }

    fn new_with_value(value: Value::Primitive, max: Value::Primitive) -> Self {
        Cell::Value(Self::import_value(value, max))
    }

    fn new_with_candidates<I>(candidates: I, max: Value::Primitive) -> Self
    where
        I: IntoIterator<Item = Value::Primitive>,
    {
        Cell::Candidates(Self::import_candidates(candidates, max))
    }

    fn view(&self) -> CellView {
        match self {
            Cell::Value(value) => CellView::Value {
                value: Self::primitive_as_usize(Self::export_value(*value)),
            },
            Cell::Candidates(candidates) => CellView::Candidates {
                candidates: Self::export_candidates(candidates)
                    .into_iter()
                    .map(Self::primitive_as_usize)
                    .collect(),
            },
        }
    }

    fn value(&self) -> Option<Value::Primitive> {
        match self {
            Cell::Value(value) => Some(Self::export_value(*value)),
            Cell::Candidates(_) => None,
        }
    }

    fn value_as_usize(&self) -> Option<usize> {
        self.value().map(|value| Self::primitive_as_usize(value))
    }

    fn candidates(&self) -> Option<Vec<Value::Primitive>> {
        match self {
            Cell::Candidates(candidates) => Some(Self::export_candidates(candidates)),
            Cell::Value(_) => None,
        }
    }

    fn delete(&mut self, max: Value::Primitive) {
        replace(self, Self::new(max));
    }

    fn set_value(&mut self, value: Value::Primitive, max: Value::Primitive) {
        replace(self, Cell::Value(Self::import_value(value, max)));
    }

    fn set_or_toggle_value(&mut self, value: Value::Primitive, max: Value::Primitive) {
        match self {
            Cell::Value(current_value) => {
                if current_value.get() == value {
                    self.delete(max);
                } else {
                    self.set_value(value, max)
                }
            }
            Cell::Candidates(_) => self.set_value(value, max),
        }
    }

    fn set_candidates<I>(&mut self, candidates: I, max: Value::Primitive)
    where
        I: IntoIterator<Item = Value::Primitive>,
    {
        replace(
            self,
            Cell::Candidates(Self::import_candidates(candidates, max)),
        );
    }
    fn toggle_candidate(&mut self, candidate: Value::Primitive, max: Value::Primitive) {
        let imported_candidate = Self::import_candidate(candidate, max);

        match self {
            Cell::Candidates(candidates) => {
                candidates.set(imported_candidate, !candidates[imported_candidate]);
            }
            Cell::Value(_) => {
                replace(
                    self,
                    Self::new_with_candidates(std::iter::once(candidate), max),
                );
            }
        }
    }
}

/// Conversion Helpers
impl<Value> Cell<Value>
where
    Value: SudokuValue,
    Value::Primitive: PrimInt + Unsigned,
{
    fn import_candidates<I: IntoIterator<Item = Value::Primitive>>(
        candidates: I,
        max: Value::Primitive,
    ) -> FixedBitSet {
        let mut candidates: FixedBitSet = candidates
            .into_iter()
            .map(|candidate| Self::import_candidate(candidate, max))
            .collect();

        candidates.grow(Self::primitive_as_usize(max) + 1);

        candidates
    }
    fn import_value(value: Value::Primitive, max: Value::Primitive) -> Value {
        assert!(value <= max);

        let value = Value::new(value).expect("Value can't be 0");

        value
    }

    fn import_candidate(candidate: Value::Primitive, max: Value::Primitive) -> usize {
        assert!(candidate <= max);

        Value::new(candidate).expect("Candidate can't be 0");

        Self::primitive_as_usize(candidate - Value::Primitive::one())
    }

    fn export_value(value: Value) -> Value::Primitive {
        value.get()
    }

    fn export_candidates(candidates: &FixedBitSet) -> Vec<Value::Primitive> {
        candidates.ones().map(Self::export_candidate).collect()
    }

    fn export_candidate(candidate: usize) -> Value::Primitive {
        <Value::Primitive as NumCast>::from(candidate + 1).unwrap()
    }

    fn primitive_as_usize(primitive: Value::Primitive) -> usize {
        // Unwrap should be safe.
        <usize as NumCast>::from(primitive).unwrap()
    }
}

impl<Value> Display for Cell<Value>
where
    Value: SudokuValue,
    Value::Primitive: PrimInt + Unsigned,
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_str(&if let Some(value) = self.value_as_usize() {
            value.to_string()
        } else {
            "_".to_string()
        })
    }
}
