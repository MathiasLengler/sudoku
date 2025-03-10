use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::ensure;
use log::*;

use crate::base::SudokuBase;
use crate::error::Result;
use crate::grid::Grid;
use crate::solver::strategic;
use crate::solver::strategic::strategies::StrategyEnum;

use super::{Generator, GeneratorSettings};

pub use dynamic_settings::*;

pub type EvaluatedGridMetric = u32;

static GENERATE_NO_GRIDS: &str = "at least one generation result";

/// A metric used to evaluate the difficulty of a grid.
#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GridMetric {
    // Based on `strategic::SolverPathIter` - a single solve path determined by the solver.
    /// Weighted sum of all strategy scores used to solve the grid. `Strategy::score() * Number of deductions made by the strategy`
    #[default]
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

impl GridMetric {
    pub fn evaluate<Base: SudokuBase>(
        self,
        grid: &Grid<Base>,
        strategies: Vec<StrategyEnum>,
    ) -> Result<EvaluatedGridMetric> {
        Ok(match self {
            GridMetric::StrategyTotalScore => strategic::SolverBuilder::new(grid.clone())
                .strategies(strategies)
                .build()
                .total_score()?
                .unwrap(),
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
    fn get_prune_strategies(&self) -> Vec<StrategyEnum> {
        self.generator_settings
            .prune
            .as_ref()
            .map(|prune| prune.strategies.clone())
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

#[derive(Debug)]
pub struct MultiShotGeneratorProgress {
    pub current_iteration: IterationsCounter,
    pub total_iterations: IterationsCounter,
    pub current_evaluated_grid_metric: EvaluatedGridMetric,
    pub best_evaluated_grid_metric: EvaluatedGridMetric,
}

#[derive(Debug)]
pub struct MultiShotGenerationReturn<Base: SudokuBase> {
    pub evaluated_grid_metric: EvaluatedGridMetric,
    pub grid: Grid<Base>,
}

impl<Base: SudokuBase + Eq> Eq for MultiShotGenerationReturn<Base> {}

impl<Base: SudokuBase + PartialEq> PartialEq for MultiShotGenerationReturn<Base> {
    fn eq(&self, other: &Self) -> bool {
        self.evaluated_grid_metric == other.evaluated_grid_metric
    }
}

impl<Base: SudokuBase + Ord> Ord for MultiShotGenerationReturn<Base> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.evaluated_grid_metric.cmp(&other.evaluated_grid_metric)
    }
}

impl<Base: SudokuBase + PartialOrd> PartialOrd for MultiShotGenerationReturn<Base> {
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

    fn generate_single(&self, iteration: IterationsCounter) -> Result<Grid<Base>> {
        debug!("Generate iteration {iteration}");

        Ok(if let Some(seed) = self.settings.generator_settings.seed {
            Generator::with_settings(GeneratorSettings {
                seed: Some(seed + u64::from(iteration)),
                ..self.settings.generator_settings.clone()
            })
            .generate()?
        } else {
            Generator::with_settings(self.settings.generator_settings.clone()).generate()?
        })
    }

    fn evaluate_grid(&self, grid: Grid<Base>) -> Result<MultiShotGenerationReturn<Base>> {
        let evaluated_grid_metric = self
            .settings
            .metric
            .evaluate(&grid, self.settings.get_prune_strategies())?;
        Ok(MultiShotGenerationReturn {
            evaluated_grid_metric,
            grid,
        })
    }

    pub fn generate(&self) -> Result<MultiShotGenerationReturn<Base>> {
        self.generate_with_inspect_evaluated_grids(|_, _| Ok(()))
    }

    pub fn generate_with_progress(
        &self,
        on_progress: impl Fn(MultiShotGeneratorProgress) -> Result<()>,
    ) -> Result<MultiShotGenerationReturn<Base>> {
        let mut ret = None;

        // `on_progress` is neither `Sync` nor `Send`, because the intended use case is a WASM/JS callback.
        // We generate exlusively on the rayon threadpool, sending the progress to the main thread.
        rayon::in_place_scope::<_, Result<_>>(|s| {
            use std::sync::mpsc;

            let (on_progress_sender, on_progress_receiver) =
                mpsc::channel::<MultiShotGeneratorProgress>();

            s.spawn(|_s| {
                ret = Some(self.generate_with_inspect_evaluated_grids(
                    move |current_iteration, evaluated_grid| {
                        let progress = MultiShotGeneratorProgress {
                            current_iteration,
                            total_iterations: self.settings.iterations,
                            current_evaluated_grid_metric: evaluated_grid.evaluated_grid_metric,
                            // TODO: track best evaluated grid metric
                            best_evaluated_grid_metric: 0,
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

    fn generate_with_inspect_evaluated_grids(
        &self,
        inspect_evaluated_grids: impl Fn(IterationsCounter, &MultiShotGenerationReturn<Base>) -> Result<()>
            + Sync
            + Send,
    ) -> Result<MultiShotGenerationReturn<Base>> {
        let process_iteration = |iteration| -> Result<_> {
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
    use crate::{
        base::consts::*,
        generator::{Generator, PruningSettings},
    };

    use super::*;

    #[test]
    fn test_one_iteration_against_single_shot_generator() {
        type Base = Base2;
        let generator_settings = GeneratorSettings {
            prune: Some(PruningSettings {
                strategies: StrategyEnum::default_solver_strategies_no_backtracking(),
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
                strategies: StrategyEnum::default_solver_strategies_no_backtracking(),
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
        type Base = Base2;

        let generator_settings = GeneratorSettings {
            prune: Some(PruningSettings {
                strategies: StrategyEnum::default_solver_strategies_no_backtracking(),
                ..Default::default()
            }),
            solution: None,
            seed: Some(42),
        };
        let multi_shot_generator_par =
            MultiShotGenerator::<Base>::with_settings(MultiShotGeneratorSettings {
                generator_settings: generator_settings.clone(),
                iterations: 3,
                parallel: true,
                ..Default::default()
            })
            .unwrap();
        let grid_par = multi_shot_generator_par
            .generate_with_progress(|progress| {
                // TODO: assert that we where called three times and that the best progress matches the return.
                dbg!(progress);
                Ok(())
            })
            .unwrap()
            .grid;
    }
}
