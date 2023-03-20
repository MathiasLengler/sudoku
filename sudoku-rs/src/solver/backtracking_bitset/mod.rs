//! Fork of [tdoku `solver_basic.cc`](https://github.com/t-dillon/tdoku/blob/master/src/solver_basic.cc)

use log::trace;

use crate::base::SudokuBase;
use crate::cell::candidates_cell::CandidatesCell;
use crate::cell::compact::candidates::{Candidates, CandidatesIter};
use crate::cell::compact::value::Value;
use crate::grid::Grid;
use crate::position::DynamicPosition;

// TODO: implement shuffle_candidates

#[derive(Debug, Clone)]
pub struct Solver<'a, Base: SudokuBase> {
    /// Grid to be solved
    grid: &'a Grid<Base>,
    /// Cached remaining candidates for each group.
    availability: GroupAvailability<Base>,
    /// Indices to non-value cells which must be solved.
    availability_indices: Vec<GroupAvailabilityIndex>,
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
        for row in 0..Base::SIDE_LENGTH {
            for column in 0..Base::SIDE_LENGTH {
                let pos = DynamicPosition { row, column };

                let cell_index = pos.cell_index::<Base>();

                let block = Base::cell_index_to_block_index(cell_index);

                let index = GroupAvailabilityIndex { row, column, block };

                if let Some(value) = grid.get(pos).value() {
                    // clue, clear group availability
                    self.availability.reserve(index, value);
                } else {
                    // Non-value cell, add to choices
                    self.availability_indices.push(index);
                }
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
                .get_mut(choice_index.into())
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
    rows: Base::CandidatesCells,
    columns: Base::CandidatesCells,
    blocks: Base::CandidatesCells,
}

impl<Base: SudokuBase> GroupAvailability<Base> {
    fn new() -> Self {
        Self::default()
    }

    fn all() -> Self {
        let mut this = Self::new();

        this.iter_mut()
            .for_each(|cell| *cell = CandidatesCell::with_candidates(Candidates::all()));

        this
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut CandidatesCell<Base>> {
        self.rows
            .as_mut()
            .iter_mut()
            .chain(self.columns.as_mut().iter_mut())
            .chain(self.blocks.as_mut().iter_mut())
    }

    fn reserve(&mut self, index: GroupAvailabilityIndex, candidate: Value<Base>) {
        // Clear candidate availability
        self.mutate(index, |cell| {
            cell.set_candidate(candidate, false);
        });
    }
    fn restore(&mut self, index: GroupAvailabilityIndex, candidate: Value<Base>) {
        // Restore candidate availability
        self.mutate(index, |cell| {
            cell.set_candidate(candidate, true);
        });
    }

    fn mutate(
        &mut self,
        index: GroupAvailabilityIndex,
        mut f: impl FnMut(&mut CandidatesCell<Base>),
    ) {
        let GroupAvailabilityIndex { row, column, block } = index;
        f(&mut self.rows.as_mut()[usize::from(row)]);
        f(&mut self.columns.as_mut()[usize::from(column)]);
        f(&mut self.blocks.as_mut()[usize::from(block)]);
    }

    fn intersection(&self, index: GroupAvailabilityIndex) -> Candidates<Base> {
        let GroupAvailabilityIndex { row, column, block } = index;
        self.rows.as_ref()[usize::from(row)]
            .candidates
            .intersection(&self.columns.as_ref()[usize::from(column)].candidates)
            .intersection(&self.blocks.as_ref()[usize::from(block)].candidates)
    }
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
struct GroupAvailabilityIndex {
    row: u8,
    column: u8,
    block: u8,
}

impl From<&GroupAvailabilityIndex> for DynamicPosition {
    fn from(index: &GroupAvailabilityIndex) -> Self {
        DynamicPosition {
            row: index.row,
            column: index.column,
        }
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

        for mut grid in grids {
            let mut solver = Solver::new(&mut grid);

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
            GroupAvailabilityIndex {
                row: 0,
                column: 3,
                block: 1,
            },
            GroupAvailabilityIndex {
                row: 0,
                column: 1,
                block: 0,
            },
            GroupAvailabilityIndex {
                row: 1,
                column: 0,
                block: 0,
            },
            GroupAvailabilityIndex {
                row: 1,
                column: 1,
                block: 0,
            },
            GroupAvailabilityIndex {
                row: 1,
                column: 2,
                block: 1,
            },
            GroupAvailabilityIndex {
                row: 1,
                column: 3,
                block: 1,
            },
            GroupAvailabilityIndex {
                row: 2,
                column: 0,
                block: 2,
            },
            GroupAvailabilityIndex {
                row: 2,
                column: 1,
                block: 2,
            },
            GroupAvailabilityIndex {
                row: 2,
                column: 2,
                block: 3,
            },
            GroupAvailabilityIndex {
                row: 2,
                column: 3,
                block: 3,
            },
            GroupAvailabilityIndex {
                row: 3,
                column: 0,
                block: 2,
            },
            GroupAvailabilityIndex {
                row: 3,
                column: 2,
                block: 3,
            },
        ];
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
