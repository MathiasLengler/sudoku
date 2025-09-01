//! Statistical analysis of grid metrics

use std::collections::HashMap;
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::Serialize;

use sudoku::base::BaseEnum;
use sudoku::base::SudokuBase;
use sudoku::generator::multi_shot::{EvaluatedGridMetric, GridMetric};
use sudoku::generator::{Generator, GeneratorSettings, PruningSettings};
use sudoku::grid::Grid;
use sudoku::match_base_enum;
use sudoku::solver::strategic::strategies::StrategyEnum;

#[derive(Parser, Debug)]
#[command(name = "metric_analysis")]
#[command(about = "Statistical analysis of grid difficulty metrics")]
struct Args {
    /// Number of grids to generate and analyze
    #[arg(short, long, default_value = "1000")]
    samples: u64,

    /// Sudoku base (2 for 4x4, 3 for 9x9, 4 for 16x16)
    #[arg(short, long, default_value = "3", value_enum)]
    base: BaseEnum,

    /// Random seed for reproducible results
    #[arg(long)]
    seed: Option<u64>,
}

#[derive(Debug, Serialize)]
struct TimedEvaluatedGridMetric {
    value: EvaluatedGridMetric,
    duration: Duration,
}

#[derive(Debug, Serialize)]
struct MetricResult {
    grid_id: u64,
    strategy_score: TimedEvaluatedGridMetric,
    strategy_application_count: TimedEvaluatedGridMetric,
    strategy_deduction_count: TimedEvaluatedGridMetric,
    strategy_average_options: TimedEvaluatedGridMetric,
    sat_step_count: TimedEvaluatedGridMetric,
    backtrack_count: TimedEvaluatedGridMetric,
    grid_givens_count: TimedEvaluatedGridMetric,
    grid_direct_candidates_count: TimedEvaluatedGridMetric,
}

#[derive(Debug)]
struct CorrelationAnalysis {
    pearson_correlations: HashMap<(&'static str, &'static str), f64>,
    metric_stats: HashMap<&'static str, MetricStats>,
    computation_times: HashMap<&'static str, Duration>,
}

#[derive(Debug)]
struct MetricStats {
    mean: f64,
    std_dev: f64,
    min: EvaluatedGridMetric,
    max: EvaluatedGridMetric,
    median: f64,
}

fn evaluate_metric_with_timing<Base: SudokuBase>(
    metric: GridMetric,
    grid: &Grid<Base>,
    strategies: &[StrategyEnum],
) -> Result<TimedEvaluatedGridMetric> {
    let start = Instant::now();
    let value = metric.evaluate(grid, strategies.to_vec())?;
    let duration = start.elapsed();
    Ok(TimedEvaluatedGridMetric { value, duration })
}

fn analyze_single_grid<Base: SudokuBase>(
    grid_id: u64,
    grid: Grid<Base>,
    strategies: &[StrategyEnum],
) -> Result<MetricResult> {
    let strategy_score = evaluate_metric_with_timing(GridMetric::StrategyScore, &grid, strategies)?;
    let strategy_application_count =
        evaluate_metric_with_timing(GridMetric::StrategyApplicationCount, &grid, strategies)?;
    let strategy_deduction_count =
        evaluate_metric_with_timing(GridMetric::StrategyDeductionCount, &grid, strategies)?;
    let strategy_average_options =
        evaluate_metric_with_timing(GridMetric::StrategyAverageOptions, &grid, strategies)?;
    let sat_step_count = evaluate_metric_with_timing(GridMetric::SatStepCount, &grid, strategies)?;
    let backtrack_count =
        evaluate_metric_with_timing(GridMetric::BacktrackCount, &grid, strategies)?;
    let grid_givens_count =
        evaluate_metric_with_timing(GridMetric::GridGivensCount, &grid, strategies)?;
    let grid_direct_candidates_count =
        evaluate_metric_with_timing(GridMetric::GridDirectCandidatesCount, &grid, strategies)?;

    Ok(MetricResult {
        grid_id,
        strategy_score,
        strategy_application_count,
        strategy_deduction_count,
        strategy_average_options,
        sat_step_count,
        backtrack_count,
        grid_givens_count,
        grid_direct_candidates_count,
    })
}

// TODO: replace with linfa
fn calculate_pearson_correlation(x: &[f64], y: &[f64]) -> f64 {
    assert_eq!(x.len(), y.len());
    let n = x.len() as f64;

    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;

    let numerator: f64 = x
        .iter()
        .zip(y)
        .map(|(xi, yi)| (xi - mean_x) * (yi - mean_y))
        .sum();
    let sum_sq_x: f64 = x.iter().map(|xi| (xi - mean_x).powi(2)).sum();
    let sum_sq_y: f64 = y.iter().map(|yi| (yi - mean_y).powi(2)).sum();

    let denominator = (sum_sq_x * sum_sq_y).sqrt();

    if denominator == 0.0 {
        0.0
    } else {
        numerator / denominator
    }
}

fn calculate_metric_stats(values: &[EvaluatedGridMetric]) -> MetricStats {
    let f64_values: Vec<f64> = values.iter().map(|&v| v as f64).collect();
    let mean = f64_values.iter().sum::<f64>() / f64_values.len() as f64;

    let variance =
        f64_values.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / f64_values.len() as f64;
    let std_dev = variance.sqrt();

    let min = *values.iter().min().unwrap();
    let max = *values.iter().max().unwrap();

    let mut sorted_values = f64_values.clone();
    sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = if sorted_values.len() % 2 == 0 {
        (sorted_values[sorted_values.len() / 2 - 1] + sorted_values[sorted_values.len() / 2]) / 2.0
    } else {
        sorted_values[sorted_values.len() / 2]
    };

    MetricStats {
        mean,
        std_dev,
        min,
        max,
        median,
    }
}

fn analyze_results(results: &[MetricResult]) -> CorrelationAnalysis {
    let metric_names = [
        "strategy_score",
        "strategy_application_count",
        "strategy_deduction_count",
        "strategy_average_options",
        "sat_step_count",
        "backtrack_count",
        "grid_givens_count",
        "grid_direct_candidates_count",
    ];

    // Extract metric values
    let metric_values: HashMap<&'static str, Vec<EvaluatedGridMetric>> = [
        (
            "strategy_score",
            results.iter().map(|r| r.strategy_score.value).collect(),
        ),
        (
            "strategy_application_count",
            results
                .iter()
                .map(|r| r.strategy_application_count.value)
                .collect(),
        ),
        (
            "strategy_deduction_count",
            results
                .iter()
                .map(|r| r.strategy_deduction_count.value)
                .collect(),
        ),
        (
            "strategy_average_options",
            results
                .iter()
                .map(|r| r.strategy_average_options.value)
                .collect(),
        ),
        (
            "sat_step_count",
            results.iter().map(|r| r.sat_step_count.value).collect(),
        ),
        (
            "backtrack_count",
            results.iter().map(|r| r.backtrack_count.value).collect(),
        ),
        (
            "grid_givens_count",
            results.iter().map(|r| r.grid_givens_count.value).collect(),
        ),
        (
            "grid_direct_candidates_count",
            results
                .iter()
                .map(|r| r.grid_direct_candidates_count.value)
                .collect(),
        ),
    ]
    .into_iter()
    .collect();

    // Calculate correlations
    let mut pearson_correlations = HashMap::new();
    for i in 0..metric_names.len() {
        for j in (i + 1)..metric_names.len() {
            let metric1 = metric_names[i];
            let metric2 = metric_names[j];

            let values1: Vec<f64> = metric_values[metric1].iter().map(|&v| v as f64).collect();
            let values2: Vec<f64> = metric_values[metric2].iter().map(|&v| v as f64).collect();

            let correlation = calculate_pearson_correlation(&values1, &values2);
            pearson_correlations.insert((metric1, metric2), correlation);
        }
    }

    // Calculate metric statistics
    let metric_stats: HashMap<&'static str, MetricStats> = metric_values
        .into_iter()
        .map(|(name, values)| (name, calculate_metric_stats(&values)))
        .collect();

    // Calculate average computation times
    let computation_times: HashMap<&'static str, Duration> = [
        (
            "strategy_score",
            results
                .iter()
                .map(|r| r.strategy_score.duration)
                .sum::<Duration>()
                / results.len() as u32,
        ),
        (
            "strategy_application_count",
            results
                .iter()
                .map(|r| r.strategy_application_count.duration)
                .sum::<Duration>()
                / results.len() as u32,
        ),
        (
            "strategy_deduction_count",
            results
                .iter()
                .map(|r| r.strategy_deduction_count.duration)
                .sum::<Duration>()
                / results.len() as u32,
        ),
        (
            "strategy_average_options",
            results
                .iter()
                .map(|r| r.strategy_average_options.duration)
                .sum::<Duration>()
                / results.len() as u32,
        ),
        (
            "sat_step_count",
            results
                .iter()
                .map(|r| r.sat_step_count.duration)
                .sum::<Duration>()
                / results.len() as u32,
        ),
        (
            "backtrack_count",
            results
                .iter()
                .map(|r| r.backtrack_count.duration)
                .sum::<Duration>()
                / results.len() as u32,
        ),
        (
            "grid_givens_count",
            results
                .iter()
                .map(|r| r.grid_givens_count.duration)
                .sum::<Duration>()
                / results.len() as u32,
        ),
        (
            "grid_direct_candidates_count",
            results
                .iter()
                .map(|r| r.grid_direct_candidates_count.duration)
                .sum::<Duration>()
                / results.len() as u32,
        ),
    ]
    .into_iter()
    .collect();

    CorrelationAnalysis {
        pearson_correlations,
        metric_stats,
        computation_times,
    }
}

fn print_analysis_summary(analysis: &CorrelationAnalysis) {
    println!("\n=== METRIC ANALYSIS SUMMARY ===\n");

    println!("📊 METRIC STATISTICS:");
    println!(
        "{:<30} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "Metric", "Mean", "StdDev", "Min", "Max", "Median"
    );
    println!("{}", "-".repeat(90));

    for (metric, stats) in &analysis.metric_stats {
        println!(
            "{:<30} {:>10.2} {:>10.2} {:>10} {:>10} {:>10.2}",
            metric, stats.mean, stats.std_dev, stats.min, stats.max, stats.median
        );
    }

    println!("\n⏱️  COMPUTATION TIMES (average per grid):");
    println!("{:<30} {:>15}", "Metric", "Time");
    println!("{}", "-".repeat(45));

    let mut time_pairs: Vec<_> = analysis.computation_times.iter().collect();
    time_pairs.sort_by_key(|(_, duration)| *duration);

    for (metric, duration) in time_pairs {
        println!("{:<30} {:>15?}", metric, duration);
    }

    println!("\n🔗 CORRELATIONS (Pearson correlation coefficient):");
    println!("{:<35} {:>10}", "Metric Pair", "Correlation");
    println!("{}", "-".repeat(45));

    let mut correlations: Vec<_> = analysis.pearson_correlations.iter().collect();
    correlations.sort_by(|a, b| b.1.abs().partial_cmp(&a.1.abs()).unwrap());

    for ((metric1, metric2), correlation) in correlations {
        println!(
            "{:<35} {:>10.3}",
            format!("{} vs {}", metric1, metric2),
            correlation
        );
    }

    println!("\n🎯 KEY INSIGHTS:");

    // Find highly correlated metrics (|correlation| > 0.8)
    let high_correlations: Vec<_> = analysis
        .pearson_correlations
        .iter()
        .filter(|(_, &corr)| corr.abs() > 0.8)
        .collect();

    if !high_correlations.is_empty() {
        println!("   • Highly correlated metrics (|r| > 0.8) - potential redundancy:");
        for ((m1, m2), corr) in high_correlations {
            println!("     - {} ↔ {}: {:.3}", m1, m2, corr);
        }
    }

    // Find fastest metrics
    let fastest_metric = analysis
        .computation_times
        .iter()
        .min_by_key(|(_, duration)| *duration)
        .unwrap();
    println!(
        "   • Fastest metric to compute: {} ({:?})",
        fastest_metric.0, fastest_metric.1
    );

    // Find slowest metrics
    let slowest_metric = analysis
        .computation_times
        .iter()
        .max_by_key(|(_, duration)| *duration)
        .unwrap();
    println!(
        "   • Slowest metric to compute: {} ({:?})",
        slowest_metric.0, slowest_metric.1
    );
}

fn run_analysis<Base: SudokuBase>() -> Result<()> {
    let args = Args::parse();

    println!(
        "🎲 Starting metric analysis for {} grids (Base {})",
        args.samples, args.base
    );

    let strategies = StrategyEnum::default_solver_strategies_no_brute_force();

    let generator_settings = GeneratorSettings {
        prune: Some(PruningSettings {
            strategies: strategies.clone(),
            ..Default::default()
        }),
        seed: args.seed,
        ..Default::default()
    };

    let pb = ProgressBar::new(args.samples).with_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta}, {per_sec})")?
    );

    let results: Result<Vec<MetricResult>> = (0..args.samples)
        .into_par_iter()
        .progress_with(pb)
        .map(|i| {
            let generator_settings = if let Some(base_seed) = args.seed {
                GeneratorSettings {
                    seed: Some(base_seed + (i)),
                    ..generator_settings.clone()
                }
            } else {
                generator_settings.clone()
            };

            let generator = Generator::<Base>::with_settings(generator_settings);
            let grid = generator.generate()?;

            analyze_single_grid(i, grid, &strategies)
        })
        .collect();

    let results = results?;

    // Perform statistical analysis
    let analysis = analyze_results(&results);
    print_analysis_summary(&analysis);

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    match_base_enum!(args.base, run_analysis::<Base>())
}
