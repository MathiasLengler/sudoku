use crate::solver::backtracking::CandidatesFilter;
use crate::solver::strategic::deduction::Deductions;
use crate::solver::strategic::strategies::StrategyEnum;
use crate::solver::{FallibleSolver, strategic::strategies::selection::StrategySet};
use crate::{base::SudokuBase, solver::strategic::strategies::STRATEGY_SCORE_FIXED_POINT_SCALE};
use crate::{
    error::{Error, Result},
    solver::strategic::strategies::map::StrategyMap,
};
use crate::{grid::Grid, solver::strategic::strategies::selection::StrategySelection};
use anyhow::ensure;
pub use builder::SolverBuilder;
use log::trace;
use std::marker::PhantomData;
pub use step::{DynamicSolveStep, SolveStep};
use strategies::{Strategy, StrategyScore};

pub mod deduction;
pub mod strategies;

mod step {
    use std::fmt::Display;

    use super::*;
    pub use dynamic::DynamicSolveStep;

    #[derive(Debug, Clone)]
    pub struct SolveStep<Base: SudokuBase> {
        pub strategy: StrategyEnum,
        pub deductions: Deductions<Base>,
    }

    impl<Base: SudokuBase> Display for SolveStep<Base> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let Self {
                strategy,
                deductions,
            } = self;
            write!(f, "Strategy {strategy}:\n{deductions}")
        }
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
        #[derive(Debug, Clone, Serialize, Deserialize)]
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
    use crate::solver::strategic::strategies::selection::StrategySet;

    #[derive(Debug)]
    pub struct SolverBuilder<Base: SudokuBase, GridMut: AsRef<Grid<Base>>> {
        grid: GridMut,
        _base: PhantomData<Base>,
    }

    impl<Base: SudokuBase, GridMut: AsRef<Grid<Base>>> SolverBuilder<Base, GridMut> {
        pub fn new(grid: GridMut) -> Self {
            Self {
                grid,
                _base: PhantomData,
            }
        }

        #[must_use]
        pub fn strategies<Strategies: StrategySelection>(
            self,
            strategies: Strategies,
        ) -> SolverBuilderWithStrategies<Base, GridMut, Strategies> {
            SolverBuilderWithStrategies {
                grid: self.grid,
                strategies,
                _base: PhantomData,
            }
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

        pub fn build(self) -> Solver<Base, GridMut, StrategySet> {
            let SolverBuilder { grid, _base } = self;
            Solver::with_strategies(grid, StrategySet::default_solver_strategies())
        }
    }

    #[derive(Debug)]
    pub struct SolverBuilderWithStrategies<
        Base: SudokuBase,
        GridMut: AsRef<Grid<Base>>,
        Strategies: StrategySelection = StrategySet,
    > {
        grid: GridMut,
        strategies: Strategies,
        _base: PhantomData<Base>,
    }

    impl<
        Base: SudokuBase,
        GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>,
        Strategies: StrategySelection,
    > SolverBuilderWithStrategies<Base, GridMut, Strategies>
    {
        pub fn build(self) -> Solver<Base, GridMut, Strategies> {
            let SolverBuilderWithStrategies {
                grid,
                strategies,
                _base,
            } = self;
            Solver::with_strategies(grid, strategies)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Solver<Base: SudokuBase, GridRef: AsRef<Grid<Base>>, Strategies: StrategySelection> {
    grid: GridRef,
    strategies: Strategies,
    _base: PhantomData<Base>,
}

impl<Base: SudokuBase, GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>>
    Solver<Base, GridMut, StrategySet>
{
    pub fn new(grid: GridMut) -> Self {
        Self::builder(grid).build()
    }

    pub fn builder(grid: GridMut) -> SolverBuilder<Base, GridMut> {
        SolverBuilder::new(grid)
    }
}

/// Methods requiring mutable access to the grid.
impl<
    Base: SudokuBase,
    GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>,
    Strategies: StrategySelection,
> Solver<Base, GridMut, Strategies>
{
    pub fn with_strategies(mut grid: GridMut, strategies: Strategies) -> Self {
        grid.as_mut()
            .set_all_direct_candidates_if_all_candidates_are_empty();

        Self {
            grid,
            strategies,
            _base: PhantomData,
        }
    }

    pub fn solve_path(&mut self) -> SolverPathIter<'_, Base, GridMut, Strategies> {
        SolverPathIter {
            solver: self,
            is_solved: false,
        }
    }
    pub fn solve_path_all(&mut self) -> SolverPathAllIter<'_, Base, GridMut, Strategies> {
        SolverPathAllIter {
            solver: self,
            is_solved: false,
        }
    }
}

/// Methods requiring only immutable access to the grid.
impl<Base: SudokuBase, GridMut: AsRef<Grid<Base>>, Strategies: StrategySelection>
    Solver<Base, GridMut, Strategies>
{
    fn validate(&self) -> Result<()> {
        ensure!(
            self.grid.as_ref().is_directly_consistent(),
            "Grid is inconsistent"
        );
        Ok(())
    }

    fn execute_strategies_iter(&self) -> impl Iterator<Item = Result<SolveStep<Base>>> + '_ {
        self.strategies.iter_strategies().filter_map(|strategy| {
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
pub struct SolverPathIter<
    'a,
    Base: SudokuBase,
    GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>,
    Strategies: StrategySelection,
> {
    solver: &'a mut Solver<Base, GridMut, Strategies>,
    is_solved: bool,
}

impl<
    Base: SudokuBase,
    GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>,
    Strategies: StrategySelection,
> SolverPathIter<'_, Base, GridMut, Strategies>
{
    /// Weighted sum of all strategy scores used to solve the grid. `Strategy::score() * Number of deductions made by the strategy`
    pub fn total_score(mut self) -> Result<Option<StrategyScore>> {
        let total_score = self.try_fold::<_, _, Result<_>>(0, |acc, res| {
            let SolveStep {
                strategy,
                deductions,
            } = res?;
            Ok(acc + (strategy.score() * StrategyScore::try_from(deductions.count())?))
        })?;

        Ok(self.is_solved.then_some(total_score))
    }

    /// The number of times each strategy was applied to the grid.
    pub fn application_count(mut self) -> Result<Option<StrategyMap<StrategyScore>>> {
        let mut strategy_map = StrategyMap::<StrategyScore>::default();

        self.try_for_each(|res| {
            let SolveStep { strategy, .. } = res?;
            *strategy_map.get_mut(strategy) += 1;
            Ok::<(), Error>(())
        })?;

        Ok(self.is_solved.then_some(strategy_map))
    }

    /// The number of times any strategy was applied to the grid.
    pub fn application_count_any(self) -> Result<Option<StrategyScore>> {
        Ok(self
            .application_count()?
            .map(|strategy_map| strategy_map.into_values().into_iter().sum()))
    }

    /// The number of times a strategy was applied to the grid.
    pub fn application_count_single(self, strategy: StrategyEnum) -> Result<Option<StrategyScore>> {
        Ok(self
            .application_count()?
            .map(|strategy_map| *strategy_map.get(strategy)))
    }

    /// Number of deductions by each strategy used to solve the grid.
    pub fn deduction_count(mut self) -> Result<Option<StrategyMap<StrategyScore>>> {
        let mut strategy_map = StrategyMap::default();

        self.try_for_each(|res| {
            let SolveStep {
                strategy,
                deductions,
            } = res?;
            *strategy_map.get_mut(strategy) += StrategyScore::try_from(deductions.count())?;
            Ok::<(), Error>(())
        })?;

        Ok(self.is_solved.then_some(strategy_map))
    }

    /// Number of deductions used to solve the grid.
    pub fn deduction_count_any(self) -> Result<Option<StrategyScore>> {
        Ok(self
            .deduction_count()?
            .map(|strategy_map| strategy_map.into_values().into_iter().sum()))
    }
    /// Number of deductions by a single strategy used to solve the grid.
    pub fn deduction_count_single(self, strategy: StrategyEnum) -> Result<Option<StrategyScore>> {
        Ok(self
            .deduction_count()?
            .map(|strategy_map| *strategy_map.get(strategy)))
    }
}

impl<
    Base: SudokuBase,
    GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>,
    Strategies: StrategySelection,
> Iterator for SolverPathIter<'_, Base, GridMut, Strategies>
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

impl<
    Base: SudokuBase,
    GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>,
    Strategies: StrategySelection,
> FallibleSolver<Base> for Solver<Base, GridMut, Strategies>
{
    type Error = Error;

    fn try_solve(&mut self) -> Result<Option<Grid<Base>>> {
        let solve_path = &mut self.solve_path();
        solve_path.try_for_each(|res| res.map(|_| ()))?;

        Ok(solve_path.is_solved.then(|| self.grid.as_ref().clone()))
    }
}

#[derive(Debug)]
pub struct SolverPathAllIter<
    'a,
    Base: SudokuBase,
    GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>,
    Strategies: StrategySelection,
> {
    solver: &'a mut Solver<Base, GridMut, Strategies>,
    is_solved: bool,
}

impl<
    Base: SudokuBase,
    GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>,
    Strategies: StrategySelection,
> SolverPathAllIter<'_, Base, GridMut, Strategies>
{
    /// The average number of strategies available to make progress. Scaled by a factor of `STRATEGY_SCORE_FIXED_POINT_SCALE`.
    pub fn average_options(mut self) -> Result<Option<StrategyScore>> {
        let (step_count, total_options) =
            self.try_fold::<_, _, Result<_>>((0u64, 0u64), |(acc_count, acc_options), res| {
                let possible_solve_steps = res?;
                Ok((
                    acc_count + 1,
                    acc_options + StrategyScore::try_from(possible_solve_steps.len())?,
                ))
            })?;

        Ok(self.is_solved.then_some(StrategyScore::try_from(
            (total_options * STRATEGY_SCORE_FIXED_POINT_SCALE) / step_count,
        )?))
    }
}

impl<
    Base: SudokuBase,
    GridMut: AsMut<Grid<Base>> + AsRef<Grid<Base>>,
    Strategies: StrategySelection,
> Iterator for SolverPathAllIter<'_, Base, GridMut, Strategies>
{
    type Item = Result<Vec<SolveStep<Base>>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.solver.grid.as_ref().is_solved() {
            self.is_solved = true;
            None
        } else {
            self.solver
                .try_all_strategies()
                .and_then(|possible_solve_steps| {
                    if let Some(solve_step) = possible_solve_steps.first() {
                        solve_step.deductions.apply(self.solver.grid.as_mut())?;
                        Ok(Some(possible_solve_steps))
                    } else {
                        Ok(None)
                    }
                })
                .transpose()
        }
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::base::consts::Base2;
    use crate::cell::Value;
    use crate::position::Position;
    use crate::solver::backtracking::ForceCandidateAtPosition;
    use crate::solver::test_util::{assert_fallible_solver_single_solution, tests_solver_samples};

    use super::*;

    tests_solver_samples! {
        |grid| {
            let mut solver = Solver::new(grid.clone());
            assert_fallible_solver_single_solution(&mut solver, &grid);
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
        let mut solver = Solver::with_strategies(
            grid.clone(),
            StrategySet::default_solver_strategies_no_brute_force(),
        );
        assert_fallible_solver_single_solution(&mut solver, &grid);

        // Delete top left value 1 => grid is ambiguous
        let ambiguous_grid = {
            let mut grid = grid.clone();
            grid[Position::top_left()].delete();
            grid.set_all_direct_candidates();
            grid
        };
        assert!(!ambiguous_grid.has_unique_solution());

        // Solver can no longer solve it
        let mut solver = Solver::with_strategies(
            ambiguous_grid.clone(),
            StrategySet::default_solver_strategies_no_brute_force(),
        );
        assert!(solver.try_solve().unwrap().is_none());

        // But solver with filter for top left cell can solve it.
        let mut solver = Solver::builder(ambiguous_grid.clone())
            .candidates_filter(&ForceCandidateAtPosition {
                pos: Position::top_left(),
                candidate: Value::default(),
            })
            .strategies(StrategySet::default_solver_strategies_no_brute_force())
            .build();
        assert_fallible_solver_single_solution(&mut solver, &grid);
    }

    #[test]
    fn test_solve_path() {
        for grid in crate::samples::base_2() {
            let mut solver = Solver::with_strategies(
                grid.clone(),
                StrategySet::default_solver_strategies_no_brute_force(),
            );
            let solve_steps = solver.solve_path().collect::<Result<Vec<_>>>().unwrap();
            println!(
                "Grid:\n{grid}\nSolve steps:\n{}",
                solve_steps.into_iter().join("\n")
            );
            // TODO: assert
        }
    }

    #[test]
    fn test_solve_path_all() {
        for grid in crate::samples::base_2() {
            let mut solver = Solver::with_strategies(
                grid.clone(),
                StrategySet::default_solver_strategies_no_brute_force(),
            );
            let all_possible_solve_steps =
                solver.solve_path_all().collect::<Result<Vec<_>>>().unwrap();
            println!(
                "Grid:\n{grid}\nSolve steps:\n{}",
                all_possible_solve_steps.into_iter().flatten().join("\n")
            );
            // TODO: assert
        }
    }

    mod snapshots {
        use super::*;

        #[test]
        fn test_solve_path_base2() {
            for (i, grid) in crate::samples::base_2().into_iter().enumerate() {
                let mut solver = Solver::with_strategies(
                    grid,
                    StrategySet::default_solver_strategies_no_brute_force(),
                );
                let solve_steps: Vec<DynamicSolveStep> = solver
                    .solve_path()
                    .map(|res| res.map(Into::into))
                    .collect::<Result<_>>()
                    .unwrap();
                insta::assert_yaml_snapshot!(format!("solve_path_base2_grid_{i}"), solve_steps);
            }
        }

        #[test]
        fn test_solve_path_all_base2() {
            for (i, grid) in crate::samples::base_2().into_iter().enumerate() {
                let mut solver = Solver::with_strategies(
                    grid,
                    StrategySet::default_solver_strategies_no_brute_force(),
                );
                let all_possible_solve_steps: Vec<Vec<DynamicSolveStep>> = solver
                    .solve_path_all()
                    .map(|res| {
                        res.map(|steps| steps.into_iter().map(Into::into).collect())
                    })
                    .collect::<Result<_>>()
                    .unwrap();
                insta::assert_yaml_snapshot!(
                    format!("solve_path_all_base2_grid_{i}"),
                    all_possible_solve_steps
                );
            }
        }

        #[test]
        fn test_solve_path_base3() {
            let grid = crate::samples::base_3().into_iter().next().unwrap();
            let mut solver = Solver::with_strategies(
                grid,
                StrategySet::default_solver_strategies_no_brute_force(),
            );
            let solve_steps: Vec<DynamicSolveStep> = solver
                .solve_path()
                .map(|res| res.map(Into::into))
                .collect::<Result<_>>()
                .unwrap();
            insta::assert_yaml_snapshot!(solve_steps);
        }

        #[test]
        fn test_solve_path_base3_all_samples() {
            for (i, grid) in crate::samples::base_3().into_iter().enumerate() {
                let mut solver = Solver::new(grid);
                let solve_steps: Vec<DynamicSolveStep> = solver
                    .solve_path()
                    .map(|res| res.map(Into::into))
                    .collect::<Result<_>>()
                    .unwrap();
                insta::assert_yaml_snapshot!(format!("solve_path_base3_grid_{i}"), solve_steps);
            }
        }
    }
}
