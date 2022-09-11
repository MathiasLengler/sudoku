use strategies::Strategy;

use crate::base::SudokuBase;
use crate::error::Result;
use crate::grid::Grid;
use deduction::Deductions;

pub mod strategies;
// API
pub mod deduction;

// TODO: return/persist chain of deductions for complete solve

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
    pub fn try_solve(&mut self) -> Result<bool> {
        loop {
            if self.grid.is_solved() {
                return Ok(true);
            }

            if let Some(deductions) = self.try_strategies()? {
                deductions.apply(self.grid);
                // Continue with strategy execution
            } else {
                // All strategies have failed.
                return Ok(false);
            }
        }
    }

    /// Tries strategies until a strategy is able to modify the grid.
    pub fn try_strategies(&mut self) -> Result<Option<Deductions<Base>>> {
        for strategy in &self.strategies {
            let deductions = strategy.execute(&mut self.grid)?;

            if !(deductions.is_empty()) {
                #[cfg(feature = "debug_print")]
                println!(
                    "{:?}: {:?}\n{}",
                    strategy,
                    deductions
                        .iter()
                        .map(|pos| pos.to_string())
                        .collect::<Vec<_>>(),
                    self.grid
                );

                return Ok(Some(deductions));
            } else {
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::*;

    use super::*;

    fn assert_solvable<Base: SudokuBase>(grid: &mut Grid<Base>) {
        grid.set_all_direct_candidates();
        grid.fix_all_values();

        let mut solver = Solver::new(grid);

        assert!(solver.try_solve().unwrap());

        assert!(grid.is_solved());
    }

    #[test]
    fn test_base_2() {
        let grids = crate::samples::base_2();

        for mut grid in grids.into_iter() {
            assert_solvable(&mut grid);
        }
    }

    #[test]
    fn test_base_3() {
        let grids = crate::samples::base_3();

        for mut grid in grids.into_iter() {
            assert_solvable(&mut grid);
        }
    }

    #[test]
    fn test_minimal() {
        let mut grid = crate::samples::minimal::<U2>();

        grid.set_all_direct_candidates();
        grid.fix_all_values();

        let mut solver = Solver::new(&mut grid);

        assert!(solver.try_solve().unwrap());

        assert!(grid.is_solved());
    }
}
