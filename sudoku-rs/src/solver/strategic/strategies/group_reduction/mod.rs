use std::convert::TryInto;

use itertools::izip;

use crate::base::SudokuBase;
use crate::cell::compact::candidates::Candidates;
use crate::cell::compact::value::Value;
use crate::grid::Grid;
use crate::position::Position;

use super::Strategy;

use self::pcp::group_candidates_reduction;

mod pcp;

#[derive(Debug)]
pub struct GroupReductionV2;

impl<Base: SudokuBase> Strategy<Base> for GroupReductionV2 {
    fn execute(&self, grid: &mut Grid<Base>) -> Vec<Position> {
        let mut modified_positions = vec![];

        Self::reduce_groups(grid.all_row_positions(), grid, &mut modified_positions);
        Self::reduce_groups(grid.all_column_positions(), grid, &mut modified_positions);
        Self::reduce_groups(grid.all_block_positions(), grid, &mut modified_positions);

        modified_positions.sort();
        modified_positions.dedup();
        modified_positions
    }
}

impl GroupReductionV2 {
    fn reduce_groups<Base: SudokuBase>(
        groups: impl Iterator<Item = impl Iterator<Item = Position>>,
        grid: &mut Grid<Base>,
        modified_positions: &mut Vec<Position>,
    ) {
        for group in groups {
            let (positions, candidates_group): (Vec<_>, Vec<_>) = group
                .filter_map(|pos| {
                    grid.get(pos)
                        .candidates()
                        .map(|candidates| (pos, candidates))
                })
                .unzip();

            let reduced_candidates_group = Self::reduce_candidates_group(&candidates_group);

            for (position, candidates, reduced_candidates) in
                izip!(positions, candidates_group, reduced_candidates_group)
            {
                if candidates != reduced_candidates {
                    println!(
                        "GroupReduction at {}: {} => {}",
                        position, candidates, reduced_candidates
                    );

                    modified_positions.push(position);

                    grid.get_mut(position).set_candidates(reduced_candidates);
                }
            }
        }
    }

    pub fn reduce_candidates_group<Base: SudokuBase>(
        candidates_group: &[Candidates<Base>],
    ) -> Vec<Candidates<Base>> {
        let mut values = vec![];
        let mut reduced_candidates_group = vec![Candidates::new(); candidates_group.len()];

        Self::walk_value_assignments(
            &candidates_group,
            &mut values,
            &mut reduced_candidates_group,
        );

        reduced_candidates_group
    }

    fn walk_value_assignments<Base: SudokuBase>(
        group: &[Candidates<Base>],
        values: &mut Vec<Value<Base>>,
        reduced_group: &mut [Candidates<Base>],
    ) {
        if let Some((candidate, rest)) = group.split_first() {
            for value in candidate.iter() {
                if values.contains(&value) {
                    continue;
                }
                values.push(value);
                Self::walk_value_assignments(rest, values, reduced_group);
                values.pop();
            }
        } else {
            for (reduced_candidates, value) in reduced_group.iter_mut().zip(values) {
                reduced_candidates.set(*value, true);
            }
        }
    }
}

#[derive(Debug)]
pub struct GroupReduction;

impl<Base: SudokuBase> Strategy<Base> for GroupReduction {
    fn execute(&self, grid: &mut Grid<Base>) -> Vec<Position> {
        let mut modified_positions = vec![];

        Self::reduce_groups(grid.all_row_positions(), grid, &mut modified_positions);
        Self::reduce_groups(grid.all_column_positions(), grid, &mut modified_positions);
        Self::reduce_groups(grid.all_block_positions(), grid, &mut modified_positions);

        modified_positions.sort();
        modified_positions.dedup();
        modified_positions
    }
}

impl GroupReduction {
    fn reduce_groups<Base: SudokuBase>(
        groups: impl Iterator<Item = impl Iterator<Item = Position>>,
        grid: &mut Grid<Base>,
        modified_positions: &mut Vec<Position>,
    ) {
        for group in groups {
            let (positions, candidates_group): (Vec<_>, Vec<_>) = group
                .filter_map(|pos| {
                    grid.get(pos)
                        .candidates()
                        .map(|candidates| (pos, candidates))
                })
                .unzip();

            let reduced_candidates_group = Self::reduce_candidates_group(&candidates_group);

            for (pos, candidates, reduced_candidates) in
                izip!(positions, candidates_group, reduced_candidates_group)
            {
                if candidates != reduced_candidates {
                    println!(
                        "GroupReduction at {}: {:?} => {:?}",
                        pos, candidates, reduced_candidates
                    );

                    grid.get_mut(pos).set_candidates(reduced_candidates);

                    modified_positions.push(pos)
                }
            }
        }
    }

    pub fn reduce_candidates_group<Base: SudokuBase>(
        candidates_group: &[Candidates<Base>],
    ) -> Vec<Candidates<Base>> {
        let candidates_group_u8 = candidates_group
            .iter()
            .map(|candidates| candidates.to_vec_u8())
            .collect::<Vec<_>>();

        let reduced_candidates_group_u8 =
            group_candidates_reduction(&candidates_group_u8, Grid::<Base>::max_value());

        reduced_candidates_group_u8
            .into_iter()
            .map(|reduced_candidates| reduced_candidates.try_into().unwrap())
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use crate::base::consts::*;

    use super::*;

    #[test]
    fn test_group_reduction() {
        let test_cases = vec![(
            vec![
                vec![1, 2],
                vec![1, 3],
                vec![2, 3],
                vec![1, 2, 3, 4, 5, 6],
                vec![1, 3, 4],
                vec![2, 3, 4, 5, 6],
            ],
            vec![
                vec![1, 2],
                vec![1, 3],
                vec![2, 3],
                vec![5, 6],
                vec![4],
                vec![5, 6],
            ],
        )];

        for (candidates_group_data, expected_reduced_candidate_group_data) in test_cases {
            let candidates_group: Vec<Candidates<U3>> = candidates_group_data
                .into_iter()
                .map(|candidates_data| candidates_data.try_into().unwrap())
                .collect();

            let reduced_candidates_group =
                GroupReductionV2::reduce_candidates_group(&candidates_group);

            let reduced_candidates_group_data: Vec<_> = reduced_candidates_group
                .into_iter()
                .map(|candidates| candidates.to_vec_u8())
                .collect();

            assert_eq!(
                reduced_candidates_group_data,
                expected_reduced_candidate_group_data
            );
        }
    }
}
