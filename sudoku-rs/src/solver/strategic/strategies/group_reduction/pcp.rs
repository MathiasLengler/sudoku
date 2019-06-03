use fixedbitset::FixedBitSet;
use gcollections::ops::*;
use gcollections::VectorStack;
use interval::interval::ToInterval;
use pcp::concept::*;
use pcp::kernel::*;
use pcp::propagation::CStoreFD;
use pcp::propagators::*;
use pcp::search::branching::{BinarySplit, Brancher, FirstSmallestVar, MiddleVal};
use pcp::search::engine::one_solution::OneSolution;
use pcp::search::propagation::Propagation;
use pcp::search::search_tree_visitor::Status::*;
use pcp::search::*;
use pcp::term::*;
use pcp::variable::ops::*;
use pcp::variable::VStoreFD;

type VStore = VStoreFD;
type CStore = CStoreFD<VStore>;
type FDSpace = Space<VStore, CStore, NoRecomputation<VStore, CStore>>;

fn one_solution_engine_interval() -> impl SearchTreeVisitor<FDSpace> {
    OneSolution::<_, VectorStack<_>, FDSpace>::new(Propagation::new(Brancher::new(
        FirstSmallestVar,
        MiddleVal,
        BinarySplit,
    )))
}

// TODO: return result
// TODO: optimize by using 0 indexed values and returning Vec<FixedBitSet>
pub(super) fn group_candidates_reduction(
    group_candidates: &[Vec<usize>],
    max_value: usize,
) -> Vec<Vec<usize>> {
    match group_candidates.len() {
        0 | 1 => return group_candidates.to_vec(),
        _ => {}
    }

    let mut space = FDSpace::empty();

    let mut values = vec![];

    // values must be in range
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
        std::iter::repeat(FixedBitSet::with_capacity(max_value + 1))
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
        .map(|bit_set| bit_set.ones().collect())
        .collect()
}
// TODO: complete sudoku solver in PCP

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_candidates() {
        let mut test_cases = vec![vec![vec![1, 5]]];

        for group_candidates in test_cases.drain(0..=0) {
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

        for (group_candidates, expected) in test_cases.drain(4..) {
            let reduced_group_candidates = group_candidates_reduction(&group_candidates, 4);

            eprintln!("group_candidates         = {:?}", group_candidates);
            eprintln!("reduced_group_candidates = {:?}", reduced_group_candidates);

            assert_eq!(reduced_group_candidates, expected);
        }
    }
}
