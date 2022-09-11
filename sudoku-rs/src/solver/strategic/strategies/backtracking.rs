use crate::base::SudokuBase;
use crate::error::Result;
use crate::grid::Grid;
use crate::solver::backtracking::Solver;
use crate::solver::strategic::deduction::{Deduction, Deductions, TryIntoDeductions};

use super::Strategy;

#[derive(Debug)]
pub struct Backtracking;

impl<Base: SudokuBase> Strategy<Base> for Backtracking {
    fn execute(&self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
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
    use crate::cell::compact::value::Value;
    use crate::samples;
    use crate::solver::strategic::deduction::IntoDeductions;

    use super::*;

    #[test]
    fn test_backtracking_base_2() {
        let mut grid = samples::base_2().first().unwrap().clone();
        grid.fix_all_values();
        grid.set_all_direct_candidates();

        let deductions = Backtracking.execute(&grid).unwrap();

        assert_eq!(
            deductions,
            IntoDeductions(vec![
                grid.deduction_at((0, 0), Value::try_from(2).unwrap())
                    .unwrap(),
                grid.deduction_at((0, 3), Value::try_from(1).unwrap())
                    .unwrap(),
                grid.deduction_at((1, 1), Value::try_from(1).unwrap())
                    .unwrap(),
                grid.deduction_at((1, 2), Value::try_from(3).unwrap())
                    .unwrap(),
                grid.deduction_at((2, 1), Value::try_from(4).unwrap())
                    .unwrap(),
                grid.deduction_at((2, 2), Value::try_from(2).unwrap())
                    .unwrap(),
                grid.deduction_at((3, 0), Value::try_from(3).unwrap())
                    .unwrap(),
                grid.deduction_at((3, 3), Value::try_from(4).unwrap())
                    .unwrap(),
            ])
            .try_into()
            .unwrap()
        );

        deductions.apply(&mut grid);
        assert!(grid.is_solved());
    }
    #[test]
    fn test_backtracking_base_3() {
        let mut grid = samples::base_3().first().unwrap().clone();
        grid.fix_all_values();
        grid.set_all_direct_candidates();

        let deductions = Backtracking.execute(&grid).unwrap();
        deductions.apply(&mut grid);
        assert!(grid.is_solved());
    }
}
