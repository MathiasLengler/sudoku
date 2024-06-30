use itertools::Itertools;

use crate::{
    base::SudokuBase,
    cell::{Candidates, Value},
    grid::group::CandidatesGroup,
};

// adapter for previous API
pub(super) fn reduce_candidates_group<Base: SudokuBase>(
    candidates_group: &[Candidates<Base>],
) -> Vec<Candidates<Base>> {
    let taken_candidates = candidates_group
        .iter()
        .fold(Candidates::new(), |acc, &candidates| acc.union(candidates));

    let missing_candidates = taken_candidates.invert();

    let mut candidates_group_vec = Vec::with_capacity(Base::SIDE_LENGTH.into());
    candidates_group_vec.extend(missing_candidates.into_iter().map(Candidates::with_single));
    candidates_group_vec.extend(candidates_group);

    dbg!(candidates_group.iter().join(","));
    dbg!(candidates_group_vec.iter().join(","));

    let candidates_group = candidates_group_vec
        .try_into()
        .expect("Candidates group to be well formed");

    reduce_real_candidates_group::<Base>(candidates_group)
        .into_iter()
        .filter(|candidates| {
            !candidates
                .to_single()
                .is_some_and(|candidate| missing_candidates.has(candidate))
        })
        .collect()
}

fn reduce_real_candidates_group<Base: SudokuBase>(
    mut candidates_group: CandidatesGroup<Base>,
) -> CandidatesGroup<Base> {
    print_debug_candidate_matrix(candidates_group.clone());

    // TODO: calculcate number cells per Value
    //  could be usefull to pre-filter candidates to be considered
    let candidate_positions = candidates_group.transpose();

    let position_count_per_candidate = candidate_positions.map(|positions| positions.count());
    println!("{position_count_per_candidate}");

    let candidates_count_per_index = candidates_group
        .clone()
        .map(|candidates| candidates.count());

    for set_size in 1..Base::SIDE_LENGTH {
        for potenital_set in candidates_count_per_index
            .iter_enumerate()
            .filter(|&(_i, candidates_count)| candidates_count <= set_size)
            .map(|(i, _candidates_count)| i)
            // TODO: sort?
            .combinations(set_size.into())
        {
            let locked_set_indexes = potenital_set
                .into_iter()
                .map(Value::from)
                .collect::<Candidates<_>>();

            let outside_set_indexes = locked_set_indexes.invert();

            let locked_candidates = candidates_group
                .iter_filter_mask(locked_set_indexes)
                .fold(Candidates::new(), |acc, candidates| acc.union(candidates));

            if locked_candidates.count() > set_size {
                continue;
            }

            println!(
                "Potential set: size={set_size}; coordinates={}; candidates={}",
                locked_set_indexes.iter().join(","),
                candidates_group
                    .iter_filter_mask(locked_set_indexes)
                    .join(","),
            );

            let removed_candidates_by_set = candidates_group
                .iter_filter_mask(outside_set_indexes)
                // Which candidates would be removed?
                .map(|not_set_candidates| not_set_candidates.intersection(locked_candidates))
                .fold(Candidates::new(), |acc, candidates| acc.union(candidates));

            if removed_candidates_by_set.is_empty() {
                println!("Not a valid locked set since it removes no candidates.");
                continue;
            }

            println!("Valid set, removing candidates");
            candidates_group
                .iter_mut_filter_mask(outside_set_indexes)
                .for_each(|candidates| *candidates = candidates.without(removed_candidates_by_set));
            return candidates_group;
        }
    }

    return candidates_group;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::consts::*;
    use crate::cell::Candidates;
    use crate::error::Result;

    #[test]
    fn test_reduce_candidates_group() {
        type TestCase = (Vec<Vec<u8>>, Vec<Vec<u8>>);

        let test_cases: Vec<TestCase> = vec![
            // Naked single
            (vec![vec![1], vec![1, 2]], vec![vec![1], vec![2]]),
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
                    vec![4, 5, 6],
                    vec![4],
                    vec![4, 5, 6],
                ],
            ),
            // Naked pair
            (
                vec![vec![2, 3, 4], vec![2, 4], vec![2, 4]],
                vec![vec![3], vec![2, 4], vec![2, 4]],
            ),
            // Naked single
            (vec![vec![1, 2], vec![2]], vec![vec![1], vec![2]]),
            // Naked pair
            (
                vec![vec![3, 4], vec![1, 3, 4], vec![1, 2, 3, 4], vec![3, 4]],
                vec![vec![3, 4], vec![1], vec![1, 2], vec![3, 4]],
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

            let reduced_candidates_group = reduce_candidates_group(&candidates_group);

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
