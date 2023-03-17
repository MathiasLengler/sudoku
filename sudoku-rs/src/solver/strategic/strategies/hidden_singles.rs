use anyhow::ensure;

use crate::base::SudokuBase;
use crate::cell::compact::value::Value;
use crate::error::Result;
use crate::grid::Grid;
use crate::position::Position;
use crate::solver::strategic::deduction::{Action, Deduction, Deductions};

use super::Strategy;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct HiddenSingles;

impl Strategy for HiddenSingles {
    fn execute<Base: SudokuBase>(&self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        ensure!(
            grid.is_directly_consistent(),
            "HiddenSingles requires a directly consistent grid"
        );

        Ok(Grid::<Base>::all_group_positions()
            .flat_map(|group_positions| {
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

                            Some(Deduction::with_action(
                                pos,
                                Action::SetValue {
                                    value: candidate_value,
                                },
                            ))
                        } else {
                            None
                        }
                    },
                )
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::samples;
    use crate::solver::strategic::strategies::test_util::assert_deductions_with_grid;

    use super::*;

    #[test]
    fn test_hidden_singles() {
        let mut grid = samples::base_2().into_iter().nth(1).unwrap().clone();

        grid.set_all_direct_candidates();
        grid.fix_all_values();

        let deductions = HiddenSingles.execute(&mut grid).unwrap();

        let expected_deductions: Deductions<_> = vec![
            //
            ((0, 1), 2),
            ((1, 2), 2),
            ((2, 3), 4),
            ((3, 0), 4),
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

        assert_deductions_with_grid(deductions, expected_deductions, &mut grid);
    }
}
