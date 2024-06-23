use std::collections::BTreeMap;

use crate::base::SudokuBase;
use crate::cell::Candidates;
use crate::error::Result;
use crate::grid::Grid;
use crate::position::Position;
use crate::solver::strategic::deduction::{Action, Deduction, Deductions, Reason};

use super::{Strategy, StrategyScore};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct NakedPairs;

impl Strategy for NakedPairs {
    fn name(self) -> &'static str {
        "NakedPairs"
    }
    fn score(self) -> StrategyScore {
        5
    }
    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>> {
        Grid::<Base>::all_group_positions()
            .flat_map(|group| {
                let candidates_group: Vec<_> = group
                    .filter_map(|pos| {
                        grid.get(pos)
                            .candidates()
                            .map(|candidates| (pos, candidates))
                    })
                    .collect();

                Self::find_naked_pairs(candidates_group)
            })
            .collect::<Deductions<Base>>()
            .merge_deductions_by_reasons()
    }
}

impl NakedPairs {
    fn find_naked_pairs<Base: SudokuBase>(
        candidates_group: Vec<(Position<Base>, Candidates<Base>)>,
    ) -> impl Iterator<Item = Deduction<Base>> {
        let mut pair_candidates_histogram: BTreeMap<Candidates<Base>, Vec<Position<Base>>> =
            BTreeMap::new();

        for (pos, pair_candidates) in candidates_group
            .iter()
            .filter(|(_, candidates)| candidates.count() == 2)
        {
            pair_candidates_histogram
                .entry(*pair_candidates)
                .and_modify(|indexes| indexes.push(*pos))
                .or_insert(vec![*pos]);
        }

        pair_candidates_histogram
            .into_iter()
            .filter_map(|(pair_candidates, positions)| {
                <[Position<Base>; 2]>::try_from(positions)
                    .ok()
                    .map(|positions| (pair_candidates, positions))
            })
            .filter_map(
                move |(pair_candidates, positions): (_, [Position<Base>; 2])| {
                    // pair_candidates is a naked pair
                    let mut deduction = Deduction::new();

                    candidates_group
                        .iter()
                        .filter(|(pos, _)| !positions.contains(pos))
                        .filter_map(|(pos, candidates)| {
                            let deleted_candidates = candidates.intersection(pair_candidates);
                            if deleted_candidates.is_empty() {
                                None
                            } else {
                                Some((pos, Action::DeleteCandidates(deleted_candidates)))
                            }
                        })
                        .for_each(|(pos, action)| {
                            deduction.actions.insert(*pos, action).unwrap();
                        });

                    if deduction.actions.is_empty() {
                        None
                    } else {
                        // Add reasons
                        let [first_pos, second_pos] = positions;
                        deduction
                            .reasons
                            .insert(first_pos, Reason::Candidates(pair_candidates))
                            .unwrap();
                        deduction
                            .reasons
                            .insert(second_pos, Reason::Candidates(pair_candidates))
                            .unwrap();

                        Some(deduction)
                    }
                },
            )
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::*;
    use crate::solver::strategic::strategies::test_util::assert_deductions;

    use super::*;

    #[test]
    fn test_find_naked_pairs() {
        type Base = Base3;
        type TestCase = (Vec<(Position<Base>, Candidates<Base>)>, Deductions<Base>);

        let test_cases: Vec<TestCase> = vec![
            // Single naked pair
            (
                // candidates group
                vec![
                    ((0, 0), vec![1, 6]),
                    ((0, 1), vec![1, 6]),
                    ((0, 2), vec![1, 2, 5]),
                    ((0, 3), vec![1, 2, 5, 6, 7]),
                    ((0, 4), vec![2, 5, 6, 7]),
                ],
                // deductions
                vec![(
                    vec![
                        // reasons
                        ((0, 0), vec![1, 6]),
                        ((0, 1), vec![1, 6]),
                    ],
                    vec![
                        // actions
                        ((0, 2), vec![1]),
                        ((0, 3), vec![1, 6]),
                        ((0, 4), vec![6]),
                    ],
                )],
            ),
            // Two naked pairs
            (
                // candidates group
                vec![
                    ((0, 0), vec![1, 2]),
                    ((0, 1), vec![1, 2]),
                    ((0, 2), vec![3, 4]),
                    ((0, 3), vec![3, 4]),
                    ((0, 4), vec![1, 2, 3, 4, 5]),
                ],
                // deductions
                vec![
                    (
                        vec![
                            // reasons
                            ((0, 0), vec![1, 2]),
                            ((0, 1), vec![1, 2]),
                        ],
                        vec![
                            // actions
                            ((0, 4), vec![1, 2]),
                        ],
                    ),
                    (
                        vec![
                            // reasons
                            ((0, 2), vec![3, 4]),
                            ((0, 3), vec![3, 4]),
                        ],
                        vec![
                            // actions
                            ((0, 4), vec![3, 4]),
                        ],
                    ),
                ],
            ),
            // Hidden pair 6,7; not found
            // Reference: https://www.sudokuwiki.org/sudoku.htm?bd=000000000904607000076804100309701080008000300050308702007502610000403208000000000
            (
                // candidates group
                vec![
                    ((0, 0), vec![1, 2, 4, 5, 8]),
                    ((0, 1), vec![1, 2, 3, 8]),
                    ((0, 2), vec![2, 3]),
                    ((0, 3), vec![1, 2, 9]),
                    ((0, 4), vec![1, 2, 3, 5, 9]),
                    ((0, 5), vec![5, 9]),
                    ((0, 6), vec![4, 5, 8, 9]),
                    ((0, 7), vec![2, 3, 4, 5, 6, 7, 9]),
                    ((0, 8), vec![3, 4, 5, 6, 7, 9]),
                ],
                // deductions
                vec![],
            ),
            // Single naked pair, already solved
            (
                // candidates group
                vec![
                    ((0, 0), vec![1, 6]),
                    ((0, 1), vec![1, 6]),
                    ((0, 2), vec![2, 5]),
                    ((0, 3), vec![2, 5, 7]),
                    ((0, 4), vec![2, 5, 7]),
                ],
                // deductions
                vec![],
            ),
        ]
        .into_iter()
        .map(|(candidates_group, deductions)| {
            (
                candidates_group
                    .into_iter()
                    .map(|(pos, candidates)| {
                        (
                            pos.try_into().unwrap(),
                            Candidates::try_from(candidates).unwrap(),
                        )
                    })
                    .collect(),
                deductions
                    .into_iter()
                    .map(|(reasons, actions)| {
                        Deduction::try_from_iters(
                            actions.into_iter().map(|(pos, candidates)| {
                                (
                                    pos,
                                    Action::DeleteCandidates(
                                        Candidates::try_from(candidates).unwrap(),
                                    ),
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
                    .collect(),
            )
        })
        .collect();

        for (candidates_group, expected_deductions) in test_cases {
            let deductions: Deductions<Base> =
                NakedPairs::find_naked_pairs(candidates_group).collect();

            assert_deductions(&deductions, &expected_deductions);
        }
    }
}
