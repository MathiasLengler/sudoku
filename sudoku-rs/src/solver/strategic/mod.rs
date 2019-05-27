use strategies::Strategy;

use crate::cell::SudokuCell;
use crate::position::Position;
use crate::Sudoku;

mod strategies;

// TODO: implement strategic solver
//  loop from beginning after each find

// TODO: bench
//  Solver
//  strategies

pub struct Solver<'s, Cell: SudokuCell> {
    sudoku: &'s mut Sudoku<Cell>,
    solved_positions: Vec<Position>,
    strategies: Vec<Box<dyn Strategy<Cell>>>,
}

impl<'s, Cell: SudokuCell> Solver<'s, Cell> {
    pub fn new(sudoku: &'s mut Sudoku<Cell>) -> Solver<'s, Cell> {
        Self {
            sudoku,
            solved_positions: vec![],
            strategies: strategies::strategies(),
        }
    }

    pub fn solve(&mut self) {
        for strategy in &self.strategies {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
