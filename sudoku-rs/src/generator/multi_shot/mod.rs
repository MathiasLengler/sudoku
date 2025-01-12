use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::ensure;
use log::*;

use crate::error::Result;
use crate::grid::Grid;
use crate::solver::strategic;
use crate::{base::SudokuBase, solver::strategic::strategies::StrategyScore};

use super::Generator;

pub type GoalScore = u32;

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

#[derive(Debug)]
pub struct Goal {
    metric: GridMetric,
    optimize: GoalOptimization,
}

impl GridMetric {
    pub fn evaluate<Base: SudokuBase>(self, _grid: &Grid<Base>) -> StrategyScore {
        todo!()
    }
}

/// A generator that generates multiple grids and selects one based on a Goal metric.
#[derive(Debug)]
pub struct MultiShotGenerator<Base: SudokuBase> {
    generator: Generator<Base>,
}

impl<Base: SudokuBase> MultiShotGenerator<Base> {
    pub fn new(generator: Generator<Base>) -> Result<Self> {
        ensure!(
            generator.settings.prune.is_some(),
            "GoalGenerator requires pruning settings"
        );
        Ok(Self { generator })
    }

    pub fn generate_for_total_strategy_score(
        &self,
        iterations: u64,
    ) -> (StrategyScore, Grid<Base>) {
        // use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
        use rayon::prelude::*;

        let progress_counter = AtomicUsize::new(0);

        // let pb = ProgressBar::new(iterations).with_style(ProgressStyle::default_bar().template(
        //     "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta}, {per_sec})",
        // )?);

        let (total_score, grid) = (0..iterations)
            .into_par_iter()
            // .progress_with(pb)
            .map(|i| -> Result<_> {
                debug!("Generate iteration {i}");

                let grid = if let Some(seed) = self.generator.settings.seed {
                    let mut new_generator = self.generator.clone();
                    new_generator.settings.seed = Some(seed + i);
                    new_generator.generate()?
                } else {
                    self.generator.generate()?
                };

                let total_score = strategic::SolverBuilder::new(grid.clone())
                    .strategies(
                        self.generator
                            .settings
                            .prune
                            .as_ref()
                            .unwrap()
                            .strategies
                            .clone(),
                    )
                    .build()
                    .total_score()?
                    .unwrap();
                Ok((total_score, grid))
            })
            .inspect(|_| {
                progress_counter.fetch_add(1, Ordering::SeqCst);
                info!(
                    "Generate progress {}/{iterations}",
                    progress_counter.load(Ordering::SeqCst)
                );
            })
            .max_by_key(|res| res.as_ref().map_or(0, |(total_score, _)| *total_score))
            .unwrap()
            .unwrap();

        (total_score, grid)
    }
}
