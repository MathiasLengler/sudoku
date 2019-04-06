use crate::cell::SudokuCell;
use crate::position::Position;
use crate::Sudoku;

pub struct Choice {
    pos: Position,
    value: usize,
}

pub struct Solver<Cell: SudokuCell> {
    choices: Vec<Choice>,
    sudoku: Sudoku<Cell>,
}

impl<Cell: SudokuCell> Solver<Cell> {
    pub fn new(sudoku: Sudoku<Cell>) -> Solver<Cell> {
        Solver {
            choices: vec![],
            sudoku,
        }
    }

    pub fn solve(&mut self) {
        let empty_positions = self.sudoku.all_empty_positions();


        loop {



        }
    }
}
