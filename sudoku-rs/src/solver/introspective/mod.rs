use log::debug;

use crate::base::{DynamicBase, SudokuBase};
use crate::cell::CandidatesAscIter;
use crate::grid::Grid;
use crate::solver::backtracking::AvailabilityFilter;
use crate::solver::strategic::strategies::DynamicStrategy;
use crate::solver::{backtracking, strategic, FallibleSolver, InfallibleSolver};

#[derive(Debug, Default)]
enum SolverImpl<Base: SudokuBase, Filter: AvailabilityFilter<Base>> {
    #[default]
    Done,
    Strategic(strategic::Solver<Base, Grid<Base>>, Filter),
    Backtracking(backtracking::Solver<Base, Grid<Base>, CandidatesAscIter<Base>, Filter>),
}

/// A base-introspective solver with focus on performance.
///
/// This is a meta-solver, which delegates the work to `backtracking::Solver` and `strategic::Solver`.
#[derive(Debug)]
pub struct Solver<Base: SudokuBase, Filter: AvailabilityFilter<Base>> {
    solver_impl: SolverImpl<Base, Filter>,
}

impl<Base: SudokuBase> Solver<Base, ()> {
    pub fn new(grid: Grid<Base>) -> Self {
        Self::new_with_filter(grid, ())
    }
}

impl<Base: SudokuBase, Filter: AvailabilityFilter<Base>> Solver<Base, Filter> {
    pub fn new_with_filter(grid: Grid<Base>, filter: Filter) -> Self {
        match Base::DYNAMIC_BASE {
            // Base 2 and 3 are small enough,
            // that the overhead of the strategy evaluation is slower than the naive backtracking solver.
            DynamicBase::Base2 | DynamicBase::Base3 => Self {
                solver_impl: SolverImpl::Backtracking(
                    backtracking::Solver::builder(grid)
                        .availability_filter(filter)
                        .build(),
                ),
            },
            // For base >= 4, a hybrid approach of strategic, then backtracking, is faster.
            DynamicBase::Base4 | DynamicBase::Base5 => Self {
                solver_impl: SolverImpl::Strategic(
                    strategic::Solver::builder(grid)
                        .strategies(DynamicStrategy::introspective_solver_base_4_plus_strategies())
                        .availability_filter(&filter)
                        .build(),
                    filter,
                ),
            },
        }
    }
}

impl<Base: SudokuBase, Filter: AvailabilityFilter<Base>> InfallibleSolver<Base>
    for Solver<Base, Filter>
{
    fn solve(&mut self) -> Option<Grid<Base>> {
        let solver_impl = std::mem::take(&mut self.solver_impl);
        match solver_impl {
            SolverImpl::Strategic(mut solver, filter) => {
                if let Ok(Some(strategic_solution)) = solver.try_solve() {
                    // TODO: does this assumption hold for ambiguous grids?
                    // Assumption: when strategic::Solver returns a solution, it is unique.
                    self.solver_impl = SolverImpl::Done;
                    Some(strategic_solution)
                } else {
                    debug!("Strategic solver failed to make progress, switching to backtracking solver");
                    // Use the mutated grid as a starting point for the backtracking solver.
                    let mut backtracking_solver = backtracking::Solver::builder(solver.into_grid())
                        .availability_filter(filter)
                        .build();
                    let res = backtracking_solver.next();
                    self.solver_impl = SolverImpl::Backtracking(backtracking_solver);
                    res
                }
            }
            SolverImpl::Backtracking(mut solver) => {
                let res = solver.next();
                self.solver_impl = SolverImpl::Backtracking(solver);
                res
            }
            SolverImpl::Done => None,
        }
    }
}

impl<Base: SudokuBase, Filter: AvailabilityFilter<Base>> Iterator for Solver<Base, Filter> {
    type Item = Grid<Base>;

    fn next(&mut self) -> Option<Self::Item> {
        self.solve()
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::Base2;
    use crate::solver::test_util::{
        assert_infallible_solution_iter_all_solutions_base_2,
        assert_infallible_solver_single_solution, tests_solver_samples,
    };
    use crate::test_util::init_test_logger;

    use super::*;

    tests_solver_samples! {
        init_test_logger(),
        |grid| {
            let solver = Solver::new(grid.clone());
            assert_infallible_solver_single_solution(solver, &grid);
        }
    }

    mod samples {
        use super::*;

        use crate::solver::test_util::tests_solver_samples;

        mod infallible_solver {
            use super::*;
            tests_solver_samples! {
                init_test_logger(),
                |grid| {
                    let solver = Solver::new(grid.clone());
                    assert_infallible_solver_single_solution(solver, &grid);
                }
            }
        }

        mod infallible_solution_iter {
            use super::*;
            use crate::solver::test_util::assert_infallible_solution_iter_single_solution;
            tests_solver_samples! {
                |grid| {
                    let solver = Solver::new(grid.clone());
                    assert_infallible_solution_iter_single_solution(solver, &grid);
                }
            }
        }
    }

    #[test]
    fn test_iter_all_solutions() {
        init_test_logger();

        let grid = Grid::<Base2>::new();
        let solver = Solver::new(grid);

        assert_infallible_solution_iter_all_solutions_base_2(solver);
    }
}
