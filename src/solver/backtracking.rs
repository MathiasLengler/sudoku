use std::fmt::{self, Display};
use std::num::NonZeroUsize;
use std::ops::RangeInclusive;

use crate::cell::SudokuCell;
use crate::position::Position;
use crate::Sudoku;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Choice {
    pos: Position,
    value: usize,
}

impl Choice {
    fn set_next(&mut self, value_range: &RangeInclusive<usize>) {
        if self.value != *value_range.end() {
            // Try next value
            self.value += 1;
        } else {
            // Queue deletion of current cell
            self.value = 0;
        }
    }
}

impl Display for Choice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}={}", self.pos, self.value)
    }
}

pub struct BacktrackingSolver<Cell: SudokuCell> {
    sudoku: Sudoku<Cell>,
    choices: Vec<Choice>,

    empty_positions: Vec<Position>,
    value_range: RangeInclusive<usize>,

    step_count: usize,
    step_limit: Option<NonZeroUsize>,
}

// TODO: solutions iterator
impl<Cell: SudokuCell> BacktrackingSolver<Cell> {
    pub fn new(sudoku: Sudoku<Cell>) -> BacktrackingSolver<Cell> {
        Self::new_with_limit(sudoku, 0)
    }

    pub fn new_with_limit(sudoku: Sudoku<Cell>, step_limit: usize) -> BacktrackingSolver<Cell> {
        let empty_positions = sudoku.all_empty_positions();
        let value_range: RangeInclusive<usize> = sudoku.value_range();

        let mut solver = BacktrackingSolver {
            sudoku,
            choices: vec![],
            empty_positions,
            value_range,
            step_count: 0,
            step_limit: NonZeroUsize::new(step_limit),
        };

        solver.init();

        solver
    }

    fn init(&mut self) {
        if let Some(first_pos) = self.empty_positions.first() {
            self.choices.push(Choice {
                pos: *first_pos,
                value: *self.value_range.start(),
            })
        };
    }

    // TODO: refactor return type
    pub fn solve(&mut self) -> bool {
        self.debug_print();

        loop {
            let step_ret = self.step();

            self.step_count += 1;

            self.debug_print();

            if let Some(step_ret) = step_ret {
                return step_ret;
            }

            if let Some(step_limit) = self.step_limit {
                if self.step_count >= step_limit.get() {
                    return false;
                }
            }
        }
    }

    // TODO: refactor return type
    fn step(&mut self) -> Option<bool> {
        match self.choices.last() {
            Some(choice) => {
                self.sudoku.set(choice.pos, Cell::new_with_value(choice.value));

                if choice.value == 0 {
                    // Backtrack
                    println!("Backtrack");

                    self.choices.pop();

                    match self.choices.last_mut() {
                        Some(prev_choice) => {
                            prev_choice.set_next(&self.value_range)
                        }
                        None => {
                            // TODO: return value?
                            // Backtracked on first position
                        }
                    }

                    return None;
                }
            }
            None => {
                // No choices left
                return if self.empty_positions.is_empty() {
                    // TODO: multiple returns?
                    // Sudoku is filled completely
                    Some(!self.sudoku.has_conflict())
                } else {
                    // We went through the whole solution space and marked all potential solutions on the way
                    Some(false)
                }
            }
        }

        if self.sudoku.has_conflict() {
            self.choices.last_mut().unwrap().set_next(&self.value_range);
        } else {
            // Go to next cell
            let next_position = match self.empty_positions.get(self.choices.len()) {
                Some(next_position) => next_position,
                // Solved
                None => return Some(true),
            };

            self.choices.push(Choice {
                pos: *next_position,
                value: *self.value_range.start(),
            })
        }

        None
    }

    fn debug_print(&self) {
        println!(
            "Solver at step {}:\n{}\nChoices = {:?}",
            self.step_count,
            self.sudoku,
            self.choices.iter().rev().map(ToString::to_string).collect::<Vec<_>>()
        );
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
            ]
        ]
            .into_iter()
            .map(TryInto::<Sudoku<OptionCell>>::try_into)
            .collect::<Result<Vec<_>>>()?;

        for (sudoku_index, sudoku) in sudokus.into_iter().enumerate() {
            eprintln!("sudoku_index = {:?}", sudoku_index);

            let mut solver = BacktrackingSolver::new_with_limit(sudoku, 1000);

            let solve_ret = solver.solve();

            assert!(solve_ret);
        }

        Ok(())
    }
}