use crate::base::SudokuBase;
use crate::error::Result;
use crate::grid::Grid;
use crate::solver::introspective;
use crate::solver::strategic::deduction::{Action, Deduction, Deductions};
use crate::solver::strategic::strategies::{Strategy, StrategyScore};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BruteForce;

impl Strategy for BruteForce {
    fn name(self) -> &'static str {
        "BruteForce"
    }
    fn score(self) -> StrategyScore {
        1_000_000
    }
    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        let mut solver = introspective::Solver::new(grid);

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
    use crate::samples;
    use crate::solver::strategic::strategies::test_util::assert_deductions_with_grid;
    use crate::{cell::Value, solver::strategic::strategies::test_util::strategy_snapshot_tests};

    use super::*;

    #[test]
    fn test_base_2() {
        let mut grid = samples::base_2().first().unwrap().clone();
        grid.fix_all_values();
        grid.set_all_direct_candidates();

        let deductions = BruteForce.execute(&grid).unwrap();

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

        assert_deductions_with_grid(&deductions, &expected_deductions, &mut grid);

        assert!(grid.is_solved());
    }

    strategy_snapshot_tests!(BruteForce, |grid| {
        assert!(grid.is_solved());
    });
}
