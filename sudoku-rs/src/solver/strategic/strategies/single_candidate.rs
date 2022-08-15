use crate::base::SudokuBase;
use crate::grid::Grid;
use crate::position::Position;
use crate::solver::strategic::strategies::deduction::StrategyDeduction;

use super::Strategy;

#[derive(Debug)]
pub struct SingleCandidate;

impl<Base: SudokuBase> Strategy<Base> for SingleCandidate {
    fn execute(&self, grid: &Grid<Base>) -> Vec<StrategyDeduction<Base>> {
        grid.all_candidates_positions()
            .into_iter()
            .filter_map(|candidate_pos| {
                let candidates = grid.get(candidate_pos).candidates().unwrap().to_vec_value();

                if candidates.len() == 1 {
                    let single_candidate = candidates[0];
                    Some(StrategyDeduction::Value {
                        pos: candidate_pos,
                        value: single_candidate,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::samples;

    use super::*;

    #[test]
    fn test_single_candidate() {
        let mut grid = samples::base_2().first().unwrap().clone();

        grid.set_all_direct_candidates();
        grid.fix_all_values();

        println!("{grid}");

        let mut deductions = SingleCandidate.execute(&mut grid);
        deductions.sort();

        assert_eq!(
            deductions,
            vec![
                StrategyDeduction::Value {
                    pos: Position { row: 0, column: 0 },
                    value: 2.try_into().unwrap()
                },
                StrategyDeduction::Value {
                    pos: Position { row: 0, column: 3 },
                    value: 1.try_into().unwrap()
                },
                StrategyDeduction::Value {
                    pos: Position { row: 1, column: 1 },
                    value: 1.try_into().unwrap()
                },
                StrategyDeduction::Value {
                    pos: Position { row: 1, column: 2 },
                    value: 3.try_into().unwrap()
                },
                StrategyDeduction::Value {
                    pos: Position { row: 2, column: 1 },
                    value: 4.try_into().unwrap()
                },
                StrategyDeduction::Value {
                    pos: Position { row: 2, column: 2 },
                    value: 2.try_into().unwrap()
                },
                StrategyDeduction::Value {
                    pos: Position { row: 3, column: 0 },
                    value: 3.try_into().unwrap()
                },
                StrategyDeduction::Value {
                    pos: Position { row: 3, column: 3 },
                    value: 4.try_into().unwrap()
                },
            ]
        );

        grid.apply_deductions(&deductions);
        assert!(grid.is_solved());
    }
}
