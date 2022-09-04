use crate::base::SudokuBase;
use crate::cell::compact::candidates::Candidates;
use crate::cell::compact::value::Value;

// TODO: refactor API, either:
//  - merge additional functionality into Candidates
//  - replicate all required functionality of Candidates on CandidatesCell

/// A sudoku cell represented as a candidates bitset.
///
/// Mirrors the API of `Cell<Base>`, but with a few exceptions:
/// - No fixed state.
/// - No cell.state()
/// - No difference between a single candidate and value.
///
/// Use-case: more compact/efficient cell state for the backtracking solver.
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone, Debug, Default)]
pub struct CandidatesCell<Base: SudokuBase> {
    candidates: Candidates<Base>,
}

impl<Base: SudokuBase> CandidatesCell<Base> {
    /// Constructs new empty cell (empty candidates and no value)
    pub fn new() -> Self {
        Self::default()
    }

    /// Constructs a new cell with a value and if it should be fixed.
    pub fn with_value(value: Value<Base>) -> Self {
        Self {
            candidates: Candidates::single(value),
        }
    }

    /// Constructs a new cell with the provided candidates
    pub fn with_candidates(candidates: Candidates<Base>) -> Self {
        Self { candidates }
    }

    /// If the cell contains a fixed or unfixed value.
    pub fn has_value(&self) -> bool {
        self.candidates.count() == 1
    }
    pub fn has_candidates(&self) -> bool {
        !self.has_value()
    }

    /// Value if any
    pub fn value(&self) -> Option<Value<Base>> {
        let mut values = self.candidates.iter();

        if let Some(value) = values.next() {
            if let None = values.next() {
                return Some(value);
            }
        }
        None
    }

    pub fn candidates(&self) -> Candidates<Base> {
        self.candidates
    }

    /// Delete contents of the cell
    pub fn delete(&mut self) {
        *self = Self::new()
    }

    /// Set the cell to a unfixed value.
    /// Deletes candidates if present.
    pub fn set_value(&mut self, value: Value<Base>) {
        *self = Self::with_value(value)
    }

    /// Set the cell to a value.
    /// If the cell contained the same value, delete the cell.
    /// Deletes candidates if present.
    ///
    /// Returns true if a new value has been set.
    pub fn set_or_toggle_value(&mut self, value: Value<Base>) -> bool {
        // Remove other candidates
        self.candidates = self.candidates.intersection(&Candidates::single(value));

        // Toggle value
        self.candidates.toggle(value);

        !self.candidates.is_empty()
    }

    /// Set the cell to the given candidates.
    /// Deletes value if present.
    pub fn set_candidates(&mut self, candidates: Candidates<Base>) {
        *self = Self::with_candidates(candidates)
    }

    /// Toggle the given candidate or last value.
    pub fn toggle_candidate(&mut self, candidate: Value<Base>) {
        self.candidates.toggle(candidate)
    }

    /// Deletes the given candidate or last value.
    pub fn delete_candidate(&mut self, candidate: Value<Base>) {
        self.candidates.delete(candidate)
    }

    /// Set the given candidate to the given enabled state.
    pub fn set_candidate(&mut self, candidate: Value<Base>, enabled: bool) {
        self.candidates.set(candidate, enabled)
    }
}
