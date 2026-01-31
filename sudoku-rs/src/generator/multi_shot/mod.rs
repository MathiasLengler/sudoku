#![allow(deprecated)]

use crate::cell::Value;
use crate::grid::Grid;
use crate::solver::strategic::{self, strategies::StrategyEnum};
use crate::solver::{FallibleSolver, InfallibleSolver, backtracking, sat};
use crate::{base::SudokuBase, solver::strategic::strategies::selection::StrategySet};
use crate::{error::Result, solver::strategic::strategies::selection::StrategySelection};
use anyhow::{Context, ensure};
use log::*;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use super::{Generator, GeneratorSettings};

pub use dynamic_settings::*;

pub type EvaluatedGridMetric = u64;
type AtomicEvaluatedGridMetric = AtomicU64;

static GENERATE_NO_GRIDS: &str = "at least one generation result";

/// A metric used to evaluate the difficulty of a grid.
#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum GridMetric {
    // Based on `strategic::SolverPathIter` - a single solve path determined by the solver.
    /// Weighted sum of all strategy scores used to solve the grid. `Strategy::score() * Number of deductions made by the strategy`
    #[default]
    StrategyScore,
    /// The number of times any strategy was applied to the grid.
    StrategyApplicationCountAny,
    /// The number of times a single strategy was applied to the grid.
    StrategyApplicationCountSingle { strategy: StrategyEnum },
    /// Number of deductions used to solve the grid.
    StrategyDeductionCountAny,
    /// Number of deductions by a single strategy used to solve the grid.
    StrategyDeductionCountSingle { strategy: StrategyEnum },
    // FIXME: this produces counterintuitive results
    //  if there are only single candidates left, all strategies except for *Singles don't make progress.
    //  The intention was to measure "needle point" strategies, which block further progress until spotted,
    //  but this metric does not reflect that.
    //  We need to somehow weigh the available strategies by their difficulty.
    /// The average number of strategies available to make progress. Scaled by a factor of `STRATEGY_SCORE_FIXED_POINT_SCALE`.
    StrategyAverageOptions,

    // Based on the PoC bin `solve_graph` - a graph of all possible solve paths.
    /// The average [branching factor](https://en.wikipedia.org/wiki/Branching_factor) of the strategy solve graph.
    /// In other words: the average number of strategies available to make progress across all nodes in the solve graph.
    #[deprecated]
    SolveGraphAverageBranchingFactor,
    /// The number of steps taken by `sat::Solver` to solve the grid.
    SatStepCount,
    /// The number of backtracking steps taken by `backtracking::Solver` to solve the grid.
    BacktrackCount,
    /// The number of givens in the grid.
    GridGivensCount,
    /// The number of candidates in the grid.
    GridDirectCandidatesCount,
    // Use normalized metrics instead of standard deviation? (0-1, gini coefficient etc.)
    /// The standard deviation of the givens value counts in the grid.
    /// E.g. how evenly distributed the givens values are.
    /// Example:
    /// 3 givens for each number => 1
    /// Only 2s and 3s => >>1
    #[deprecated]
    GridGivensValueCountDeviation,
}

impl GridMetric {
    pub fn evaluate<Base: SudokuBase>(
        self,
        grid: &Grid<Base>,
        strategies: impl StrategySelection,
    ) -> Result<EvaluatedGridMetric> {
        static STRATEGIC_SOLVER_ERROR_MESSAGE: &str = "Strategic solver failed to solve the grid";
        let get_strategic_solver = || {
            strategic::SolverBuilder::new(grid.clone())
                .strategies(strategies)
                .build()
        };

        // TODO: implement remaining metrics
        Ok(match self {
            GridMetric::StrategyScore => get_strategic_solver()
                .solve_path()
                .total_score()?
                .context(STRATEGIC_SOLVER_ERROR_MESSAGE)?,
            GridMetric::StrategyApplicationCountAny => get_strategic_solver()
                .solve_path()
                .application_count_any()?
                .context(STRATEGIC_SOLVER_ERROR_MESSAGE)?,
            GridMetric::StrategyApplicationCountSingle { strategy } => get_strategic_solver()
                .solve_path()
                .application_count_single(strategy)?
                .context(STRATEGIC_SOLVER_ERROR_MESSAGE)?,
            GridMetric::StrategyDeductionCountAny => get_strategic_solver()
                .solve_path()
                .deduction_count_any()?
                .context(STRATEGIC_SOLVER_ERROR_MESSAGE)?,
            GridMetric::StrategyDeductionCountSingle { strategy } => get_strategic_solver()
                .solve_path()
                .deduction_count_single(strategy)?
                .context(STRATEGIC_SOLVER_ERROR_MESSAGE)?,
            GridMetric::StrategyAverageOptions => get_strategic_solver()
                .solve_path_all()
                .average_options()?
                .context(STRATEGIC_SOLVER_ERROR_MESSAGE)?,
            GridMetric::SolveGraphAverageBranchingFactor => todo!(),
            GridMetric::SatStepCount => {
                let mut solver = sat::Solver::new(grid);
                solver
                    .try_solve()?
                    .context("SAT solver failed to solve the grid")?;
                solver.step_count()
            }
            GridMetric::BacktrackCount => {
                let mut solver = backtracking::Solver::new(grid);
                solver
                    .solve()
                    .context("Backtracking solver failed to solve the grid")?;
                solver.backtrack_count
            }
            GridMetric::GridGivensCount => grid.all_value_positions().len().try_into()?,
            GridMetric::GridDirectCandidatesCount => grid
                .all_candidates_positions()
                .into_iter()
                .map(|pos| EvaluatedGridMetric::from(grid.direct_candidates(pos).count()))
                .sum(),
            GridMetric::GridGivensValueCountDeviation => {
                // Count occurrences of each value in the grid's givens
                let value_counts: Vec<u64> = Value::<Base>::all()
                    .map(|value| {
                        grid.all_value_positions()
                            .iter()
                            .filter(|&&pos| grid.get(pos).value() == Some(value))
                            .count() as u64
                    })
                    .collect();

                // These casts are safe: value_counts.len() <= 25 (max base 5), counts are small
                #[allow(clippy::cast_precision_loss)]
                let n = value_counts.len() as f64;

                let sum: u64 = value_counts.iter().sum();
                #[allow(clippy::cast_precision_loss)]
                let mean = sum as f64 / n;

                // Calculate variance: sum of squared differences from mean
                #[allow(clippy::cast_precision_loss)]
                let variance: f64 = value_counts
                    .iter()
                    .map(|&count| {
                        let diff = count as f64 - mean;
                        diff * diff
                    })
                    .sum::<f64>()
                    / n;

                // Standard deviation, scaled by 1000 for precision
                // (similar to STRATEGY_SCORE_FIXED_POINT_SCALE)
                // std_dev is always non-negative, and scaled value fits in u64
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                { (variance.sqrt() * 1000.0) as u64 }
            }
        })
    }
}

#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GoalOptimization {
    Minimize,
    #[default]
    Maximize,
}

pub type IterationsCounter = u32;

#[derive(Debug)]
pub struct MultiShotGeneratorSettings<Base: SudokuBase> {
    pub generator_settings: GeneratorSettings<Base>,
    pub iterations: IterationsCounter,
    pub metric: GridMetric,
    pub optimize: GoalOptimization,
    pub parallel: bool,
}

impl<Base: SudokuBase + Default> Default for MultiShotGeneratorSettings<Base> {
    fn default() -> Self {
        Self {
            generator_settings: GeneratorSettings::default(),
            iterations: 1,
            metric: GridMetric::default(),
            optimize: GoalOptimization::default(),
            parallel: false,
        }
    }
}

impl<Base: SudokuBase> MultiShotGeneratorSettings<Base> {
    fn get_prune_strategies(&self) -> StrategySet {
        self.generator_settings
            .prune
            .as_ref()
            .map(|prune| prune.strategies)
            .unwrap()
    }
}

mod dynamic_settings {
    use super::*;

    use crate::error::Error;
    use crate::generator::DynamicGeneratorSettings;

    #[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DynamicMultiShotGeneratorSettings {
        pub generator_settings: DynamicGeneratorSettings,
        pub iterations: IterationsCounter,
        pub metric: GridMetric,
        pub optimize: GoalOptimization,
        pub parallel: bool,
    }

    impl<Base: SudokuBase> TryFrom<DynamicMultiShotGeneratorSettings>
        for MultiShotGeneratorSettings<Base>
    {
        type Error = Error;

        fn try_from(
            dynamic_multi_shot_generator_settings: DynamicMultiShotGeneratorSettings,
        ) -> Result<Self> {
            let DynamicMultiShotGeneratorSettings {
                generator_settings,
                iterations,
                metric,
                optimize,
                parallel,
            } = dynamic_multi_shot_generator_settings;

            Ok(Self {
                generator_settings: generator_settings.try_into()?,
                iterations,
                metric,
                optimize,
                parallel,
            })
        }
    }
}

#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "kind"
)]
pub enum MultiShotGeneratorProgress {
    Started {
        current_iteration: IterationsCounter,
        total_iterations: IterationsCounter,
    },
    Finished {
        current_iteration: IterationsCounter,
        total_iterations: IterationsCounter,
        current_evaluated_grid_metric: EvaluatedGridMetric,
        best_evaluated_grid_metric: EvaluatedGridMetric,
    },
}

impl MultiShotGeneratorProgress {
    pub fn current_iteration(&self) -> IterationsCounter {
        match self {
            MultiShotGeneratorProgress::Started {
                current_iteration, ..
            }
            | MultiShotGeneratorProgress::Finished {
                current_iteration, ..
            } => *current_iteration,
        }
    }

    pub fn total_iterations(&self) -> IterationsCounter {
        match self {
            MultiShotGeneratorProgress::Started {
                total_iterations, ..
            }
            | MultiShotGeneratorProgress::Finished {
                total_iterations, ..
            } => *total_iterations,
        }
    }
}

#[derive(Debug)]
pub struct EvaluatedGrid<Base: SudokuBase> {
    pub evaluated_grid_metric: EvaluatedGridMetric,
    pub grid: Grid<Base>,
}

impl<Base: SudokuBase + Eq> Eq for EvaluatedGrid<Base> {}

impl<Base: SudokuBase + PartialEq> PartialEq for EvaluatedGrid<Base> {
    fn eq(&self, other: &Self) -> bool {
        self.evaluated_grid_metric == other.evaluated_grid_metric
    }
}

impl<Base: SudokuBase + Ord> Ord for EvaluatedGrid<Base> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.evaluated_grid_metric.cmp(&other.evaluated_grid_metric)
    }
}

impl<Base: SudokuBase + PartialOrd> PartialOrd for EvaluatedGrid<Base> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// A generator that generates multiple grids and selects one based on a Goal metric.
#[derive(Debug)]
pub struct MultiShotGenerator<Base: SudokuBase> {
    settings: MultiShotGeneratorSettings<Base>,
}

impl<Base: SudokuBase> MultiShotGenerator<Base> {
    pub fn with_settings(settings: MultiShotGeneratorSettings<Base>) -> Result<Self> {
        ensure!(
            settings.generator_settings.prune.is_some(),
            "MultiShotGenerator requires pruning settings"
        );
        ensure!(
            settings.iterations > 0,
            "MultiShotGenerator requires at least one iteration"
        );
        Ok(Self { settings })
    }

    fn iterations_iter(&self) -> impl Iterator<Item = IterationsCounter> + use<Base> {
        0..self.settings.iterations
    }

    #[allow(dead_code)]
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

    fn iterations_par_iter(
        &self,
    ) -> impl IndexedParallelIterator<Item = IterationsCounter> + use<Base> {
        (0..self.settings.iterations).into_par_iter()
    }

    #[allow(dead_code)]
    #[cfg(feature = "terminal")]
    fn iterations_par_iter_progress_bar(
        &self,
    ) -> impl IndexedParallelIterator<Item = IterationsCounter> {
        use indicatif::{ParallelProgressIterator, ProgressStyle};

        self.iterations_par_iter().progress_with_style(ProgressStyle::default_bar().template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta}, {per_sec})",
        ).expect("Progress bar template to be valid"))
    }

    #[allow(dead_code)]
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

    fn generate_single(&self, iteration: IterationsCounter) -> Result<Grid<Base>> {
        debug!("Generate iteration {iteration}");

        Ok(if let Some(seed) = self.settings.generator_settings.seed {
            Generator::with_settings(GeneratorSettings {
                seed: Some(seed + u64::from(iteration)),
                ..self.settings.generator_settings.clone()
            })
            // TODO: generate_with_progress
            //  this would allow us to show progress bar for each iteration
            //  Reference: https://docs.rs/indicatif/latest/indicatif/struct.MultiProgress.html
            //  Could become a performance issue, since the upper progress callback will be executed more often.
            .generate()?
        } else {
            Generator::with_settings(self.settings.generator_settings.clone()).generate()?
        })
    }

    fn evaluate_grid(&self, grid: Grid<Base>) -> Result<EvaluatedGrid<Base>> {
        let evaluated_grid_metric = self
            .settings
            .metric
            .evaluate(&grid, self.settings.get_prune_strategies())?;
        Ok(EvaluatedGrid {
            evaluated_grid_metric,
            grid,
        })
    }

    pub fn generate(&self) -> Result<EvaluatedGrid<Base>> {
        self.generate_with_inspect(|_| Ok(()), |_, _| Ok(()))
    }

    pub fn generate_with_progress(
        &self,
        mut on_progress: impl FnMut(MultiShotGeneratorProgress) -> Result<()>,
    ) -> Result<EvaluatedGrid<Base>> {
        let mut ret = None;

        // `on_progress` is neither `Sync` nor `Send`, because the intended use case is a WASM/JS callback.
        // We generate exlusively on the rayon threadpool, sending the progress to the main thread.
        rayon::in_place_scope::<_, Result<_>>(|s| {
            use std::sync::mpsc;

            let (on_progress_sender, on_progress_receiver) =
                mpsc::channel::<MultiShotGeneratorProgress>();

            s.spawn(|_s| {
                let best_evaluated_grid_metric =
                    AtomicEvaluatedGridMetric::new(match self.settings.optimize {
                        GoalOptimization::Minimize => EvaluatedGridMetric::MAX,
                        GoalOptimization::Maximize => EvaluatedGridMetric::MIN,
                    });

                let on_start_progress_sender: mpsc::Sender<MultiShotGeneratorProgress> =
                    on_progress_sender.clone();

                ret = Some(self.generate_with_inspect(
                    |current_iteration| {
                        let progress = MultiShotGeneratorProgress::Started {
                            current_iteration,
                            total_iterations: self.settings.iterations,
                        };
                        on_start_progress_sender
                            .send(progress)
                            .expect("Failed to send progress");
                        Ok(())
                    },
                    move |current_iteration, evaluated_grid| {
                        let best_evaluated_grid_metric = match self.settings.optimize {
                            GoalOptimization::Minimize => best_evaluated_grid_metric
                                .fetch_min(evaluated_grid.evaluated_grid_metric, Ordering::SeqCst)
                                .min(evaluated_grid.evaluated_grid_metric),
                            GoalOptimization::Maximize => best_evaluated_grid_metric
                                .fetch_max(evaluated_grid.evaluated_grid_metric, Ordering::SeqCst)
                                .max(evaluated_grid.evaluated_grid_metric),
                        };

                        let progress = MultiShotGeneratorProgress::Finished {
                            current_iteration,
                            total_iterations: self.settings.iterations,
                            current_evaluated_grid_metric: evaluated_grid.evaluated_grid_metric,
                            best_evaluated_grid_metric,
                        };

                        on_progress_sender
                            .send(progress)
                            .expect("Failed to send progress");
                        Ok(())
                    },
                ));
            });

            for progress in on_progress_receiver {
                on_progress(progress)?;
            }

            Ok(())
        })?;
        ret.expect("Spawned thread to set a return value")
    }

    fn generate_with_inspect(
        &self,
        inspect_iteration_start: impl Fn(IterationsCounter) -> Result<()> + Sync + Send,
        inspect_evaluated_grids: impl Fn(IterationsCounter, &EvaluatedGrid<Base>) -> Result<()>
        + Sync
        + Send,
    ) -> Result<EvaluatedGrid<Base>> {
        let process_iteration = |iteration| -> Result<_> {
            inspect_iteration_start(iteration)?;
            let grid = self.generate_single(iteration)?;
            let evaluated_grid = self.evaluate_grid(grid)?;
            inspect_evaluated_grids(iteration, &evaluated_grid)?;
            Ok(evaluated_grid)
        };

        Ok(if self.settings.parallel {
            let evaluated_grids = self.iterations_par_iter().map(process_iteration);
            match self.settings.optimize {
                GoalOptimization::Minimize => evaluated_grids
                    .try_reduce_with(|a, b| Ok(Ord::min(a, b)))
                    .expect(GENERATE_NO_GRIDS)?,
                GoalOptimization::Maximize => evaluated_grids
                    .try_reduce_with(|a, b| Ok(Ord::max(a, b)))
                    .expect(GENERATE_NO_GRIDS)?,
            }
        } else {
            let mut evaluated_grids = self.iterations_iter().map(process_iteration);

            match self.settings.optimize {
                GoalOptimization::Minimize => evaluated_grids
                    .try_fold(None, |acc, item| -> Result<_> {
                        let item = item?;
                        Ok(Some(match acc {
                            None => item,
                            Some(current_min) => Ord::min(current_min, item),
                        }))
                    })?
                    .expect(GENERATE_NO_GRIDS),
                GoalOptimization::Maximize => evaluated_grids
                    .try_fold(None, |acc, item| -> Result<_> {
                        let item = item?;
                        Ok(Some(match acc {
                            None => item,
                            Some(current_max) => Ord::max(current_max, item),
                        }))
                    })?
                    .expect(GENERATE_NO_GRIDS),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        base::consts::*,
        generator::{Generator, PruningSettings},
        solver::strategic::strategies::*,
    };

    mod grid_metric {
        use super::*;
        use crate::samples;
        use rstest::rstest;

        mod evaluate {
            use super::*;
            use crate::test_util::init_test_logger;

            #[rstest]
            #[case::strategy_score(0, GridMetric::StrategyScore, 8)]
            #[case::strategy_score(1, GridMetric::StrategyScore, 12)]
            #[case::strategy_score(2, GridMetric::StrategyScore, 12)]
            #[case::strategy_application_count_any(0, GridMetric::StrategyApplicationCountAny, 1)]
            #[case::strategy_application_count_any(1, GridMetric::StrategyApplicationCountAny, 4)]
            #[case::strategy_application_count_any(2, GridMetric::StrategyApplicationCountAny, 2)]
            #[case::strategy_application_count_single_naked_singles(0, GridMetric::StrategyApplicationCountSingle {strategy: NakedSingles.into() }, 1)]
            #[case::strategy_application_count_single_naked_singles(1, GridMetric::StrategyApplicationCountSingle {strategy: NakedSingles.into() }, 4)]
            #[case::strategy_application_count_single_naked_singles(2, GridMetric::StrategyApplicationCountSingle {strategy: NakedSingles.into() }, 2)]
            #[case::strategy_application_count_single_hidden_singles(0, GridMetric::StrategyApplicationCountSingle {strategy: HiddenSingles.into() }, 0)]
            #[case::strategy_application_count_single_hidden_singles(1, GridMetric::StrategyApplicationCountSingle {strategy: HiddenSingles.into() }, 0)]
            #[case::strategy_application_count_single_hidden_singles(2, GridMetric::StrategyApplicationCountSingle {strategy: HiddenSingles.into() }, 0)]
            #[case::strategy_application_count_single_naked_pairs(0, GridMetric::StrategyApplicationCountSingle {strategy: NakedPairs.into() }, 0)]
            #[case::strategy_application_count_single_naked_pairs(1, GridMetric::StrategyApplicationCountSingle {strategy: NakedPairs.into() }, 0)]
            #[case::strategy_application_count_single_naked_pairs(2, GridMetric::StrategyApplicationCountSingle {strategy: NakedPairs.into() }, 0)]
            #[case::strategy_deduction_count(0, GridMetric::StrategyDeductionCountAny, 8)]
            #[case::strategy_deduction_count(1, GridMetric::StrategyDeductionCountAny, 12)]
            #[case::strategy_deduction_count(2, GridMetric::StrategyDeductionCountAny, 12)]
            // TODO: StrategyDeductionCountSingle
            #[case::strategy_average_options(0, GridMetric::StrategyAverageOptions, 2000)]
            #[case::strategy_average_options(1, GridMetric::StrategyAverageOptions, 2750)]
            #[case::strategy_average_options(2, GridMetric::StrategyAverageOptions, 3000)]
            // #[case::solve_graph_average_branching_factor(
            //     0,
            //     GridMetric::SolveGraphAverageBranchingFactor,
            //     0
            // )]
            #[case::sat_step_count(0, GridMetric::SatStepCount, 1)]
            #[case::sat_step_count(1, GridMetric::SatStepCount, 1)]
            #[case::sat_step_count(2, GridMetric::SatStepCount, 1)]
            #[case::backtrack_count(0, GridMetric::BacktrackCount, 0)]
            #[case::backtrack_count(1, GridMetric::BacktrackCount, 0)]
            #[case::backtrack_count(2, GridMetric::BacktrackCount, 0)]
            #[case::grid_givens_count(0, GridMetric::GridGivensCount, 8)]
            #[case::grid_givens_count(1, GridMetric::GridGivensCount, 4)]
            #[case::grid_givens_count(2, GridMetric::GridGivensCount, 4)]
            #[case::grid_direct_candidates_count(0, GridMetric::GridDirectCandidatesCount, 8)]
            #[case::grid_direct_candidates_count(1, GridMetric::GridDirectCandidatesCount, 28)]
            #[case::grid_givens_value_count_deviation(0, GridMetric::GridGivensValueCountDeviation, 0)]
            #[case::grid_givens_value_count_deviation(1, GridMetric::GridGivensValueCountDeviation, 707)]
            #[case::grid_givens_value_count_deviation(2, GridMetric::GridGivensValueCountDeviation, 0)]
            fn test_base_2(
                #[case] grid_sample_index: usize,
                #[case] grid_metric: GridMetric,
                #[case] expected: EvaluatedGridMetric,
            ) {
                type Base = Base2;

                init_test_logger();

                let grid_sample = samples::grid::<Base>(grid_sample_index);

                let strategies = StrategyEnum::default_solver_strategies_no_brute_force();

                assert_eq!(
                    grid_metric.evaluate(&grid_sample, strategies).unwrap(),
                    expected
                );
            }

            #[rstest]
            #[case::strategy_score(1, GridMetric::StrategyScore, 47)]
            #[case::strategy_application_count_any(1, GridMetric::StrategyApplicationCountAny, 4)]
            #[case::strategy_application_count_single_naked_singles(1, GridMetric::StrategyApplicationCountSingle {strategy: NakedSingles.into() }, 4)]
            #[case::strategy_application_count_single_hidden_singles(1, GridMetric::StrategyApplicationCountSingle {strategy: HiddenSingles.into() }, 0)]
            #[case::strategy_application_count_single_naked_pairs(1, GridMetric::StrategyApplicationCountSingle {strategy: NakedPairs.into() }, 0)]
            #[case::strategy_application_count_single_naked_singles(6, GridMetric::StrategyApplicationCountSingle {strategy: NakedSingles.into() }, 19)]
            #[case::strategy_application_count_single_hidden_singles(6, GridMetric::StrategyApplicationCountSingle {strategy: HiddenSingles.into() }, 10)]
            #[case::strategy_application_count_single_naked_pairs(6, GridMetric::StrategyApplicationCountSingle {strategy: NakedPairs.into() }, 1)]
            #[case::strategy_application_count_single_locked_sets(6, GridMetric::StrategyApplicationCountSingle {strategy: LockedSets.into() }, 2)]
            #[case::strategy_application_count_single_group_intersection_both(6, GridMetric::StrategyApplicationCountSingle {strategy: GroupIntersectionBoth.into() }, 2)]
            #[case::strategy_application_count_single_x_wing(6, GridMetric::StrategyApplicationCountSingle {strategy: XWing.into() }, 0)]
            #[case::strategy_deduction_count_any(1, GridMetric::StrategyDeductionCountAny, 47)]
            // TODO: StrategyDeductionCountSingle
            #[case::strategy_average_options(1, GridMetric::StrategyAverageOptions, 4000)]
            #[case::sat_step_count(0, GridMetric::SatStepCount, 77)]
            #[case::sat_step_count(1, GridMetric::SatStepCount, 1)]
            #[case::backtrack_count(0, GridMetric::BacktrackCount, 13357)]
            #[case::backtrack_count(1, GridMetric::BacktrackCount, 0)]
            #[case::grid_givens_count(0, GridMetric::GridGivensCount, 21)]
            #[case::grid_givens_count(1, GridMetric::GridGivensCount, 34)]
            #[case::grid_direct_candidates_count(0, GridMetric::GridDirectCandidatesCount, 254)]
            #[case::grid_direct_candidates_count(1, GridMetric::GridDirectCandidatesCount, 115)]
            fn test_base_3(
                #[case] grid_sample_index: usize,
                #[case] grid_metric: GridMetric,
                #[case] expected: EvaluatedGridMetric,
            ) {
                type Base = Base3;
                init_test_logger();

                let grid_sample = samples::grid::<Base>(grid_sample_index);

                let strategies = StrategyEnum::default_solver_strategies_no_brute_force();

                assert_eq!(
                    grid_metric.evaluate(&grid_sample, strategies).unwrap(),
                    expected
                );
            }
        }
    }

    #[test]
    fn test_one_iteration_against_single_shot_generator() {
        type Base = Base2;
        let generator_settings = GeneratorSettings {
            prune: Some(PruningSettings {
                strategies: StrategyEnum::default_solver_strategies_no_brute_force(),
                ..Default::default()
            }),
            solution: None,
            seed: Some(42),
        };
        let generator: Generator<Base2> = Generator::with_settings(generator_settings.clone());
        let single_shot_grid = generator.generate().unwrap();

        let multi_shot_generator =
            MultiShotGenerator::<Base>::with_settings(MultiShotGeneratorSettings {
                generator_settings,
                iterations: 1,
                ..Default::default()
            })
            .unwrap();

        let multi_shot_grid = multi_shot_generator.generate().unwrap().grid;
        assert_eq!(
            single_shot_grid, multi_shot_grid,
            "Single shot and multi shot grids should be equal"
        );
    }

    #[test]
    fn test_parallel_vs_sequential() {
        type Base = Base2;

        let generator_settings = GeneratorSettings {
            prune: Some(PruningSettings {
                strategies: StrategyEnum::default_solver_strategies_no_brute_force(),
                ..Default::default()
            }),
            solution: None,
            seed: Some(42),
        };
        let multi_shot_generator_par =
            MultiShotGenerator::<Base>::with_settings(MultiShotGeneratorSettings {
                generator_settings: generator_settings.clone(),
                iterations: 100,
                parallel: true,
                ..Default::default()
            })
            .unwrap();
        let grid_par = multi_shot_generator_par.generate().unwrap().grid;

        let multi_shot_generator_seq =
            MultiShotGenerator::<Base>::with_settings(MultiShotGeneratorSettings {
                generator_settings,
                iterations: 100,
                parallel: false,
                ..Default::default()
            })
            .unwrap();

        let grid_seq = multi_shot_generator_seq.generate().unwrap().grid;

        assert_eq!(
            grid_par, grid_seq,
            "parallel should have no effect on the output grid"
        );
    }

    #[test]
    fn test_generate_with_progress() {
        type Base = Base3;

        let generator_settings = GeneratorSettings {
            prune: Some(PruningSettings {
                strategies: StrategySet::with_single(NakedSingles.into()),
                ..Default::default()
            }),
            solution: None,
            seed: Some(42),
        };
        let iterations = 3;
        let multi_shot_generator_par =
            MultiShotGenerator::<Base>::with_settings(MultiShotGeneratorSettings {
                generator_settings: generator_settings.clone(),
                iterations,
                parallel: true,
                ..Default::default()
            })
            .unwrap();

        let mut progress_vec = vec![];

        let evaluated_grid = multi_shot_generator_par
            .generate_with_progress(|progress| {
                progress_vec.push(progress);
                Ok(())
            })
            .unwrap();

        assert_eq!(
            progress_vec.len(),
            usize::try_from(iterations * 2).unwrap(),
            "Progress vector should have 2 * iterations elements, on start and finish"
        );

        for progress in &progress_vec {
            assert_eq!(progress.total_iterations(), iterations);
            assert!(
                progress.current_iteration() < progress.total_iterations(),
                "Progress current iteration should be less than total iterations count"
            );
            if let MultiShotGeneratorProgress::Finished {
                current_evaluated_grid_metric,
                best_evaluated_grid_metric,
                ..
            } = progress
            {
                assert!(current_evaluated_grid_metric <= best_evaluated_grid_metric);
            }
        }
        let best_progress = progress_vec
            .into_iter()
            .max_by_key(|progress| {
                if let MultiShotGeneratorProgress::Finished {
                    current_evaluated_grid_metric,
                    ..
                } = *progress
                {
                    current_evaluated_grid_metric
                } else {
                    EvaluatedGridMetric::MIN
                }
            })
            .unwrap();

        let MultiShotGeneratorProgress::Finished {
            current_evaluated_grid_metric,
            best_evaluated_grid_metric,
            ..
        } = best_progress
        else {
            panic!("Best progress should be a finished progress")
        };
        assert_eq!(current_evaluated_grid_metric, best_evaluated_grid_metric);

        assert_eq!(
            current_evaluated_grid_metric, evaluated_grid.evaluated_grid_metric,
            "Best progress evaluated grid metric should be equal to the returned grid metric"
        );
    }
}
