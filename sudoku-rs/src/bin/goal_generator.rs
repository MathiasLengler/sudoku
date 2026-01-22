use env_logger::Env;

use sudoku::base::consts::*;
use sudoku::error::Result;
use sudoku::generator::multi_shot::{
    EvaluatedGrid, GoalOptimization, GridMetric, MultiShotGenerator, MultiShotGeneratorSettings,
};
use sudoku::generator::{GeneratorSettings, PruningSettings};
use sudoku::solver::strategic::strategies::*;

type Base = Base3;

fn main() -> Result<()> {
    env_logger::Builder::from_env(
        Env::default().default_filter_or("info,varisat=warn,sudoku::generator::goal=debug"),
    )
    .format_indent(Some(0))
    .init();

    let generator = MultiShotGenerator::<Base>::with_settings(MultiShotGeneratorSettings {
        generator_settings: GeneratorSettings {
            prune: Some(PruningSettings {
                strategies: StrategyEnum::default_solver_strategies_no_brute_force(),
                ..Default::default()
            }),
            ..Default::default()
        },
        iterations: 100_000,
        metric: GridMetric::StrategyDeductionCountSingle {
            strategy: XWing.into(),
        },
        optimize: GoalOptimization::Maximize,
        parallel: true,
    })?;

    let EvaluatedGrid {
        grid,
        evaluated_grid_metric,
    } = generator.generate_with_progress(|progress| {
        if progress.current_iteration() % 1000 == 0 {
            println!("Progress: {progress:?}");
        }
        Ok(())
    })?;

    println!("Evaluated grid metric: {}\n{grid}", evaluated_grid_metric);

    Ok(())
}
