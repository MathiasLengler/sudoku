/// Fork of: https://github.com/t-dillon/tdoku/blob/master/src/solver_basic.cc
use crate::base::SudokuBase;
use crate::cell::candidates_cell::CandidatesCell;
use crate::cell::compact::candidates::Candidates;
use crate::cell::compact::value::Value;
use crate::grid::Grid;
use crate::position::Position;
use std::collections::VecDeque;

// TODO: implement MoveBestTodoToFront heuristic optimization

#[derive(Debug, Clone)]
pub struct Solver<'a, Base: SudokuBase> {
    /// Grid to be solved
    grid: &'a Grid<Base>,
    /// Cached remaining candidates for each group.
    group_availability: GroupAvailability<Base>,
    /// Stack of indices to non-value cells to be solved.
    choice_indices: VecDeque<GroupAvailabilityIndex>,
    /// Stack of the currently selected value for choice_indices.
    choices: Vec<Value<Base>>,
}

impl<'a, Base: SudokuBase> Solver<'a, Base> {
    pub fn new(grid: &'a Grid<Base>) -> Self {
        let mut this = Self {
            grid,
            group_availability: GroupAvailability::all(),
            choice_indices: Default::default(),
            choices: Default::default(),
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
                    self.choice_indices.push_back(index);
                }
            }
        }
    }

    // TODO: make resumable; seems to be a tradeoff between:
    //  - fast solution counting while return last solution, if any
    //  - more state tracking while returning every solution
    pub fn try_solve(&mut self) -> Option<Grid<Base>> {
        // dbg!(&self.choice_indices);

        if let Some(choice_index) = self.choice_indices.pop_front() {
            let candidates = self.group_availability.intersection(choice_index);

            // println!("{choice_index:?}: {candidates}");

            for candidate in candidates.iter() {
                // Clear candidate availability
                self.group_availability.mutate(choice_index, |cell| {
                    cell.set_candidate(candidate, false);
                });

                self.choices.push(candidate);

                if self.choice_indices.is_empty() {
                    // Current choices are a solution
                    let mut solution_grid = self.grid.clone();

                    for (choice_pos, choice) in solution_grid
                        .all_candidates_positions()
                        .into_iter()
                        .zip(self.choices.iter().copied())
                    {
                        solution_grid.get_mut(choice_pos).set_value(choice)
                    }

                    return Some(solution_grid);
                } else {
                    // Recursively solve remaining cells, returning the first solution, if any.
                    if let Some(solution) = self.try_solve() {
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

            self.choice_indices.push_front(choice_index);

            None
        } else {
            todo!()
        }
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

    fn get(&self, index: GroupAvailabilityIndex, mut f: impl FnMut(&CandidatesCell<Base>)) {
        let GroupAvailabilityIndex { row, column, block } = index;
        f(&self.rows.as_ref()[usize::from(row)]);
        f(&self.columns.as_ref()[usize::from(column)]);
        f(&self.blocks.as_ref()[usize::from(block)]);
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
            .candidates()
            .intersection(&self.columns.as_ref()[usize::from(column)].candidates())
            .intersection(&self.blocks.as_ref()[usize::from(block)].candidates())
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct GroupAvailabilityIndex {
    row: u8,
    column: u8,
    block: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::consts::*;
    use crate::solver::test_util::assert_solve_result;
    use std::mem::size_of;

    #[test]
    fn test_base_2() {
        let grids = crate::samples::base_2();

        for mut grid in grids.into_iter() {
            let mut solver = Solver::new(&grid);

            let solve_result = solver.try_solve();

            assert_solve_result(solve_result);
        }
    }

    #[test]
    fn test_base_3() {
        let grids = crate::samples::base_3();

        for mut grid in grids.into_iter() {
            let mut solver = Solver::new(&mut grid);

            let solve_result = solver.try_solve();

            assert_solve_result(solve_result);
        }
    }

    #[test]
    fn test_size() {
        dbg!(size_of::<Solver::<U2>>());
        dbg!(size_of::<Solver::<U3>>());
        dbg!(size_of::<Solver::<U4>>());
        dbg!(size_of::<Solver::<U5>>());
    }
}
