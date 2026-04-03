use std::fmt::Debug;

use crate::base::SudokuBase;
use crate::error::Result;
use crate::grid::Grid;

pub mod backtracking;
pub mod strategic;

pub mod introspective;

pub mod sat;

// TODO: exact-cover based solver:
//  Could be more performant for bigger sudokus
//  References:
//  Section 4.4 Sequential DLX algorithm (0.3 seconds?)
//   https://github.com/huaminghuangtw/Parallel-Sudoku-Solver/blob/master/Project_Report.pdf
//  Rust exact cover solver
//   https://github.com/jw013/exact-cover-rs

// TODO: change solution type
//  Grid<Base> => Grid<Base, Value<Base>>
//  Pros:
//   solutions are smaller
//   solutions are guaranteed to contain only values, no empty/candidates cells
//   no confusion about fix/unfixed values
//   grid could provide solution-specific helpers
//    is_solution_for(grid: Grid<Base>)
//    is_valid_(solution)
//  Cons:
//   requires more conversions/allocations?
//    evaluate usages of solutions
//   many Grid methods are unavailable
//    implement more generally where necessary

/// A infallible sudoku solver.
pub trait InfallibleSolver<Base: SudokuBase> {
    /// Attempt to find a single solution.
    ///
    /// # Returns
    ///
    /// - `Some(solution)` if the solver found a solution.
    /// - `None` if the solver found no solution.
    ///
    /// A solver *may*:
    /// - return the same solution forever
    /// - return all valid solutions for the sudoku in some order, then `None`.
    fn solve(&mut self) -> Option<Grid<Base>>;
}

/// A fallible sudoku solver.
pub trait FallibleSolver<Base: SudokuBase> {
    type Error: Debug;

    /// Attempt to find a single solution.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(solution))` if the solver found a solution.
    /// - `Ok(None)` if the solver found no solution.
    /// - `Err(err)` if the solver encountered an error while solving.
    ///
    /// A solver *may*:
    /// - return the same solution forever
    /// - return all valid solutions for the sudoku in some order, then `None`.
    fn try_solve(&mut self) -> Result<Option<Grid<Base>>, Self::Error>;
}

/// An iterator over solutions.
///
/// Yields solutions as `Grid<Base>`
pub trait InfallibleSolutionIter<Base: SudokuBase>: Iterator<Item = Grid<Base>> {}
impl<Base: SudokuBase, S: Iterator<Item = Grid<Base>>> InfallibleSolutionIter<Base> for S {}

/// An fallible iterator over solutions
///
/// Yields solutions as `Result<Grid<Base>>`
pub trait FallibleSolutionIter<Base: SudokuBase>: Iterator<Item = Result<Grid<Base>>> {}
impl<Base: SudokuBase, S: Iterator<Item = Result<Grid<Base>>>> FallibleSolutionIter<Base> for S {}

#[cfg(test)]
pub(crate) mod test_util {
    use std::collections::HashSet;
    use std::marker::PhantomData;

    use crate::base::consts::Base2;

    use super::*;

    macro_rules! tests_solver_samples {
        ($setup:expr, |$grid:ident| $block:block) => {
            #[test]
            fn test_samples_base_2() {
                $setup;

                let grids = crate::samples::base_2();

                for $grid in grids $block
            }
            #[test]
            fn test_samples_base_2_solved() {
                $setup;

                let $grid = crate::samples::base_2_solved();

                $block
            }
            #[test]
            fn test_samples_base_3() {
                $setup;

                let grids = crate::samples::base_3();

                for $grid in grids $block
            }
            #[cfg(not(debug_assertions))]
            #[test]
            fn test_samples_base_4() {
                $setup;

                let grids = crate::samples::base_4();

                for $grid in grids $block
            }
        };
        (|$grid:ident| $block:block) => {
            tests_solver_samples!((), |$grid| $block);
        };
    }

    pub(crate) use tests_solver_samples;

    pub(crate) fn assert_infallible_solver_single_solution<Base: SudokuBase>(
        solver: &mut impl InfallibleSolver<Base>,
        puzzle: &Grid<Base>,
    ) {
        let solution = solver.solve().expect("Solver should return a solution");

        assert_solution(&solution, puzzle);
    }

    pub(crate) fn assert_fallible_solver_single_solution<Base: SudokuBase>(
        solver: &mut impl FallibleSolver<Base>,
        puzzle: &Grid<Base>,
    ) {
        let solution = solver
            .try_solve()
            .expect("Solver should not return an error")
            .expect("Solver should return a solution");

        assert_solution(&solution, puzzle);
    }

    pub(crate) fn assert_solution<Base: SudokuBase>(solution: &Grid<Base>, puzzle: &Grid<Base>) {
        solution.assert_is_solution_for(puzzle);
    }

    pub(crate) fn assert_infallible_solution_iter_single_solution<Base: SudokuBase>(
        mut solution_iter: impl InfallibleSolutionIter<Base>,
        puzzle: &Grid<Base>,
    ) {
        let solution = solution_iter
            .next()
            .expect("Solution iterator should yield at least one solution");

        assert_solution(&solution, puzzle);

        assert!(
            solution_iter.next().is_none(),
            "Solution iterator should yield not more than one solution"
        );
    }

    pub(crate) fn assert_infallible_solution_iter_all_solutions_base_2(
        solution_iter: impl InfallibleSolutionIter<Base2>,
    ) {
        const NUMBER_OF_BASE_2_SOLUTIONS: usize = 288;

        let solutions = solution_iter
            .take(NUMBER_OF_BASE_2_SOLUTIONS + 1)
            .collect::<Vec<_>>();

        assert_eq!(solutions.len(), NUMBER_OF_BASE_2_SOLUTIONS);

        for solution in &solutions {
            assert!(solution.is_solved());
        }

        let unique_solutions = solutions.into_iter().collect::<HashSet<_>>();

        assert_eq!(unique_solutions.len(), NUMBER_OF_BASE_2_SOLUTIONS);
    }

    pub(crate) fn assert_fallible_solution_iter_as_infallible<Base: SudokuBase>(
        fallible_solution_iter: impl FallibleSolutionIter<Base>,
    ) -> impl InfallibleSolutionIter<Base> {
        struct AssertFallibleSolutionIterAdapter<
            Base: SudokuBase,
            IFallible: FallibleSolutionIter<Base>,
        >
        where
            Base: SudokuBase,
        {
            fallible_solution_iter: IFallible,
            _base: PhantomData<Base>,
        }
        impl<Base: SudokuBase, IFallible: FallibleSolutionIter<Base>> Iterator
            for AssertFallibleSolutionIterAdapter<Base, IFallible>
        where
            Base: SudokuBase,
        {
            type Item = Grid<Base>;

            fn next(&mut self) -> Option<Self::Item> {
                self.fallible_solution_iter.next().map(|solution_res| {
                    solution_res.expect("FallibleSolutionIter should not return an error")
                })
            }
        }

        AssertFallibleSolutionIterAdapter {
            fallible_solution_iter,
            _base: PhantomData,
        }
    }
}
