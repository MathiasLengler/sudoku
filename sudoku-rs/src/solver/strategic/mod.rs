use log::trace;

use strategies::Strategy;

use crate::base::SudokuBase;
use crate::error::Result;
use crate::grid::Grid;
use crate::solver::strategic::deduction::Deductions;
use crate::solver::strategic::strategies::DynamicStrategy;

pub mod deduction;
pub mod strategies;

// TODO: return/persist chain of deductions for complete solve

#[derive(Debug)]
pub struct Solver<'g, Base: SudokuBase> {
    grid: &'g mut Grid<Base>,
    strategies: Vec<DynamicStrategy>,
}

impl<'g, Base: SudokuBase> Solver<'g, Base> {
    pub fn new(grid: &'g mut Grid<Base>) -> Solver<'g, Base> {
        Self::new_with_strategies(grid, DynamicStrategy::all())
    }

    pub fn new_with_strategies(
        grid: &'g mut Grid<Base>,
        strategies: Vec<DynamicStrategy>,
    ) -> Solver<'g, Base> {
        grid.set_all_direct_candidates_if_all_candidates_are_empty();

        Self { grid, strategies }
    }

    pub fn try_solve(&mut self) -> Result<Option<Grid<Base>>> {
        Ok(loop {
            if self.grid.is_solved() {
                break Some(self.grid.clone());
            }

            if let Some((_, deductions)) = self.try_strategies()? {
                deductions.apply(self.grid)?;
                // Continue with strategy execution
            } else {
                // All strategies failed to make progress.
                break None;
            }
        })
    }

    /// Tries executing strategies until one strategy is able to make at least one deduction.
    pub fn try_strategies(&self) -> Result<Option<(DynamicStrategy, Deductions<Base>)>> {
        for strategy in &self.strategies {
            let deductions = Strategy::execute(strategy, self.grid)?;

            if !(deductions.is_empty()) {
                trace!("{strategy:?}:\n{}\n{}", deductions, self.grid);

                return Ok(Some((*strategy, deductions)));
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

        assert!(solver.try_solve().unwrap().is_some());

        assert!(grid.is_solved());
    }

    #[test]
    fn test_base_2() {
        let grids = crate::samples::base_2();

        for mut grid in grids {
            assert_solvable(&mut grid);
        }
    }

    #[test]
    fn test_base_3() {
        let grids = crate::samples::base_3();

        for mut grid in grids {
            assert_solvable(&mut grid);
        }
    }

    #[test]
    fn test_minimal() {
        let mut grid = crate::samples::minimal::<Base2>();

        grid.set_all_direct_candidates();
        grid.fix_all_values();

        let mut solver = Solver::new(&mut grid);

        assert!(solver.try_solve().unwrap().is_some());

        assert!(grid.is_solved());
    }
}
