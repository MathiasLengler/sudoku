
use crate::cell::SudokuCell;
use crate::position::Position;
use crate::Sudoku;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Choice {
    pos: Position,
    value: usize,
}

pub struct BacktrackingSolver<Cell: SudokuCell> {
    choices: Vec<Choice>,
    sudoku: Sudoku<Cell>,
}

impl<Cell: SudokuCell> BacktrackingSolver<Cell> {
    pub fn new(sudoku: Sudoku<Cell>) -> BacktrackingSolver<Cell> {
        BacktrackingSolver {
            choices: vec![],
            sudoku,
        }
    }

    pub fn solve(&mut self) -> bool {
        let empty_positions = self.sudoku.all_empty_positions();
        let value_range = self.sudoku.value_range();

        let first_pos = match empty_positions.first() {
            Some(first_pos) => first_pos,
            // No empty positions
            None => return !self.sudoku.has_conflict(),
        };

        self.choices.push(Choice {
            pos: *first_pos,
            value: *value_range.start(),
        });

        loop {
            match self.choices.last() {
                Some(choice) =>
                    self.sudoku.set(choice.pos, Cell::new_with_value(choice.value)),
                // Backtracked on first position, unsolvable
                None => return false,
            }

            if self.sudoku.has_conflict() {
                // Try next value or backtrack
                let mut choice = self.choices.pop().unwrap();

                if choice.value != *value_range.end() {

                    // Try next value
                    choice.value += 1;

                    self.choices.push(choice);
                } else {
                    // Backtrack, do not add
                }
            } else {
                // Go to next cell

                let next_position = match empty_positions.get(self.choices.len()) {
                    Some(next_position) => next_position,
                    // Solved
                    None => return true,
                };

                self.choices.push(Choice {
                    pos: *next_position,
                    value: *value_range.start(),
                })
            }
        }
    }
}
