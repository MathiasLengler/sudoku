use std::marker::PhantomData;

use log::trace;

pub use builder::SolverBuilder;
use strategies::Strategy;

use crate::base::SudokuBase;
use crate::error::{Error, Result};
use crate::grid::Grid;
use crate::solver::backtracking::AvailabilityFilter;
use crate::solver::strategic::deduction::Deductions;
use crate::solver::strategic::strategies::DynamicStrategy;
use crate::solver::FallibleSolver;

pub mod deduction;
pub mod strategies;

// TODO: return/persist chain of deductions for complete solve

// TODO: `solver.grade`
//  add "difficulty" score for each strategy
//  sum difficulty for each applied strategy
//  Reference: sudokuwiki "Grader"/"Solve path"

mod builder {
    use super::*;

    #[derive(Debug)]
    pub struct SolverBuilder<Base: SudokuBase, GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>> {
        grid: GridMut,
        strategies: Vec<DynamicStrategy>,
        _base: PhantomData<Base>,
    }

    impl<Base: SudokuBase, GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>>
        SolverBuilder<Base, GridMut>
    {
        pub fn new(grid: GridMut) -> Self {
            Self {
                grid,
                strategies: vec![],
                _base: PhantomData,
            }
        }
    }

    impl<Base: SudokuBase, GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>>
        SolverBuilder<Base, GridMut>
    {
        #[must_use]
        pub fn strategies(mut self, strategies: Vec<DynamicStrategy>) -> Self {
            self.strategies = strategies;
            self
        }
    }

    impl<Base: SudokuBase, GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>>
        SolverBuilder<Base, GridMut>
    {
        /// Filter the available candidates which the solver can use to find a solution.
        #[must_use]
        pub fn availability_filter<Filter: AvailabilityFilter<Base>>(
            mut self,
            filter: &Filter,
        ) -> Self {
            filter.apply_to_grid_candidates(self.grid.as_mut());
            self
        }
    }

    impl<Base: SudokuBase, GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>>
        SolverBuilder<Base, GridMut>
    {
        pub fn build(self) -> Solver<Base, GridMut> {
            let SolverBuilder {
                grid,
                strategies,
                _base,
            } = self;
            Solver::new_with_strategies(
                grid,
                if strategies.is_empty() {
                    DynamicStrategy::default_solver_strategies()
                } else {
                    strategies
                },
            )
        }
    }
}

#[derive(Debug, Clone)]
pub struct Solver<Base: SudokuBase, GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>> {
    grid: GridMut,
    // TODO: generic: AsRef: IntoIterator<DynamicStrategy>
    //  `Generator::try_delete_cell_at_pos` would not need to clone its strategies
    strategies: Vec<DynamicStrategy>,
    _base: PhantomData<Base>,
}

impl<Base: SudokuBase, GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>> Solver<Base, GridMut> {
    pub fn new(grid: GridMut) -> Self {
        Self::builder(grid).build()
    }

    pub fn builder(grid: GridMut) -> SolverBuilder<Base, GridMut> {
        SolverBuilder::new(grid)
    }

    pub fn new_with_strategies(mut grid: GridMut, strategies: Vec<DynamicStrategy>) -> Self {
        grid.as_mut()
            .set_all_direct_candidates_if_all_candidates_are_empty();

        Self {
            grid,
            strategies,
            _base: PhantomData,
        }
    }

    /// Tries executing strategies until one strategy is able to make at least one deduction.
    pub fn try_strategies(&self) -> Result<Option<(DynamicStrategy, Deductions<Base>)>> {
        for strategy in &self.strategies {
            trace!("Executing strategy: {strategy:?}");

            let deductions = Strategy::execute(*strategy, self.grid.as_ref())?;

            if !deductions.is_empty() {
                trace!(
                    "{strategy:?} made progress:\n{}\n{}",
                    deductions,
                    self.grid.as_ref()
                );

                return Ok(Some((*strategy, deductions)));
            }
        }
        Ok(None)
    }

    pub fn into_grid(self) -> GridMut {
        self.grid
    }
}

impl<Base: SudokuBase, GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>> FallibleSolver<Base>
    for Solver<Base, GridMut>
{
    type Error = Error;

    fn try_solve(&mut self) -> Result<Option<Grid<Base>>> {
        Ok(loop {
            if self.grid.as_ref().is_solved() {
                break Some(self.grid.as_ref().clone());
            }

            if let Some((_, deductions)) = self.try_strategies()? {
                deductions.apply(self.grid.as_mut())?;
                // Continue with strategy execution
            } else {
                // All strategies failed to make progress.
                break None;
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::Base2;
    use crate::cell::{Candidates, Value};
    use crate::position::Position;
    use crate::solver::backtracking::GroupAvailabilityIndex;
    use crate::solver::test_util::{assert_fallible_solver_single_solution, tests_solver_samples};

    use super::*;

    tests_solver_samples! {
        |grid| {
            let solver = Solver::new(grid.clone());
            assert_fallible_solver_single_solution(solver, &grid);
        }
    }

    #[test]
    fn test_availability_filter_denied_candidates_grid() {
        type Base = Base2;

        let grid = {
            let mut grid: Grid<Base> = "
            1040
            0000
            0000
            0102
            "
            .parse()
            .unwrap();

            grid.unfix_all_values();
            grid
        };

        assert!(grid.is_minimal());

        // Solver can solve the input grid
        let solver = Solver::new_with_strategies(
            grid.clone(),
            DynamicStrategy::default_solver_strategies_no_backtracking(),
        );
        assert_fallible_solver_single_solution(solver, &grid);

        // Delete top left value 1 => grid is ambiguous
        let ambiguous_grid = {
            let mut grid = grid.clone();
            grid[Position::top_left()].delete();
            grid.set_all_direct_candidates();
            grid
        };
        assert!(!ambiguous_grid.has_unique_solution());

        // Solver can no longer solve it
        let mut solver = Solver::new_with_strategies(
            ambiguous_grid.clone(),
            DynamicStrategy::default_solver_strategies_no_backtracking(),
        );
        assert!(solver.try_solve().unwrap().is_none());

        // But, solver with filter for top left cell can solve it.
        let solver = Solver::builder(ambiguous_grid.clone())
            .strategies(DynamicStrategy::default_solver_strategies_no_backtracking())
            .availability_filter(&|available_candidates, index| {
                if index == GroupAvailabilityIndex::default() {
                    Candidates::with_single(Value::default())
                } else {
                    available_candidates
                }
            })
            .build();
        assert_fallible_solver_single_solution(solver, &grid);
    }
}
