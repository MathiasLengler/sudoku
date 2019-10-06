use std::fmt::{self, Debug, Display, Formatter};
use std::hash::Hash;
use std::mem::replace;
use std::num::NonZeroU8;

use fixedbitset::FixedBitSet;
use num::{cast, ToPrimitive};

use crate::cell::view::CellView;

pub mod view;

// TODO: set_candidates_bit_set optimization
//  assert len
pub trait SudokuCell: Clone + Display + Debug + Ord + Eq + Hash + Send {
    /// Constructs new empty cell (empty candidates and no value)
    fn new(max: usize) -> Self;
    /// Constructs a new cell with a set value
    fn new_with_value(value: usize, max: usize) -> Self;
    /// Constructs a new cell with the provided candidates
    fn new_with_candidates<I>(candidates: I, max: usize) -> Self
    where
        I: IntoIterator<Item = usize>;

    fn view(&self) -> CellView;

    /// Value if any.
    fn value(&self) -> Option<usize>;

    /// Candidates if any
    fn candidates(&self) -> Option<Vec<usize>>;

    fn delete(&mut self, max: usize) -> Self;

    fn set_value(&mut self, value: usize, max: usize);

    /// Returns true if a new value has been set.
    fn set_or_toggle_value(&mut self, value: usize, max: usize) -> bool;
    fn set_candidates<I>(&mut self, candidates: I, max: usize)
    where
        I: IntoIterator<Item = usize>;

    fn toggle_candidate(&mut self, candidate: usize, max: usize);

    fn delete_candidate(&mut self, candidate: usize, max: usize);
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug)]
pub enum Cell {
    Value(NonZeroU8),
    Candidates(FixedBitSet),
}

impl SudokuCell for Cell {
    fn new(max: usize) -> Self {
        Self::new_with_candidates(std::iter::empty(), max)
    }

    fn new_with_value(value: usize, max: usize) -> Self {
        if value == 0 {
            Self::new(max)
        } else {
            Cell::Value(Self::import_value(value, max))
        }
    }

    fn new_with_candidates<I>(candidates: I, max: usize) -> Self
    where
        I: IntoIterator<Item = usize>,
    {
        Cell::Candidates(Self::import_candidates(candidates, max))
    }

    fn view(&self) -> CellView {
        match self {
            Cell::Value(value) => CellView::Value {
                value: Self::export_value(*value),
            },
            Cell::Candidates(candidates) => CellView::Candidates {
                candidates: Self::export_candidates(candidates),
            },
        }
    }

    fn value(&self) -> Option<usize> {
        match self {
            Cell::Value(value) => Some(Self::export_value(*value)),
            Cell::Candidates(_) => None,
        }
    }

    fn candidates(&self) -> Option<Vec<usize>> {
        match self {
            Cell::Candidates(candidates) => Some(Self::export_candidates(candidates)),
            Cell::Value(_) => None,
        }
    }

    fn delete(&mut self, max: usize) -> Self {
        replace(self, Self::new(max))
    }

    fn set_value(&mut self, value: usize, max: usize) {
        replace(self, Self::new_with_value(value, max));
    }

    fn set_or_toggle_value(&mut self, value: usize, max: usize) -> bool {
        match self {
            Cell::Value(current_value) => {
                if Self::export_value(*current_value) == value {
                    self.delete(max);
                    false
                } else {
                    self.set_value(value, max);
                    true
                }
            }
            Cell::Candidates(_) => {
                self.set_value(value, max);
                true
            }
        }
    }

    fn set_candidates<I>(&mut self, candidates: I, max: usize)
    where
        I: IntoIterator<Item = usize>,
    {
        replace(
            self,
            Cell::Candidates(Self::import_candidates(candidates, max)),
        );
    }
    fn toggle_candidate(&mut self, candidate: usize, max: usize) {
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

    fn delete_candidate(&mut self, candidate: usize, max: usize) {
        let imported_candidate = Self::import_candidate(candidate, max);

        match self {
            Cell::Candidates(candidates) => {
                candidates.set(imported_candidate, false);
            }
            Cell::Value(_) => {}
        }
    }
}

/// Conversion Helpers
impl Cell {
    fn import_candidates<I: IntoIterator<Item = usize>>(candidates: I, max: usize) -> FixedBitSet {
        // TODO: rewrite with extend
        let mut candidates: FixedBitSet = candidates
            .into_iter()
            .map(|candidate| Self::import_candidate(candidate, max))
            .collect();

        candidates.grow(max + 1);

        candidates
    }
    fn import_value(value: usize, max: usize) -> NonZeroU8 {
        assert!(value <= max);

        let value_as_primitive: u8 =
            cast(value).expect("Could not convert value to cell value primitive");

        let value = NonZeroU8::new(value_as_primitive).expect("Value can't be 0");

        value
    }

    fn import_candidate(candidate: usize, max: usize) -> usize {
        assert!(candidate != 0 && candidate <= max);

        candidate - 1
    }

    fn export_value(value: NonZeroU8) -> usize {
        value.get().to_usize().unwrap()
    }

    fn export_candidates(candidates: &FixedBitSet) -> Vec<usize> {
        candidates.ones().map(Self::export_candidate).collect()
    }

    fn export_candidate(candidate: usize) -> usize {
        candidate + 1
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_str(&if let Some(value) = self.value() {
            value.to_string()
        } else {
            "_".to_string()
        })
    }
}
