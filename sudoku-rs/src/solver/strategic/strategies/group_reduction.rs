use itertools::izip;

use crate::base::SudokuBase;
use crate::cell::Candidates;
use crate::cell::Value;
use crate::error::Result;
use crate::grid::Grid;
use crate::solver::strategic::deduction::{Action, Deduction, Deductions};

use super::Strategy;

// TODO: optimize
//  - https://en.wikipedia.org/wiki/Strongly_connected_component
//  - https://opensourc.es/blog/sudoku/

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct GroupReduction;

impl Strategy for GroupReduction {
    fn name(self) -> &'static str {
        "GroupReduction"
    }
    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        Ok(Grid::<Base>::all_group_positions()
            .map(|group| {
                let (positions, candidates_group): (Vec<_>, Vec<_>) = group
                    .filter_map(|pos| {
                        grid.get(pos)
                            .candidates()
                            .map(|candidates| (pos, candidates))
                    })
                    .unzip();

                let reduced_candidates_group = Self::reduce_candidates_group(&candidates_group);

                let mut deduction = Deduction::new();

                for (position, candidates, reduced_candidates) in
                    izip!(positions, candidates_group, reduced_candidates_group)
                {
                    if candidates != reduced_candidates {
                        deduction.actions.insert(
                            position,
                            Action::DeleteCandidates(candidates.without(reduced_candidates)),
                        )?;
                    }
                }

                if deduction.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(deduction))
                }
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect())
    }
}

impl GroupReduction {
    pub fn reduce_candidates_group<Base: SudokuBase>(
        candidates_group: &[Candidates<Base>],
    ) -> Vec<Candidates<Base>> {
        let mut values = Vec::with_capacity(candidates_group.len());
        let mut reduced_candidates_group = vec![Candidates::new(); candidates_group.len()];

        let mut assigned_values = Candidates::new();

        Self::walk_value_assignments(
            candidates_group,
            &mut values,
            &mut assigned_values,
            &mut reduced_candidates_group,
        );

        reduced_candidates_group
    }

    fn walk_value_assignments<Base: SudokuBase>(
        group: &[Candidates<Base>],
        values: &mut Vec<Value<Base>>,
        assigned_values: &mut Candidates<Base>,
        reduced_group: &mut [Candidates<Base>],
    ) {
        if let Some((candidate, rest)) = group.split_first() {
            for value in candidate.into_iter() {
                if assigned_values.has(value) {
                    continue;
                }
                assigned_values.insert(value);
                values.push(value);
                Self::walk_value_assignments(rest, values, assigned_values, reduced_group);
                values.pop();
                assigned_values.delete(value);
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
    use crate::solver::strategic::strategies::test_util::assert_deductions_with_grid;

    use super::*;

    #[test]
    fn test_reduce_candidates_group() {
        type TestCase = (Vec<Vec<u8>>, Vec<Vec<u8>>);

        let test_cases: Vec<TestCase> = vec![
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
            // Naked single
            (
                vec![
                    vec![2], //
                    vec![1, 2, 3],
                    vec![1, 2, 3],
                ],
                vec![
                    vec![2], //
                    vec![1, 3],
                    vec![1, 3],
                ],
            ),
            // Naked pair
            (
                vec![
                    vec![1, 3],
                    vec![1, 3],
                    vec![1, 2, 3, 4, 5],
                    vec![1, 2, 3, 4, 5],
                    vec![1, 2, 3, 4, 5],
                ],
                vec![
                    vec![1, 3],
                    vec![1, 3],
                    vec![2, 4, 5],
                    vec![2, 4, 5],
                    vec![2, 4, 5],
                ],
            ),
            // Naked tripple {3,3,3}
            (
                vec![
                    vec![1, 2, 3], //
                    vec![1, 2, 3],
                    vec![1, 2, 3],
                    vec![1, 2, 3, 4, 5, 6, 7],
                    vec![1, 2, 3, 4, 5, 6, 7],
                    vec![1, 2, 3, 4, 5, 6, 7],
                    vec![1, 2, 3, 4, 5, 6, 7],
                ],
                vec![
                    vec![1, 2, 3], //
                    vec![1, 2, 3],
                    vec![1, 2, 3],
                    vec![4, 5, 6, 7],
                    vec![4, 5, 6, 7],
                    vec![4, 5, 6, 7],
                    vec![4, 5, 6, 7],
                ],
            ),
            // Naked tripple {3,3,2}
            (
                vec![
                    vec![1, 2, 3], //
                    vec![1, 2, 3],
                    vec![1, 2],
                    vec![1, 2, 3, 4, 5, 6, 7],
                    vec![1, 2, 3, 4, 5, 6, 7],
                    vec![1, 2, 3, 4, 5, 6, 7],
                    vec![1, 2, 3, 4, 5, 6, 7],
                ],
                vec![
                    vec![1, 2, 3], //
                    vec![1, 2, 3],
                    vec![1, 2],
                    vec![4, 5, 6, 7],
                    vec![4, 5, 6, 7],
                    vec![4, 5, 6, 7],
                    vec![4, 5, 6, 7],
                ],
            ),
            // Naked tripple {3,2,2}
            (
                vec![
                    vec![1, 2, 3], //
                    vec![1, 2],
                    vec![2, 3],
                    vec![1, 2, 3, 4, 5, 6, 7],
                    vec![1, 2, 3, 4, 5, 6, 7],
                    vec![1, 2, 3, 4, 5, 6, 7],
                    vec![1, 2, 3, 4, 5, 6, 7],
                ],
                vec![
                    vec![1, 2, 3], //
                    vec![1, 2],
                    vec![2, 3],
                    vec![4, 5, 6, 7],
                    vec![4, 5, 6, 7],
                    vec![4, 5, 6, 7],
                    vec![4, 5, 6, 7],
                ],
            ),
            // Naked tripple {2,2,2}
            (
                vec![
                    vec![1, 2], //
                    vec![2, 3],
                    vec![1, 3],
                    vec![1, 2, 3, 4, 5, 6, 7],
                    vec![1, 2, 3, 4, 5, 6, 7],
                    vec![1, 2, 3, 4, 5, 6, 7],
                    vec![1, 2, 3, 4, 5, 6, 7],
                ],
                vec![
                    vec![1, 2], //
                    vec![2, 3],
                    vec![1, 3],
                    vec![4, 5, 6, 7],
                    vec![4, 5, 6, 7],
                    vec![4, 5, 6, 7],
                    vec![4, 5, 6, 7],
                ],
            ),
            // Naked quad
            // Reference: https://www.sudokuwiki.org/sudoku.htm?bd=000030086000020040090078520371856294900142375400397618200703859039205467700904132
            (
                vec![
                    vec![1, 5], //
                    vec![1, 5, 6, 8],
                    vec![1, 5, 6, 8],
                    vec![1, 6],
                    vec![1, 2, 4, 5],
                    vec![2, 4, 5, 7],
                    vec![3, 5, 6, 7, 8],
                    vec![3, 4, 6],
                ],
                vec![
                    vec![1, 5], //
                    vec![1, 5, 6, 8],
                    vec![1, 5, 6, 8],
                    vec![1, 6],
                    vec![2, 4],
                    vec![2, 4, 7],
                    vec![3, 7],
                    vec![3, 4],
                ],
            ),
            // Hidden pair 6,7
            // Reference: https://www.sudokuwiki.org/sudoku.htm?bd=000000000904607000076804100309701080008000300050308702007502610000403208000000000
            (
                vec![
                    vec![1, 2, 4, 5, 8], //
                    vec![1, 2, 3, 8],
                    vec![2, 3],
                    vec![1, 2, 9],
                    vec![1, 2, 3, 5, 9],
                    vec![5, 9],
                    vec![4, 5, 8, 9],
                    vec![2, 3, 4, 5, 6, 7, 9],
                    vec![3, 4, 5, 6, 7, 9],
                ],
                vec![
                    vec![1, 2, 4, 5, 8], //
                    vec![1, 2, 3, 8],
                    vec![2, 3],
                    vec![1, 2, 9],
                    vec![1, 2, 3, 5, 9],
                    vec![5, 9],
                    vec![4, 5, 8, 9],
                    vec![6, 7],
                    vec![6, 7],
                ],
            ),
        ];

        for (candidates_group_data, expected_reduced_candidate_group_data) in test_cases {
            let candidates_group: Vec<Candidates<Base3>> = candidates_group_data
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
    /// `https://www.sudokuwiki.org/Naked_Candidates`
    /// "Naked Pairs examples : Load Example"
    /// `https://www.sudokuwiki.org/sudoku.htm?bd=400000938032094100095300240370609004529001673604703090957008300003900400240030709`
    #[test]
    fn test_naked_pairs() {
        let mut grid: Grid<Base3> =
            "400000938032094100095300240370609004529001673604703090957008300003900400240030709"
                .parse()
                .unwrap();

        grid.set_all_direct_candidates();

        let deductions = GroupReduction.execute(&grid).unwrap();

        let expected_deductions: Deductions<_> = vec![
            vec![
                // Pair 1,6 in row 0
                ((0, 3), vec![1]),
                ((0, 4), vec![1, 6]),
                ((0, 5), vec![6]),
            ],
            vec![
                // Pair 1,6 in block 0,0
                ((2, 0), vec![1]),
            ],
            vec![
                // Pair 6,7 in row 2
                ((2, 0), vec![7]),
                ((2, 4), vec![6, 7]),
            ],
            vec![
                // Pair 4,8 in block 1,1
                ((3, 4), vec![8]),
                ((5, 4), vec![8]),
            ],
            vec![
                // Pair 5,8 in block 1,2
                ((3, 7), vec![5, 8]),
                ((5, 8), vec![5]),
            ],
        ]
        .into_iter()
        .map(|positioned_candidates| {
            Deduction::try_from_actions(positioned_candidates.into_iter().map(
                |(pos, candidates)| {
                    (
                        pos.try_into().unwrap(),
                        Action::DeleteCandidates(Candidates::try_from(candidates).unwrap()),
                    )
                },
            ))
            .unwrap()
        })
        .collect();

        assert_deductions_with_grid(&deductions, &expected_deductions, &mut grid);
    }

    /// Reference:
    /// ` https://www.sudokuwiki.org/Hidden_Candidates`
    /// "Three Hidden Pairs : Load Example"
    /// `https://www.sudokuwiki.org/sudoku.htm?bd=720408030080000047401076802810739000000851000000264080209680413340000008168943275`
    #[test]
    fn test_hidden_pairs() {
        let mut grid: Grid<Base3> =
            "720408030080000047401076802810739000000851000000264080209680413340000008168943275"
                .parse()
                .unwrap();

        grid.set_all_direct_candidates();

        let deductions = GroupReduction.execute(&grid).unwrap();

        let expected_deductions: Deductions<_> = vec![
            vec![
                // Hidden pair 2,4 and hidden single 6 in block 1,0
                ((3, 2), vec![5, 6]),
                ((4, 2), vec![3, 6, 7]),
                ((4, 0), vec![9]),
            ],
            vec![
                // Hidden pair 2,4 in column 2
                ((3, 2), vec![5, 6]),
                ((4, 2), vec![3, 6, 7]),
            ],
            vec![
                // Hidden pair 3,7 and hidden single 1 in block 1,2
                ((4, 6), vec![6, 9]),
                ((5, 6), vec![1, 5, 9]),
                ((5, 8), vec![9]),
            ],
            vec![
                // Hidden pair 3,7 in column 6
                ((4, 6), vec![6, 9]),
                ((5, 6), vec![1, 5, 9]),
            ],
        ]
        .into_iter()
        .map(|positioned_candidates| {
            Deduction::try_from_actions(positioned_candidates.into_iter().map(
                |(pos, candidates)| {
                    (
                        pos.try_into().unwrap(),
                        Action::DeleteCandidates(Candidates::try_from(candidates).unwrap()),
                    )
                },
            ))
            .unwrap()
        })
        .collect();

        assert_deductions_with_grid(&deductions, &expected_deductions, &mut grid);
    }
}
