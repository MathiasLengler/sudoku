use std::num::NonZeroUsize;

use crate::cell::SudokuCell;
use crate::grid::Grid;
use crate::position::Position;
use crate::solver::backtracking::choice::Choice;

mod choice;

// TODO: how to externally drive and visualize solver (steps)
//  make step into an iterator over step results

#[derive(Debug)]
pub struct Solver<'s, Cell: SudokuCell> {
    grid: &'s mut Grid<Cell>,
    /// Cached
    empty_positions: Vec<Position>,
    /// Choices stack
    choices: Vec<Choice>,
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

impl<'s, Cell: SudokuCell> Solver<'s, Cell> {
    pub fn new(grid: &'s mut Grid<Cell>) -> Solver<'s, Cell> {
        Self::new_with_settings(grid, Default::default())
    }

    pub fn new_with_settings(grid: &'s mut Grid<Cell>, settings: Settings) -> Solver<'s, Cell> {
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
                self.grid.direct_candidates(*first_pos),
                *first_pos,
                self.settings.shuffle_candidates,
            ))
        };
    }

    fn try_solve(&mut self) -> Option<Grid<Cell>> {
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
                self.grid.set_value(choice.position(), choice.selection());

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
                                self.grid.direct_candidates(*next_position),
                                *next_position,
                                self.settings.shuffle_candidates,
                            ));

                            StepResult::NextCell
                        }
                        None => {
                            choice.set_next();

                            // TODO: move clone to iterator (streaming iterator problem)
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
        use crossterm::Crossterm;
        use lazy_static::lazy_static;
        use std::time::Duration;

        lazy_static! {
            static ref CROSSTERM: Crossterm = Crossterm::new();
        }

        CROSSTERM
            .terminal()
            .clear(crossterm::ClearType::All)
            .unwrap();

        CROSSTERM
            .terminal()
            .write(format!(
                "Solver at step {}:\n{}\nStep result: {:?}\nChoices: {}\nCurrent Choice: {:?}",
                self.step_count,
                self.grid,
                step_result,
                self.choices.len(),
                self.choices.last()
            ))
            .unwrap();

        ::std::thread::sleep(Duration::from_nanos(1));
    }
}

impl<'s, Cell: SudokuCell> Solver<'s, Cell> {
    /// Panics if the grid has no solution
    pub fn has_unique_solution(grid: &Grid<Cell>) -> bool {
        let mut grid = grid.clone();
        let mut solver = Solver::new(&mut grid);

        assert!(solver.next().is_some());

        solver.next().is_none()
    }

    /// Returns the solution to the grid only if it is the only possible solution
    pub fn unique_solution(grid: &Grid<Cell>) -> Option<Grid<Cell>> {
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

impl<'s, Cell: SudokuCell> Iterator for Solver<'s, Cell> {
    type Item = Grid<Cell>;

    fn next(&mut self) -> Option<Self::Item> {
        self.try_solve()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::cell::Cell;

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

    fn assert_solve_result<Cell: SudokuCell>(solve_result: Option<Grid<Cell>>) {
        assert!(solve_result.is_some());

        let sudoku = solve_result.unwrap();

        assert!(sudoku.is_solved());
    }

    fn assert_iter(solver: Solver<Cell>) {
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
        let mut grid = Grid::<Cell>::new(2);
        let solver = Solver::new(&mut grid);

        assert_iter(solver);
    }

    #[test]
    fn test_test_iter_all_solutions_shuffle_candidates() {
        let mut grid = Grid::<Cell>::new(2);
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
