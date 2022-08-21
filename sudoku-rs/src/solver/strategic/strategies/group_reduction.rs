use itertools::izip;

use crate::base::SudokuBase;
use crate::cell::compact::candidates::Candidates;
use crate::cell::compact::value::Value;
use crate::error::Result;
use crate::grid::Grid;
use crate::solver::strategic::deduction::{Deduction, Deductions, TryIntoDeductions};

use super::Strategy;

// TODO: optimize
//  - https://en.wikipedia.org/wiki/Strongly_connected_component
//  - https://opensourc.es/blog/sudoku/

#[derive(Debug)]
pub struct GroupReduction;

impl<Base: SudokuBase> Strategy<Base> for GroupReduction {
    fn execute(&self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        TryIntoDeductions(Grid::<Base>::all_group_positions().flat_map(|group| {
            let (positions, candidates_group): (Vec<_>, Vec<_>) = group
                .filter_map(|pos| {
                    grid.get(pos)
                        .candidates()
                        .map(|candidates| (pos, candidates))
                })
                .unzip();

            let reduced_candidates_group = Self::reduce_candidates_group(&candidates_group);

            izip!(positions, candidates_group, reduced_candidates_group).filter_map(
                |(position, candidates, reduced_candidates)| {
                    if candidates != reduced_candidates {
                        Some(Deduction::with_remaining_candidates(
                            position,
                            candidates,
                            reduced_candidates,
                        ))
                    } else {
                        None
                    }
                },
            )
        }))
        .try_into()
    }
}

impl GroupReduction {
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

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use crate::base::consts::*;
    use crate::solver::strategic::deduction::IntoDeductions;

    use super::*;

    #[test]
    fn test_group_reduction() {
        let test_cases: Vec<(Vec<Vec<u8>>, Vec<Vec<u8>>)> = vec![
            (
                vec![
                    vec![1, 6],
                    vec![1, 6],
                    vec![1, 2, 5],
                    vec![1, 2, 5, 6, 7],
                    vec![2, 5, 6, 7],
                ],
                vec![
                    vec![1, 6],
                    vec![1, 6],
                    vec![2, 5],
                    vec![2, 5, 7],
                    vec![2, 5, 7],
                ],
            ),
            (
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
            ),
            (
                vec![vec![1, 2, 3, 4], vec![2, 4], vec![2, 4]],
                vec![vec![1, 3], vec![2, 4], vec![2, 4]],
            ),
            (
                vec![vec![1, 3], vec![1, 3, 4]],
                vec![vec![1, 3], vec![1, 3, 4]],
            ),
            (vec![vec![1, 2], vec![2]], vec![vec![1], vec![2]]),
            (
                vec![vec![3, 4], vec![1, 3, 4], vec![1, 2, 3, 4], vec![3, 4]],
                vec![vec![3, 4], vec![1], vec![2], vec![3, 4]],
            ),
            (
                vec![vec![2, 3, 4], vec![2, 3, 4], vec![1, 3], vec![1, 3, 4]],
                vec![vec![2, 3, 4], vec![2, 3, 4], vec![1, 3], vec![1, 3, 4]],
            ),
        ];

        for (candidates_group_data, expected_reduced_candidate_group_data) in test_cases {
            let candidates_group: Vec<Candidates<U3>> = candidates_group_data
                .into_iter()
                .map(|candidates_data| candidates_data.try_into().unwrap())
                .collect();

            let reduced_candidates_group =
                GroupReduction::reduce_candidates_group(&candidates_group);

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

    /// Reference:
    ///  https://www.sudokuwiki.org/Naked_Candidates
    ///  "Naked Pairs examples : Load Example"
    ///  https://www.sudokuwiki.org/sudoku.htm?bd=400000938032094100095300240370609004529001673604703090957008300003900400240030709
    #[test]
    fn test_naked_pairs() {
        let mut grid: Grid<U3> =
            "400000938032094100095300240370609004529001673604703090957008300003900400240030709"
                .try_into()
                .unwrap();

        grid.fix_all_values();
        grid.set_all_direct_candidates();

        let deductions = GroupReduction.execute(&grid).unwrap();

        assert_eq!(
            deductions,
            IntoDeductions(vec![
                grid.deduction_at((0, 3), Candidates::try_from(vec![2, 5]).unwrap())
                    .unwrap(),
                grid.deduction_at((0, 4), Candidates::try_from(vec![2, 5, 7]).unwrap())
                    .unwrap(),
                grid.deduction_at((0, 5), Candidates::try_from(vec![2, 5, 7]).unwrap())
                    .unwrap(),
                grid.deduction_at((2, 0), Candidates::try_from(vec![8]).unwrap())
                    .unwrap(),
                grid.deduction_at((2, 4), Candidates::try_from(vec![1, 8]).unwrap())
                    .unwrap(),
                grid.deduction_at((3, 4), Candidates::try_from(vec![2, 5]).unwrap())
                    .unwrap(),
                grid.deduction_at((3, 7), Candidates::try_from(vec![1, 2]).unwrap())
                    .unwrap(),
                grid.deduction_at((5, 4), Candidates::try_from(vec![2, 5]).unwrap())
                    .unwrap(),
                grid.deduction_at((5, 8), Candidates::try_from(vec![1, 2]).unwrap())
                    .unwrap(),
            ])
            .try_into()
            .unwrap()
        );
    }
}
