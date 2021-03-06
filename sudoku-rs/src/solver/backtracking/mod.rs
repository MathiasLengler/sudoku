use std::num::NonZeroUsize;

use crate::base::SudokuBase;
use crate::grid::Grid;
use crate::position::Position;
use crate::solver::backtracking::choice::Choice;

mod choice;

// TODO: how to externally drive and visualize solver (steps)
//  make step into an iterator over step results

#[derive(Debug)]
pub struct Solver<'s, Base: SudokuBase> {
    grid: &'s mut Grid<Base>,
    /// Cached
    empty_positions: Vec<Position>,
    /// Choices stack
    choices: Vec<Choice<Base>>,
    /// Step limit checking
    step_count: usize,
    /// Settings
    settings: Settings,
}

#[derive(Debug, Default)]
pub struct Settings {
    pub step_limit: Option<NonZeroUsize>,
    pub shuffle_candidates: bool,
}

#[derive(Debug)]
enum StepResult {
    Solution,
    /// Sudoku was filled completely
    Filled,
    /// Went through the whole solution space and marked all potential solutions on the way
    Finished,
    Backtrack,
    NextCell,
}

impl<'s, Base: SudokuBase> Solver<'s, Base> {
    pub fn new(grid: &'s mut Grid<Base>) -> Solver<'s, Base> {
        Self::new_with_settings(grid, Default::default())
    }

    pub fn new_with_settings(grid: &'s mut Grid<Base>, settings: Settings) -> Solver<'s, Base> {
        let empty_positions = grid.all_candidates_positions();

        let mut solver = Solver {
            grid,
            choices: Vec::with_capacity(empty_positions.len()),
            empty_positions,
            step_count: 0,
            settings,
        };

        solver.init();

        solver
    }

    pub fn into_empty_positions(self) -> Vec<Position> {
        self.empty_positions
    }

    fn init(&mut self) {
        if let Some(first_pos) = self.empty_positions.first() {
            self.choices.push(Choice::new(
                self.grid.direct_candidates(*first_pos).to_vec_value(),
                *first_pos,
                self.settings.shuffle_candidates,
            ))
        };
    }

    fn try_solve(&mut self) -> Option<Grid<Base>> {
        loop {
            let step_result = self.step();

            self.step_count += 1;

            #[cfg(feature = "debug_print")]
            self.debug_print(&step_result);

            match step_result {
                StepResult::Solution => return Some(self.grid.clone()),
                StepResult::Filled | StepResult::Finished => return None,
                _ => {}
            }

            if let Some(step_limit) = self.settings.step_limit {
                if self.step_count >= step_limit.get() {
                    return None;
                }
            }
        }
    }

    fn step(&mut self) -> StepResult {
        let choices_len = self.choices.len();

        match self.choices.last_mut() {
            Some(choice) => {
                let cell = self.grid.get_mut(choice.position());

                if let Some(value) = choice.selection() {
                    cell.set_value(value);
                } else {
                    cell.delete();
                }

                if choice.is_exhausted() {
                    // Backtrack
                    self.choices.pop();

                    if let Some(prev_choice) = self.choices.last_mut() {
                        prev_choice.set_next()
                    }

                    StepResult::Backtrack
                } else {
                    match self.empty_positions.get(choices_len) {
                        Some(next_position) => {
                            self.choices.push(Choice::new(
                                self.grid.direct_candidates(*next_position).to_vec_value(),
                                *next_position,
                                self.settings.shuffle_candidates,
                            ));

                            StepResult::NextCell
                        }
                        None => {
                            choice.set_next();

                            StepResult::Solution
                        }
                    }
                }
            }
            None => {
                if self.empty_positions.is_empty() {
                    StepResult::Filled
                } else {
                    StepResult::Finished
                }
            }
        }
    }

    #[cfg(feature = "debug_print")]
    fn debug_print(&self, step_result: &StepResult) {
        use crossterm::{cursor, style::Print, terminal, QueueableCommand};
        use std::io::{prelude::*, stdout};

        let mut stdout = stdout();
        stdout
            .queue(terminal::Clear(terminal::ClearType::All))
            .unwrap();
        stdout.queue(cursor::MoveTo(0, 0)).unwrap();
        stdout
            .queue(Print(format!(
                "Solver at step {}:
{}
Step result: {:?}
Choices: {}
Current Choice: {:?}",
                self.step_count,
                self.grid,
                step_result,
                self.choices.len(),
                self.choices.last()
            )))
            .unwrap();

        stdout.flush().unwrap();
    }
}

impl<'s, Base: SudokuBase> Solver<'s, Base> {
    /// Panics if the grid has no solution
    pub fn has_unique_solution(grid: &Grid<Base>) -> bool {
        let mut grid = grid.clone();
        let mut solver = Solver::new(&mut grid);

        assert!(solver.next().is_some());

        solver.next().is_none()
    }

    /// Returns the solution to the grid only if it is the only possible solution
    pub fn unique_solution(grid: &Grid<Base>) -> Option<Grid<Base>> {
        let mut grid = grid.clone();
        let mut solver = Solver::new(&mut grid);
        let first_solution = solver.next();
        let second_solution = solver.next();

        match (first_solution, second_solution) {
            (Some(solution), None) => Some(solution),
            _ => None,
        }
    }
}

impl<'s, Base: SudokuBase> Iterator for Solver<'s, Base> {
    type Item = Grid<Base>;

    fn next(&mut self) -> Option<Self::Item> {
        self.try_solve()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use typenum::consts::*;

    use super::*;

    // Input space (
    //      [empty, partial, full] sudoku,
    //      [conflict/ no conflict],
    //      [0, 1, n] solutions
    // )

    // TODO: test filled sudoku with conflict
    // TODO: test filled sudoku without conflict
    // TODO: test partial filled sudoku without conflict and no possible solution
    // TODO: test partial filled sudoku without conflict and one possible solution
    // TODO: test partial filled sudoku without conflict and multiple possible solutions
    // TODO: test partial filled sudoku with conflict (implies no solutions)

    fn assert_solve_result<Base: SudokuBase>(solve_result: Option<Grid<Base>>) {
        assert!(solve_result.is_some());

        let sudoku = solve_result.unwrap();

        assert!(sudoku.is_solved());
    }

    fn assert_iter<Base: SudokuBase>(solver: Solver<'_, Base>) {
        const NUMBER_OF_2X2_SOLUTIONS: usize = 288;

        let solutions = solver.collect::<Vec<_>>();

        assert_eq!(NUMBER_OF_2X2_SOLUTIONS, solutions.len());

        solutions
            .iter()
            .for_each(|solution| assert!(solution.is_solved()));

        let unique_solutions = solutions.into_iter().collect::<HashSet<_>>();

        assert_eq!(NUMBER_OF_2X2_SOLUTIONS, unique_solutions.len());
    }

    #[test]
    fn test_iter_all_solutions() {
        let mut grid = Grid::<U2>::new();
        let solver = Solver::new(&mut grid);

        assert_iter(solver);
    }

    #[test]
    fn test_test_iter_all_solutions_shuffle_candidates() {
        let mut grid = Grid::<U2>::new();
        let solver = Solver::new_with_settings(
            &mut grid,
            Settings {
                shuffle_candidates: true,
                step_limit: Default::default(),
            },
        );

        assert_iter(solver);
    }

    #[test]
    fn test_base_2() {
        let sudokus = crate::samples::base_2();

        for (_sudoku_index, mut sudoku) in sudokus.into_iter().enumerate() {
            let mut solver = Solver::new(&mut sudoku);

            let solve_result = solver.try_solve();

            assert_solve_result(solve_result);
        }
    }

    #[test]
    fn test_base_3() {
        let sudokus = crate::samples::base_3();

        for (_sudoku_index, mut sudoku) in sudokus.into_iter().enumerate() {
            let mut solver = Solver::new(&mut sudoku);

            let solve_result = solver.try_solve();

            assert_solve_result(solve_result);
        }
    }
}
