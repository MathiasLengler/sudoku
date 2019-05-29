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
use pcp::search::engine::all_solution::AllSolution;
use pcp::search::monitor::Monitor;
use pcp::search::engine::one_solution::OneSolution;
use pcp::search::propagation::Propagation;
use pcp::search::branching::{Brancher, FirstSmallestVar, MiddleVal, BinarySplit};
use pcp::search::statistics::Statistics;
use gcollections::VectorStack;

// TODO: candidate reduction
// TODO: sudoku solver/strategy

pub fn candidate_reduction(base: usize, list_of_candidates: Vec<Vec<i32>>) {
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
    // let mut search = one_solution_engine();

    let mut statistics = Statistics::new();
    let mut search: AllSolution<Monitor<Statistics,
    OneSolution<_, VectorStack<_>, FDSpace>>> = AllSolution::new(Monitor::new(&mut statistics,
                                                   OneSolution::new(Propagation::new(Brancher::new(FirstSmallestVar, MiddleVal, BinarySplit)))));

    search.start(&space);
    let (frozen_space, status) = search.enter(space);
    let space = frozen_space.unfreeze();

    // Print result.

    dbg!(statistics.num_solution);

    match status {
        Satisfiable => {
            println!("Satisfiable");
            eprintln!("{:#?}", space.vstore.iter().collect::<Vec<_>>());
            /*                // At this stage, dom.lower() == dom.upper().
                            print!("{}, ", dom.lower());*/
        }
        Unsatisfiable => println!("Unsatisfiable"),
        EndOfSearch => println!("Search terminated or was interrupted."),
        Unknown(_) => unreachable!(
            "After the search step, the problem instance should be either satisfiable or unsatisfiable.")
    }
}

pub fn nqueens(n: usize) {
    let mut space = FDSpace::empty();

    let mut queens = vec![];
    // 2 queens can't share the same line.
    for _ in 0..n {
        queens.push(Box::new(space.vstore.alloc(IntervalSet::new(1, n as i32))) as Var<VStore>);
    }
    for i in 0..n - 1 {
        for j in i + 1..n {
            // 2 queens can't share the same diagonal.
            let q1 = (i + 1) as i32;
            let q2 = (j + 1) as i32;
            // Xi + i != Xj + j reformulated as: Xi != Xj + j - i
            space.cstore.alloc(Box::new(XNeqY::new(
                queens[i].bclone(), Box::new(Addition::new(queens[j].bclone(), q2 - q1)) as Var<VStore>)));
            // Xi - i != Xj - j reformulated as: Xi != Xj - j + i
            space.cstore.alloc(Box::new(XNeqY::new(
                queens[i].bclone(), Box::new(Addition::new(queens[j].bclone(), -q2 + q1)) as Var<VStore>)));
        }
    }
    // 2 queens can't share the same column.
    // join_distinct(&mut space.vstore, &mut space.cstore, queens);
    space.cstore.alloc(Box::new(Distinct::new(queens)));

    // Search step.
    let mut search = one_solution_engine();
    search.start(&space);
    let (frozen_space, status) = search.enter(space);
    let space = frozen_space.unfreeze();

    // Print result.
    match status {
        Satisfiable => {
            print!("{}-queens problem is satisfiable. The first solution is:\n[", n);
            for dom in space.vstore.iter() {
                // At this stage, dom.lower() == dom.upper().
                print!("{}, ", dom.lower());
            }
            println!("]");
        }
        Unsatisfiable => println!("{}-queens problem is unsatisfiable.", n),
        EndOfSearch => println!("Search terminated or was interrupted."),
        Unknown(_) => unreachable!(
            "After the search step, the problem instance should be either satisfiable or unsatisfiable.")
    }
}

fn main() {
    // nqueens(12);

    candidate_reduction(2, vec![
        vec![1, 2, 3, 4],
        vec![1, 2],
        vec![1, 2, 3],
        vec![1, 2],
    ])
}