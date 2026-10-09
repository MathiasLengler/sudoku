use crate::base::SudokuBase;
use crate::cell::Candidates;
use crate::cell::Value;
use crate::error::Result;
use crate::grid::Grid;
use crate::grid::group::CandidatesGroup;
use crate::position::{Coordinate, Position};
use crate::solver::strategic::deduction::{Action, Deduction, Deductions, Reason};
use crate::solver::strategic::strategies::Strategy;
use crate::solver::strategic::strategies::StrategyScore;

#[doc(hidden)]
pub mod bench_cases;
pub mod v2;
pub mod v3;

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

    /// One locked set per group per call, see [`v3::find_locked_set`].
    /// Bench cases: v3 stays under 2 µs per group up to Base5, where v2 reaches 1.3 s on cyclic
    /// candidates without a set; v2 is faster only by a few hundred ns on groups of full cells.
    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        Grid::<Base>::all_group_positions()
            .filter_map(|group| {
                // FIXME: optimize conversion of group iters to Group<T> (TrustedGroupSizeIter)
                //  Grid could provide Group<Base, T> iteration/indexing directly.
                let positions: Vec<Position<Base>> = group.collect();
                let candidates_group: CandidatesGroup<Base> = positions
                    .iter()
                    .map(|&pos| grid[pos].to_candidates())
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap();
                let position = |coordinate: Coordinate<Base>| positions[coordinate.get_usize()];

                let set = v3::find_locked_set(&candidates_group)?;
                Some(Deduction::try_from_iters(
                    set.eliminations(&candidates_group)
                        .map(|(coordinate, deleted)| {
                            (position(coordinate), Action::DeleteCandidates(deleted))
                        }),
                    set.reasons(&candidates_group)
                        // A solved set cell has no candidates to highlight.
                        .filter(|&(coordinate, _kept)| {
                            grid[position(coordinate)].candidates().is_some()
                        })
                        .map(|(coordinate, kept)| (position(coordinate), Reason::Candidates(kept))),
                ))
            })
            .collect::<Result<Deductions<Base>>>()?
            .merge_deductions_by_reasons()
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

    type PositionedValues = Vec<((u8, u8), Vec<u8>)>;

    /// (reasons, delete-candidate actions) per deduction.
    fn expected_deductions(
        deductions: Vec<(PositionedValues, PositionedValues)>,
    ) -> Deductions<Base3> {
        deductions
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
            .collect()
    }

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

        let expected_deductions = expected_deductions(vec![
            (
                // Pair 1,6 in row 0 and block 0,0
                vec![((0, 1), vec![1, 6]), ((0, 2), vec![1, 6])],
                vec![
                    ((0, 3), vec![1]),
                    ((0, 4), vec![1, 6]),
                    ((0, 5), vec![6]),
                    ((2, 0), vec![1]),
                ],
            ),
            (
                // Pair 6,7 in row 2
                vec![((2, 5), vec![6, 7]), ((2, 8), vec![6, 7])],
                vec![((2, 0), vec![7]), ((2, 4), vec![6, 7])],
            ),
            (
                // Pair 4,8 in block 1,1
                vec![((4, 3), vec![4, 8]), ((4, 4), vec![4, 8])],
                vec![((3, 4), vec![8]), ((5, 4), vec![8])],
            ),
            (
                // Pair 5,8 in block 1,2
                vec![((3, 6), vec![5, 8]), ((5, 6), vec![5, 8])],
                vec![((3, 7), vec![5, 8]), ((5, 8), vec![5])],
            ),
        ]);

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

        let expected_deductions = expected_deductions(vec![
            (
                // Hidden pair 2,4 in block 1,0 and column 2
                vec![((3, 2), vec![2, 4]), ((4, 2), vec![2, 4])],
                vec![((3, 2), vec![5, 6]), ((4, 2), vec![3, 6, 7])],
            ),
            (
                // Hidden pair 3,7 in block 1,2 and column 6
                vec![((4, 6), vec![3, 7]), ((5, 6), vec![3, 7])],
                vec![((4, 6), vec![6, 9]), ((5, 6), vec![1, 5, 9])],
            ),
        ]);

        assert_deductions_with_grid(&deductions, &expected_deductions, &mut grid);
    }

    strategy_snapshot_tests!(LockedSets);
}
