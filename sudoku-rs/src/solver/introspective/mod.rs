use log::debug;

use crate::base::{DynamicBase, SudokuBase};
use crate::cell::CandidatesAscIter;
use crate::grid::Grid;
use crate::solver::strategic::strategies::DynamicStrategy;
use crate::solver::{backtracking, strategic, FallibleSolver, InfallibleSolver};

#[derive(Debug, Default)]
enum SolverImpl<Base: SudokuBase> {
    #[default]
    Done,
    Strategic(strategic::Solver<Base, Grid<Base>>),
    Backtracking(backtracking::Solver<Base, Grid<Base>, CandidatesAscIter<Base>, ()>),
}

/// A base-introspective solver with focus on performance.
///
/// This is a meta-solver, which delegates the work to `backtracking::Solver` and `strategic::Solver`.
#[derive(Debug)]
pub struct Solver<Base: SudokuBase> {
    solver_impl: SolverImpl<Base>,
}

impl<Base: SudokuBase> Solver<Base> {
    pub fn new(grid: Grid<Base>) -> Self {
        Self {
            solver_impl: match Base::DYNAMIC_BASE {
                // Base 2 and 3 are small enough,
                // that the overhead of the strategy evaluation is slower than the naive backtracking solver.
                DynamicBase::Base2 | DynamicBase::Base3 => {
                    SolverImpl::Backtracking(backtracking::Solver::new(grid))
                }
                // For base >= 4, a hybrid approach of strategic, then backtracking, is faster.
                DynamicBase::Base4 | DynamicBase::Base5 => {
                    SolverImpl::Strategic(strategic::Solver::new_with_strategies(
                        grid,
                        DynamicStrategy::introspective_solver_base_4_plus_strategies(),
                    ))
                }
            },
        }
    }
}

impl<Base: SudokuBase> InfallibleSolver<Base> for Solver<Base> {
    fn solve(&mut self) -> Option<Grid<Base>> {
        let solver_impl = std::mem::take(&mut self.solver_impl);
        match solver_impl {
            SolverImpl::Strategic(mut solver) => {
                if let Ok(Some(strategic_solution)) = solver.try_solve() {
                    // Assumption: when strategic::Solver returns a solution, it is unique.
                    self.solver_impl = SolverImpl::Done;
                    Some(strategic_solution)
                } else {
                    // Strategic solver failed to find solution, but possibly made progress.
                    // Use the mutated grid as a starting point for the backtracking solver.
                    debug!("Strategic solver failed to make progress, switching to backtracking solver");
                    let mut backtracking_solver = backtracking::Solver::new(solver.into_grid());
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

impl<Base: SudokuBase> Iterator for Solver<Base> {
    type Item = Grid<Base>;

    fn next(&mut self) -> Option<Self::Item> {
        self.solve()
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::Base2;
    use crate::solver::test_util::{
        assert_infallible_solver_single_solution, assert_solution_iter_all_solutions_base_2,
        tests_solver_samples,
    };
    use crate::test_util::init_test_logger;

    use super::*;

    #[test]
    fn test_iter_all_solutions() {
        init_test_logger();

        let grid = Grid::<Base2>::new();
        let solver = Solver::new(grid);

        assert_solution_iter_all_solutions_base_2(solver);
    }

    tests_solver_samples! {
        init_test_logger(),
        |grid| {
            let solver = Solver::new(grid.clone());
            assert_infallible_solver_single_solution(solver, &grid);
        }
    }
}
