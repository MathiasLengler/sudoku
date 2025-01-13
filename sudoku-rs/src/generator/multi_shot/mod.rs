use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::ensure;
use log::*;

use crate::error::Result;
use crate::grid::Grid;
use crate::solver::strategic;
use crate::solver::strategic::strategies::StrategyEnum;
use crate::{base::SudokuBase, solver::strategic::strategies::StrategyScore};

use super::{Generator, GeneratorSettings};

pub type EvaluatedGridMetric = u32;

/// A metric used to evaluate the difficulty of a grid.
#[derive(Debug, Clone, Copy)]
pub enum GridMetric {
    // Based on `strategic::SolverPathIter` - a single solve path determined by the solver.
    /// Weighted sum of all strategy scores used to solve the grid. `Strategy::score() * Number of deductions made by the strategy`
    StrategyTotalScore,
    /// How often each strategy was executed, successful or not, to solve the grid.
    StrategyExecutionCount,
    /// How often each strategy was successfully executed to solve the grid.
    StrategyApplicationCount,
    /// Number of deductions used to solve the grid.
    StrategyDeductionCount,
    /// The average number of strategies available to make progress.
    StrategyOptionsAverage,

    // Based on the PoC bin `solve_graph` - a graph of all possible solve paths.
    /// The average [branching factor](https://en.wikipedia.org/wiki/Branching_factor) of the strategy solve graph.
    /// In other words: the average number of strategies available to make progress.
    /// E.g. the average number of available stratgies to make progress.
    SolveGraphAverageBranchingFactor,
    /// The number of steps taken by `sat::Solver` to solve the grid.
    SatStepCount,
    /// The number of steps taken by `backtracking::Solver` to solve the grid.
    BacktrackingStepCount,
    /// The number of givens in the grid.
    GridGivens,
    // Use normalized metrics instead of standard deviation? (0-1, gini coefficient etc.)
    /// The standard deviation of the givens value counts in the grid.
    /// E.g. how evenly distributed the givens values are.
    /// Example:
    /// 3 givens for each number => 1
    /// Only 2s and 3s => >>1
    GridGivensValueCountDeviation,
}

#[derive(Debug)]
pub enum GoalOptimization {
    Minimize,
    Maximize,
}

type IterationsCounter = u32;

#[derive(Debug)]
pub struct MultiShotGeneratorSettings<Base: SudokuBase> {
    pub generator_settings: GeneratorSettings<Base>,
    pub iterations: IterationsCounter,
    pub metric: GridMetric,
    pub optimize: GoalOptimization,
    pub parallel: bool,
}

impl GridMetric {
    pub fn evaluate<Base: SudokuBase>(
        self,
        grid: &Grid<Base>,
        strategies: Vec<StrategyEnum>,
    ) -> Result<EvaluatedGridMetric> {
        Ok(match self {
            GridMetric::StrategyTotalScore => {
                let total_score = strategic::SolverBuilder::new(grid.clone())
                    .strategies(strategies)
                    .build()
                    .total_score()?
                    .unwrap();
                total_score
            }
            GridMetric::StrategyExecutionCount => todo!(),
            GridMetric::StrategyApplicationCount => todo!(),
            GridMetric::StrategyDeductionCount => todo!(),
            GridMetric::StrategyOptionsAverage => todo!(),
            GridMetric::SolveGraphAverageBranchingFactor => todo!(),
            GridMetric::SatStepCount => todo!(),
            GridMetric::BacktrackingStepCount => todo!(),
            GridMetric::GridGivens => todo!(),
            GridMetric::GridGivensValueCountDeviation => todo!(),
        })
    }
}

/// A generator that generates multiple grids and selects one based on a Goal metric.
#[derive(Debug)]
pub struct MultiShotGenerator<Base: SudokuBase> {
    settings: MultiShotGeneratorSettings<Base>,
}

impl<Base: SudokuBase> MultiShotGenerator<Base> {
    pub fn new(settings: MultiShotGeneratorSettings<Base>) -> Result<Self> {
        ensure!(
            settings.generator_settings.prune.is_some(),
            "GoalGenerator requires pruning settings"
        );
        Ok(Self { settings })
    }

    fn iterations_iter(&self) -> impl Iterator<Item = IterationsCounter> {
        0..self.settings.iterations
    }

    fn iter_progress_log<'a, IterItem>(
        &'a self,
        iter: impl Iterator<Item = IterItem> + 'a,
    ) -> impl Iterator<Item = IterItem> + 'a {
        let mut progress_counter = 0usize;

        iter.inspect(move |_| {
            progress_counter += 1;
            info!(
                "Sequential generate progress {}/{}",
                progress_counter, self.settings.iterations
            );
        })
    }

    fn iterations_par_iter(&self) -> impl IndexedParallelIterator<Item = IterationsCounter> {
        (0..self.settings.iterations).into_par_iter()
    }

    #[cfg(feature = "terminal")]
    fn iterations_par_iter_progress_bar(
        &self,
    ) -> impl IndexedParallelIterator<Item = IterationsCounter> {
        use indicatif::{ParallelProgressIterator, ProgressStyle};

        self.iterations_par_iter().progress_with_style(ProgressStyle::default_bar().template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta}, {per_sec})",
        ).expect("Progress bar template to be valid"))
    }

    fn par_iter_progress_log<'a, ParIterItem: Send>(
        &'a self,
        par_iter: impl IndexedParallelIterator<Item = ParIterItem> + 'a,
    ) -> impl IndexedParallelIterator<Item = ParIterItem> + 'a {
        let progress_counter = AtomicUsize::new(0);

        par_iter.inspect(move |_| {
            progress_counter.fetch_add(1, Ordering::SeqCst);
            info!(
                "Parallel generate progress {}/{}",
                progress_counter.load(Ordering::SeqCst),
                self.settings.iterations
            );
        })
    }

    fn generate_single(&self, i: IterationsCounter) -> Result<Grid<Base>> {
        debug!("Generate iteration {i}");

        Ok(if let Some(seed) = self.settings.generator_settings.seed {
            Generator::with_settings(GeneratorSettings {
                seed: Some(seed + u64::from(i)),
                ..self.settings.generator_settings.clone()
            })
            .generate()?
        } else {
            Generator::with_settings(self.settings.generator_settings.clone()).generate()?
        })
    }

    pub fn generate_for_total_strategy_score(&self) -> (StrategyScore, Grid<Base>) {
        let (total_score, grid) = self
            .iterations_par_iter()
            .map(|i| self.generate_single(i))
            .map(|grid_res| -> Result<_> {
                let grid = grid_res?;
                let evaluated_grid_metric = self.settings.metric.evaluate(
                    &grid,
                    self.settings
                        .generator_settings
                        .prune
                        .as_ref()
                        .unwrap()
                        .strategies
                        .clone(),
                )?;
                Ok((evaluated_grid_metric, grid))
            })
            .max_by_key(|res| res.as_ref().map_or(0, |(total_score, _)| *total_score))
            .unwrap()
            .unwrap();

        (total_score, grid)
    }
}
