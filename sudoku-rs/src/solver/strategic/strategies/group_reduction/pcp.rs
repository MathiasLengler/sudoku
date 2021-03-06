use std::convert::TryInto;

use fixedbitset::FixedBitSet;
use gcollections::ops::*;
use interval::interval::ToInterval;
use pcp::concept::*;
use pcp::kernel::*;
use pcp::propagators::*;
use pcp::search::search_tree_visitor::Status::*;
use pcp::search::*;
use pcp::term::*;
use pcp::variable::ops::*;

use crate::solver::constraint::pcp_utils::{one_solution_engine_interval, FDSpace, VStore};

// TODO: return result
// TODO: optimize by using 0 indexed values and returning Vec<FixedBitSet>
pub(super) fn group_candidates_reduction(
    group_candidates: &[Vec<u8>],
    max_value: u8,
) -> Vec<Vec<u8>> {
    match group_candidates.len() {
        0 | 1 => return group_candidates.to_vec(),
        _ => {}
    }

    let mut space = FDSpace::empty();

    let mut values = vec![];

    // Define values constrained to range
    for _candidates in group_candidates {
        values
            .push(Box::new(space.vstore.alloc((1, max_value as i32).to_interval())) as Var<VStore>);
    }

    // values must be one of the given candidates
    for (value, candidates) in values.iter().zip(group_candidates) {
        for candidate in 1..=max_value {
            if !candidates.contains(&candidate) {
                //                dbg!(candidate);
                space.cstore.alloc(Box::new(XNeqY::new(
                    value.bclone(),
                    Box::new(Constant::new(candidate as i32)),
                )));
            }
        }
    }

    // values must all be different
    space.cstore.alloc(Box::new(Distinct::new(values)));

    //    dbg!(&space.cstore);

    // Search step.
    let mut search = one_solution_engine_interval();

    let mut reduced_group_candidates: Vec<FixedBitSet> =
        std::iter::repeat(FixedBitSet::with_capacity(usize::from(max_value) + 1))
            .take(group_candidates.len())
            .collect();

    search.start(&space);
    loop {
        let (frozen_space, status) = search.enter(space);
        space = frozen_space.unfreeze();

        match status {
            Satisfiable => {
//                eprintln!("space.vstore = {:?}", space.vstore.iter().map(|dom| (dom.lower(), dom.upper())).collect::<Vec<_>>());

                space.vstore.iter().zip(&mut reduced_group_candidates)
                    .for_each(|(dom, bit_set)| {
                        if dom.lower() != dom.upper() {
//                            eprintln!("Open interval {:?}", dom);

                            let dom_as_range = (dom.lower() as usize)..((dom.upper() as usize) + 1);

                            bit_set.set_range(dom_as_range, true);
                        } else {
                            bit_set.insert(dom.lower() as usize);
                        };
                    });
            }
            Unsatisfiable => {
//                eprintln!(
//                    "Unsatisfiable: {:?}\nreduced_group_candidates = {:?}",
//                    group_candidates,
//                    reduced_group_candidates
//                        .into_iter()
//                        .map(|bit_set| bit_set.ones().collect::<Vec<_>>())
//                        .collect::<Vec<_>>()
//                );
                return group_candidates.to_vec();
            }
            EndOfSearch => {
                break;
            }
            Unknown(_) => unreachable!(
                "After the search step, the problem instance should be either satisfiable or unsatisfiable.")
        }
    }

    reduced_group_candidates
        .into_iter()
        .map(|bit_set| {
            bit_set
                .ones()
                .map(|candidate| candidate.try_into().unwrap())
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_candidates() {
        let mut test_cases = vec![vec![vec![1, 5]]];

        for group_candidates in test_cases.drain(..) {
            let reduced_group_candidates = group_candidates_reduction(&group_candidates, 5);

            eprintln!("group_candidates         = {:?}", group_candidates);
            eprintln!("reduced_group_candidates = {:?}", reduced_group_candidates);
        }
    }

    #[test]
    fn test_group_candidates_reduction() {
        let mut test_cases = vec![
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

        for (group_candidates, expected) in test_cases.drain(..) {
            let reduced_group_candidates = group_candidates_reduction(&group_candidates, 4);

            eprintln!("group_candidates         = {:?}", group_candidates);
            eprintln!("reduced_group_candidates = {:?}", reduced_group_candidates);

            assert_eq!(reduced_group_candidates, expected);
        }
    }
}
