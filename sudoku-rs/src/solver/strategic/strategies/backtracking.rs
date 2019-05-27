use crate::cell::SudokuCell;
use crate::position::Position;
use crate::solver::backtracking::BacktrackingSolver;
use crate::Sudoku;

use super::Strategy;

pub(in crate::solver::strategic) struct Backtracking;

impl<Cell: SudokuCell> Strategy<Cell> for Backtracking {
    fn name(&self) -> &'static str {
        "Backtracking"
    }

    fn execute(&self, sudoku: &mut Sudoku<Cell>) -> Vec<Position> {
        let mut solver = BacktrackingSolver::new(sudoku);

        if let Some(_) = solver.next() {
            solver.empty_positions().to_vec()
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use crate::cell::Cell;
    use crate::samples;

    use super::*;

    #[test]
    fn test_backtracking() {
        let mut sudoku: Sudoku<Cell<NonZeroUsize>> = samples::base_3().first().unwrap().clone();

        let candidates_len = sudoku.grid().all_candidates_positions().len();

        sudoku.fix_all_values();

        let modified_positions = Backtracking.execute(&mut sudoku);

        assert!(sudoku.is_solved());

        assert_eq!(modified_positions.len(), modified_positions.len());
    }
}
