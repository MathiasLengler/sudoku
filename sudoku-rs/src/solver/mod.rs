use crate::base::SudokuBase;
use crate::grid::Grid;
use std::fmt::Debug;

pub mod backtracking;
pub mod strategic;

pub mod introspective;

#[cfg(feature = "sat")]
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
pub trait SolutionIter<Base: SudokuBase>: Iterator<Item = Grid<Base>> {}

impl<Base: SudokuBase, S: Iterator<Item = Grid<Base>>> SolutionIter<Base> for S {}

#[cfg(test)]
mod test_util {
    use std::collections::HashSet;

    use crate::base::consts::Base2;
    use crate::base::SudokuBase;
    use crate::grid::Grid;

    use super::*;

    macro_rules! tests_solver_samples {
        ($setup: expr, |$grid:ident| $block:block) => {
            #[test]
            fn test_samples_base_2() {
                $setup;

                let grids = crate::samples::base_2();

                for $grid in grids $block
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

            #[test]
            fn test_samples_solved() {
                $setup;

                let $grid = crate::samples::base_2_solved();

                $block
            }
        };
        (|$grid:ident| $block:block) => {
            tests_solver_samples!((), |$grid| $block);
        };
    }

    pub(crate) use tests_solver_samples;

    pub(crate) fn assert_infallible_solver_single_solution<Base: SudokuBase>(
        mut solver: impl InfallibleSolver<Base>,
        puzzle: &Grid<Base>,
    ) {
        let solution = solver.solve().expect("Solver should return a solution");

        assert_solution(&solution, puzzle);
    }

    pub(crate) fn assert_fallible_solver_single_solution<Base: SudokuBase>(
        mut solver: impl FallibleSolver<Base>,
        puzzle: &Grid<Base>,
    ) {
        let solution = solver
            .try_solve()
            .expect("Solver should not return an error")
            .expect("Solver should return a solution");

        assert_solution(&solution, puzzle);
    }

    pub(crate) fn assert_solution<Base: SudokuBase>(solution: &Grid<Base>, puzzle: &Grid<Base>) {
        assert!(
            solution.is_solved(),
            "The solution should be solved, instead got:\n{solution}"
        );

        for value_pos in puzzle.all_value_positions() {
            let puzzle_value = puzzle[value_pos].value().unwrap();
            let solution_value = solution[value_pos].value().unwrap();
            assert_eq!(
                puzzle_value, solution_value,
                "The returned solution is not a valid solution for the puzzle: different values at {value_pos}"
            );
        }
    }

    pub(crate) fn assert_solution_iter_single_solution<Base: SudokuBase>(
        mut solver: impl SolutionIter<Base>,
        puzzle: &Grid<Base>,
    ) {
        let solution = solver
            .next()
            .expect("Solver should produce at least one solution");

        assert_solution(&solution, puzzle);

        assert!(
            solver.next().is_none(),
            "Solver should produce not more than one solution"
        );
    }

    pub(crate) fn assert_solution_iter_all_solutions_base_2(solver: impl SolutionIter<Base2>) {
        const NUMBER_OF_BASE_2_SOLUTIONS: usize = 288;

        let solutions = solver
            .take(NUMBER_OF_BASE_2_SOLUTIONS + 1)
            .collect::<Vec<_>>();

        assert_eq!(solutions.len(), NUMBER_OF_BASE_2_SOLUTIONS);

        solutions
            .iter()
            .for_each(|solution| assert!(solution.is_solved()));

        let unique_solutions = solutions.into_iter().collect::<HashSet<_>>();

        assert_eq!(unique_solutions.len(), NUMBER_OF_BASE_2_SOLUTIONS);
    }
}
