use std::marker::PhantomData;

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

// TODO: `solver.grade`
//  add "difficulty" score for each strategy
//  sum difficulty for each applied strategy
// Reference: sudokuwiki "Grader"/"Solve path"

#[derive(Debug)]
pub struct Solver<Base: SudokuBase, GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>> {
    grid: GridMut,
    // TODO: generic: AsRef: IntoIterator<DynamicStrategy>
    //  `Generator::try_delete_cell_at_pos` would not need to clone its strategies
    strategies: Vec<DynamicStrategy>,
    _base: PhantomData<Base>,
}

impl<Base: SudokuBase, GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>> Solver<Base, GridMut> {
    pub fn new(grid: GridMut) -> Self {
        Self::new_with_strategies(grid, DynamicStrategy::default_solver_strategies())
    }

    pub fn new_with_strategies(mut grid: GridMut, strategies: Vec<DynamicStrategy>) -> Self {
        grid.as_mut()
            .set_all_direct_candidates_if_all_candidates_are_empty();

        Self {
            grid,
            strategies,
            _base: PhantomData,
        }
    }

    pub fn try_solve(&mut self) -> Result<Option<Grid<Base>>> {
        Ok(loop {
            if self.grid.as_ref().is_solved() {
                break Some(self.grid.as_ref().clone());
            }

            if let Some((_, deductions)) = self.try_strategies()? {
                deductions.apply(self.grid.as_mut())?;
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
            trace!("Executing strategy: {strategy:?}");

            let deductions = Strategy::execute(*strategy, self.grid.as_ref())?;

            if !deductions.is_empty() {
                trace!(
                    "{strategy:?} made progress:\n{}\n{}",
                    deductions,
                    self.grid.as_ref()
                );

                return Ok(Some((*strategy, deductions)));
            }
        }
        Ok(None)
    }

    pub fn into_grid(self) -> GridMut {
        self.grid
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::*;

    use super::*;

    fn assert_solvable<Base: SudokuBase>(mut grid: Grid<Base>) {
        grid.set_all_direct_candidates();
        grid.fix_all_values();

        let mut solver = Solver::new(&mut grid);
        assert!(solver.try_solve().unwrap().is_some());
        assert!(grid.is_solved());
    }

    #[test]
    fn test_base_2() {
        let grids = crate::samples::base_2();

        for grid in grids {
            assert_solvable(grid);
        }
    }

    #[test]
    fn test_base_3() {
        let grids = crate::samples::base_3();

        for grid in grids {
            assert_solvable(grid);
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
