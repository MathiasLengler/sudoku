use clap::Parser;
use env_logger::Env;
use log::*;
use std::time::Instant;
use sudoku::{
    DynamicSudoku,
    base::BaseEnum,
    error::Result,
    generator::{
        DynamicGeneratorSettings, DynamicPruningSettings,
        multi_shot::{DynamicMultiShotGeneratorSettings, GoalOptimization, GridMetric},
    },
    solver::strategic::strategies::{XWing, selection::StrategySet},
};

fn parse_generator_settings(s: &str) -> Result<DynamicMultiShotGeneratorSettings> {
    let settings = serde_json::from_str(s)?;
    Ok(settings)
}

fn default_generator_settings() -> DynamicMultiShotGeneratorSettings {
    DynamicMultiShotGeneratorSettings {
        generator_settings: DynamicGeneratorSettings {
            base: BaseEnum::Base3,
            prune: Some(DynamicPruningSettings {
                strategies: StrategySet::default_solver_strategies_no_brute_force(),
                ..Default::default()
            }),
            ..Default::default()
        },
        iterations: 10_000,
        metric: GridMetric::StrategyDeductionCountSingle {
            strategy: XWing.into(),
        },
        optimize: GoalOptimization::Maximize,
        parallel: true,
    }
}

/// Generate a single Sudoku puzzle using the multi-shot generator
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Multi-shot generator settings in JSON format
    #[arg(short, long, value_parser = parse_generator_settings, default_value_t = default_generator_settings())]
    generator_settings: DynamicMultiShotGeneratorSettings,
}

fn main() -> Result<()> {
    let args = Args::parse();

    env_logger::Builder::from_env(
        Env::default().default_filter_or("info,varisat=warn,sudoku::generator::multi_shot=info"),
    )
    .format_indent(Some(0))
    .init();

    debug!("{:?}", args);

    let before = Instant::now();

    let mut i = 0;
    let grid = DynamicSudoku::generate_multi_shot(args.generator_settings, |progress| {
        i += 1;
        if i % 1_000 == 0 {
            info!("Generation progress: {progress:?}");
        }
        Ok(())
    })?;
    println!("{grid}");
    let after = Instant::now();
    let total_time = after - before;

    dbg!(total_time);

    Ok(())
}
