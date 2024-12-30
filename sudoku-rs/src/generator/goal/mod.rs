use anyhow::ensure;
use log::info;

use crate::error::Result;
use crate::grid::Grid;
use crate::solver::strategic;
use crate::{base::SudokuBase, solver::strategic::strategies::StrategyScore};

use super::Generator;

#[derive(Debug)]
pub struct GoalGenerator<Base: SudokuBase> {
    generator: Generator<Base>,
}

impl<Base: SudokuBase> GoalGenerator<Base> {
    pub fn new(generator: Generator<Base>) -> Result<Self> {
        ensure!(
            generator.settings.prune.is_some(),
            "GoalGenerator requires pruning settings"
        );
        ensure!(
            generator.settings.seed.is_none(),
            "GoalGenerator does not support seeding"
        );
        Ok(Self { generator })
    }

    pub fn generate_for_total_strategy_score(
        &self,
        iterations: u64,
    ) -> (StrategyScore, Grid<Base>) {
        // use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
        use rayon::prelude::*;

        // let pb = ProgressBar::new(iterations).with_style(ProgressStyle::default_bar().template(
        //     "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta}, {per_sec})",
        // )?);

        let (total_score, grid) = (0..iterations)
            .into_par_iter()
            // .progress_with(pb)
            .map(|i| -> Result<_> {
                info!("Iteration {}", i);

                let grid = self.generator.generate()?;

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
            .max_by_key(|res| res.as_ref().map_or(0, |(total_score, _)| *total_score))
            .unwrap()
            .unwrap();

        (total_score, grid)
    }
}
