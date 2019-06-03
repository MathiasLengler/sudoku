use strategies::Strategy;

use crate::cell::SudokuCell;
use crate::Sudoku;

mod strategies;

// TODO: bench
//  Solver
//  strategies

pub struct Solver<'s, Cell: SudokuCell> {
    sudoku: &'s mut Sudoku<Cell>,
    strategies: Vec<Box<dyn Strategy<Cell>>>,
}

impl<'s, Cell: SudokuCell> Solver<'s, Cell> {
    pub fn new(sudoku: &'s mut Sudoku<Cell>) -> Solver<'s, Cell> {
        Self {
            sudoku,
            strategies: strategies::strategies(),
        }
    }

    // TODO: unique solution?
    pub fn try_solve(&mut self) -> bool {
        self.sudoku.fix_all_values();
        self.sudoku.set_all_direct_candidates();

        loop {
            if self.sudoku.is_solved() {
                return true;
            }

            let mut modified = false;

            for strategy in &self.strategies {
                let modified_positions = strategy.execute(&mut self.sudoku);

                if !modified_positions.is_empty() {
                    println!(
                        "{}: {:?}",
                        strategy.name(),
                        modified_positions
                            .into_iter()
                            .map(|pos| pos.to_string())
                            .collect::<Vec<_>>()
                    );

                    println!("{}", self.sudoku);

                    modified = true;

                    break;
                }
            }

            if modified {
                // Continue with strategy execution
            } else {
                // All strategies have failed.
                return false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_2() {
        let mut sudokus = crate::samples::base_2();

        for (sudoku_index, mut sudoku) in sudokus.drain(1..2).enumerate() {
            println!("#{}:\n{}", sudoku_index, sudoku);

            let mut solver = Solver::new(&mut sudoku);

            assert!(solver.try_solve());

            assert!(sudoku.is_solved());
        }
    }

    #[test]
    fn test_base_3() {
        let sudokus = crate::samples::base_3();

        for (sudoku_index, mut sudoku) in sudokus.into_iter().enumerate() {
            println!("#{}:\n{}", sudoku_index, sudoku);

            let mut solver = Solver::new(&mut sudoku);

            assert!(solver.try_solve());

            assert!(sudoku.is_solved());
        }
    }

    #[test]
    fn test_critical() {
        let mut sudoku = crate::samples::critical(2);

        let mut solver = Solver::new(&mut sudoku);

        assert!(solver.try_solve());

        assert!(sudoku.is_solved());
    }
}
