use crate::base::SudokuBase;
use crate::error::Result;
use crate::grid::Grid;
use crate::solver::strategic::deduction::{Deduction, Deductions, TryIntoDeductions};

use super::Strategy;

#[derive(Debug)]
pub struct SingleCandidate;

impl<Base: SudokuBase> Strategy<Base> for SingleCandidate {
    fn execute(&self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
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
    use crate::position::Position;
    use crate::samples;
    use crate::solver::strategic::deduction::IntoDeductions;

    use super::*;

    #[test]
    fn test_single_candidate() {
        let mut grid = samples::base_2().first().unwrap().clone();

        grid.set_all_direct_candidates();
        grid.fix_all_values();

        println!("{grid}");

        let deductions = SingleCandidate.execute(&mut grid).unwrap();

        // TODO: rewrite using grid.deduction_at
        assert_eq!(
            deductions,
            IntoDeductions(vec![
                Deduction::with_value(
                    Position { row: 0, column: 0 },
                    grid.get(Position { row: 0, column: 0 })
                        .candidates()
                        .unwrap(),
                    2.try_into().unwrap()
                )
                .unwrap(),
                Deduction::with_value(
                    Position { row: 0, column: 3 },
                    grid.get(Position { row: 0, column: 3 })
                        .candidates()
                        .unwrap(),
                    1.try_into().unwrap()
                )
                .unwrap(),
                Deduction::with_value(
                    Position { row: 1, column: 1 },
                    grid.get(Position { row: 1, column: 1 })
                        .candidates()
                        .unwrap(),
                    1.try_into().unwrap()
                )
                .unwrap(),
                Deduction::with_value(
                    Position { row: 1, column: 2 },
                    grid.get(Position { row: 1, column: 2 })
                        .candidates()
                        .unwrap(),
                    3.try_into().unwrap()
                )
                .unwrap(),
                Deduction::with_value(
                    Position { row: 2, column: 1 },
                    grid.get(Position { row: 2, column: 1 })
                        .candidates()
                        .unwrap(),
                    4.try_into().unwrap()
                )
                .unwrap(),
                Deduction::with_value(
                    Position { row: 2, column: 2 },
                    grid.get(Position { row: 2, column: 2 })
                        .candidates()
                        .unwrap(),
                    2.try_into().unwrap()
                )
                .unwrap(),
                Deduction::with_value(
                    Position { row: 3, column: 0 },
                    grid.get(Position { row: 3, column: 0 })
                        .candidates()
                        .unwrap(),
                    3.try_into().unwrap()
                )
                .unwrap(),
                Deduction::with_value(
                    Position { row: 3, column: 3 },
                    grid.get(Position { row: 3, column: 3 })
                        .candidates()
                        .unwrap(),
                    4.try_into().unwrap()
                )
                .unwrap(),
            ])
            .try_into()
            .unwrap()
        );

        deductions.apply(&mut grid);

        assert!(grid.is_solved());
    }
}
