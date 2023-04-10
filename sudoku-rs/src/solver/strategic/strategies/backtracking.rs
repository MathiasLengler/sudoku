use crate::base::SudokuBase;
use crate::error::Result;
use crate::grid::Grid;
use crate::solver::backtracking_bitset::Solver;
use crate::solver::strategic::deduction::{Action, Deduction, Deductions};

use super::Strategy;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Backtracking;

impl Strategy for Backtracking {
    fn execute<Base: SudokuBase>(&self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        let mut solver = Solver::new(grid);

        if let Some(solved_grid) = solver.next() {
            Ok(grid
                .all_candidates_positions()
                .into_iter()
                .map(|pos| {
                    Deduction::with_action(
                        pos,
                        Action::SetValue(solved_grid.get(pos).value().unwrap()),
                    )
                })
                .collect())
        } else {
            Ok(Deductions::default())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::cell::Value;
    use crate::samples;
    use crate::solver::strategic::strategies::test_util::assert_deductions_with_grid;

    use super::*;

    #[test]
    fn test_backtracking_base_2() {
        let mut grid = samples::base_2().first().unwrap().clone();
        grid.fix_all_values();
        grid.set_all_direct_candidates();

        let deductions = Backtracking.execute(&grid).unwrap();

        let expected_deductions: Deductions<_> = vec![
            ((0, 0), 2),
            ((0, 3), 1),
            ((1, 1), 1),
            ((1, 2), 3),
            ((2, 1), 4),
            ((2, 2), 2),
            ((3, 0), 3),
            ((3, 3), 4),
        ]
        .into_iter()
        .map(|(pos, value)| {
            Deduction::with_action(
                pos.try_into().unwrap(),
                Action::SetValue(Value::try_from(value).unwrap()),
            )
        })
        .collect();

        assert_deductions_with_grid(deductions, expected_deductions, &mut grid);

        assert!(grid.is_solved());
    }
    #[test]
    fn test_backtracking_base_3() {
        let mut grid = samples::base_3().first().unwrap().clone();
        grid.fix_all_values();
        grid.set_all_direct_candidates();

        let deductions = Backtracking.execute(&grid).unwrap();
        deductions.apply(&mut grid).unwrap();
        assert!(grid.is_solved());
    }
}
