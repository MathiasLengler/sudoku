use crate::base::SudokuBase;
use crate::error::Result;
use crate::grid::Grid;
use crate::position::Position;
use crate::solver::backtracking::Solver;
use crate::solver::strategic::deduction::{Deduction, Deductions, TryIntoDeductions};

use super::Strategy;

#[derive(Debug)]
pub struct Backtracking;

impl<Base: SudokuBase> Strategy<Base> for Backtracking {
    fn execute(&self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        // todo!("port Backtracking Strategy to StrategyDeduction")

        let mut solver_grid = grid.clone();

        let mut solver = Solver::new(&mut solver_grid);

        if solver.next().is_some() {
            TryIntoDeductions(solver.into_empty_positions().into_iter().map(|pos| {
                Deduction::with_value(
                    pos,
                    grid.get(pos).candidates().unwrap(),
                    solver_grid.get(pos).value().unwrap(),
                )
            }))
            .try_into()
        } else {
            Ok(Deductions::default())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::samples;

    use super::*;

    #[test]
    fn test_backtracking() {
        let mut grid = samples::base_3().first().unwrap().clone();
        grid.fix_all_values();
        grid.set_all_direct_candidates();

        // TODO: assert deductions
        let deductions = Backtracking.execute(&grid).unwrap();

        // TODO: fix panic
        grid.apply_deductions(&deductions);

        assert!(grid.is_solved());
    }
}
