use itertools::Itertools;
use log::{debug, trace};
use serde::de;

use crate::{
    base::SudokuBase,
    cell::{Candidates, Value},
    grid::group::{CandidatesGroup, Group},
    solver::strategic::deduction::Deduction,
};

// adapter for previous API
pub fn reduce_candidates_group<Base: SudokuBase>(
    candidates_group: &[Candidates<Base>],
) -> Vec<Candidates<Base>> {
    let taken_candidates = candidates_group
        .iter()
        .fold(Candidates::new(), |acc, &candidates| acc.union(candidates));

    let missing_candidates = taken_candidates.invert();

    let mut candidates_group_vec = Vec::with_capacity(Base::SIDE_LENGTH.into());
    candidates_group_vec.extend(missing_candidates.into_iter().map(Candidates::with_single));
    candidates_group_vec.extend(candidates_group);

    let candidates_group = candidates_group_vec
        .try_into()
        .expect("Candidates group to be well formed");

    reduce_complete_candidates_group::<Base>(candidates_group)
        .into_iter()
        .filter(|candidates| {
            !candidates
                .to_single()
                .is_some_and(|candidate| missing_candidates.has(candidate))
        })
        .collect()
}

fn iter_alternating<T>(mut i: impl DoubleEndedIterator<Item = T>) -> impl Iterator<Item = T> {
    let mut front = true;
    std::iter::from_fn(move || {
        let res = if front { i.next() } else { i.next_back() };
        front = !front;
        res
    })
}

fn find_locked_set<Base: SudokuBase>(
    candidates_group: CandidatesGroup<Base>,
) -> Option<Deduction<Base>> {
    None
}

// TODO: change API: return deduction
fn reduce_complete_candidates_group<Base: SudokuBase>(
    mut candidates_group: CandidatesGroup<Base>,
) -> CandidatesGroup<Base> {
    debug!("Searching for locked set in:\n{candidates_group}");

    let candidates_counts = candidates_group
        .clone()
        .map(|candidates| candidates.count());
    trace!("Candidates counts: {candidates_counts}");

    // TODO: calculcate number cells per Value
    //  could be usefull to pre-filter candidates to be considered
    //  Example: Base3 Hidden single (5) - sparse
    //  It should be obvious that candidate 5 is only contained in one cell.
    //  Does this generalize for larger sets?
    let candidate_positions = candidates_group.transpose();
    debug!("Candidate positions:\n{candidate_positions}");

    let candidates_position_counts = candidate_positions
        .clone()
        .map(|candidates| candidates.count());
    trace!("Candidates position counts: {candidates_position_counts}");

    let mut evaluated_locked_set_count_per_set_size = Group::<Base, u32>::default();

    // Search order:
    // - Naked single (1)
    // - Hidden single (MAX - 1)
    // - Naked pair (2)
    // - Hidden pair (MAX -2)
    // - ...
    // let set_size_values = {
    //     let mut all_values = Value::all();
    //     all_values.next_back();
    //     iter_alternating(all_values)
    // };

    // FIXME: Other approach: split search for hidden/nacked sets
    //  Idea: search for naked sets in transposed candidates. Does this find hidden sets in the original candidates?
    for (set_size_value, candidates_group, candidates_counts, is_transposed) in Value::<Base>::all()
        .take((Base::MAX_VALUE / 2).into())
        .flat_map(|set_size| {
            [
                (set_size, &candidates_group, &candidates_counts, false), // Naked
                (
                    set_size,
                    &candidate_positions,
                    &candidates_position_counts,
                    true,
                ), // Hidden
            ]
        })
        .filter(
            |(set_size_value, _candidates_group, _candidates_counts, is_transposed)| {
                !(Base::BASE % 2 == 0 // Base is even
                    && *is_transposed
                    && set_size_value.get() == Base::MAX_VALUE / 2)
            },
        )
    {
        // TODO: clean up logging for transposed/hidden sets
        trace!(
            "Locked set size {set_size_value}, transposed: {is_transposed}:\n{candidates_group}"
        );

        let set_size = set_size_value.get();
        let potential_locked_set_indexes = candidates_counts
            .iter_enumerate()
            .filter(|&(_i, candidates_count)| {
                // Set members for set sizes > 1 require at least 2 candidates.
                if set_size == 1 {
                    candidates_count == 1
                } else {
                    (2..=set_size).contains(&candidates_count)
                }
            })
            .map(|(i, _candidates_count)| i)
            .collect::<Candidates<_>>();

        trace!("Potential locked set indexes: {potential_locked_set_indexes}");

        // TODO: sort?
        //  idea: find the most likely locked set first
        // TODO: optimization: build locked set iteratively for set_size_value > 2
        //  idea: if two cells combine to a locked candidate count greater than set_size_value, we can skip all combinations which contain those two cells.
        //  This smells like a tree pruning search.
        //  additional criteria: only consider a candidates index if it has some overlap with the current locked set
        for locked_set_indexes in potential_locked_set_indexes.combinations(set_size_value) {
            *evaluated_locked_set_count_per_set_size.get_mut(if is_transposed {
                (Value::<Base>::max().get() - set_size_value.get())
                    .try_into()
                    .unwrap()
            } else {
                set_size_value.into()
            }) += 1;

            trace!("Evaluating locked set indexes {locked_set_indexes}");

            let locked_candidates = candidates_group
                .iter_index_mask(locked_set_indexes)
                .fold(Candidates::new(), |acc, candidates| acc.union(candidates));

            trace!("Locked candidates: {locked_candidates}");

            let locked_candidates_count = locked_candidates.count();
            if locked_candidates_count > set_size {
                trace!("Not a valid locked set, locked candidates count {locked_candidates_count} > set size {set_size_value}");
                continue;
            }

            trace!("Potential locked set, locked candidates count {locked_candidates_count} <= set size {set_size_value}");

            let outside_set_indexes = locked_set_indexes.invert();

            trace!(
                "Outside locked set indexes {outside_set_indexes} with candidates:\n{}",
                candidates_group
                    .iter_index_mask(outside_set_indexes)
                    .join("\n")
            );

            let removed_candidates_by_set = candidates_group
                .iter_index_mask(outside_set_indexes)
                // Which candidates would be removed?
                .map(|not_set_candidates| not_set_candidates.intersection(locked_candidates))
                .fold(Candidates::new(), |acc, candidates| acc.union(candidates));

            if removed_candidates_by_set.is_empty() {
                trace!("Not a valid locked set since it removes no candidates.");
                continue;
            }

            debug!(
                "Valid locked set {locked_candidates} at indexes {locked_set_indexes} with candidates:\n{}\nRemoves candidates {removed_candidates_by_set} from indexes {outside_set_indexes}",
                candidates_group
                    .iter_index_mask(locked_set_indexes)
                    .join("\n")
            );
            let mut candidates_group = candidates_group.clone();
            candidates_group
                .iter_mut_index_mask(outside_set_indexes)
                .for_each(|candidates| *candidates = candidates.without(removed_candidates_by_set));

            if is_transposed {
                candidates_group = candidates_group.transpose();
            }

            trace!("evaluated_locked_set_count_per_set_size: {evaluated_locked_set_count_per_set_size}");
            return candidates_group;
        }
    }

    debug!("No locked set found");
    candidates_group
}

#[cfg(test)]
mod tests {
    use log::info;

    use super::*;
    use crate::base::consts::*;
    use crate::cell::Candidates;
    use crate::solver::strategic::strategies::GroupReduction;
    use crate::test_util::init_test_logger;

    #[test]
    fn test_reduce_candidates_group() {
        type TestCase = (Vec<Vec<u8>>, Vec<Vec<u8>>);

        init_test_logger();

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
                    vec![1, 3, 4, 5],
                    vec![2, 3, 4, 5, 6],
                ],
                vec![
                    vec![1, 2],
                    vec![1, 3],
                    vec![2, 3],
                    vec![4, 5, 6],
                    vec![4, 5],
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
                vec![vec![3, 4], vec![1, 2, 3, 4], vec![1, 2, 3, 4], vec![3, 4]],
                vec![vec![3, 4], vec![1, 2], vec![1, 2], vec![3, 4]],
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

        for (i, (candidates_group_data, expected_reduced_candidate_group_data)) in
            test_cases.into_iter().enumerate()
        {
            let candidates_group: Vec<Candidates<Base3>> = candidates_group_data
                .clone()
                .into_iter()
                .map(|candidates_data| candidates_data.try_into().unwrap())
                .collect();

            let reduced_candidates_group = reduce_candidates_group(&candidates_group);

            let reduced_candidates_group_data: Vec<_> = reduced_candidates_group
                .into_iter()
                .map(|candidates| candidates.to_vec_u8())
                .collect();

            assert_eq!(
                reduced_candidates_group_data, expected_reduced_candidate_group_data,
                "input #{i}: {candidates_group_data:?}"
            );
        }
    }

    type TestCase<Base> = (&'static str, CandidatesGroup<Base>, CandidatesGroup<Base>);

    fn candidates_group_from_test_data<Base: SudokuBase>(
        candidates_group_data: Vec<Vec<u8>>,
    ) -> CandidatesGroup<Base> {
        CandidatesGroup::try_from(
            candidates_group_data
                .into_iter()
                .map(|candidates| Candidates::try_from(candidates).unwrap())
                .collect::<Vec<_>>(),
        )
        .unwrap()
    }

    fn assert_reduce_complete_candidates_group<Base: SudokuBase>(
        (base_test_case_name, input, expected_output): TestCase<Base>,
    ) {
        assert_reduce_complete_candidates_group_single(
            base_test_case_name,
            &input,
            &expected_output,
        );
        assert_reduce_complete_candidates_group_single(
            &format!("{base_test_case_name} - reversed"),
            &input.clone().reverse(),
            &expected_output.clone().reverse(),
        );
        // TODO: shuffle candidate positions
        // TODO: re-label candidates
    }

    fn assert_reduce_complete_candidates_group_single<Base: SudokuBase>(
        test_case_name: &str,
        input: &CandidatesGroup<Base>,
        expected_output: &CandidatesGroup<Base>,
    ) {
        info!("Test case: {test_case_name}");

        let actual_output = reduce_complete_candidates_group(input.clone());

        assert_eq!(
            &actual_output, expected_output,
            "Test case {test_case_name}:\n{actual_output}!=\n{expected_output}"
        );
    }

    #[test]
    fn test_reduce_complete_candidates_group_base_2() {
        type Base = Base2;

        init_test_logger();

        let test_cases: Vec<TestCase<Base>> = vec![
            (
                "Naked single (2) - filled",
                vec![
                    //
                    vec![2],
                    vec![1, 2, 3, 4],
                    vec![1, 2, 3, 4],
                    vec![1, 2, 3, 4],
                ],
                vec![
                    //
                    vec![2],
                    vec![1, 3, 4],
                    vec![1, 3, 4],
                    vec![1, 3, 4],
                ],
            ),
            (
                "Naked single (2) - sparse",
                vec![
                    //
                    vec![2],
                    vec![1, 2],
                    vec![2, 3],
                    vec![3, 4],
                ],
                vec![
                    //
                    vec![2],
                    vec![1],
                    vec![3],
                    vec![3, 4],
                ],
            ),
            (
                "Hidden single (2) - filled",
                vec![
                    //
                    vec![1, 2, 3, 4],
                    vec![1, 3, 4],
                    vec![1, 3, 4],
                    vec![1, 3, 4],
                ],
                vec![
                    //
                    vec![2],
                    vec![1, 3, 4],
                    vec![1, 3, 4],
                    vec![1, 3, 4],
                ],
            ),
            (
                "Hidden single (2) - sparse",
                vec![
                    //
                    vec![2, 3],
                    vec![1, 3],
                    vec![3, 4],
                    vec![1, 4],
                ],
                vec![
                    //
                    vec![2],
                    vec![1, 3],
                    vec![3, 4],
                    vec![1, 4],
                ],
            ),
            (
                "Naked pair (2,4) - filled",
                vec![
                    //
                    vec![2, 4],
                    vec![2, 4],
                    vec![1, 2, 3, 4],
                    vec![1, 2, 3, 4],
                ],
                vec![
                    //
                    vec![2, 4],
                    vec![2, 4],
                    vec![1, 3],
                    vec![1, 3],
                ],
            ),
            // TODO: add real-world cases
        ]
        .into_iter()
        .map(|(test_case_name, input, expected_output)| {
            (
                test_case_name,
                candidates_group_from_test_data(input),
                candidates_group_from_test_data(expected_output),
            )
        })
        .collect();

        for test_case in test_cases {
            assert_reduce_complete_candidates_group(test_case);
        }
    }

    #[test]
    fn test_reduce_complete_candidates_group_base_3() {
        type Base = Base3;

        init_test_logger();

        let test_cases: Vec<TestCase<Base>> = vec![
            (
                "Naked single (5) - outside filled",
                vec![
                    //
                    vec![5],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                ],
                vec![
                    //
                    vec![5],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                ],
            ),
            (
                "Naked single (5) - outside pair chain",
                vec![
                    //
                    vec![5],
                    vec![1, 2],
                    vec![1, 3],
                    vec![2, 3],
                    vec![4, 5],
                    vec![5, 6],
                    vec![7, 8],
                    vec![8, 9],
                    vec![7, 9],
                ],
                vec![
                    //
                    vec![5],
                    vec![1, 2],
                    vec![1, 3],
                    vec![2, 3],
                    vec![4],
                    vec![6],
                    vec![7, 8],
                    vec![8, 9],
                    vec![7, 9],
                ],
            ),
            (
                "Naked single (5) - outside singles",
                vec![
                    //
                    vec![5],
                    vec![1, 5],
                    vec![2],
                    vec![3],
                    vec![4],
                    vec![6],
                    vec![7],
                    vec![8],
                    vec![9],
                ],
                vec![
                    //
                    vec![5],
                    vec![1],
                    vec![2],
                    vec![3],
                    vec![4],
                    vec![6],
                    vec![7],
                    vec![8],
                    vec![9],
                ],
            ),
            (
                "Hidden single (5) - outside filled",
                vec![
                    //
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                ],
                vec![
                    //
                    vec![5],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                ],
            ),
            (
                "Hidden single (5) - outside sparse",
                vec![
                    //
                    vec![4, 5],
                    vec![1, 2],
                    vec![1, 3],
                    vec![2, 3],
                    vec![4, 6],
                    vec![4, 6],
                    vec![7, 8],
                    vec![8, 9],
                    vec![7, 9],
                ],
                vec![
                    //
                    vec![5],
                    vec![1, 2],
                    vec![1, 3],
                    vec![2, 3],
                    vec![4, 6],
                    vec![4, 6],
                    vec![7, 8],
                    vec![8, 9],
                    vec![7, 9],
                ],
            ),
            (
                "Naked pair (4,6) - outside filled",
                vec![
                    //
                    vec![4, 6],
                    vec![4, 6],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                ],
                vec![
                    //
                    vec![4, 6],
                    vec![4, 6],
                    vec![1, 2, 3, 5, 7, 8, 9],
                    vec![1, 2, 3, 5, 7, 8, 9],
                    vec![1, 2, 3, 5, 7, 8, 9],
                    vec![1, 2, 3, 5, 7, 8, 9],
                    vec![1, 2, 3, 5, 7, 8, 9],
                    vec![1, 2, 3, 5, 7, 8, 9],
                    vec![1, 2, 3, 5, 7, 8, 9],
                ],
            ),
            (
                "Naked pair (4,6) - outside sparse",
                vec![
                    //
                    vec![4, 6],
                    vec![4, 6],
                    vec![1, 2],
                    vec![2, 3],
                    vec![1, 3],
                    vec![4, 5, 6, 7, 9],
                    vec![5, 6, 7, 8],
                    vec![4, 5, 6, 7, 8, 9],
                    vec![5, 7, 8, 9],
                ],
                vec![
                    //
                    vec![4, 6],
                    vec![4, 6],
                    vec![1, 2],
                    vec![2, 3],
                    vec![1, 3],
                    vec![5, 7, 9],
                    vec![5, 7, 8],
                    vec![5, 7, 8, 9],
                    vec![5, 7, 8, 9],
                ],
            ),
            (
                "Naked pair (4,6) - outside singles",
                vec![
                    //
                    vec![4, 6],
                    vec![4, 6],
                    vec![1, 2, 4],
                    vec![1, 2],
                    vec![3],
                    vec![5],
                    vec![7],
                    vec![8],
                    vec![9],
                ],
                vec![
                    //
                    vec![4, 6],
                    vec![4, 6],
                    vec![1, 2],
                    vec![1, 2],
                    vec![3],
                    vec![5],
                    vec![7],
                    vec![8],
                    vec![9],
                ],
            ),
            (
                "Hidden pair (4,6) - outside filled",
                vec![
                    //
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 5, 7, 8, 9],
                    vec![1, 2, 3, 5, 7, 8, 9],
                    vec![1, 2, 3, 5, 7, 8, 9],
                    vec![1, 2, 3, 5, 7, 8, 9],
                    vec![1, 2, 3, 5, 7, 8, 9],
                    vec![1, 2, 3, 5, 7, 8, 9],
                    vec![1, 2, 3, 5, 7, 8, 9],
                ],
                vec![
                    //
                    vec![4, 6],
                    vec![4, 6],
                    vec![1, 2, 3, 5, 7, 8, 9],
                    vec![1, 2, 3, 5, 7, 8, 9],
                    vec![1, 2, 3, 5, 7, 8, 9],
                    vec![1, 2, 3, 5, 7, 8, 9],
                    vec![1, 2, 3, 5, 7, 8, 9],
                    vec![1, 2, 3, 5, 7, 8, 9],
                    vec![1, 2, 3, 5, 7, 8, 9],
                ],
            ),
            (
                "Hidden pair (4,6) - outside sparse",
                vec![
                    //
                    vec![4, 5, 6],
                    vec![4, 6],
                    vec![1, 2],
                    vec![2, 3],
                    vec![1, 3],
                    vec![5, 7],
                    vec![7, 8],
                    vec![8, 9],
                    vec![5, 9],
                ],
                vec![
                    //
                    vec![4, 6],
                    vec![4, 6],
                    vec![1, 2],
                    vec![2, 3],
                    vec![1, 3],
                    vec![5, 7],
                    vec![7, 8],
                    vec![8, 9],
                    vec![5, 9],
                ],
            ),
            (
                "Naked tripple (3,5,7) - complete - outside filled",
                vec![
                    //
                    vec![3, 5, 7],
                    vec![3, 5, 7],
                    vec![3, 5, 7],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                ],
                vec![
                    //
                    vec![3, 5, 7],
                    vec![3, 5, 7],
                    vec![3, 5, 7],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                ],
            ),
            (
                "Naked tripple (3,5,7) - complete - outside sparse",
                vec![
                    //
                    vec![3, 5, 7],
                    vec![3, 5, 7],
                    vec![3, 5, 7],
                    vec![1, 2, 3],
                    vec![2, 4],
                    vec![4, 6],
                    vec![6, 8],
                    vec![8, 9],
                    vec![1, 9],
                ],
                vec![
                    //
                    vec![3, 5, 7],
                    vec![3, 5, 7],
                    vec![3, 5, 7],
                    vec![1, 2],
                    vec![2, 4],
                    vec![4, 6],
                    vec![6, 8],
                    vec![8, 9],
                    vec![1, 9],
                ],
            ),
            (
                "Naked tripple (3,5,7) - distributed - outside filled",
                vec![
                    //
                    vec![3, 5],
                    vec![5, 7],
                    vec![3, 7],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                ],
                vec![
                    //
                    vec![3, 5],
                    vec![5, 7],
                    vec![3, 7],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                ],
            ),
            (
                "Naked tripple (3,5,7) - distributed - outside sparse",
                vec![
                    //
                    vec![3, 5],
                    vec![5, 7],
                    vec![3, 7],
                    vec![1, 2, 3],
                    vec![2, 4],
                    vec![4, 6],
                    vec![6, 8],
                    vec![8, 9],
                    vec![1, 9],
                ],
                vec![
                    //
                    vec![3, 5],
                    vec![5, 7],
                    vec![3, 7],
                    vec![1, 2],
                    vec![2, 4],
                    vec![4, 6],
                    vec![6, 8],
                    vec![8, 9],
                    vec![1, 9],
                ],
            ),
            (
                "Hidden tripple (3,5,7) - complete - outside filled",
                vec![
                    //
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                ],
                vec![
                    //
                    vec![3, 5, 7],
                    vec![3, 5, 7],
                    vec![3, 5, 7],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                ],
            ),
            (
                "Hidden tripple (3,5,7) - complete - outside sparse",
                vec![
                    //
                    vec![1, 3, 5, 7],
                    vec![3, 5, 7],
                    vec![3, 5, 7],
                    vec![1, 2],
                    vec![2, 4],
                    vec![4, 6],
                    vec![6, 8],
                    vec![8, 9],
                    vec![1, 9],
                ],
                vec![
                    //
                    vec![3, 5, 7],
                    vec![3, 5, 7],
                    vec![3, 5, 7],
                    vec![1, 2],
                    vec![2, 4],
                    vec![4, 6],
                    vec![6, 8],
                    vec![8, 9],
                    vec![1, 9],
                ],
            ),
            (
                "Hidden tripple (3,5,7) - distributed - outside filled",
                vec![
                    //
                    vec![1, 2, 3, 4, 5, 6, 8, 9],
                    vec![1, 2, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 6, 7, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                ],
                vec![
                    //
                    vec![3, 5],
                    vec![5, 7],
                    vec![3, 7],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                    vec![1, 2, 4, 6, 8, 9],
                ],
            ),
            (
                "Hidden tripple (3,5,7) - distributed - outside sparse",
                vec![
                    //
                    vec![1, 3, 5],
                    vec![5, 7],
                    vec![3, 7],
                    vec![1, 2],
                    vec![2, 4],
                    vec![4, 6],
                    vec![6, 8],
                    vec![8, 9],
                    vec![1, 9],
                ],
                vec![
                    //
                    vec![3, 5],
                    vec![5, 7],
                    vec![3, 7],
                    vec![1, 2],
                    vec![2, 4],
                    vec![4, 6],
                    vec![6, 8],
                    vec![8, 9],
                    vec![1, 9],
                ],
            ),
            (
                "Naked quad (2,4,6,8) - complete - outside filled",
                vec![
                    //
                    vec![2, 4, 6, 8],
                    vec![2, 4, 6, 8],
                    vec![2, 4, 6, 8],
                    vec![2, 4, 6, 8],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                ],
                vec![
                    //
                    vec![2, 4, 6, 8],
                    vec![2, 4, 6, 8],
                    vec![2, 4, 6, 8],
                    vec![2, 4, 6, 8],
                    vec![1, 3, 5, 7, 9],
                    vec![1, 3, 5, 7, 9],
                    vec![1, 3, 5, 7, 9],
                    vec![1, 3, 5, 7, 9],
                    vec![1, 3, 5, 7, 9],
                ],
            ),
            (
                "Naked quad (2,4,6,8) - complete - outside sparse",
                vec![
                    //
                    vec![2, 4, 6, 8],
                    vec![2, 4, 6, 8],
                    vec![2, 4, 6, 8],
                    vec![2, 4, 6, 8],
                    vec![1, 3, 4],
                    vec![3, 5],
                    vec![5, 7],
                    vec![7, 9],
                    vec![1, 9],
                ],
                vec![
                    //
                    vec![2, 4, 6, 8],
                    vec![2, 4, 6, 8],
                    vec![2, 4, 6, 8],
                    vec![2, 4, 6, 8],
                    vec![1, 3],
                    vec![3, 5],
                    vec![5, 7],
                    vec![7, 9],
                    vec![1, 9],
                ],
            ),
            (
                "Naked quad (2,4,6,8) - distributed - outside filled",
                vec![
                    //
                    vec![2, 4],
                    vec![4, 6],
                    vec![6, 8],
                    vec![2, 8],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                ],
                vec![
                    //
                    vec![2, 4],
                    vec![4, 6],
                    vec![6, 8],
                    vec![2, 8],
                    vec![1, 3, 5, 7, 9],
                    vec![1, 3, 5, 7, 9],
                    vec![1, 3, 5, 7, 9],
                    vec![1, 3, 5, 7, 9],
                    vec![1, 3, 5, 7, 9],
                ],
            ),
            (
                "Naked quad (2,4,6,8) - distributed - outside sparse",
                vec![
                    //
                    vec![2, 4],
                    vec![4, 6],
                    vec![6, 8],
                    vec![2, 8],
                    vec![1, 3, 4],
                    vec![3, 5],
                    vec![5, 7],
                    vec![7, 9],
                    vec![1, 9],
                ],
                vec![
                    //
                    vec![2, 4],
                    vec![4, 6],
                    vec![6, 8],
                    vec![2, 8],
                    vec![1, 3],
                    vec![3, 5],
                    vec![5, 7],
                    vec![7, 9],
                    vec![1, 9],
                ],
            ),
            (
                "Hidden quad (2,4,6,8) - complete - outside filled",
                vec![
                    //
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
                    vec![1, 3, 5, 7, 9],
                    vec![1, 3, 5, 7, 9],
                    vec![1, 3, 5, 7, 9],
                    vec![1, 3, 5, 7, 9],
                    vec![1, 3, 5, 7, 9],
                ],
                vec![
                    //
                    vec![2, 4, 6, 8],
                    vec![2, 4, 6, 8],
                    vec![2, 4, 6, 8],
                    vec![2, 4, 6, 8],
                    vec![1, 3, 5, 7, 9],
                    vec![1, 3, 5, 7, 9],
                    vec![1, 3, 5, 7, 9],
                    vec![1, 3, 5, 7, 9],
                    vec![1, 3, 5, 7, 9],
                ],
            ),
            (
                "Hidden quad (2,4,6,8) - complete - outside sparse",
                vec![
                    //
                    vec![1, 2, 4, 6, 8],
                    vec![2, 4, 6, 8],
                    vec![2, 4, 6, 8],
                    vec![2, 4, 6, 8],
                    vec![1, 3],
                    vec![3, 5],
                    vec![5, 7],
                    vec![7, 9],
                    vec![1, 9],
                ],
                vec![
                    //
                    vec![2, 4, 6, 8],
                    vec![2, 4, 6, 8],
                    vec![2, 4, 6, 8],
                    vec![2, 4, 6, 8],
                    vec![1, 3],
                    vec![3, 5],
                    vec![5, 7],
                    vec![7, 9],
                    vec![1, 9],
                ],
            ),
            (
                "Hidden quad (2,4,6,8) - distributed - outside filled",
                vec![
                    //
                    vec![1, 2, 3, 4, 5, 7, 9],
                    vec![1, 3, 4, 5, 6, 7, 9],
                    vec![1, 3, 5, 6, 7, 8, 9],
                    vec![1, 2, 3, 5, 7, 8, 9],
                    vec![1, 3, 5, 7, 9],
                    vec![1, 3, 5, 7, 9],
                    vec![1, 3, 5, 7, 9],
                    vec![1, 3, 5, 7, 9],
                    vec![1, 3, 5, 7, 9],
                ],
                vec![
                    //
                    vec![2, 4],
                    vec![4, 6],
                    vec![6, 8],
                    vec![2, 8],
                    vec![1, 3, 5, 7, 9],
                    vec![1, 3, 5, 7, 9],
                    vec![1, 3, 5, 7, 9],
                    vec![1, 3, 5, 7, 9],
                    vec![1, 3, 5, 7, 9],
                ],
            ),
            (
                "Hidden quad (2,4,6,8) - distributed - outside sparse",
                vec![
                    //
                    vec![1, 2, 4],
                    vec![4, 6],
                    vec![6, 8],
                    vec![2, 8],
                    vec![1, 3],
                    vec![3, 5],
                    vec![5, 7],
                    vec![7, 9],
                    vec![1, 9],
                ],
                vec![
                    //
                    vec![2, 4],
                    vec![4, 6],
                    vec![6, 8],
                    vec![2, 8],
                    vec![1, 3],
                    vec![3, 5],
                    vec![5, 7],
                    vec![7, 9],
                    vec![1, 9],
                ],
            ),
        ]
        // TODO: add real-world test cases
        .into_iter()
        .map(|(test_case_name, input, expected_output)| {
            (
                test_case_name,
                candidates_group_from_test_data(input),
                candidates_group_from_test_data(expected_output),
            )
        })
        .collect();

        for test_case in test_cases {
            // if test_case.0 != "Naked pair (4,6) - outside singles" {
            //     continue;
            // }
            assert_reduce_complete_candidates_group(test_case);
        }
    }

    #[test]
    fn test_reduce_complete_candidates_group_vs_v1() {
        // v2 only applies a single deduction. To be comparable with v1, we need to apply it recursively.
        fn v2_recusive<Base: SudokuBase>(
            candidates_group: &CandidatesGroup<Base>,
        ) -> CandidatesGroup<Base> {
            let reduced = reduce_complete_candidates_group(candidates_group.clone());
            if &reduced == candidates_group {
                return reduced;
            }
            v2_recusive(&reduced)
        }

        type Base = Base2;

        let mut i = 0;

        for input in std::iter::repeat(Candidates::<Base>::iter_all_lexicographical())
            .take(4)
            .multi_cartesian_product()
            .map(|candidates_group| CandidatesGroup::<Base>::try_from(candidates_group).unwrap())
        {
            if input.iter().any(|candidates| candidates.is_empty()) {
                continue;
            }
            let reduced_v1 = CandidatesGroup::<Base>::try_from(
                GroupReduction::reduce_candidates_group_v1(input.as_slice()),
            )
            .unwrap();
            if reduced_v1.iter().any(|candidates| candidates.is_empty()) {
                continue;
            }

            let reduced_v2 = v2_recusive(&input);

            assert_eq!(
                reduced_v1, reduced_v2,
                "\nInput:\n{input}V1:\n{reduced_v1}!= V2:\n{reduced_v2}"
            );

            i += 1;
        }
        dbg!(i);
    }
}
