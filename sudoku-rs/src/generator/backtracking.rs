use crate::cell::SudokuCell;
use crate::solver::backtracking::{BacktrackingSolver, BacktrackingSolverSettings};
use crate::Sudoku;

pub struct BacktrackingGenerator<Cell: SudokuCell> {
    solver: BacktrackingSolver<Cell>,
}

impl<Cell: SudokuCell> BacktrackingGenerator<Cell> {
    pub fn new(base: usize) -> Self {
        Self {
            solver: BacktrackingSolver::new_with_settings(
                Sudoku::<Cell>::new(base),
                BacktrackingSolverSettings::new(0, true),
            ),
        }
    }

    pub fn generate(mut self) -> Sudoku<Cell> {
        self.solver.next().unwrap()
    }
}
