use strategies::Strategy;

use crate::cell::SudokuBase;
use crate::grid::Grid;
use crate::position::Position;

pub mod strategies;

#[derive(Debug)]
pub struct Solver<'s, Base: SudokuBase> {
    grid: &'s mut Grid<Base>,
    strategies: Vec<Box<dyn Strategy<Base>>>,
}

impl<'s, Base: SudokuBase> Solver<'s, Base> {
    pub fn new(grid: &'s mut Grid<Base>) -> Solver<'s, Base> {
        Self::new_with_strategies(grid, strategies::all_strategies())
    }

    pub fn new_with_strategies(
        grid: &'s mut Grid<Base>,
        strategies: Vec<Box<dyn Strategy<Base>>>,
    ) -> Solver<'s, Base> {
        Self { grid, strategies }
    }

    // TODO: unique solution?
    pub fn try_solve(&mut self) -> bool {
        loop {
            if self.grid.is_solved() {
                return true;
            }

            if let Some(_modified_positions) = self.try_strategies() {
                // Continue with strategy execution
            } else {
                // All strategies have failed.
                return false;
            }
        }
    }

    /// Tries strategies until a strategy is able to modify the grid.
    pub fn try_strategies(&mut self) -> Option<Vec<Position>> {
        for strategy in self.strategies.iter() {
            let modified_positions = strategy.execute(&mut self.grid);

            if !modified_positions.is_empty() {
                println!(
                    "{:?}: {:?}\n{}",
                    strategy,
                    modified_positions
                        .iter()
                        .map(|pos| pos.to_string())
                        .collect::<Vec<_>>(),
                    self.grid
                );

                return Some(modified_positions);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_2() {
        let mut grids = crate::samples::base_2();

        for (grid_index, mut grid) in grids.drain(..).enumerate() {
            grid.set_all_direct_candidates();
            grid.fix_all_values();

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
            grid.set_all_direct_candidates();
            grid.fix_all_values();

            println!("#{}:\n{}", grid_index, grid);

            let mut solver = Solver::new(&mut grid);

            assert!(solver.try_solve());

            assert!(grid.is_solved());
        }
    }

    #[test]
    fn test_minimal() {
        let mut grid = crate::samples::minimal(2);

        grid.set_all_direct_candidates();
        grid.fix_all_values();

        let mut solver = Solver::new(&mut grid);

        assert!(solver.try_solve());

        assert!(grid.is_solved());
    }
}
