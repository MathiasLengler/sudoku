use strategies::Strategy;

use crate::base::SudokuBase;
use crate::grid::Grid;
use crate::position::Position;
use crate::solver::strategic::strategies::deduction::StrategyDeduction;

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

            if let Some((deductions)) = self.try_strategies() {
                self.grid.apply_deductions(&deductions);

                // Continue with strategy execution
            } else {
                // All strategies have failed.
                return false;
            }
        }
    }

    /// Tries strategies until a strategy is able to modify the grid.
    pub fn try_strategies(&mut self) -> Option<Vec<StrategyDeduction<Base>>> {
        self.strategies.iter().find_map(|strategy| {
            let deductions = strategy.execute(&mut self.grid);

            if deductions.is_empty() {
                None
            } else {
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

                return Some(deductions);
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use typenum::consts::*;

    use super::*;

    fn assert_solvable<Base: SudokuBase>(grid: &mut Grid<Base>) {
        grid.set_all_direct_candidates();
        grid.fix_all_values();

        let mut solver = Solver::new(grid);

        assert!(solver.try_solve());

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

        assert!(solver.try_solve());

        assert!(grid.is_solved());
    }
}
