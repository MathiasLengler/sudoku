//! Adaptation of [tdoku `solver_basic.cc`](https://github.com/t-dillon/tdoku/blob/master/src/solver_basic.cc)

use std::fmt::Debug;

use log::trace;

pub use builder::SolverBuilder;
use group_availability::GroupAvailability;
pub use group_availability::{AvailabilityDenyList, AvailabilityFilter, GroupAvailabilityIndex};

use crate::base::SudokuBase;
use crate::cell::{Candidates, CandidatesAscIter, CandidatesIterator, CandidatesRandIter};
use crate::grid::Grid;
use crate::position::Position;
use crate::rng::CrateRng;

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

    pub guess_count: u64,

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
        /// Visit candidates in a random order, instead of ascending.
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
            guess_count: 0,
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
        solution_grid
    }

    pub fn try_solve(&mut self) -> Option<Grid<Base>> {
        while let Some(candidates) = self.candidates_iters.last() {
            if let Some(candidate) = candidates.peek() {
                // TODO: only update if there are multiple candidates
                self.guess_count += 1;

                let choice_index = self.availability_indexes[self.candidates_iters.len() - 1];
                self.availability.delete(choice_index, candidate);

                if self.candidates_iters.len() == self.availability_indexes.len() {
                    // Found solution
                    let solution_grid = self.build_solution_grid();

                    // Continue at next candidate
                    self.candidates_iters.last_mut().unwrap().next();
                    self.availability.insert(choice_index, candidate);

                    return Some(solution_grid);
                } else {
                    // Next cell
                    let next_i = self.candidates_iters.len();
                    self.move_best_choice_to_front(next_i);
                    let next_availability_index = self.availability_indexes[next_i];
                    self.push_candidates_iter(next_availability_index);
                }
            } else {
                // Backtrack
                self.candidates_iters.pop().unwrap();
                let candidates_iters_len = self.candidates_iters.len();
                if let Some(prev_candidates) = self.candidates_iters.last_mut() {
                    if let Some(prev_candidate) = prev_candidates.peek() {
                        let prev_choice_index = self.availability_indexes[candidates_iters_len - 1];
                        self.availability.insert(prev_choice_index, prev_candidate);
                    }

                    prev_candidates.next();
                }
            }
        }

        if self.availability_indexes.is_empty() && !self.has_returned_pre_filled_grid_solution {
            self.has_returned_pre_filled_grid_solution = true;

            let grid = self.grid();
            if grid.is_solved() {
                Some(grid.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<Base: SudokuBase, GridRef: AsRef<Grid<Base>>, ICandidates: CandidatesIterator<Base>>
    Solver<Base, GridRef, ICandidates, AvailabilityDenyList<Base>>
where
    Self: Clone,
{
    // TODO: measure solution space split fairness
    //  for efficient parallelization both solvers should search the approx. same solution space.

    /// Split the current solver into two.
    ///
    /// The two solvers will search distinct solution spaces of the same sudoku.
    /// Each solver will produce unique solutions in its search space.
    ///
    /// Used for parallel solving.
    pub fn split(self) -> (Self, Option<Self>) {
        // Find yet to be solved index with at least two available candidates
        let candidates_iters_len = self.candidates_iters.len();
        let Some((split_availability_index, split_available_candidates, _)) = self
            .availability_indexes[candidates_iters_len..]
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
            // TODO: benchmark: min vs max while generating
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

        let cell_index = usize::from(Position::from(split_availability_index).cell_index());
        let left_filter_slice = left.availability.filter.as_slice_mut().unwrap();
        left_filter_slice[cell_index] = left_filter_slice[cell_index].union(right_candidates);

        let right_filter_slice = right.availability.filter.as_slice_mut().unwrap();
        right_filter_slice[cell_index] = right_filter_slice[cell_index].union(left_candidates);

        (left, Some(right))
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

#[cfg(feature = "parallel")]
mod parallel {
    use rayon::iter::{split, Split};
    use rayon::prelude::*;

    use super::*;

    impl<Base: SudokuBase, GridRef: AsRef<Grid<Base>>, ICandidates: CandidatesIterator<Base>>
        IntoParallelIterator for Solver<Base, GridRef, ICandidates, AvailabilityDenyList<Base>>
    where
        Self: Clone + Send,
    {
        type Iter = Split<Self, fn(Self) -> (Self, Option<Self>)>;
        type Item = Self;

        fn into_par_iter(self) -> Self::Iter {
            split(self, Solver::split)
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
    use ndarray::Array2;

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
    fn test_solved() {
        let grid = crate::samples::base_2_solved();

        let solver = Solver::new(&grid);
        assert_solver_single_solution(solver);
    }

    #[test]
    fn test_filter_denylist() {
        type Base = Base2;

        let grid = Grid::<Base>::new();
        let mut denylist = Array2::default((Base::SIDE_LENGTH.into(), Base::SIDE_LENGTH.into()));
        denylist[(0, 0)] = vec![1, 3]
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
            .availability_filter(Array2::default((
                Base::SIDE_LENGTH.into(),
                Base::SIDE_LENGTH.into(),
            )))
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
            .availability_filter(Array2::default((
                Base::SIDE_LENGTH.into(),
                Base::SIDE_LENGTH.into(),
            )))
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
            .availability_filter(Array2::default((
                Base::SIDE_LENGTH.into(),
                Base::SIDE_LENGTH.into(),
            )))
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
            .availability_filter(Array2::default((
                Base::SIDE_LENGTH.into(),
                Base::SIDE_LENGTH.into(),
            )))
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
            .availability_filter(Array2::default((
                Base::SIDE_LENGTH.into(),
                Base::SIDE_LENGTH.into(),
            )))
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
