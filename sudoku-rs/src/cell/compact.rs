use std::fmt::{self, Debug, Display, Formatter};

use cell_state::CellState;

use crate::base::SudokuBase;
use crate::cell::compact::candidates::Candidates;
use crate::cell::view::CellView;

pub mod candidates;
mod cell_state;

/// Memory efficient representation of a single Sudoku cell.
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug)]
pub struct Cell<Base: SudokuBase>(CellState<Base>);

impl<Base: SudokuBase> Cell<Base> {
    /// Constructs new empty cell (empty candidates and no value)
    pub fn new() -> Self {
        Self(CellState::new())
    }

    /// Constructs a new cell with a value and if it should be fixed.
    pub fn with_value(value: u8, fixed: bool) -> Self {
        Self(CellState::with_value(value, fixed))
    }

    /// Constructs a new cell with the provided candidates
    pub fn with_candidates<I>(candidates: I) -> Self
    where
        I: IntoIterator<Item = u8>,
    {
        Self(CellState::with_candidates(candidates))
    }

    /// Convenient view of the cell.
    pub fn view(&self) -> CellView {
        self.0.view()
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
        self.0.fix()
    }

    /// Unfix a value if it was fixed.
    pub fn unfix(&mut self) {
        self.0.unfix()
    }

    /// Value if any, either fixed or unfixed.
    pub fn value(&self) -> Option<u8> {
        self.0.value()
    }

    /// Candidates if any
    pub fn candidates(&self) -> Option<Vec<u8>> {
        self.0.candidates()
    }

    /// Delete contents of the cell
    ///
    /// # Panics
    ///
    /// Panics it the cell is fixed.
    pub fn delete(&mut self) {
        self.0.delete()
    }

    /// Set the cell to a unfixed value.
    /// Deletes candidates if present.
    ///
    /// # Panics
    ///
    /// Panics it the cell is fixed.
    pub fn set_value(&mut self, value: u8) {
        self.0.set_value(value)
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
    pub fn set_or_toggle_value(&mut self, value: u8) -> bool {
        self.0.set_or_toggle_value(value)
    }

    /// Set the cell to the given candidates.
    /// Deletes value if present.
    ///
    /// Candidates can be anything convertible into the `Candidates` struct.
    /// This is commonly a `Vec<u8>` via its implementation of `IntoIterator<Item = u8>`
    /// or a `Candidates` struct directly.
    ///
    /// # Panics
    ///
    /// Panics it the cell is fixed.
    pub fn set_candidates<I: Into<Candidates<Base>>>(&mut self, candidates: I) {
        self.0.set_candidates(candidates)
    }

    /// Toggle the given candidate.
    /// Deletes value if present and sets the single candidate.
    ///
    /// # Panics
    ///
    /// Panics it the cell is fixed.
    pub fn toggle_candidate(&mut self, candidate: u8) {
        self.0.toggle_candidate(candidate)
    }

    /// Deletes the given candidate if the cell contains candidates.
    ///
    /// # Panics
    ///
    /// Panics it the cell is fixed.
    pub fn delete_candidate(&mut self, candidate: u8) {
        self.0.delete_candidate(candidate)
    }
}

impl<Base: SudokuBase> Display for Cell<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
