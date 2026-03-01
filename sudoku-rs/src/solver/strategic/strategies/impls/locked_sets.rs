use itertools::izip;

use crate::base::SudokuBase;
use crate::cell::Candidates;
use crate::cell::Value;
use crate::error::Result;
use crate::grid::Grid;
use crate::grid::group::CandidatesGroup;
use crate::position::Position;
use crate::solver::strategic::deduction::{Action, Deduction, Deductions, Reason};
use crate::solver::strategic::strategies::Strategy;
use crate::solver::strategic::strategies::StrategyScore;

pub mod v2;

// TODO: optimize
//  - https://en.wikipedia.org/wiki/Strongly_connected_component
//  - https://opensourc.es/blog/sudoku/
//  This seems to be the bottleneck for the goal generator
// TODO: parameterize strategy
//  set size
//  naked/hidden
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LockedSets;

impl Strategy for LockedSets {
    fn name(self) -> &'static str {
        "LockedSets"
    }

    fn score(self) -> StrategyScore {
        50
    }
    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        Grid::<Base>::all_group_positions()
            .map(|group| {
                // let group = group
                //     .map(|pos| grid[pos].to_candidates())
                //     .next_chunk()
                //     .unwrap();

                // FIXME: optimize conversion of group iters to Group<T> (TrustedGroupSizeIter)
                //  Grid could provide Group<Base, T> iteration/indexing directly.

                let candidates_group: CandidatesGroup<Base> = group
                    .clone()
                    .map(|pos| grid[pos].to_candidates())
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap();

                // Collect positions for later use when building reasons
                let group_positions: Vec<Position<Base>> = group.clone().collect();

                // let (positions, candidates_group): (Vec<_>, Vec<_>) = group
                //     .filter_map(|pos| {
                //         grid.get(pos)
                //             .candidates()
                //             .map(|candidates| (pos, candidates))
                //     })
                //     .unzip();

                // TODO: v1 vs v2 has no clear-cut performance winner
                //  For small candidates groups, v1 is faster, up to 10x (Strategies/LockedSets/execute/sample_grid_hidden_pairs)
                //  For large candidates groups, v2 is way faster, up to 6ms vs 220ns / 20_000x speed-up (Strategies/LockedSets/v2/find_locked_set/all)
                // Either optimize v2 to be faster or at least comparable to v1 in all cases.
                // Or use introspective implementation, which switches between the two implementation based on a heuristic.
                let locked_set_result = v2::find_locked_set(&candidates_group);

                let mut deduction = Deduction::new();

                for (position, candidates, reduced_candidates) in izip!(
                    group,
                    candidates_group,
                    locked_set_result.reduced_candidates_group
                ) {
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
                    // Add reasons for the locked set if found
                    if let Some(locked_set_info) = locked_set_result.locked_set_info {
                        for coordinate in locked_set_info.locked_set_coordinates {
                            let position = group_positions[coordinate.get_usize()];
                            // Intersect with the cell's actual candidates, since not all cells
                            // in a locked set necessarily contain all locked candidates
                            // (e.g., a naked triple {1,2}, {1,3}, {2,3} has locked candidates {1,2,3}).
                            let cell_candidates = grid[position].to_candidates();
                            let reason_candidates = locked_set_info.locked_candidates.intersection(cell_candidates);
                            if !reason_candidates.is_empty() {
                                deduction
                                    .reasons
                                    .insert(position, Reason::Candidates(reason_candidates))?;
                            }
                        }
                    }
                    Ok(Some(deduction))
                }
            })
            // TODO: copy `.filter_map(Result::transpose)` trick to other strategies
            .filter_map(Result::transpose)
            .collect()
    }
}

impl LockedSets {
    pub fn find_locked_set_v1<Base: SudokuBase>(
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
            for value in candidate {
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
    use crate::base::consts::*;
    use crate::solver::strategic::strategies::test_util::{
        assert_deductions_with_grid, strategy_snapshot_tests,
    };
    use crate::test_util::init_test_logger;

    use super::*;

    #[test]
    fn test_find_locked_set() {
        type TestCase = (Vec<Vec<u8>>, Vec<Vec<u8>>);

        let test_cases: Vec<TestCase> = vec![
            // Naked pair
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
            // Naked tripple
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
            // Naked pair
            (
                vec![vec![1, 2, 3, 4], vec![2, 4], vec![2, 4]],
                vec![vec![1, 3], vec![2, 4], vec![2, 4]],
            ),
            // Naked pair
            (
                vec![vec![1, 3], vec![1, 3, 4]],
                vec![vec![1, 3], vec![1, 3, 4]],
            ),
            // Naked single
            (vec![vec![1, 2], vec![2]], vec![vec![1], vec![2]]),
            // Naked pair
            (
                vec![vec![3, 4], vec![1, 3, 4], vec![1, 2, 3, 4], vec![3, 4]],
                vec![vec![3, 4], vec![1], vec![2], vec![3, 4]],
            ),
            // Naked pair
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

            let reduced_candidates_group = LockedSets::find_locked_set_v1(&candidates_group);

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

        let deductions = LockedSets.execute(&grid).unwrap();

        let expected_deductions: Deductions<_> = vec![
            (
                vec![
                    // reasons: Pair 1,6 at positions (0,1) and (0,2)
                    ((0, 1), vec![1, 6]),
                    ((0, 2), vec![1, 6]),
                ],
                vec![
                    // actions: Pair 1,6 in row 0
                    ((0, 3), vec![1]),
                    ((0, 4), vec![1, 6]),
                    ((0, 5), vec![6]),
                ],
            ),
            (
                vec![
                    // reasons: Pair 1,6 at positions (0,1) and (0,2) in block 0,0
                    ((0, 1), vec![1, 6]),
                    ((0, 2), vec![1, 6]),
                ],
                vec![
                    // actions: Pair 1,6 in block 0,0
                    ((2, 0), vec![1]),
                ],
            ),
            (
                vec![
                    // reasons: Pair 6,7 at positions (2,5) and (2,8)
                    ((2, 5), vec![6, 7]),
                    ((2, 8), vec![6, 7]),
                ],
                vec![
                    // actions: Pair 6,7 in row 2
                    ((2, 0), vec![7]),
                    ((2, 4), vec![6, 7]),
                ],
            ),
            (
                vec![
                    // reasons: Pair 4,8 at positions (4,3) and (4,4) in block 1,1
                    ((4, 3), vec![4, 8]),
                    ((4, 4), vec![4, 8]),
                ],
                vec![
                    // actions: Pair 4,8 in block 1,1
                    ((3, 4), vec![8]),
                    ((5, 4), vec![8]),
                ],
            ),
            (
                vec![
                    // reasons: Pair 5,8 at positions (3,6) and (5,6) in block 1,2
                    ((3, 6), vec![5, 8]),
                    ((5, 6), vec![5, 8]),
                ],
                vec![
                    // actions: Pair 5,8 in block 1,2
                    ((3, 7), vec![5, 8]),
                    ((5, 8), vec![5]),
                ],
            ),
        ]
        .into_iter()
        .map(|(reasons, actions)| {
            Deduction::try_from_iters(
                actions.into_iter().map(|(pos, candidates)| {
                    (
                        pos,
                        Action::DeleteCandidates(Candidates::try_from(candidates).unwrap()),
                    )
                }),
                reasons.into_iter().map(|(pos, candidates)| {
                    (
                        pos,
                        Reason::Candidates(Candidates::try_from(candidates).unwrap()),
                    )
                }),
            )
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
        init_test_logger();

        let mut grid: Grid<Base3> =
            "720408030080000047401076802810739000000851000000264080209680413340000008168943275"
                .parse()
                .unwrap();

        grid.set_all_direct_candidates();

        let deductions = LockedSets.execute(&grid).unwrap();

        let expected_deductions: Deductions<_> = vec![
            (
                vec![
                    // reasons: Hidden pair 2,4 at positions (3,2) and (4,2)
                    ((3, 2), vec![2, 4]),
                    ((4, 2), vec![2, 4]),
                ],
                vec![
                    // actions: Hidden pair 2,4 in block 1,0 and column 2
                    ((3, 2), vec![5, 6]),
                    ((4, 2), vec![3, 6, 7]),
                ],
            ),
            (
                vec![
                    // reasons: Hidden pair 3,7 at positions (4,6) and (5,6)
                    ((4, 6), vec![3, 7]),
                    ((5, 6), vec![3, 7]),
                ],
                vec![
                    // actions: Hidden pair 3,7 in block 1,2 and column 6
                    ((4, 6), vec![6, 9]),
                    ((5, 6), vec![1, 5, 9]),
                ],
            ),
        ]
        .into_iter()
        .map(|(reasons, actions)| {
            Deduction::try_from_iters(
                actions.into_iter().map(|(pos, candidates)| {
                    (
                        pos,
                        Action::DeleteCandidates(Candidates::try_from(candidates).unwrap()),
                    )
                }),
                reasons.into_iter().map(|(pos, candidates)| {
                    (
                        pos,
                        Reason::Candidates(Candidates::try_from(candidates).unwrap()),
                    )
                }),
            )
            .unwrap()
        })
        .collect();

        assert_deductions_with_grid(&deductions, &expected_deductions, &mut grid);
    }

    strategy_snapshot_tests!(LockedSets);
}
