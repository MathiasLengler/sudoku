use crate::base::SudokuBase;
use crate::cell::{Candidates, Value};
use crate::grid::Grid;
use crate::position::Position;

fn default_apply_to_grid_candidates<Base: SudokuBase>(
    filter: &impl CandidatesFilter<Base>,
    grid: &mut Grid<Base>,
) {
    filter
        .all_denied_candidates()
        .for_each(|(pos, denied_candidates)| {
            let cell = &mut grid[pos];
            let candidates = cell
                .candidates()
                .expect("CandidatesFilter to only target cells with candidates");
            cell.set_candidates(candidates.without(denied_candidates));
        });
}

pub type DeniedCandidatesGrid<Base> = Grid<Base, Candidates<Base>>;

// TODO: test implementations

/// A constraint for a sudoku solver.
/// It limits which candidates are considered for a solution.
pub trait CandidatesFilter<Base: SudokuBase> {
    /// Whether the filter keeps the `available_candidates` unchanged for all `index`es.
    ///
    /// Defaults to `false`.
    const IS_NOOP: bool = false;

    /// For a given `index`, return the candidates that are denied by this filter, e.g. *not* available for a solution.
    fn denied_candidates(&self, pos: Position<Base>) -> Candidates<Base>;

    /// A iterator over all positions for which a candidate is denied by this filter.
    fn all_denied_candidates(&self) -> impl Iterator<Item = (Position<Base>, Candidates<Base>)> {
        Position::all()
            .map(move |pos| (pos, self.denied_candidates(pos)))
            .filter(|(_, candidates)| !candidates.is_empty())
    }

    /// Apply this filter to a given grid.
    ///
    /// The default implementation iterates over the candidates in the grid
    /// and replaces them with the result of `self.filter`.
    fn apply_to_grid_candidates(&self, grid: &mut Grid<Base>)
    where
        Self: Sized,
    {
        default_apply_to_grid_candidates(self, grid);
    }
}

// No-op
impl<Base: SudokuBase> CandidatesFilter<Base> for () {
    const IS_NOOP: bool = true;

    fn denied_candidates(&self, _pos: Position<Base>) -> Candidates<Base> {
        Candidates::new()
    }

    fn all_denied_candidates(&self) -> impl Iterator<Item = (Position<Base>, Candidates<Base>)> {
        std::iter::empty()
    }
}

/// A `Grid` containing `Candidates` defines candidates which are *not* available.
impl<Base: SudokuBase> CandidatesFilter<Base> for DeniedCandidatesGrid<Base> {
    fn denied_candidates(&self, pos: Position<Base>) -> Candidates<Base> {
        *self.get(pos)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DisallowedCandidateAtPosition<Base: SudokuBase> {
    pub pos: Position<Base>,
    pub candidate: Value<Base>,
}

impl<Base: SudokuBase> DisallowedCandidateAtPosition<Base> {
    fn denied_candidates_at_pos(self) -> Candidates<Base> {
        Candidates::with_single(self.candidate)
    }
}

impl<Base: SudokuBase> CandidatesFilter<Base> for DisallowedCandidateAtPosition<Base> {
    fn denied_candidates(&self, pos: Position<Base>) -> Candidates<Base> {
        if pos == self.pos {
            self.denied_candidates_at_pos()
        } else {
            Candidates::new()
        }
    }
    fn all_denied_candidates(&self) -> impl Iterator<Item = (Position<Base>, Candidates<Base>)> {
        std::iter::once((self.pos, self.denied_candidates_at_pos()))
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ForceCandidateAtPosition<Base: SudokuBase> {
    pub pos: Position<Base>,
    pub candidate: Value<Base>,
}

impl<Base: SudokuBase> ForceCandidateAtPosition<Base> {
    fn denied_candidates_at_pos(self) -> Candidates<Base> {
        Candidates::all().without(Candidates::with_single(self.candidate))
    }
}

impl<Base: SudokuBase> CandidatesFilter<Base> for ForceCandidateAtPosition<Base> {
    fn denied_candidates(&self, pos: Position<Base>) -> Candidates<Base> {
        if pos == self.pos {
            self.denied_candidates_at_pos()
        } else {
            Candidates::new()
        }
    }
    fn all_denied_candidates(&self) -> impl Iterator<Item = (Position<Base>, Candidates<Base>)> {
        std::iter::once((self.pos, self.denied_candidates_at_pos()))
    }
}
