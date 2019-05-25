use std::num::NonZeroUsize;

use crate::cell::SudokuCell;
use crate::position::Position;
use crate::solver::backtracking::choice::Choice;
use crate::Sudoku;

mod choice;

// TODO: how to externally drive and visualize solver (steps)
//  make step into an iterator over step results
//  common Solver trait?

pub struct BacktrackingSolver<Cell: SudokuCell> {
    sudoku: Sudoku<Cell>,
    /// Cached
    empty_positions: Vec<Position>,
    /// Choices stack
    choices: Vec<Choice>,
    /// Step limit checking
    step_count: usize,
    /// Settings
    settings: BacktrackingSolverSettings,
}

#[derive(Debug, Default)]
pub struct BacktrackingSolverSettings {
    pub step_limit: Option<NonZeroUsize>,
    pub shuffle_candidates: bool,
}

#[derive(Debug)]
enum StepResult<Cell: SudokuCell> {
    Solution(Box<Sudoku<Cell>>),
    /// Sudoku was filled completely
    Filled,
    /// Went through the whole solution space and marked all potential solutions on the way
    Finished,
    Backtrack,
    NextCell,
}

impl<Cell: SudokuCell> BacktrackingSolver<Cell> {
    pub fn new(sudoku: Sudoku<Cell>) -> BacktrackingSolver<Cell> {
        Self::new_with_settings(sudoku, Default::default())
    }

    pub fn new_with_settings(
        sudoku: Sudoku<Cell>,
        settings: BacktrackingSolverSettings,
    ) -> BacktrackingSolver<Cell> {
        let empty_positions = sudoku.grid().all_candidates_positions();

        let mut solver = BacktrackingSolver {
            sudoku,
            choices: Vec::with_capacity(empty_positions.len()),
            empty_positions,
            step_count: 0,
            settings,
        };

        solver.init();

        solver
    }

    pub fn into_sudoku(self) -> Sudoku<Cell> {
        self.sudoku
    }

    fn init(&mut self) {
        if let Some(first_pos) = self.empty_positions.first() {
            self.choices.push(Choice::new(
                &self.sudoku,
                *first_pos,
                self.settings.shuffle_candidates,
            ))
        };
    }

    fn try_solve(&mut self) -> Option<Sudoku<Cell>> {
        loop {
            let step_result = self.step();

            self.step_count += 1;

            self.debug_print(&step_result);

            match step_result {
                StepResult::Solution(sudoku) => return Some(*sudoku),
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

    fn step(&mut self) -> StepResult<Cell> {
        let choices_len = self.choices.len();

        match self.choices.last_mut() {
            Some(choice) => {
                self.sudoku.set_value(choice.position(), choice.selection());

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
                                &self.sudoku,
                                *next_position,
                                self.settings.shuffle_candidates,
                            ));

                            StepResult::NextCell
                        }
                        None => {
                            choice.set_next();

                            StepResult::Solution(Box::new(self.sudoku.clone()))
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

    #[cfg(not(feature = "debug_print"))]
    fn debug_print(&self, _step_result: &StepResult<Cell>) {
        // Do nothing
    }

    #[cfg(feature = "debug_print")]
    fn debug_print(&self, step_result: &StepResult<Cell>) {
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
                self.sudoku,
                step_result,
                self.choices.len(),
                self.choices.last()
            ))
            .unwrap();

        ::std::thread::sleep(Duration::from_nanos(1));
    }
}

impl<Cell: SudokuCell> BacktrackingSolver<Cell> {
    pub fn has_unique_solution(sudoku: &Sudoku<Cell>) -> bool {
        let mut solver = Self::new(sudoku.clone());

        assert!(solver.next().is_some());

        solver.next().is_none()
    }
}

impl<Cell: SudokuCell> Iterator for BacktrackingSolver<Cell> {
    type Item = Sudoku<Cell>;

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

    fn assert_solve_result<Cell: SudokuCell>(solve_result: Option<Sudoku<Cell>>) {
        assert!(solve_result.is_some());

        let sudoku = solve_result.unwrap();

        assert!(sudoku.is_solved());
    }

    fn assert_iter(solver: BacktrackingSolver<Cell>) {
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
        let solver = BacktrackingSolver::new(Sudoku::<Cell>::new(2));

        assert_iter(solver);
    }

    #[test]
    fn test_test_iter_all_solutions_shuffle_candidates() {
        let solver = BacktrackingSolver::new_with_settings(
            Sudoku::<Cell>::new(2),
            BacktrackingSolverSettings {
                shuffle_candidates: true,
                step_limit: Default::default(),
            },
        );

        assert_iter(solver);
    }

    #[test]
    fn test_base_2() {
        let sudokus = crate::samples::base_2();

        for (_sudoku_index, sudoku) in sudokus.into_iter().enumerate() {
            let mut solver = BacktrackingSolver::new(sudoku);

            let solve_result = solver.try_solve();

            assert_solve_result(solve_result);
        }
    }

    #[test]
    fn test_base_3() {
        let sudokus = crate::samples::base_3();

        for (_sudoku_index, sudoku) in sudokus.into_iter().enumerate() {
            let mut solver = BacktrackingSolver::new(sudoku);

            let solve_result = solver.try_solve();

            assert_solve_result(solve_result);
        }
    }
}
