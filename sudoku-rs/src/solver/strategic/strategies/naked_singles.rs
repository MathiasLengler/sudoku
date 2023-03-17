use crate::base::SudokuBase;
use crate::error::Result;
use crate::grid::Grid;
use crate::solver::strategic::deduction::{Action, Deduction, Deductions};

use super::Strategy;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct NakedSingles;

impl Strategy for NakedSingles {
    fn execute<Base: SudokuBase>(&self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        Ok(grid
            .all_candidates_positions()
            .into_iter()
            .filter_map(|candidate_pos| {
                let candidates = grid.get(candidate_pos).candidates().unwrap();

                if candidates.count() == 1 {
                    let single_candidate = candidates.iter().next().unwrap();
                    Some(Deduction::with_action(
                        candidate_pos,
                        Action::SetValue {
                            value: single_candidate,
                        },
                    ))
                } else {
                    None
                }
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::cell::compact::value::Value;
    use crate::samples;

    use super::*;

    #[test]
    fn test_single_candidate() {
        let mut grid = samples::base_2().first().unwrap().clone();
        grid.set_all_direct_candidates();
        grid.fix_all_values();

        let deductions = NakedSingles.execute(&mut grid).unwrap();

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
                pos,
                Action::SetValue {
                    value: Value::try_from(value).unwrap(),
                },
            )
        })
        .collect();

        assert_eq!(
            deductions, expected_deductions,
            "{deductions}\n!=\n{expected_deductions}"
        );

        deductions.apply(&mut grid).unwrap();

        assert!(grid.is_solved());
    }
}
