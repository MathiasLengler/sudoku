use std::num::NonZeroUsize;

use crate::base::SudokuBase;
use crate::grid::Grid;
use crate::position::Position;
use crate::solver::backtracking::choice::{CandidatesProcessor, Choice};

mod choice;

// TODO: how to externally drive and visualize solver (steps)
//  make step into an iterator over step results

#[derive(Debug, Copy, Clone, Default)]
pub enum CandidatesVisitOrder {
    #[default]
    Asc,
    Desc,
    Random,
    RandomSeed(u64),
}

#[derive(Debug)]
pub struct Solver<'s, Base: SudokuBase> {
    grid: &'s mut Grid<Base>,
    /// Cached
    empty_positions: Vec<Position<Base>>,
    /// Choices stack
    choices: Vec<Choice<Base>>,
    /// Step limit checking
    step_count: usize,
    /// Settings
    settings: Settings,
    /// Initialized by `settings.candidates_visit_order`
    candidates_processor: CandidatesProcessor,
}

#[derive(Debug, Default)]
pub struct Settings {
    pub step_limit: Option<NonZeroUsize>,
    pub candidates_visit_order: CandidatesVisitOrder,
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
        Self::new_with_settings(grid, Settings::default())
    }

    pub fn new_with_settings(grid: &'s mut Grid<Base>, settings: Settings) -> Solver<'s, Base> {
        let empty_positions = grid.all_candidates_positions();

        let mut solver = Solver {
            grid,
            choices: Vec::with_capacity(empty_positions.len()),
            empty_positions,
            step_count: 0,
            candidates_processor: settings.candidates_visit_order.into(),
            settings,
        };

        solver.init();

        solver
    }

    fn init(&mut self) {
        if let Some(first_pos) = self.empty_positions.first().copied() {
            self.push_choice(first_pos);
        };
    }

    fn push_choice(&mut self, pos: Position<Base>) {
        self.choices.push(Choice::new(
            self.grid.direct_candidates(pos).to_vec_value(),
            &mut self.candidates_processor,
        ));
    }

    fn try_solve(&mut self) -> Option<Grid<Base>> {
        loop {
            let step_result = self.step();

            self.step_count += 1;

            #[cfg(feature = "solver_debug_print")]
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
                let cell = self.grid.get_mut(self.empty_positions[choices_len - 1]);

                if let Some(value) = choice.selection() {
                    cell.set_value(value);
                } else {
                    cell.delete();
                }

                if choice.is_exhausted() {
                    // Backtrack
                    self.choices.pop();

                    if let Some(prev_choice) = self.choices.last_mut() {
                        prev_choice.set_next();
                    }

                    StepResult::Backtrack
                } else if let Some(next_position) = self.empty_positions.get(choices_len).copied() {
                    self.push_choice(next_position);

                    StepResult::NextCell
                } else {
                    choice.set_next();

                    StepResult::Solution
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

    #[cfg(feature = "solver_debug_print")]
    fn debug_print(&self, step_result: &StepResult) {
        use crossterm::{cursor, style::Print, terminal, QueueableCommand};
        use std::io::{prelude::*, stdout};
        use std::time::Duration;

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

        std::thread::sleep(Duration::from_millis(50));
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
    use crate::base::consts::*;
    use crate::solver::test_util::{assert_solve_result, assert_solver_solutions_base_2};

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

    #[test]
    fn test_iter_all_solutions() {
        let mut grid = Grid::<Base2>::new();
        let solver = Solver::new(&mut grid);

        assert_solver_solutions_base_2(solver);
    }

    #[test]
    fn test_test_iter_all_solutions_shuffle_candidates() {
        let mut grid = Grid::<Base2>::new();
        let solver = Solver::new_with_settings(
            &mut grid,
            Settings {
                candidates_visit_order: CandidatesVisitOrder::Random,
                step_limit: None,
            },
        );

        assert_solver_solutions_base_2(solver);
    }

    #[test]
    fn test_base_2() {
        let grids = crate::samples::base_2();

        for mut grid in grids {
            let mut solver = Solver::new(&mut grid);

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
}
