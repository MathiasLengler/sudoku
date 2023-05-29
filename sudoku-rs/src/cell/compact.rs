use std::fmt::{self, Debug, Display, Formatter};

use anyhow::bail;

pub use candidates::{Candidates, CandidatesIter};
pub(crate) use cell_state::CellState;
pub use value::Value;

use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicCell;
use crate::error::Error;

mod candidates;
mod cell_state;
mod value;

/// Memory efficient representation of a single Sudoku cell.
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug, Default)]
pub struct Cell<Base: SudokuBase>(CellState<Base>);

impl<Base: SudokuBase> Cell<Base> {
    /// Constructs new empty cell (empty candidates and no value)
    pub fn new() -> Self {
        Self(CellState::new())
    }

    /// Constructs a new cell with a value and if it should be fixed.
    pub fn with_value(value: Value<Base>, fixed: bool) -> Self {
        Self(CellState::with_value(value, fixed))
    }

    /// Constructs a new cell with the provided candidates
    pub fn with_candidates(candidates: Candidates<Base>) -> Self {
        Self(CellState::with_candidates(candidates))
    }

    /// Expose internal `CellState`
    pub(crate) fn state(&self) -> &CellState<Base> {
        &self.0
    }

    /// If the cell contains a fixed or unfixed value.
    pub fn has_value(&self) -> bool {
        self.0.has_value()
    }
    pub fn has_unfixed_value(&self) -> bool {
        self.0.has_unfixed_value()
    }
    pub fn has_fixed_value(&self) -> bool {
        self.0.has_fixed_value()
    }
    pub fn has_candidates(&self) -> bool {
        self.0.has_candidates()
    }

    /// Fix the cell to the current value if it was unfixed.
    ///
    /// # Panics
    ///
    /// Panics it the cell does not contain a value
    pub fn fix(&mut self) {
        self.0.fix();
    }

    /// Unfix a value if it was fixed.
    pub fn unfix(&mut self) {
        self.0.unfix();
    }

    /// Value if any, either fixed or unfixed.
    pub fn value(&self) -> Option<Value<Base>> {
        self.0.value()
    }

    /// Candidates if any
    pub fn candidates(&self) -> Option<Candidates<Base>> {
        self.0.candidates()
    }

    /// Delete contents of the cell
    ///
    /// # Panics
    ///
    /// Panics it the cell is fixed.
    pub fn delete(&mut self) {
        self.0.delete();
    }

    /// Set the cell to a unfixed value.
    /// Deletes candidates if present.
    ///
    /// # Panics
    ///
    /// Panics it the cell is fixed.
    pub fn set_value(&mut self, value: Value<Base>) {
        self.0.set_value(value);
    }

    /// Set the cell to a unfixed value.
    /// If the cell contained the same value, delete the cell.
    /// Deletes candidates if present.
    ///
    /// Returns true if a new value has been set.
    ///
    /// # Panics
    ///
    /// Panics it the cell is fixed.
    pub fn set_or_toggle_value(&mut self, value: Value<Base>) -> bool {
        self.0.set_or_toggle_value(value)
    }

    /// Set the cell to the given candidates.
    /// Deletes value if present.
    ///
    /// # Panics
    ///
    /// Panics it the cell is fixed.
    pub fn set_candidates(&mut self, candidates: Candidates<Base>) {
        self.0.set_candidates(candidates);
    }

    /// Toggle the given candidate.
    /// Deletes value if present and sets the single candidate.
    ///
    /// # Panics
    ///
    /// Panics it the cell is fixed.
    pub fn toggle_candidate(&mut self, candidate: Value<Base>) {
        self.0.toggle_candidate(candidate);
    }

    /// Set the given candidate if the cell contains candidates.
    ///
    /// # Panics
    ///
    /// Panics it the cell is fixed.
    pub fn set_candidate(&mut self, candidate: Value<Base>) {
        self.0.set_candidate(candidate);
    }

    /// Deletes the given candidate if the cell contains candidates.
    ///
    /// # Panics
    ///
    /// Panics it the cell is fixed.
    pub fn delete_candidate(&mut self, candidate: Value<Base>) {
        self.0.delete_candidate(candidate);
    }
}

impl<Base: SudokuBase> Display for Cell<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<Base: SudokuBase> TryFrom<DynamicCell> for Cell<Base> {
    type Error = Error;

    fn try_from(cell_view: DynamicCell) -> Result<Self, Self::Error> {
        Ok(match cell_view {
            DynamicCell::Value { value, fixed } => {
                if let Some(value) = Value::new(value.0)? {
                    Cell::with_value(value, fixed)
                } else if !fixed {
                    Cell::new()
                } else {
                    bail!("An empty cell can't be fixed")
                }
            }
            DynamicCell::Candidates { candidates } => {
                Cell::with_candidates(candidates.0.try_into()?)
            }
        })
    }
}

impl<Base: SudokuBase> From<(Candidates<Base>, bool)> for Cell<Base> {
    fn from((candidates, is_fixed_value): (Candidates<Base>, bool)) -> Self {
        if let Some(value) = candidates.to_single() {
            Self::with_value(value, is_fixed_value)
        } else {
            Self::with_candidates(candidates)
        }
    }
}
