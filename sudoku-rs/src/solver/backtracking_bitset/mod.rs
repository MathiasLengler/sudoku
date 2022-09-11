/// Fork of: https://github.com/t-dillon/tdoku/blob/master/src/solver_basic.cc
use crate::base::{consts::Base3, SudokuBase};
use crate::cell::candidates_cell::CandidatesCell;
use crate::cell::compact::candidates::Candidates;
use crate::cell::compact::value::Value;
use crate::grid::Grid;
use crate::position::Position;

#[derive(Debug, Clone)]
pub struct Solver<'a, Base: SudokuBase> {
    /// Grid to be solved
    grid: &'a Grid<Base>,
    /// Cached remaining candidates for each group.
    group_availability: GroupAvailability<Base>,
    /// Stack of indices to non-value cells to be solved.
    choice_indices: Vec<GroupAvailabilityIndex>,
    /// Stack of the currently selected value for choice_indices.
    choices: Vec<(Value<Base>, Position)>,

    pub guess_count: u64,
}

// TODO: remove asm debug wrappers
pub fn test_asm_move_best_choice_to_front(solver: &mut Solver<'_, Base3>, choice_indices_i: usize) {
    solver.move_best_choice_to_front(choice_indices_i);
}

pub fn test_asm_raw_intersection(a: u16, b: u16, c: u16) -> u16 {
    a & b & c
}

pub fn test_asm_candidates_intersection(
    a: Candidates<Base3>,
    b: Candidates<Base3>,
    c: Candidates<Base3>,
) -> Candidates<Base3> {
    a.intersection(&b).intersection(&c)
}

impl<'a, Base: SudokuBase> Solver<'a, Base> {
    pub fn new(grid: &'a Grid<Base>) -> Self {
        let mut this = Self {
            grid,
            group_availability: GroupAvailability::all(),
            choice_indices: Default::default(),
            choices: Default::default(),
            guess_count: 0,
        };

        this.initialize(grid);

        this
    }

    fn initialize(&mut self, grid: &Grid<Base>) {
        for row in 0..Base::SIDE_LENGTH {
            for column in 0..Base::SIDE_LENGTH {
                let cell_index = u16::from(row) * u16::from(Base::SIDE_LENGTH) + u16::from(column);
                let block = Base::cell_index_to_block_index(cell_index);

                let index = GroupAvailabilityIndex { row, column, block };

                if let Some(value) = grid.get(Position { row, column }).value() {
                    // clue, clear group availability
                    self.group_availability
                        .mutate(index, |cell| cell.set_candidate(value, false))
                } else {
                    // Non-value cell, add to choices
                    self.choice_indices.push(index);
                }
            }
        }
    }

    pub fn move_best_choice_to_front(&mut self, choice_indices_i: usize) {
        use std::mem::swap;

        if let Some((first_index, rest)) = self.choice_indices[choice_indices_i..].split_first_mut()
        {
            let first_count = self.group_availability.intersection(*first_index).count();
            if first_count <= 1 {
                return;
            }

            let mut better_count = first_count;
            let mut better_index = None;

            for next_index in rest {
                if better_count <= 1 {
                    break;
                }
                let next_count = self.group_availability.intersection(*next_index).count();
                if next_count < better_count {
                    better_count = next_count;
                    better_index = Some(next_index);
                }
            }

            if let Some(better_index) = better_index {
                // println!(
                //     "swapping {first_count} @ {first_index:?} with {better_count} @ {better_index:?}"
                // );
                swap(first_index, better_index);
            }
        }
    }

    fn try_solve_index(&mut self, choice_indices_i: usize) -> Option<Grid<Base>> {
        self.move_best_choice_to_front(choice_indices_i);

        let choice_index = self.choice_indices[choice_indices_i];

        let candidates = self.group_availability.intersection(choice_index);

        for candidate in candidates.iter() {
            self.guess_count += 1;

            // Clear candidate availability
            self.group_availability.mutate(choice_index, |cell| {
                cell.set_candidate(candidate, false);
            });

            self.choices.push((
                candidate,
                Position {
                    row: choice_index.row,
                    column: choice_index.column,
                },
            ));

            if choice_indices_i == self.choice_indices.len() - 1 {
                // Current choices are a solution
                let mut solution_grid = self.grid.clone();

                for (value, position) in self.choices.iter() {
                    solution_grid.get_mut(*position).set_value(*value)
                }

                return Some(solution_grid);
            } else {
                // Recursively solve remaining cells, returning the first solution, if any.
                if let Some(solution) = self.try_solve_index(choice_indices_i + 1) {
                    return Some(solution);
                } else {
                    // Backtrack
                }
            }

            self.choices.pop();

            // Restore candidate availability
            self.group_availability.mutate(choice_index, |cell| {
                cell.set_candidate(candidate, true);
            });
        }
        None
    }

    // TODO: make resumable; seems to be a tradeoff between:
    //  - fast solution counting while return last solution, if any
    //  - more state tracking while returning every solution
    pub fn try_solve(&mut self) -> Option<Grid<Base>> {
        self.try_solve_index(0)
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
        Default::default()
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

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
struct GroupAvailabilityIndex {
    row: u8,
    column: u8,
    block: u8,
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use crate::base::consts::*;
    use crate::solver::test_util::assert_solve_result;

    use super::*;

    #[test]
    fn test_base_2() {
        let grids = crate::samples::base_2();

        for grid in grids.into_iter() {
            let mut solver = Solver::new(&grid);

            let solve_result = solver.try_solve();

            dbg!(solver.guess_count);

            assert_solve_result(solve_result);
        }
    }

    #[test]
    fn test_base_3() {
        let grids = crate::samples::base_3();

        for mut grid in grids.into_iter() {
            let mut solver = Solver::new(&mut grid);

            let solve_result = solver.try_solve();

            dbg!(solver.guess_count);

            assert_solve_result(solve_result);
        }
    }

    #[test]
    fn test_move_best_choice_to_front() {
        let mut grid = crate::samples::base_2()[1].clone();
        grid.set_all_direct_candidates();
        println!("{grid}");
        let mut solver = Solver::new(&grid);
        let mut expected_choice_indices = vec![
            GroupAvailabilityIndex {
                row: 0,
                column: 1,
                block: 0,
            },
            GroupAvailabilityIndex {
                row: 0,
                column: 3,
                block: 1,
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
        assert_eq!(solver.choice_indices, expected_choice_indices);

        solver.move_best_choice_to_front(0);
        expected_choice_indices.swap(0, 1);
        assert_eq!(solver.choice_indices, expected_choice_indices);

        solver.move_best_choice_to_front(4);
        expected_choice_indices.swap(4, 11);
        assert_eq!(solver.choice_indices, expected_choice_indices);
    }

    #[ignore]
    #[test]
    fn test_size() {
        dbg!(size_of::<Solver::<'_, U2>>());
        dbg!(size_of::<Solver::<'_, U3>>());
        dbg!(size_of::<Solver::<'_, U4>>());
        dbg!(size_of::<Solver::<'_, U5>>());
    }
}
