use std::num::NonZeroUsize;
use std::time::Duration;

use lazy_static::lazy_static;

use crate::cell::SudokuCell;
use crate::position::Position;
use crate::solver::backtracking::choice::Choice;
use crate::Sudoku;

mod choice;

#[derive(Debug)]
enum StepResult<Cell: SudokuCell> {
    Solution(Box<Sudoku<Cell>>),
    // Sudoku was filled completely
    Filled,
    // We went through the whole solution space and marked all potential solutions on the way
    Finished,
    Backtrack,
    NextCandidate,
    NextCell,
}

pub struct BacktrackingSolver<Cell: SudokuCell> {
    sudoku: Sudoku<Cell>,
    choices: Vec<Choice<Cell>>,

    empty_positions: Vec<Position>,

    step_count: usize,
    step_limit: Option<NonZeroUsize>,
    debug_print: bool,
}

impl<Cell: SudokuCell> Iterator for BacktrackingSolver<Cell> {
    type Item = Sudoku<Cell>;

    fn next(&mut self) -> Option<Self::Item> {
        self.try_solve()
    }
}

impl<Cell: SudokuCell> BacktrackingSolver<Cell> {
    pub fn new(sudoku: Sudoku<Cell>) -> BacktrackingSolver<Cell> {
        Self::new_with_limit(sudoku, 0, false)
    }

    pub fn new_with_limit(sudoku: Sudoku<Cell>, step_limit: usize, debug_print: bool) -> BacktrackingSolver<Cell> {
        let empty_positions = sudoku.empty_positions();

        let mut solver = BacktrackingSolver {
            sudoku,
            choices: vec![],
            empty_positions,
            step_count: 0,
            step_limit: NonZeroUsize::new(step_limit),
            debug_print,
        };

        solver.init();

        solver
    }

    fn init(&mut self) {
        if let Some(first_pos) = self.empty_positions.first() {
            self.choices.push(Choice::new(*first_pos, &self.sudoku))
        };
    }

    fn try_solve(&mut self) -> Option<Sudoku<Cell>> {
        loop {
            let step_result = self.step();

            self.step_count += 1;

            self.debug_print(&step_result);

            match step_result {
                StepResult::Solution(sudoku) => return Some(*sudoku),
                StepResult::Filled => return None,
                StepResult::Finished => return None,
                _ => {}
            }

            if let Some(step_limit) = self.step_limit {
                if self.step_count >= step_limit.get() {
                    return None;
                }
            }
        }
    }

    fn step(&mut self) -> StepResult<Cell> {
        match self.choices.last() {
            Some(choice) => {
                self.sudoku.set(choice.position(), choice.selection());

                if choice.is_exhausted() {
                    // Backtrack
                    self.choices.pop();

                    if let Some(prev_choice) = self.choices.last_mut() {
                        prev_choice.set_next()
                    }

                    StepResult::Backtrack
                } else if self.sudoku.has_conflict_at(choice.position()) {
                    self.choices.last_mut().unwrap().set_next();

                    StepResult::NextCandidate
                } else {
                    match self.empty_positions.get(self.choices.len()) {
                        Some(next_position) => {
                            self.choices.push(Choice::new(*next_position, &self.sudoku));

                            StepResult::NextCell
                        }
                        None => StepResult::Solution(Box::new(self.sudoku.clone())),
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

    fn debug_print(&self, step_result: &StepResult<Cell>) {
        use crossterm::Crossterm;

        lazy_static! {
            static ref CROSSTERM: Crossterm = Crossterm::new();
        }

        if self.debug_print {
            CROSSTERM.terminal().clear(crossterm::ClearType::All).unwrap();

            CROSSTERM.terminal().write(format!(
                "Solver at step {}:\n{}\n{:?}\nChoices = {:?}",
                self.step_count,
                self.sudoku,
                step_result,
                self.choices.len(),
            )).unwrap();

            ::std::thread::sleep(Duration::from_nanos(1));
        }
    }
}


#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use crate::cell::OptionCell;
    use crate::error::Result;

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
    // TODO: test empty sudoku and multiple possible solutions
    // TODO: test multiple calls

    fn assert_solve_result<Cell: SudokuCell>(solve_result: Option<Sudoku<Cell>>) {
        assert!(solve_result.is_some());

        let sudoku = solve_result.unwrap();

//        println!("{}", sudoku);

        assert!(!sudoku.has_conflict());

        assert!(sudoku.empty_positions().is_empty());
    }

    #[test]
    fn test_base_2() -> Result<()> {
        let sudokus = vec![
            vec![
                vec![0, 3, 4, 0],
                vec![4, 0, 0, 2],
                vec![1, 0, 0, 3],
                vec![0, 2, 1, 0],
            ],
            vec![
                vec![1, 0, 4, 0],
                vec![0, 0, 0, 0],
                vec![0, 0, 0, 0],
                vec![0, 1, 0, 2],
            ],
            vec![
                vec![0, 0, 1, 0],
                vec![4, 0, 0, 0],
                vec![0, 0, 0, 2],
                vec![0, 3, 0, 0],
            ],
        ]
            .into_iter()
            .map(TryInto::<Sudoku<OptionCell>>::try_into)
            .collect::<Result<Vec<_>>>()?;

        for (sudoku_index, sudoku) in sudokus.into_iter().enumerate() {
            eprintln!("sudoku_index = {:?}", sudoku_index);

            let mut solver = BacktrackingSolver::new_with_limit(sudoku, 1000, false);

            let solve_result = solver.try_solve();

            assert_solve_result(solve_result);
        }

        Ok(())
    }

    #[test]
    fn test_base_3() -> Result<()> {
        let sudokus = vec![
            // 11 Star difficulty
            vec![
                vec![8, 0, 0, 0, 0, 0, 0, 0, 0],
                vec![0, 0, 3, 6, 0, 0, 0, 0, 0],
                vec![0, 7, 0, 0, 9, 0, 2, 0, 0],
                vec![0, 5, 0, 0, 0, 7, 0, 0, 0],
                vec![0, 0, 0, 0, 4, 5, 7, 0, 0],
                vec![0, 0, 0, 1, 0, 0, 0, 3, 0],
                vec![0, 0, 1, 0, 0, 0, 0, 6, 8],
                vec![0, 0, 8, 5, 0, 0, 0, 1, 0],
                vec![0, 9, 0, 0, 0, 0, 4, 0, 0],
            ]
        ]
            .into_iter()
            .map(TryInto::<Sudoku<OptionCell>>::try_into)
            .collect::<Result<Vec<_>>>()?;

        for (sudoku_index, sudoku) in sudokus.into_iter().enumerate() {
            eprintln!("sudoku_index = {:?}", sudoku_index);

            let mut solver = BacktrackingSolver::new(sudoku);

            let solve_result = solver.try_solve();

            assert_solve_result(solve_result);
        }

        Ok(())
    }
}