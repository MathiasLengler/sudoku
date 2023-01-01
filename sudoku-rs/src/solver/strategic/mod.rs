use log::trace;

use deduction::Deductions;
use strategies::Strategy;

use crate::base::SudokuBase;
use crate::error::Result;
use crate::grid::Grid;
use crate::solver::strategic::strategies::DynamicStrategy;

pub mod strategies;
// API
pub mod deduction;

// TODO: return/persist chain of deductions for complete solve

#[derive(Debug)]
pub struct Solver<'s, Base: SudokuBase> {
    grid: &'s mut Grid<Base>,
    strategies: Vec<DynamicStrategy>,
}

impl<'s, Base: SudokuBase> Solver<'s, Base> {
    pub fn new(grid: &'s mut Grid<Base>) -> Solver<'s, Base> {
        Self::new_with_strategies(grid, DynamicStrategy::all())
    }

    pub fn new_with_strategies(
        grid: &'s mut Grid<Base>,
        strategies: Vec<DynamicStrategy>,
    ) -> Solver<'s, Base> {
        Self { grid, strategies }
    }

    pub fn try_solve(&mut self) -> Option<Result<Grid<Base>>> {
        loop {
            if self.grid.is_solved() {
                return Some(Ok(self.grid.clone()));
            }

            match self.try_strategies() {
                Ok(Some(deductions)) => {
                    deductions.apply(self.grid);
                    // Continue with strategy execution
                }
                Ok(None) => {
                    // All strategies have failed.
                    return None;
                }
                Err(err) => {
                    return Some(Err(err));
                }
            }
        }
    }

    /// Tries strategies until a strategy is able to modify the grid.
    pub fn try_strategies(&mut self) -> Result<Option<Deductions<Base>>> {
        for strategy in &self.strategies {
            let deductions = strategy.execute(&mut self.grid)?;

            if !(deductions.is_empty()) {
                trace!(
                    "{:?}: {:?}\n{}",
                    strategy,
                    deductions
                        .iter()
                        .map(|pos| pos.to_string())
                        .collect::<Vec<_>>(),
                    self.grid
                );

                return Ok(Some(deductions));
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

        assert!(solver.try_solve().unwrap().is_ok());

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

        assert!(solver.try_solve().unwrap().is_ok());

        assert!(grid.is_solved());
    }
}
