use crate::base::SudokuBase;
use crate::error::Result;
use crate::grid::Grid;
use crate::solver::strategic::deduction::{Deduction, Deductions, TryIntoDeductions};

use super::Strategy;

#[derive(Debug, Copy, Clone)]
pub struct SingleCandidate;

impl Strategy for SingleCandidate {
    fn execute<Base: SudokuBase>(&self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        TryIntoDeductions(
            grid.all_candidates_positions()
                .into_iter()
                .filter_map(|candidate_pos| {
                    let candidates = grid.get(candidate_pos).candidates().unwrap();

                    if candidates.count() == 1 {
                        let single_candidate = candidates.iter().next().unwrap();
                        Some(Deduction::with_value(
                            candidate_pos,
                            candidates,
                            single_candidate,
                        ))
                    } else {
                        None
                    }
                }),
        )
        .try_into()
    }
}

#[cfg(test)]
mod tests {
    use crate::cell::compact::value::Value;
    use crate::samples;
    use crate::solver::strategic::deduction::IntoDeductions;

    use super::*;

    #[test]
    fn test_single_candidate() {
        let mut grid = samples::base_2().first().unwrap().clone();

        grid.set_all_direct_candidates();
        grid.fix_all_values();

        let deductions = SingleCandidate.execute(&mut grid).unwrap();

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
}
