use pcp::kernel::*;
use pcp::propagators::*;
use pcp::variable::ops::*;
use pcp::term::*;
use pcp::search::search_tree_visitor::Status::*;
use pcp::search::*;
use pcp::concept::*;
use interval::ops::Range;
use interval::interval_set::*;
use gcollections::ops::*;
use fixedbitset::FixedBitSet;

// TODO: candidate reduction
// TODO: sudoku solver/strategy

pub fn candidate_reduction(max_value: usize, list_of_candidates: Vec<Vec<i32>>) -> Vec<Vec<usize>> {
    let mut space = FDSpace::empty();

    let mut values = vec![];

    // values must be in range
    for candidates in &list_of_candidates {
        let interval_set = candidates.iter()
            .map(|candidate| (*candidate, *candidate))
            .collect::<Vec<_>>().to_interval_set();

        values.push(Box::new(space.vstore.alloc(interval_set)) as Var<VStore>);
    }

    // values must all be different
    space.cstore.alloc(Box::new(Distinct::new(values)));

    // Search step.
    let mut search = one_solution_engine();

//    let mut statistics = Statistics::new();
//    let mut search: AllSolution<Monitor<Statistics,
//    OneSolution<_, VectorStack<_>, FDSpace>>> = AllSolution::new(Monitor::new(&mut statistics,
//                                                   OneSolution::new(Propagation::new(Brancher::new(FirstSmallestVar, MiddleVal, BinarySplit)))));

    let mut new_list_of_candidates: Vec<FixedBitSet> = std::iter::repeat(FixedBitSet::with_capacity(max_value + 1)).take(list_of_candidates.len()).collect();

    search.start(&space);
    loop {
        let (frozen_space, status) = search.enter(space);
        space = frozen_space.unfreeze();

        // Print result.

        //dbg!(statistics.num_solution);

        match status {
            Satisfiable => {
                println!("Satisfiable");
                eprintln!("{:?}", space.vstore.iter().map(|dom| dom.lower()).collect::<Vec<_>>());

                space.vstore.iter().zip(&mut new_list_of_candidates)
                    .for_each(|(dom, bit_set)| {
                        let value = dom.lower();

                        bit_set.insert(value as usize);
                    });
                /*                // At this stage, dom.lower() == dom.upper().
                                print!("{}, ", dom.lower());*/
            }
            Unsatisfiable => println!("Unsatisfiable"),
            EndOfSearch => {
                println!("Search terminated or was interrupted.");
                break;
            }
            Unknown(_) => unreachable!(
                "After the search step, the problem instance should be either satisfiable or unsatisfiable.")
        }
    }

    new_list_of_candidates.into_iter().map(|bit_set| bit_set.ones().collect()).collect()
}

fn main() {
    let new_list_of_candidates = candidate_reduction(4, vec![
        vec![3, 4],
        vec![1, 3, 4],
        vec![1, 2, 3, 4],
        vec![3, 4],
    ]);

    eprintln!("new_list_of_candidates = {:?}", new_list_of_candidates);
}