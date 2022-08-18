use crate::base::SudokuBase;
use crate::cell::compact::value::Value;
use crate::error::Result;
use crate::grid::Grid;
use crate::position::Position;
use crate::solver::strategic::deduction::{Deduction, Deductions, TryIntoDeductions};

use super::Strategy;

#[derive(Debug)]
pub struct HiddenSingles;

impl<Base: SudokuBase> Strategy<Base> for HiddenSingles {
    fn execute(&self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        TryIntoDeductions(
            Grid::<Base>::all_group_positions().flat_map(|group_positions| {
                #[derive(Debug, Copy, Clone, Default)]
                struct CandidateStats {
                    count: u8,
                    last_pos: Option<Position>,
                }

                // TODO: evaluate better data structure
                //  - stack allocated
                //  - Value<Base> API, less conversions
                let mut candidate_histogram =
                    vec![CandidateStats::default(); Grid::<Base>::max_value_usize()];

                for group_position in group_positions {
                    if let Some(candidates) = grid.get(group_position).candidates() {
                        for candidate in candidates.iter() {
                            let candidate = candidate.into_u8() - 1;

                            let candidate_index = usize::from(candidate);

                            candidate_histogram[candidate_index].count += 1;
                            candidate_histogram[candidate_index].last_pos = Some(group_position);
                        }
                    }
                }

                candidate_histogram.into_iter().enumerate().filter_map(
                    |(candidate_value, stats)| {
                        if stats.count == 1 {
                            // This candidate is unique in this group.
                            let pos = stats.last_pos.unwrap();

                            let candidate_value =
                                Value::<Base>::try_from(u8::try_from(candidate_value + 1).unwrap())
                                    .unwrap();

                            Some(Deduction::with_value(
                                pos,
                                grid.get(pos).candidates().unwrap(),
                                candidate_value,
                            ))
                        } else {
                            None
                        }
                    },
                )
            }),
        )
        .try_into()
    }
}

#[cfg(test)]
mod tests {
    use crate::cell::view::{f, v};
    use crate::samples;

    use super::*;

    #[test]
    fn test_hidden_singles() {
        // TODO: port test to deduction asserts

        // let mut grid = samples::base_2().into_iter().nth(1).unwrap().clone();
        //
        // grid.set_all_direct_candidates();
        // grid.fix_all_values();
        //
        // let mut modified_positions = HiddenSingles.execute(&mut grid);
        //
        // modified_positions.sort();
        //
        // assert_eq!(
        //     modified_positions,
        //     vec![
        //         Position { row: 0, column: 1 },
        //         Position { row: 1, column: 2 },
        //         Position { row: 2, column: 3 },
        //         Position { row: 3, column: 0 },
        //     ]
        // );
        //
        // let mut expected_grid = Grid::try_from(vec![
        //     vec![f(1), v(2), f(4), v(0)],
        //     vec![v(0), v(0), v(2), v(0)],
        //     vec![v(0), v(0), v(0), v(4)],
        //     vec![v(4), f(1), v(0), f(2)],
        // ])
        // .unwrap();
        //
        // expected_grid.set_all_direct_candidates();
        //
        // assert_eq!(grid, expected_grid);
    }
}
