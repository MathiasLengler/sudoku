use crate::base::SudokuBase;
use crate::cell::{Candidates, Value};
use crate::grid::Grid;
use crate::position::Position;

pub type DeniedCandidatesGrid<Base> = Grid<Base, Candidates<Base>>;

/// A constraint for a sudoku solver.
/// It limits which candidates are considered for a solution.
pub trait CandidatesFilter<Base: SudokuBase> {
    /// For a given `index`, return the candidates that are denied by this filter, e.g. *not* available for a solution.
    fn denied_candidates(&self, pos: Position<Base>) -> Candidates<Base>;

    /// A iterator over all positions for which a candidate is denied by this filter.
    ///
    /// The default implementation iterates over all positions and returns those with non-empty `denied_candidates`.
    fn all_denied_candidates(&self) -> impl Iterator<Item = (Position<Base>, Candidates<Base>)> {
        Position::all()
            .map(move |pos| (pos, self.denied_candidates(pos)))
            .filter(|(_, candidates)| !candidates.is_empty())
    }

    /// Apply this filter to a given grid.
    ///
    /// The default implementation iterates over all denied candidates and removes them from the corresponding cell's candidates.
    fn apply_to_grid_candidates(&self, grid: &mut Grid<Base>) {
        self.all_denied_candidates()
            .for_each(|(pos, denied_candidates)| {
                let cell = &mut grid[pos];
                let candidates = cell
                    .candidates()
                    .expect("CandidatesFilter to only target cells with candidates");
                cell.set_candidates(candidates.without(denied_candidates));
            });
    }
}

/// A no-op filter that does not deny any candidates.
///
/// Used as a default filter.
impl<Base: SudokuBase> CandidatesFilter<Base> for () {
    fn denied_candidates(&self, _pos: Position<Base>) -> Candidates<Base> {
        Candidates::new()
    }

    fn all_denied_candidates(&self) -> impl Iterator<Item = (Position<Base>, Candidates<Base>)> {
        std::iter::empty()
    }

    fn apply_to_grid_candidates(&self, _grid: &mut Grid<Base>) {
        // No-op
    }
}

/// A `Grid` containing `Candidates` defines candidates which are *not* available.
impl<Base: SudokuBase> CandidatesFilter<Base> for DeniedCandidatesGrid<Base> {
    fn denied_candidates(&self, pos: Position<Base>) -> Candidates<Base> {
        *self.get(pos)
    }
}

/// A filter that disallows a single candidate at a specific position.
///
/// Used in `Generator::try_delete_cell_at_pos` to check if a pruned cell resulted in multiple solutions.
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

/// A filter that only allows a single candidate at a specific position.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::consts::*;
    use crate::cell::Cell;
    use crate::samples::base_2_candidates_coordinates;

    fn assert_self_consistent_filter(filter: &impl CandidatesFilter<Base2>) {
        for (pos, denied_candidates) in filter.all_denied_candidates() {
            // The returned `denied_candidates` from `all_denied_candidates` match `denied_candidates()` for the same position.
            assert_eq!(denied_candidates, filter.denied_candidates(pos));
            // `denied_candidates` is non-empty.
            assert!(!denied_candidates.is_empty());
        }

        // All non-denied positions return empty candidates.
        let denied_positions = filter
            .all_denied_candidates()
            .map(|(pos, _)| pos)
            .collect::<std::collections::HashSet<_>>();
        for non_denied_pos in Position::<Base2>::all().filter(|pos| !denied_positions.contains(pos))
        {
            assert!(filter.denied_candidates(non_denied_pos).is_empty());
        }

        // => `denied_candidates` and `all_denied_candidates` are consistent with each other.

        let canidates_test_grids = [
            // All empty candidates
            Grid::new(),
            // All full candidates
            Grid::filled_with(Cell::with_candidates(Candidates::all())),
            // All possible candidates
            base_2_candidates_coordinates(),
        ];
        for test_grid in canidates_test_grids {
            let filtered_grid = {
                let mut grid = test_grid.clone();
                filter.apply_to_grid_candidates(&mut grid);
                grid
            };

            for pos in Position::<Base2>::all() {
                let original_candidates = test_grid[pos].candidates().unwrap();
                let filtered_candidates = filtered_grid[pos].candidates().unwrap();

                // The filter does not introduce candidates that were not present before.
                // i.e., filtered candidates must be a subset of original candidates.
                assert!(
                    filtered_candidates.without(original_candidates).is_empty(),
                    "Filter introduced new candidates at {pos:?}"
                );

                // The filter only removes candidates that were denied.
                let removed_candidates = original_candidates.without(filtered_candidates);
                let denied_candidates = filter.denied_candidates(pos);
                assert!(
                    removed_candidates.without(denied_candidates).is_empty(),
                    "Filter removed candidates that were not denied at {pos:?}"
                );
            }
        }
    }

    #[test]
    fn test_noop_filter() {
        let filter = ();

        for pos in Position::<Base2>::all() {
            assert!(filter.denied_candidates(pos).is_empty());
        }

        assert!(CandidatesFilter::<Base2>::all_denied_candidates(&filter)
            .next()
            .is_none());

        let grid = base_2_candidates_coordinates();
        let filtered_grid = {
            let mut grid = grid.clone();
            filter.apply_to_grid_candidates(&mut grid);
            grid
        };

        assert_eq!(filtered_grid, grid);

        assert_self_consistent_filter(&filter);
    }

    #[test]
    fn test_denied_candidates_grid_filter() {
        let mut denied_candiates_grid = Grid::<Base2, Candidates<Base2>>::new();
        // [] - [3] = []
        denied_candiates_grid[Position::top_left()] =
            Candidates::with_single(3.try_into().unwrap());
        // [1,2] - [2] = [1]
        denied_candiates_grid[Position::top_right()] =
            Candidates::with_single(2.try_into().unwrap());
        // [3,4] - [1,2,4] = [3]
        denied_candiates_grid[Position::bottom_left()] =
            Candidates::try_from(vec![1, 2, 4]).unwrap();
        // [1,2,3,4] - [1,2,3,4] = []
        denied_candiates_grid[Position::bottom_right()] = Candidates::all();

        let grid = base_2_candidates_coordinates();
        let filtered_grid = {
            let mut grid = grid.clone();
            denied_candiates_grid.apply_to_grid_candidates(&mut grid);
            grid
        };

        let mut expected_filtered_grid = grid.clone();
        expected_filtered_grid[Position::top_left()].set_candidates(Candidates::new());
        expected_filtered_grid[Position::top_right()]
            .set_candidates(Candidates::with_single(1.try_into().unwrap()));
        expected_filtered_grid[Position::bottom_left()]
            .set_candidates(Candidates::with_single(3.try_into().unwrap()));
        expected_filtered_grid[Position::bottom_right()].set_candidates(Candidates::new());
        let filter = denied_candiates_grid;

        assert_eq!(filtered_grid, expected_filtered_grid);

        assert_self_consistent_filter(&filter);
    }

    #[test]
    fn test_disallowed_candidate_at_position() {
        let filter = DisallowedCandidateAtPosition {
            // [3,4]
            pos: Position::bottom_right(),
            candidate: Value::max(),
        };
        let all_denied_candidates: Vec<_> = filter.all_denied_candidates().collect();
        assert_eq!(
            all_denied_candidates,
            vec![(
                Position::bottom_right(),
                Candidates::with_single(Value::max())
            )]
        );

        assert_self_consistent_filter(&filter);
    }

    #[test]
    fn test_force_candidate_at_position() {
        let filter = ForceCandidateAtPosition {
            // [3,4]
            pos: Position::bottom_right(),
            candidate: Value::max(),
        };
        let all_denied_candidates: Vec<_> = filter.all_denied_candidates().collect();
        assert_eq!(
            all_denied_candidates,
            vec![(
                Position::bottom_right(),
                Candidates::all().without(Candidates::with_single(Value::max()))
            )]
        );

        assert_self_consistent_filter(&filter);
    }
}
