use strategies::Strategy;

use crate::cell::SudokuCell;
use crate::grid::Grid;

mod strategies;

#[derive(Debug)]
pub struct Solver<'s, Cell: SudokuCell> {
    grid: &'s mut Grid<Cell>,
    strategies: Vec<Box<dyn Strategy<Cell>>>,
}

impl<'s, Cell: SudokuCell> Solver<'s, Cell> {
    pub fn new(grid: &'s mut Grid<Cell>) -> Solver<'s, Cell> {
        grid.fix_all_values();
        grid.set_all_direct_candidates();

        Self {
            grid,
            strategies: strategies::strategies(),
        }
    }

    // TODO: unique solution?
    pub fn try_solve(&mut self) -> bool {
        loop {
            if self.grid.is_solved() {
                return true;
            }

            let mut modified = false;

            for strategy in &self.strategies {
                let modified_positions = strategy.execute(&mut self.grid);

                if !modified_positions.is_empty() {
                    //                    println!(
                    //                        "{}: {:?}",
                    //                        strategy.name(),
                    //                        modified_positions
                    //                            .into_iter()
                    //                            .map(|pos| pos.to_string())
                    //                            .collect::<Vec<_>>()
                    //                    );
                    //
                    //                    println!("{}", self.sudoku);

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
        let mut grids = crate::samples::base_2();

        for (grid_index, mut grid) in grids.drain(..).enumerate() {
            println!("#{}:\n{}", grid_index, grid);

            let mut solver = Solver::new(&mut grid);

            assert!(solver.try_solve());

            assert!(grid.is_solved());
        }
    }

    #[test]
    fn test_base_3() {
        let grids = crate::samples::base_3();

        for (grid_index, mut grid) in grids.into_iter().enumerate() {
            println!("#{}:\n{}", grid_index, grid);

            let mut solver = Solver::new(&mut grid);

            assert!(solver.try_solve());

            assert!(grid.is_solved());
        }
    }

    #[test]
    fn test_minimal() {
        let mut grid = crate::samples::minimal(2);

        let mut solver = Solver::new(&mut grid);

        assert!(solver.try_solve());

        assert!(grid.is_solved());
    }
}
