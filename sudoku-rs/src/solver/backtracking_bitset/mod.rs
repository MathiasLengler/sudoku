//! Adaptation of [tdoku `solver_basic.cc`](https://github.com/t-dillon/tdoku/blob/master/src/solver_basic.cc)

use log::trace;

pub use builder::SolverBuilder;
use group_availability::GroupAvailability;
pub use group_availability::{AvailabilityDenyList, AvailabilityFilter, GroupAvailabilityIndex};

use crate::base::SudokuBase;
use crate::cell::{CandidatesAscIter, CandidatesIterator, CandidatesRandIter};
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
    availability_indexes: Vec<GroupAvailabilityIndex<Base>>,

    /// A list of iterators producing value assignments for each associated `availability_indices`.
    /// Can be inspected with `peek` to infer the current value assignment.
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

// TODO: add constructor/builder for CandidatesRandIter with seed
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

#[cfg(test)]
mod tests {
    use ndarray::Array2;

    use crate::base::consts::*;
    use crate::rng::new_crate_rng_with_seed;
    use crate::solver::test_util::{assert_solve_result, assert_solver_solutions_base_2};

    use super::*;

    #[test]
    fn test_iter_all_solutions() {
        let grid = Grid::<Base2>::new();
        let solver = Solver::new(&grid);

        assert_solver_solutions_base_2(solver);
    }

    #[test]
    fn test_test_iter_all_solutions_rng() {
        let grid = Grid::<Base2>::new();
        let solver = Solver::builder(&grid)
            .rng(new_crate_rng_with_seed(Some(1)))
            .build();

        assert_solver_solutions_base_2(solver);
    }

    #[test]
    fn test_base_2() {
        let grids = crate::samples::base_2();

        for grid in grids {
            let mut solver = Solver::new(&grid);

            let solve_result = solver.try_solve();

            assert_solve_result(solve_result);
        }
    }

    #[test]
    fn test_base_3() {
        let grids = crate::samples::base_3();

        for grid in grids {
            let mut solver = Solver::new(&grid);

            let solve_result = solver.try_solve();

            assert_solve_result(solve_result);
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

        let mut solver = Solver::new(&grid);
        let solve_result = solver.try_solve();
        assert_solve_result(solve_result);

        assert!(solver.try_solve().is_none());
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
}
