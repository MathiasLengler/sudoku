use crate::base::SudokuBase;
use crate::cell::CandidatesAscIter;
use crate::grid::Grid;
use crate::solver::strategic::strategies::DynamicStrategy;
use crate::solver::{backtracking, strategic};
use log::debug;

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
            solver_impl: match Base::BASE {
                // Base 2 and 3 are small enough,
                // that the overhead of the strategy evaluation is slower than the naive backtracking solver.
                2 | 3 => SolverImpl::Backtracking(backtracking::Solver::new(grid)),
                // For base >= 4, a hybrid approach of strategic, then backtracking, is faster.
                4 | 5 => SolverImpl::Strategic(strategic::Solver::new_with_strategies(
                    grid,
                    DynamicStrategy::introspective_solver_base_4_plus_strategies(),
                )),
                unexpected_base => panic!("Unexpected base: {unexpected_base}"),
            },
        }
    }

    pub fn try_solve(&mut self) -> Option<Grid<Base>> {
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
        self.try_solve()
    }
}

#[cfg(test)]
mod tests {
    use env_logger::Env;

    use crate::base::consts::Base2;
    use crate::solver::test_util::{
        assert_solver_all_solutions_base_2, assert_solver_single_solution,
    };

    use super::*;

    #[cfg(feature = "log")]
    fn init_logger() {
        let _ = env_logger::Builder::from_env(
            Env::default().default_filter_or("trace,sudoku::solver::backtracking=off"),
        )
        .is_test(true)
        .try_init();
    }

    #[cfg(not(feature = "log"))]
    fn init_logger() {}

    #[test]
    fn test_iter_all_solutions() {
        init_logger();

        let grid = Grid::<Base2>::new();
        let solver = Solver::new(grid);

        assert_solver_all_solutions_base_2(solver);
    }

    #[test]
    fn test_base_2() {
        init_logger();

        let grids = crate::samples::base_2();

        for grid in grids {
            let solver = Solver::new(grid.clone());
            assert_solver_single_solution(solver, &grid);
        }
    }

    #[test]
    fn test_base_3() {
        init_logger();

        let grids = crate::samples::base_3();

        for grid in grids {
            let solver = Solver::new(grid.clone());
            assert_solver_single_solution(solver, &grid);
        }
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn test_base_4() {
        init_logger();

        let grids = crate::samples::base_4();

        for grid in grids {
            let solver = Solver::new(grid.clone());
            assert_solver_single_solution(solver, &grid);
        }
    }
}
