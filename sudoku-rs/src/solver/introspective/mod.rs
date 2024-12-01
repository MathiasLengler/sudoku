use crate::base::{BaseEnum, SudokuBase};
use crate::cell::CandidatesAscIter;
use crate::grid::Grid;
use crate::solver::backtracking::CandidatesFilter;
use crate::solver::sat;
use crate::solver::{backtracking, InfallibleSolver};

#[derive(Debug, Default)]
enum SolverImpl<Base: SudokuBase, GridRef: AsRef<Grid<Base>>, Filter: CandidatesFilter<Base>> {
    #[default]
    Done,
    Sat(sat::SolverIter<Base>),
    Backtracking(backtracking::Solver<Base, GridRef, CandidatesAscIter<Base>, Filter>),
}

/// A base-introspective solver with focus on performance.
///
/// This is a meta-solver, which delegates the work to the other solvers.
#[derive(Debug)]
pub struct Solver<Base: SudokuBase, GridRef: AsRef<Grid<Base>>, Filter: CandidatesFilter<Base>> {
    solver_impl: SolverImpl<Base, GridRef, Filter>,
}

impl<Base: SudokuBase, GridRef: AsRef<Grid<Base>>> Solver<Base, GridRef, ()> {
    pub fn new(grid: GridRef) -> Self {
        Self::new_with_filter(grid, ())
    }
}

impl<Base: SudokuBase, GridRef: AsRef<Grid<Base>>, Filter: CandidatesFilter<Base>>
    Solver<Base, GridRef, Filter>
{
    pub fn new_with_filter(grid: GridRef, filter: Filter) -> Self {
        match Base::ENUM {
            // Base 2 and 3 are small enough,
            // that the overhead of the strategy evaluation is slower than the naive backtracking solver.
            BaseEnum::Base2 | BaseEnum::Base3 => Self {
                solver_impl: SolverImpl::Backtracking(
                    backtracking::Solver::builder(grid)
                        .candidates_filter(filter)
                        .build(),
                ),
            },
            // For base >= 4, sat solver is faster
            BaseEnum::Base4 | BaseEnum::Base5 => Self {
                solver_impl: SolverImpl::Sat(
                    sat::Solver::new_with_candidates_filter(grid, &filter).into_iter(),
                ),
            },
        }
    }
}

impl<Base: SudokuBase, GridRef: AsRef<Grid<Base>>, Filter: CandidatesFilter<Base>>
    InfallibleSolver<Base> for Solver<Base, GridRef, Filter>
{
    fn solve(&mut self) -> Option<Grid<Base>> {
        let solver_impl = std::mem::take(&mut self.solver_impl);
        match solver_impl {
            SolverImpl::Backtracking(mut solver) => {
                let res = solver.next();
                self.solver_impl = SolverImpl::Backtracking(solver);
                res
            }
            SolverImpl::Sat(mut solver) => {
                let res = solver.next();
                self.solver_impl = SolverImpl::Sat(solver);
                res.transpose().unwrap()
            }
            SolverImpl::Done => None,
        }
    }
}

impl<Base: SudokuBase, GridRef: AsRef<Grid<Base>>, Filter: CandidatesFilter<Base>> Iterator
    for Solver<Base, GridRef, Filter>
{
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
