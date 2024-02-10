//! Adaptation of [tdoku `solver_basic.cc`](https://github.com/t-dillon/tdoku/blob/master/src/solver_basic.cc)

use std::fmt::{Debug, Display, Formatter};
use std::sync::atomic::{AtomicU64, Ordering};

pub use availability_filter::AvailabilityFilter;
pub use availability_filter::DeniedCandidatesGrid;
use log::trace;

pub use builder::SolverBuilder;
use group_availability::GroupAvailability;
pub use group_availability::GroupAvailabilityIndex;

use crate::base::SudokuBase;
use crate::cell::{Candidates, CandidatesAscIter, CandidatesIterator, CandidatesRandIter};
use crate::grid::Grid;
use crate::position::Position;
use crate::rng::CrateRng;

pub(crate) mod availability_filter;
pub(crate) mod group_availability;

#[derive(Debug, Clone)]
pub struct Solver<
    Base: SudokuBase,
    GridRef: AsRef<Grid<Base>>,
    ICandidates: CandidatesIterator<Base>,
    Filter: AvailabilityFilter<Base>,
> {
    /// Grid to be solved
    grid: GridRef,
    /// Cached remaining candidates for each group.
    availability: GroupAvailability<Base, Filter>,
    /// Indexes to non-value cells which must be solved.
    ///
    /// # Invariants
    ///
    /// Length equals `grid.all_candidates_positions().len()`
    availability_indexes: Vec<GroupAvailabilityIndex<Base>>,

    /// A list of iterators producing value assignments for each associated `availability_indices`.
    /// Elements can be inspected with `peek` to infer the current value assignment.
    /// Used as the central backtracking stack.
    ///
    /// # Invariants
    ///
    /// Length less than or equal to `availability_indexes.len()`
    candidates_iters: Vec<ICandidates>,

    candidates_iter_init_context: ICandidates::InitContext,

    pub backtrack_count: u64,

    has_returned_pre_filled_grid_solution: bool,
}

mod builder {
    use super::*;

    #[derive(Debug)]
    pub struct SolverBuilder<
        Base: SudokuBase,
        GridRef: AsRef<Grid<Base>>,
        ICandidates: CandidatesIterator<Base>,
        Filter: AvailabilityFilter<Base>,
    > {
        grid: GridRef,
        availability: GroupAvailability<Base, Filter>,
        candidates_iter_init_context: ICandidates::InitContext,
    }

    impl<Base: SudokuBase, GridRef: AsRef<Grid<Base>>>
        SolverBuilder<Base, GridRef, CandidatesAscIter<Base>, ()>
    {
        pub fn new(grid: GridRef) -> Self {
            Self {
                grid,
                availability: GroupAvailability::all(),
                candidates_iter_init_context: (),
            }
        }
    }

    impl<Base: SudokuBase, GridRef: AsRef<Grid<Base>>, Filter: AvailabilityFilter<Base>>
        SolverBuilder<Base, GridRef, CandidatesAscIter<Base>, Filter>
    {
        // TODO: evaluate rng generic
        //  this could enable the use of Rng references, eliminating the need for `new_crate_rng_from_rng`,
        //  since a single Rng can be shared across solvers.
        /// Visit candidates in a random order, instead of ascending.
        pub fn rng(
            self,
            rng: CrateRng,
        ) -> SolverBuilder<Base, GridRef, CandidatesRandIter<Base>, Filter> {
            let Self {
                grid,
                availability,
                candidates_iter_init_context: (),
            } = self;

            SolverBuilder {
                grid,
                availability,
                candidates_iter_init_context: rng,
            }
        }
    }

    impl<Base: SudokuBase, GridRef: AsRef<Grid<Base>>, ICandidates: CandidatesIterator<Base>>
        SolverBuilder<Base, GridRef, ICandidates, ()>
    {
        /// Filter the available candidates which the solver can use to find a solution.
        pub fn availability_filter<Filter: AvailabilityFilter<Base>>(
            self,
            filter: Filter,
        ) -> SolverBuilder<Base, GridRef, ICandidates, Filter> {
            let Self {
                grid,
                availability,
                candidates_iter_init_context,
            } = self;

            SolverBuilder {
                grid,
                availability: availability.with_filter(filter),
                candidates_iter_init_context,
            }
        }
    }

    impl<
            Base: SudokuBase,
            GridRef: AsRef<Grid<Base>>,
            ICandidates: CandidatesIterator<Base>,
            Filter: AvailabilityFilter<Base>,
        > SolverBuilder<Base, GridRef, ICandidates, Filter>
    {
        pub fn build(self) -> Solver<Base, GridRef, ICandidates, Filter> {
            let SolverBuilder {
                grid,
                availability,
                candidates_iter_init_context,
            } = self;
            Solver::new_with(grid, availability, candidates_iter_init_context)
        }
    }
}

/// Convenience constructors
impl<Base: SudokuBase, GridRef: AsRef<Grid<Base>>>
    Solver<Base, GridRef, CandidatesAscIter<Base>, ()>
{
    pub fn new(grid: GridRef) -> Self {
        SolverBuilder::new(grid).build()
    }

    pub fn builder(grid: GridRef) -> SolverBuilder<Base, GridRef, CandidatesAscIter<Base>, ()> {
        SolverBuilder::new(grid)
    }
}

enum StepResult<Base: SudokuBase> {
    Solution(Grid<Base>),
    NextCell,
    Backtrack,
    Done,
}

enum FueledSolveResult<Base: SudokuBase> {
    OutOfFuel,
    Result(Option<Grid<Base>>),
}

impl<
        Base: SudokuBase,
        GridRef: AsRef<Grid<Base>>,
        ICandidates: CandidatesIterator<Base>,
        Filter: AvailabilityFilter<Base>,
    > Solver<Base, GridRef, ICandidates, Filter>
{
    pub(crate) fn new_with(
        grid: GridRef,
        availability: GroupAvailability<Base, Filter>,
        candidates_iter_init_context: ICandidates::InitContext,
    ) -> Self {
        let mut this = Self {
            grid,
            availability,
            availability_indexes: vec![],
            candidates_iters: vec![],
            candidates_iter_init_context,
            backtrack_count: 0,
            has_returned_pre_filled_grid_solution: false,
        };

        this.initialize();

        this
    }

    fn grid(&self) -> &Grid<Base> {
        self.grid.as_ref()
    }

    fn initialize(&mut self) {
        for pos in Position::<Base>::all() {
            let index: GroupAvailabilityIndex<Base> = pos.into();

            if let Some(value) = self.grid().get(pos).value() {
                // clue, clear group availability
                self.availability.delete(index, value);
            } else {
                // Non-value cell, add to choices
                self.availability_indexes.push(index);
            }
        }

        self.move_best_choice_to_front(0);
        if let Some(availability_index) = self.availability_indexes.first().copied() {
            self.push_candidates_iter(availability_index);
        }
    }

    fn push_candidates_iter(&mut self, availability_index: GroupAvailabilityIndex<Base>) {
        let candidates = self
            .availability
            .available_candidates_at(availability_index);
        self.candidates_iters
            .push(ICandidates::from_candidates_with_init_context(
                candidates,
                &mut self.candidates_iter_init_context,
            ));
    }

    pub fn move_best_choice_to_front(&mut self, front_i: usize) {
        use std::mem::swap;

        debug_assert!(self.candidates_iters.get(front_i).is_none());

        if let Some((first_index, rest)) = self.availability_indexes[front_i..].split_first_mut() {
            let first_count = self
                .availability
                .available_candidates_at(*first_index)
                .count();
            if first_count <= 1 {
                return;
            }

            let mut better_count = first_count;
            let mut better_index = None;

            for next_index in rest {
                if better_count <= 1 {
                    break;
                }
                let next_count = self
                    .availability
                    .available_candidates_at(*next_index)
                    .count();
                if next_count < better_count {
                    better_count = next_count;
                    better_index = Some(next_index);
                }
            }

            if let Some(better_index) = better_index {
                trace!(
                    "swapping {first_count} @ {first_index:?} with {better_count} @ {better_index:?}"
                );
                swap(first_index, better_index);
            }
        }
    }

    fn build_solution_grid(&self) -> Grid<Base> {
        let mut solution_grid = self.grid().clone();
        for (candidates_iter, choice_index) in self
            .candidates_iters
            .iter()
            .zip(self.availability_indexes.iter())
        {
            solution_grid
                .get_mut((*choice_index).into())
                .set_value(candidates_iter.peek().unwrap());
        }
        debug_assert!(solution_grid.is_solved());
        solution_grid
    }

    fn step(&mut self) -> StepResult<Base> {
        if let Some(candidates) = self.candidates_iters.last() {
            if let Some(candidate) = candidates.peek() {
                let choice_index = self.availability_indexes[self.candidates_iters.len() - 1];
                self.availability.delete(choice_index, candidate);

                if self.candidates_iters.len() == self.availability_indexes.len() {
                    // Found solution
                    let solution_grid = self.build_solution_grid();

                    // Continue at next candidate
                    self.candidates_iters.last_mut().unwrap().next();
                    self.availability.insert(choice_index, candidate);

                    StepResult::Solution(solution_grid)
                } else {
                    // Next cell
                    let next_i = self.candidates_iters.len();
                    self.move_best_choice_to_front(next_i);
                    let next_availability_index = self.availability_indexes[next_i];
                    self.push_candidates_iter(next_availability_index);

                    StepResult::NextCell
                }
            } else {
                // Backtrack
                self.backtrack_count += 1;
                self.candidates_iters.pop().unwrap();
                let candidates_iters_len = self.candidates_iters.len();
                if let Some(prev_candidates) = self.candidates_iters.last_mut() {
                    if let Some(prev_candidate) = prev_candidates.peek() {
                        let prev_choice_index = self.availability_indexes[candidates_iters_len - 1];
                        self.availability.insert(prev_choice_index, prev_candidate);
                    }
                    prev_candidates.next();
                }
                StepResult::Backtrack
            }
        } else if self.availability_indexes.is_empty()
            && !self.has_returned_pre_filled_grid_solution
        {
            self.has_returned_pre_filled_grid_solution = true;

            let grid = self.grid();
            if grid.is_solved() {
                StepResult::Solution(grid.clone())
            } else {
                StepResult::Done
            }
        } else {
            StepResult::Done
        }
    }

    fn try_solve_with_fuel(&mut self, fuel: u64) -> FueledSolveResult<Base> {
        for _ in 0..fuel {
            match self.step() {
                StepResult::Solution(solution) => return FueledSolveResult::Result(Some(solution)),
                StepResult::Done => return FueledSolveResult::Result(None),
                _ => {}
            }
        }

        FueledSolveResult::OutOfFuel
    }

    pub fn try_solve(&mut self) -> Option<Grid<Base>> {
        loop {
            match self.step() {
                StepResult::Solution(solution) => return Some(solution),
                StepResult::Done => return None,
                _ => {}
            }
        }
    }
}

impl<
        Base: SudokuBase,
        GridRef: AsRef<Grid<Base>>,
        ICandidates: CandidatesIterator<Base>,
        Filter: AvailabilityFilter<Base>,
    > Iterator for Solver<Base, GridRef, ICandidates, Filter>
{
    type Item = Grid<Base>;

    fn next(&mut self) -> Option<Self::Item> {
        self.try_solve()
    }
}

impl<
        Base: SudokuBase,
        GridRef: AsRef<Grid<Base>>,
        ICandidates: CandidatesIterator<Base>,
        Filter: AvailabilityFilter<Base>,
    > Display for Solver<Base, GridRef, ICandidates, Filter>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use tabled::{Table, Tabled};

        #[derive(Tabled)]
        struct BacktrackingStackEntry<Base: SudokuBase, ICandidates: Display> {
            // From availability_indexes
            pos: Position<Base>,
            // From candidates_iters
            candidates: ICandidates,
        }

        let Self {
            // TODO
            availability,
            availability_indexes,
            candidates_iters,
            backtrack_count,
            has_returned_pre_filled_grid_solution,
            ..
        } = self;

        write!(f, "backtracking::Solver:\nGrid:\n{}\n", self.grid())?;

        let mut availability_preview_grid = self.grid().clone();
        for &index in availability_indexes {
            let candidates = availability.available_candidates_at(index);
            availability_preview_grid[index.into()].set_candidates(candidates);
        }

        write!(f, "Availability grid:\n{availability_preview_grid}\n")?;

        let backtracking_stack = std::iter::zip(availability_indexes, candidates_iters).map(
            |(&availability_index, candidates)| BacktrackingStackEntry {
                pos: availability_index.into(),
                candidates,
            },
        );

        let backtracking_stack_table = Table::new(backtracking_stack);

        write!(f, "{backtracking_stack_table}\nbacktrack_count: {backtrack_count}, has_returned_pre_filled_grid_solution: {has_returned_pre_filled_grid_solution}")
    }
}

pub static SPLIT_COUNT: AtomicU64 = AtomicU64::new(0);

impl<Base: SudokuBase, GridRef: AsRef<Grid<Base>>, ICandidates: CandidatesIterator<Base>>
    Solver<Base, GridRef, ICandidates, DeniedCandidatesGrid<Base>>
where
    Self: Clone,
{
    // TODO: measure solution space split fairness
    //  for efficient parallelization both solvers should search the approx. same solution space.

    // Alternative implementation without Filter: DeniedCandidatesGrid
    //  find split cell with least candidates >= 2
    //  candidates.len() cloned solvers:
    //   copy grid, set values for each candidates_iter.peek, select on candidate from split cell, set as value in cloned grid, re-initialize solver

    // FIXME: this estimate is hilariously inaccurate
    fn estimate_search_space(&self) -> u64 {
        let availability_indexes_to_be_solved =
            &self.availability_indexes[self.candidates_iters.len()..];

        let total_candidates_iter_len: u64 = self
            .candidates_iters
            .iter()
            .map(|candidates_iter| candidates_iter.len() as u64)
            .sum();

        dbg!(total_candidates_iter_len);

        let total_available_candidates: u64 = availability_indexes_to_be_solved
            .iter()
            .map(|&availability_index| -> u64 {
                self.availability
                    .available_candidates_at(availability_index)
                    .count()
                    .into()
            })
            .sum();

        dbg!(total_available_candidates);

        total_candidates_iter_len + total_available_candidates
    }

    /// Split the current solver into two.
    ///
    /// The two solvers will search distinct solution spaces of the same sudoku.
    /// Each solver will produce unique solutions in its search space.
    ///
    /// Used for parallel solving.
    pub fn split(self) -> (Self, Option<Self>) {
        SPLIT_COUNT.fetch_add(1, Ordering::Release);

        // Heuristic: is it worth it to split the solver?
        // let estimated_search_space = self.estimate_search_space();
        //
        // dbg!(estimated_search_space);

        // Find yet to be solved index with at least two available candidates
        let candidates_iters_len = self.candidates_iters.len();
        let availability_indexes_to_be_solved = &self.availability_indexes[candidates_iters_len..];

        let Some((split_availability_index, split_available_candidates, _)) =
            availability_indexes_to_be_solved
                .iter()
                .filter_map(|&availability_index| {
                    let available_candidates = self
                        .availability
                        .available_candidates_at(availability_index);
                    let available_candidates_count = available_candidates.count();
                    if available_candidates_count > 1 {
                        Some((
                            availability_index,
                            available_candidates,
                            available_candidates_count,
                        ))
                    } else {
                        None
                    }
                })
                .min_by_key(|(_, _, available_candidates_count)| *available_candidates_count)
        else {
            return (self, None);
        };

        let available_candidates_vec = split_available_candidates.to_vec_value();

        // We have at least two candidates
        debug_assert!(available_candidates_vec.len() >= 2);

        let (left_values, right_values) =
            available_candidates_vec.split_at(available_candidates_vec.len() / 2);

        // Both halves have at least on candidate
        debug_assert!(!left_values.is_empty());
        debug_assert!(!right_values.is_empty());

        let left_candidates: Candidates<Base> = left_values.iter().copied().collect();
        let right_candidates: Candidates<Base> = right_values.iter().copied().collect();

        let mut left = self;
        let mut right = left.clone();

        // Remove availability for each other
        let pos = Position::from(split_availability_index);
        left.availability.filter[pos] = left.availability.filter[pos].union(right_candidates);
        right.availability.filter[pos] = right.availability.filter[pos].union(left_candidates);

        left.backtrack_count = 0;
        right.backtrack_count = 0;

        (left, Some(right))
    }
}

#[cfg(feature = "parallel")]
mod parallel {
    use std::cmp;

    use rayon::iter::{split, Split};
    use rayon::prelude::*;

    use super::*;

    impl<Base: SudokuBase, GridRef: AsRef<Grid<Base>>, ICandidates: CandidatesIterator<Base>>
        IntoParallelIterator for Solver<Base, GridRef, ICandidates, DeniedCandidatesGrid<Base>>
    where
        Self: Clone + Send,
    {
        type Iter = Split<Self, fn(Self) -> (Self, Option<Self>)>;
        type Item = Self;

        fn into_par_iter(self) -> Self::Iter {
            split(self, Solver::split)
        }
    }

    impl<Base: SudokuBase, GridRef: AsRef<Grid<Base>>, ICandidates: CandidatesIterator<Base>>
        Solver<Base, GridRef, ICandidates, DeniedCandidatesGrid<Base>>
    where
        Self: Clone + Send,
    {
        // TODO: find root cause for sporadic crash:
        //  thread '<unknown>' has overflowed its stack
        // Assumption: way too aggressive splitting
        //  also kills perf

        // Possible solution:
        //  switch to lower level rayon API
        //  we could provide hints about the size of the search space
        //  the current implementation using split has no knowledge about the search space.
        //  This could result in lop-sided splitting, resulting in overhead.

        // Possible solution:
        //  Refuse to split if search space is too small
        //   unclear how to estimate the size of the search space

        // Possible solution:
        //  interrupt iteration
        //  a single .next call can take an arbitrary amount of time.
        //  if the is a upper limit, we could identify slow solvers and split them further
        //  Better: adaptive splitting:
        //   - start with a few/a single solver
        //   - search with fuel amount X
        //   - if found solution, ret
        //   - if found no solution left, ret
        //   - if out of fuel, split and run in parallel

        pub fn has_any_solution(self) -> bool {
            self.any_solution().is_some()
        }

        pub fn any_solution(self) -> Option<Grid<Base>> {
            // self.any_solution_sequential()
            // self.any_solution_split_par_iter_split()
            self.any_solution_pre_split()
            // self.any_solution_pre_split_histogram()
            // self.any_solution_fuel_join_recursive()
        }

        fn any_solution_sequential(mut self) -> Option<Grid<Base>> {
            self.next()
        }

        fn any_solution_split_par_iter_split(self) -> Option<Grid<Base>> {
            self.into_par_iter()
                .find_map_any(|mut solver| solver.next())
        }

        fn pre_split_solvers(self) -> Vec<Self> {
            let mut split_solvers = vec![self];

            // TODO: expose initial parallel factor
            //  define for each base?
            // current value tuned for Base4
            // seq: 314ms => any_solution_pre_split: 54.5ms
            for _ in 0..12 {
                split_solvers = split_solvers
                    .into_iter()
                    .flat_map(|solver| {
                        let (left, right) = solver.split();
                        [Some(left), right]
                    })
                    .flatten()
                    .collect();
            }
            split_solvers
        }

        fn any_solution_pre_split(self) -> Option<Grid<Base>> {
            let split_solvers = self.pre_split_solvers();
            split_solvers
                .into_par_iter()
                .find_map_any(|mut solver| solver.next())
        }

        #[cfg(feature = "histogram")]
        fn any_solution_pre_split_histogram(self) -> Option<Grid<Base>> {
            use hdrhistogram::Histogram;

            let split_solvers = self.pre_split_solvers();

            let mut histogram = Histogram::<u64>::new(3).unwrap().into_sync();

            let res = split_solvers
                .into_par_iter()
                .map_with(histogram.recorder(), |recorder, mut solver| {
                    // let clone = solver.clone();

                    let ret = solver.next();
                    recorder.record(solver.backtrack_count).unwrap();
                    // if solver.backtrack_count <= 2 {
                    //     println!("solver with two guesses: {clone}");
                    //
                    //     std::process::exit(1);
                    // }

                    ret
                })
                .find_map_any(|res| res);

            histogram.refresh();

            for v in histogram.iter_recorded() {
                println!("{}: {}", v.value_iterated_to(), v.count_at_value());
            }

            res
        }

        fn any_solution_fuel_join_recursive(mut self) -> Option<Grid<Base>> {
            const FUEL: u64 = 20_000_000;

            // Fork of: rayon-1.8.0/src/iter/plumbing/mod.rs:256
            /// A splitter controls the policy for splitting into smaller work items.
            ///
            /// Thief-splitting is an adaptive policy that starts by splitting into
            /// enough jobs for every worker thread, and then resets itself whenever a
            /// job is actually stolen into a different thread.
            #[derive(Clone, Copy)]
            struct Splitter {
                /// The `splits` tell us approximately how many remaining times we'd
                /// like to split this job.  We always just divide it by two though, so
                /// the effective number of pieces will be `next_power_of_two()`.
                splits: usize,
            }

            impl Splitter {
                #[inline]
                fn new() -> Splitter {
                    Splitter {
                        splits: rayon::current_num_threads(),
                    }
                }

                #[inline]
                fn try_split(&mut self, stolen: bool) -> bool {
                    let Splitter { splits } = *self;

                    if stolen {
                        // This job was stolen!  Reset the number of desired splits to the
                        // thread count, if that's more than we had remaining anyway.
                        self.splits = cmp::max(rayon::current_num_threads(), self.splits / 2);
                        true
                    } else if splits > 0 {
                        // We have splits remaining, make it so.
                        self.splits /= 2;
                        true
                    } else {
                        // Not stolen, and no more splits -- we're done!
                        false
                    }
                }
            }

            fn inner<
                Base: SudokuBase,
                GridRef: AsRef<Grid<Base>>,
                ICandidates: CandidatesIterator<Base>,
            >(
                mut this: Solver<Base, GridRef, ICandidates, DeniedCandidatesGrid<Base>>,
                migrated: bool,
                mut splitter: Splitter,
            ) -> Option<Grid<Base>>
            where
                Solver<Base, GridRef, ICandidates, DeniedCandidatesGrid<Base>>: Clone + Send,
            {
                loop {
                    return match this.try_solve_with_fuel(FUEL) {
                        FueledSolveResult::OutOfFuel => {
                            if splitter.try_split(migrated) {
                                let (left, right) = this.split();
                                if let Some(right) = right {
                                    let (left_res, right_res) = rayon::join_context(
                                        |context| inner(left, context.migrated(), splitter),
                                        |context| inner(right, context.migrated(), splitter),
                                    );

                                    left_res.or(right_res)
                                } else {
                                    this = left;
                                    continue;
                                }
                            } else {
                                continue;
                            }
                        }
                        FueledSolveResult::Result(opt_grid) => opt_grid,
                    };
                }
            }

            inner(self, false, Splitter::new())
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO: test filled sudoku with conflict
    // TODO: test filled sudoku without conflict
    // TODO: test partial filled sudoku without conflict and no possible solution
    // TODO: test partial filled sudoku without conflict and one possible solution
    // TODO: test partial filled sudoku without conflict and multiple possible solutions
    // TODO: test partial filled sudoku with conflict (implies no solutions)

    use itertools::chain;

    use crate::base::consts::*;
    use crate::rng::new_crate_rng_with_seed;
    use crate::solver::test_util::{
        assert_solver_all_solutions_base_2, assert_solver_single_solution,
    };

    use super::*;

    #[test]
    fn test_iter_all_solutions() {
        let grid = Grid::<Base2>::new();
        let solver = Solver::new(&grid);

        assert_solver_all_solutions_base_2(solver);
    }

    #[test]
    fn test_test_iter_all_solutions_rng() {
        let grid = Grid::<Base2>::new();
        let solver = Solver::builder(&grid)
            .rng(new_crate_rng_with_seed(Some(1)))
            .build();

        assert_solver_all_solutions_base_2(solver);
    }

    #[test]
    fn test_base_2() {
        let grids = crate::samples::base_2();

        for grid in grids {
            let solver = Solver::new(&grid);
            assert_solver_single_solution(solver);
        }
    }

    #[test]
    fn test_base_3() {
        let grids = crate::samples::base_3();

        for grid in grids {
            let solver = Solver::new(&grid);
            assert_solver_single_solution(solver);
        }
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn test_base_4() {
        let grids = crate::samples::base_4();

        for grid in grids {
            let solver = Solver::new(&grid);
            assert_solver_single_solution(solver);
        }
    }

    #[test]
    fn test_solved() {
        let grid = crate::samples::base_2_solved();

        let solver = Solver::new(&grid);
        assert_solver_single_solution(solver);
    }

    #[test]
    fn test_move_best_choice_to_front() {
        let mut grid = crate::samples::base_2()[1].clone();
        grid.set_all_direct_candidates();
        let mut solver = Solver::new(&grid);
        let mut expected_choice_indexes = vec![
            (0, 3),
            (0, 1),
            (1, 0),
            (1, 1),
            (1, 2),
            (1, 3),
            (2, 0),
            (2, 1),
            (2, 2),
            (2, 3),
            (3, 0),
            (3, 2),
        ]
        .into_iter()
        .map(|(row, column)| {
            GroupAvailabilityIndex::<Base2>::from(Position::try_from((row, column)).unwrap())
        })
        .collect::<Vec<_>>();
        assert_eq!(solver.availability_indexes, expected_choice_indexes);

        solver.move_best_choice_to_front(4);
        expected_choice_indexes.swap(4, 11);
        assert_eq!(solver.availability_indexes, expected_choice_indexes);
    }

    #[test]
    fn test_filter_denylist() {
        type Base = Base2;

        let grid = Grid::<Base>::new();
        let mut denylist = Grid::new();
        denylist[Position::default()] = vec![1, 3]
            .into_iter()
            .map(|v| v.try_into().unwrap())
            .collect();
        let solver = Solver::builder(&grid).availability_filter(denylist).build();

        for solution in solver.clone() {
            assert!(![1, 3].contains(&solution.get(Position::default()).value().unwrap().get()));
        }

        assert_eq!(solver.count(), 144);
    }

    fn assert_single_solution_with_split<Base: SudokuBase>(
        grid: &Grid<Base>,
        assert_is_splittable: bool,
    ) {
        let solver = Solver::builder(grid)
            .availability_filter(Grid::new())
            .build();

        let (left_solver, Some(right_solver)) = solver.split() else {
            if assert_is_splittable {
                panic!("Solver should be splittable")
            } else {
                return;
            }
        };

        // Both solvers chained together should still produce a single solution
        assert_solver_single_solution(left_solver.chain(right_solver));
    }

    #[test]
    fn test_split_sample_base_2() {
        for grid in crate::samples::base_2() {
            // Some base 2 sample grids contain only single candidate cells, which currently can't be split.
            assert_single_solution_with_split(&grid, false);
        }
    }

    #[test]
    fn test_split_sample_base_3() {
        for grid in crate::samples::base_3() {
            assert_single_solution_with_split(&grid, true);
        }
    }

    #[test]
    fn test_split_all_base_2() {
        type Base = Base2;

        let grid = Grid::<Base>::new();
        let solver = Solver::builder(grid)
            .availability_filter(Grid::new())
            .build();

        let (left_solver, Some(right_solver)) = solver.split() else {
            panic!("Solver should be splittable")
        };

        assert_solver_all_solutions_base_2(left_solver.chain(right_solver));
    }

    #[test]
    fn test_split_twice() {
        type Base = Base2;

        let grid = Grid::<Base>::new();
        let solver = Solver::builder(grid)
            .availability_filter(Grid::new())
            .build();

        let (l, Some(r)) = solver.split() else {
            panic!("Solver should be splittable")
        };

        let (ll, Some(lr)) = l.split() else {
            panic!("Solver should be splittable")
        };

        assert_solver_all_solutions_base_2(chain!(ll, lr, r,));
    }

    #[test]
    fn test_split_recursive() {
        type Base = Base2;

        let grid = Grid::<Base>::new();
        let solver = Solver::builder(grid)
            .availability_filter(Grid::new())
            .build();

        let mut split_solvers = vec![solver];

        for _ in 0..10 {
            split_solvers = split_solvers
                .into_iter()
                .flat_map(|solver| {
                    let (left, right) = solver.split();
                    [Some(left), right]
                })
                .flatten()
                .collect();
        }

        assert_solver_all_solutions_base_2(split_solvers.into_iter().flatten());
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn test_par_iter() {
        type Base = Base2;

        use rayon::prelude::*;

        let grid = Grid::<Base>::new();
        let solver = Solver::builder(grid)
            .availability_filter(Grid::new())
            .build();

        assert_solver_all_solutions_base_2(
            solver
                .into_par_iter()
                .flat_map_iter(|solver| solver)
                .collect::<Vec<_>>()
                .into_iter(),
        );
    }
}
