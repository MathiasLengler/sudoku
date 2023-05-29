//! Fork of [tdoku `solver_basic.cc`](https://github.com/t-dillon/tdoku/blob/master/src/solver_basic.cc)

use log::trace;

use crate::base::SudokuBase;
use crate::cell::Value;
use crate::cell::{Candidates, CandidatesIter};
use crate::grid::Grid;
use crate::position::Coordinate;
use crate::position::Position;
use crate::unsafe_utils::{get_unchecked, get_unchecked_mut};

// TODO: implement shuffle_candidates

#[derive(Debug, Clone)]
pub struct Solver<'a, Base: SudokuBase> {
    /// Grid to be solved
    grid: &'a Grid<Base>,
    /// Cached remaining candidates for each group.
    availability: GroupAvailability<Base>,
    /// Indices to non-value cells which must be solved.
    availability_indices: Vec<GroupAvailabilityIndex<Base>>,
    /// A list of iterators producing value assignments for each associated `availability_indices`.
    /// Can be inspected with `peek` to infer the current value assignment.
    candidates_iters: Vec<CandidatesIter<Base>>,

    pub guess_count: u64,

    has_returned_pre_filled_grid_solution: bool,
}

impl<'a, Base: SudokuBase> Solver<'a, Base> {
    pub fn new(grid: &'a Grid<Base>) -> Self {
        let mut this = Self {
            grid,
            availability: GroupAvailability::all(),
            availability_indices: vec![],
            candidates_iters: vec![],
            guess_count: 0,
            has_returned_pre_filled_grid_solution: false,
        };

        this.initialize(grid);

        this
    }

    fn initialize(&mut self, grid: &Grid<Base>) {
        for pos in Position::<Base>::all() {
            let (row, column) = pos.to_row_and_column();

            let index = GroupAvailabilityIndex {
                row,
                column,
                block: pos.to_block(),
            };

            if let Some(value) = grid.get(pos).value() {
                // clue, clear group availability
                self.availability.reserve(index, value);
            } else {
                // Non-value cell, add to choices
                self.availability_indices.push(index);
            }
        }

        self.move_best_choice_to_front(0);
        if let Some(availability_index) = self.availability_indices.first() {
            self.candidates_iters
                .push(self.availability.intersection(*availability_index).iter());
        }
    }

    pub fn move_best_choice_to_front(&mut self, front_i: usize) {
        use std::mem::swap;

        debug_assert!(self.candidates_iters.get(front_i).is_none());

        if let Some((first_index, rest)) = self.availability_indices[front_i..].split_first_mut() {
            let first_count = self.availability.intersection(*first_index).count();
            if first_count <= 1 {
                return;
            }

            let mut better_count = first_count;
            let mut better_index = None;

            for next_index in rest {
                if better_count <= 1 {
                    break;
                }
                let next_count = self.availability.intersection(*next_index).count();
                if next_count < better_count {
                    better_count = next_count;
                    better_index = Some(next_index);
                }
            }

            if let Some(better_index) = better_index {
                trace!(
                    "swapping {first_count} @ {first_index:?} with {better_count} @ {better_index:?}"
                );
                swap(first_index, better_index);
            }
        }
    }

    fn build_solution_grid(&self) -> Grid<Base> {
        let mut solution_grid = self.grid.clone();
        for (candidates_iter, choice_index) in self
            .candidates_iters
            .iter()
            .zip(self.availability_indices.iter())
        {
            solution_grid
                .get_mut((*choice_index).into())
                .set_value(candidates_iter.peek().unwrap());
        }
        solution_grid
    }

    pub fn try_solve(&mut self) -> Option<Grid<Base>> {
        while let Some(candidates) = self.candidates_iters.last() {
            if let Some(candidate) = candidates.peek() {
                // TODO: only update if there are multiple candidates
                self.guess_count += 1;

                let choice_index = self.availability_indices[self.candidates_iters.len() - 1];
                self.availability.reserve(choice_index, candidate);

                if self.candidates_iters.len() == self.availability_indices.len() {
                    // Found solution
                    let solution_grid = self.build_solution_grid();

                    // Continue at next candidate
                    self.candidates_iters.last_mut().unwrap().next();
                    self.availability.restore(choice_index, candidate);

                    return Some(solution_grid);
                } else {
                    // Next cell
                    let next_i = self.candidates_iters.len();
                    self.move_best_choice_to_front(next_i);
                    let next_choice_index = self.availability_indices[next_i];
                    let next_candidates_iter =
                        self.availability.intersection(next_choice_index).iter();
                    self.candidates_iters.push(next_candidates_iter);
                }
            } else {
                // Backtrack
                self.candidates_iters.pop().unwrap();
                let candidates_iters_len = self.candidates_iters.len();
                if let Some(prev_candidates) = self.candidates_iters.last_mut() {
                    if let Some(prev_candidate) = prev_candidates.peek() {
                        let prev_choice_index = self.availability_indices[candidates_iters_len - 1];
                        self.availability.restore(prev_choice_index, prev_candidate);
                    }

                    prev_candidates.next();
                }
            }
        }

        if self.availability_indices.is_empty() && !self.has_returned_pre_filled_grid_solution {
            self.has_returned_pre_filled_grid_solution = true;

            if self.grid.is_solved() {
                Some(self.grid.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<'s, Base: SudokuBase> Iterator for Solver<'s, Base> {
    type Item = Grid<Base>;

    fn next(&mut self) -> Option<Self::Item> {
        self.try_solve()
    }
}

#[derive(Debug, Clone, Default)]
struct GroupAvailability<Base: SudokuBase> {
    rows: Base::CandidatesGroup,
    columns: Base::CandidatesGroup,
    blocks: Base::CandidatesGroup,
}

impl<Base: SudokuBase> GroupAvailability<Base> {
    fn new() -> Self {
        Self::default()
    }

    fn all() -> Self {
        let mut this = Self::new();

        this.iter_mut()
            .for_each(|candidates| *candidates = Candidates::all());

        this
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Candidates<Base>> {
        self.rows
            .as_mut()
            .iter_mut()
            .chain(self.columns.as_mut().iter_mut())
            .chain(self.blocks.as_mut().iter_mut())
    }

    fn reserve(&mut self, index: GroupAvailabilityIndex<Base>, candidate: Value<Base>) {
        // Clear candidate availability
        self.mutate(index, |candidates| {
            candidates.set(candidate, false);
        });
    }
    fn restore(&mut self, index: GroupAvailabilityIndex<Base>, candidate: Value<Base>) {
        // Restore candidate availability
        self.mutate(index, |candidates| {
            candidates.set(candidate, true);
        });
    }

    fn mutate(
        &mut self,
        index: GroupAvailabilityIndex<Base>,
        mut f: impl FnMut(&mut Candidates<Base>),
    ) {
        let (row, column, block) = index.into_usize_tuple();

        // Safety: relies on invariants:
        // - Coordinate::<Base>::get: `coordinate < Base::SIDE_LENGTH`
        // - Base::CandidatesCells: array length equals `Base::SIDE_LENGTH`
        // Therefore the indexes remain in-bounds.
        let (row_candidates_cell, column_candidates_cell, block_candidates_cell) = unsafe {
            (
                get_unchecked_mut(self.rows.as_mut(), row),
                get_unchecked_mut(self.columns.as_mut(), column),
                get_unchecked_mut(self.blocks.as_mut(), block),
            )
        };

        f(row_candidates_cell);
        f(column_candidates_cell);
        f(block_candidates_cell);
    }

    fn intersection(&self, index: GroupAvailabilityIndex<Base>) -> Candidates<Base> {
        let (row, column, block) = index.into_usize_tuple();

        // Safety: relies on invariants:
        // - Coordinate::<Base>::get: `coordinate < Base::SIDE_LENGTH`
        // - Base::CandidatesCells: array length equals `Base::SIDE_LENGTH`
        // Therefore the indexes remain in-bounds.
        let (row_candidates, column_candidates, block_candidates) = unsafe {
            (
                get_unchecked(self.rows.as_ref(), row),
                get_unchecked(self.columns.as_ref(), column),
                get_unchecked(self.blocks.as_ref(), block),
            )
        };

        row_candidates
            .intersection(*column_candidates)
            .intersection(*block_candidates)
    }
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
struct GroupAvailabilityIndex<Base: SudokuBase> {
    row: Coordinate<Base>,
    column: Coordinate<Base>,
    block: Coordinate<Base>,
}

impl<Base: SudokuBase> GroupAvailabilityIndex<Base> {
    fn into_usize_tuple(self) -> (usize, usize, usize) {
        let GroupAvailabilityIndex { row, column, block } = self;
        let row = usize::from(row.get());
        let column = usize::from(column.get());
        let block = usize::from(block.get());
        (row, column, block)
    }
}

impl<Base: SudokuBase> From<GroupAvailabilityIndex<Base>> for Position<Base> {
    fn from(index: GroupAvailabilityIndex<Base>) -> Self {
        (index.row, index.column).into()
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::*;
    use crate::solver::test_util::{assert_solve_result, assert_solver_solutions_base_2};

    use super::*;

    #[test]
    fn test_iter_all_solutions() {
        let grid = Grid::<Base2>::new();
        let solver = Solver::new(&grid);

        assert_solver_solutions_base_2(solver);
    }

    #[test]
    fn test_base_2() {
        let grids = crate::samples::base_2();

        for grid in grids {
            let mut solver = Solver::new(&grid);

            let solve_result = solver.try_solve();

            assert_solve_result(solve_result);
        }
    }

    #[test]
    fn test_base_3() {
        let grids = crate::samples::base_3();

        for grid in grids {
            let mut solver = Solver::new(&grid);

            let solve_result = solver.try_solve();

            assert_solve_result(solve_result);
        }
    }

    #[test]
    fn test_move_best_choice_to_front() {
        let mut grid = crate::samples::base_2()[1].clone();
        grid.set_all_direct_candidates();
        let mut solver = Solver::new(&grid);
        let mut expected_choice_indices = vec![
            (0, 3, 1),
            (0, 1, 0),
            (1, 0, 0),
            (1, 1, 0),
            (1, 2, 1),
            (1, 3, 1),
            (2, 0, 2),
            (2, 1, 2),
            (2, 2, 3),
            (2, 3, 3),
            (3, 0, 2),
            (3, 2, 3),
        ]
        .into_iter()
        .map(|(row, column, block)| GroupAvailabilityIndex::<Base2> {
            row: Coordinate::try_from(row).unwrap(),
            column: Coordinate::try_from(column).unwrap(),
            block: Coordinate::try_from(block).unwrap(),
        })
        .collect::<Vec<_>>();
        assert_eq!(solver.availability_indices, expected_choice_indices);

        solver.move_best_choice_to_front(4);
        expected_choice_indices.swap(4, 11);
        assert_eq!(solver.availability_indices, expected_choice_indices);
    }

    #[test]
    fn test_solved() {
        let grid = crate::samples::base_2_solved();

        let mut solver = Solver::new(&grid);
        let solve_result = solver.try_solve();
        assert_solve_result(solve_result);

        assert!(solver.try_solve().is_none());
    }
}
