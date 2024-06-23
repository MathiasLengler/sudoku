use crate::base::SudokuBase;
use crate::cell::{Candidates, Value};
use crate::grid::Grid;
use crate::position::Position;
use crate::solver::backtracking::GroupAvailabilityIndex;

fn default_apply_to_grid_candidates<Base: SudokuBase>(
    filter: &impl AvailabilityFilter<Base>,
    grid: &mut Grid<Base>,
) {
    grid.all_candidates_positions().into_iter().for_each(|pos| {
        let cell = &mut grid[pos];
        let candidates = cell.candidates().unwrap();
        cell.set_candidates(filter.filter(candidates, pos.into()));
    });
}

pub type DeniedCandidatesGrid<Base> = Grid<Base, Candidates<Base>>;

// TODO: test implementations

// TODO: rename to CandidatesFilter
/// A constraint for a sudoku solver.
/// It limits which candidates are considered for a solution.
pub trait AvailabilityFilter<Base: SudokuBase> {
    /// Whether the filter keeps the `available_candidates` unchanged for all `index`es.
    ///
    /// Defaults to `false`.
    const IS_NOOP: bool = false;

    /// For a given `available_candidates` at `index`, return `available_candidates` with zero or more candidates removed.
    ///
    /// The returned candidates are considered available for a solution.
    ///
    /// Must be a pure function.
    fn filter(
        &self,
        available_candidates: Candidates<Base>,
        index: GroupAvailabilityIndex<Base>,
    ) -> Candidates<Base>;

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
impl<Base: SudokuBase> AvailabilityFilter<Base> for () {
    const IS_NOOP: bool = true;

    fn filter(
        &self,
        available_candidates: Candidates<Base>,
        _index: GroupAvailabilityIndex<Base>,
    ) -> Candidates<Base> {
        available_candidates
    }

    fn apply_to_grid_candidates(&self, _grid: &mut Grid<Base>) {
        // no-op
    }
}

/// A function can modify `available_candidates`
impl<
        Base: SudokuBase,
        F: Fn(Candidates<Base>, GroupAvailabilityIndex<Base>) -> Candidates<Base>,
    > AvailabilityFilter<Base> for F
{
    fn filter(
        &self,
        available_candidates: Candidates<Base>,
        index: GroupAvailabilityIndex<Base>,
    ) -> Candidates<Base> {
        self(available_candidates, index)
    }
}

/// A `Grid` containing `Candidates` defines candidates which are *not* available.
impl<Base: SudokuBase> AvailabilityFilter<Base> for DeniedCandidatesGrid<Base> {
    fn filter(
        &self,
        available_candidates: Candidates<Base>,
        index: GroupAvailabilityIndex<Base>,
    ) -> Candidates<Base> {
        let denied_candidates = *self.get(index.into());

        available_candidates.without(denied_candidates)
    }
}

/// If `Some`, a `Option<Grid>` containing `Candidates` defines candidates which are *not* available.
/// If `None`, no-op.
impl<Base: SudokuBase> AvailabilityFilter<Base> for Option<DeniedCandidatesGrid<Base>> {
    fn filter(
        &self,
        available_candidates: Candidates<Base>,
        index: GroupAvailabilityIndex<Base>,
    ) -> Candidates<Base> {
        if let Some(denylist) = self {
            denylist.filter(available_candidates, index)
        } else {
            available_candidates
        }
    }

    fn apply_to_grid_candidates(&self, grid: &mut Grid<Base>) {
        if let Some(denylist) = self {
            default_apply_to_grid_candidates(denylist, grid);
        }
    }
}

// TODO: implement `AvailabilityFilter`
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DisallowedCandidateAtPosition<Base: SudokuBase> {
    pub position: Position<Base>,
    pub candidate: Value<Base>,
}
