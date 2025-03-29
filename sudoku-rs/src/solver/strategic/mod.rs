use std::marker::PhantomData;

use anyhow::ensure;
use log::trace;

pub use builder::SolverBuilder;
pub use step::{DynamicSolveStep, SolveStep};
use strategies::{Strategy, StrategyScore};

use crate::base::SudokuBase;
use crate::error::{Error, Result};
use crate::grid::Grid;
use crate::solver::backtracking::CandidatesFilter;
use crate::solver::strategic::deduction::Deductions;
use crate::solver::strategic::strategies::StrategyEnum;
use crate::solver::FallibleSolver;

pub mod deduction;
pub mod strategies;

mod step {
    use super::*;
    pub use dynamic::DynamicSolveStep;

    #[derive(Debug)]
    pub struct SolveStep<Base: SudokuBase> {
        pub strategy: StrategyEnum,
        pub deductions: Deductions<Base>,
    }

    impl<Base: SudokuBase> TryFrom<DynamicSolveStep> for SolveStep<Base> {
        type Error = Error;

        fn try_from(step: DynamicSolveStep) -> Result<Self> {
            Ok(SolveStep {
                strategy: step.strategy,
                deductions: step.deductions.try_into()?,
            })
        }
    }

    mod dynamic {
        use serde::{Deserialize, Serialize};

        use crate::solver::strategic::deduction::transport::TransportDeductions;

        use super::*;

        #[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
        #[derive(Debug, Serialize, Deserialize)]
        pub struct DynamicSolveStep {
            pub strategy: StrategyEnum,
            pub deductions: TransportDeductions,
        }

        impl<Base: SudokuBase> From<SolveStep<Base>> for DynamicSolveStep {
            fn from(solve_step: SolveStep<Base>) -> Self {
                Self {
                    strategy: solve_step.strategy,
                    deductions: solve_step.deductions.into(),
                }
            }
        }
    }
}

mod builder {
    use super::*;

    #[derive(Debug)]
    pub struct SolverBuilder<Base: SudokuBase, GridMut: AsRef<Grid<Base>>> {
        grid: GridMut,
        strategies: Vec<StrategyEnum>,
        _base: PhantomData<Base>,
    }

    impl<Base: SudokuBase, GridMut: AsRef<Grid<Base>>> SolverBuilder<Base, GridMut> {
        pub fn new(grid: GridMut) -> Self {
            Self {
                grid,
                strategies: vec![],
                _base: PhantomData,
            }
        }

        #[must_use]
        pub fn strategies(mut self, strategies: Vec<StrategyEnum>) -> Self {
            self.strategies = strategies;
            self
        }
    }

    impl<Base: SudokuBase, GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>>
        SolverBuilder<Base, GridMut>
    {
        /// Filter the available candidates which the solver can use to find a solution.
        #[must_use]
        pub fn candidates_filter<Filter: CandidatesFilter<Base>>(
            mut self,
            filter: &Filter,
        ) -> Self {
            filter.apply_to_grid_candidates(self.grid.as_mut());
            self
        }

        pub fn build(self) -> Solver<Base, GridMut> {
            let SolverBuilder {
                grid,
                strategies,
                _base,
            } = self;
            Solver::new_with_strategies(
                grid,
                if strategies.is_empty() {
                    StrategyEnum::default_solver_strategies()
                } else {
                    strategies
                },
            )
        }
    }
}

#[derive(Debug, Clone)]
pub struct Solver<Base: SudokuBase, GridMut: AsRef<Grid<Base>>> {
    grid: GridMut,
    // TODO: generic: AsRef: IntoIterator<DynamicStrategy>
    //  `Generator::try_delete_cell_at_pos` would not need to clone its strategies
    strategies: Vec<StrategyEnum>,
    _base: PhantomData<Base>,
}

/// Methods requiring mutable access to the grid.
impl<Base: SudokuBase, GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>> Solver<Base, GridMut> {
    pub fn new(grid: GridMut) -> Self {
        Self::builder(grid).build()
    }

    pub fn new_with_strategies(mut grid: GridMut, strategies: Vec<StrategyEnum>) -> Self {
        grid.as_mut()
            .set_all_direct_candidates_if_all_candidates_are_empty();

        Self {
            grid,
            strategies,
            _base: PhantomData,
        }
    }

    pub fn solve_route(&mut self) -> SolverPathIter<Base, GridMut> {
        SolverPathIter {
            solver: self,
            is_solved: false,
        }
    }
}

/// Methods requiring only immutable access to the grid.
impl<Base: SudokuBase, GridMut: AsRef<Grid<Base>>> Solver<Base, GridMut> {
    pub fn builder(grid: GridMut) -> SolverBuilder<Base, GridMut> {
        SolverBuilder::new(grid)
    }

    fn validate(&self) -> Result<()> {
        ensure!(
            self.grid.as_ref().is_directly_consistent(),
            "Grid is inconsistent"
        );
        Ok(())
    }

    fn execute_strategies_iter(&self) -> impl Iterator<Item = Result<SolveStep<Base>>> + '_ {
        self.strategies.iter().filter_map(|&strategy| {
            trace!("Executing strategy: {strategy:?}");
            Strategy::execute(strategy, self.grid.as_ref())
                .map(|deductions| {
                    (!deductions.is_empty()).then(|| {
                        trace!(
                            "{strategy:?} made progress:\n{deductions}\n{}",
                            self.grid.as_ref()
                        );
                        SolveStep {
                            strategy,
                            deductions,
                        }
                    })
                })
                .transpose()
        })
    }

    /// Tries executing all strategies and returns all deductions made by each strategy.
    pub fn try_all_strategies(&self) -> Result<Vec<SolveStep<Base>>> {
        self.validate()?;

        self.execute_strategies_iter().collect()
    }

    /// Tries executing strategies until one strategy is able to make at least one deduction.
    pub fn try_strategies(&self) -> Result<Option<SolveStep<Base>>> {
        self.validate()?;

        self.execute_strategies_iter().next().transpose()
    }

    pub fn into_grid(self) -> GridMut {
        self.grid
    }
}

#[derive(Debug)]
pub struct SolverPathIter<'a, Base: SudokuBase, GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>> {
    solver: &'a mut Solver<Base, GridMut>,
    is_solved: bool,
}

impl<Base: SudokuBase, GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>>
    SolverPathIter<'_, Base, GridMut>
{
    pub fn is_solved(&self) -> bool {
        self.is_solved
    }

    // TODO: return map of strategy -> number of deductions
    /// Weighted sum of all strategy scores used to solve the grid. `Strategy::score() * Number of deductions made by the strategy`
    pub fn total_score(mut self) -> Result<Option<StrategyScore>> {
        let total_score = self.try_fold::<_, _, Result<_>>(0, |total_score, res| {
            let SolveStep {
                strategy,
                deductions,
            } = res?;
            Ok(total_score + (strategy.score() * StrategyScore::try_from(deductions.count())?))
        })?;

        Ok(self.is_solved().then_some(total_score))
    }

    pub fn execution_count() {
        todo!()
    }
}

impl<Base: SudokuBase, GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>> Iterator
    for SolverPathIter<'_, Base, GridMut>
{
    type Item = Result<SolveStep<Base>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.solver.grid.as_ref().is_solved() {
            self.is_solved = true;
            None
        } else {
            Some(
                self.solver
                    .try_strategies()
                    .transpose()?
                    .and_then(|solve_step| {
                        solve_step.deductions.apply(self.solver.grid.as_mut())?;
                        Ok(solve_step)
                    }),
            )
        }
    }
}

impl<Base: SudokuBase, GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>> FallibleSolver<Base>
    for Solver<Base, GridMut>
{
    type Error = Error;

    fn try_solve(&mut self) -> Result<Option<Grid<Base>>> {
        let solve_route_iter = &mut self.solve_route();
        solve_route_iter.try_for_each(|res| res.map(|_| ()))?;

        Ok(solve_route_iter
            .is_solved()
            .then(|| self.grid.as_ref().clone()))
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::Base2;
    use crate::cell::Value;
    use crate::position::Position;
    use crate::solver::backtracking::ForceCandidateAtPosition;
    use crate::solver::test_util::{assert_fallible_solver_single_solution, tests_solver_samples};

    use super::*;

    tests_solver_samples! {
        |grid| {
            let solver = Solver::new(grid.clone());
            assert_fallible_solver_single_solution(solver, &grid);
        }
    }

    #[test]
    fn test_candidates_filter_denied_candidates_grid() {
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
            StrategyEnum::default_solver_strategies_no_brute_force(),
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
            StrategyEnum::default_solver_strategies_no_brute_force(),
        );
        assert!(solver.try_solve().unwrap().is_none());

        // But solver with filter for top left cell can solve it.
        let solver = Solver::builder(ambiguous_grid.clone())
            .strategies(StrategyEnum::default_solver_strategies_no_brute_force())
            .candidates_filter(&ForceCandidateAtPosition {
                pos: Position::top_left(),
                candidate: Value::default(),
            })
            .build();
        assert_fallible_solver_single_solution(solver, &grid);
    }
}
