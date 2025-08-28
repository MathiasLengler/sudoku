//! Statistical analysis of grid metrics
//!
//! This binary addresses the TODO in `multi_shot/mod.rs`:
//! - Analyzes correlations between different metrics
//! - Identifies redundant metrics
//! - Measures computation speed of each metric
//! - Provides foundation for correlation with human difficulty ratings
//!
//! Usage:
//! ```bash
//! cargo run --bin metric_analysis --features="log,parallel,terminal" -- --samples 1000 --base 3 --output metrics_analysis.csv
//! ```

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde::Serialize;

use sudoku::base::consts::*;
use sudoku::base::SudokuBase;
use sudoku::generator::multi_shot::{EvaluatedGridMetric, GridMetric};
use sudoku::generator::{Generator, GeneratorSettings, PruningSettings};
use sudoku::grid::Grid;
use sudoku::solver::strategic::strategies::StrategyEnum;

#[derive(Parser, Debug)]
#[command(name = "metric_analysis")]
#[command(about = "Statistical analysis of grid difficulty metrics")]
struct Args {
    /// Number of grids to generate and analyze
    #[arg(short, long, default_value = "1000")]
    samples: usize,

    /// Sudoku base (2 for 4x4, 3 for 9x9, 4 for 16x16)
    #[arg(short, long, default_value = "3")]
    base: u8,

    /// Output CSV file path
    #[arg(short, long, default_value = "metrics_analysis.csv")]
    output: String,

    /// Random seed for reproducible results
    #[arg(long)]
    seed: Option<u64>,

    /// Enable parallel processing
    #[arg(long, default_value = "true")]
    parallel: bool,
}

#[derive(Debug, Serialize)]
struct MetricResult {
    grid_id: usize,
    strategy_score: EvaluatedGridMetric,
    strategy_application_count: EvaluatedGridMetric,
    strategy_deduction_count: EvaluatedGridMetric,
    strategy_average_options: EvaluatedGridMetric,
    sat_step_count: EvaluatedGridMetric,
    backtrack_count: EvaluatedGridMetric,
    grid_givens_count: EvaluatedGridMetric,
    grid_direct_candidates_count: EvaluatedGridMetric,

    // Computation times (in nanoseconds)
    strategy_score_time_ns: u64,
    strategy_application_count_time_ns: u64,
    strategy_deduction_count_time_ns: u64,
    strategy_average_options_time_ns: u64,
    sat_step_count_time_ns: u64,
    backtrack_count_time_ns: u64,
    grid_givens_count_time_ns: u64,
    grid_direct_candidates_count_time_ns: u64,
}

#[derive(Debug)]
struct CorrelationAnalysis {
    pearson_correlations: HashMap<(String, String), f64>,
    metric_stats: HashMap<String, MetricStats>,
    computation_times: HashMap<String, Duration>,
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
) -> Result<(EvaluatedGridMetric, Duration)> {
    let start = Instant::now();
    let result = metric.evaluate(grid, strategies.to_vec())?;
    let duration = start.elapsed();
    Ok((result, duration))
}

fn analyze_single_grid<Base: SudokuBase>(
    grid_id: usize,
    grid: Grid<Base>,
    strategies: &[StrategyEnum],
) -> Result<MetricResult> {
    let (strategy_score, strategy_score_time) =
        evaluate_metric_with_timing(GridMetric::StrategyScore, &grid, strategies)?;
    let (strategy_application_count, strategy_application_count_time) =
        evaluate_metric_with_timing(GridMetric::StrategyApplicationCount, &grid, strategies)?;
    let (strategy_deduction_count, strategy_deduction_count_time) =
        evaluate_metric_with_timing(GridMetric::StrategyDeductionCount, &grid, strategies)?;
    let (strategy_average_options, strategy_average_options_time) =
        evaluate_metric_with_timing(GridMetric::StrategyAverageOptions, &grid, strategies)?;
    let (sat_step_count, sat_step_count_time) =
        evaluate_metric_with_timing(GridMetric::SatStepCount, &grid, strategies)?;
    let (backtrack_count, backtrack_count_time) =
        evaluate_metric_with_timing(GridMetric::BacktrackCount, &grid, strategies)?;
    let (grid_givens_count, grid_givens_count_time) =
        evaluate_metric_with_timing(GridMetric::GridGivensCount, &grid, strategies)?;
    let (grid_direct_candidates_count, grid_direct_candidates_count_time) =
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
        strategy_score_time_ns: strategy_score_time.as_nanos() as u64,
        strategy_application_count_time_ns: strategy_application_count_time.as_nanos() as u64,
        strategy_deduction_count_time_ns: strategy_deduction_count_time.as_nanos() as u64,
        strategy_average_options_time_ns: strategy_average_options_time.as_nanos() as u64,
        sat_step_count_time_ns: sat_step_count_time.as_nanos() as u64,
        backtrack_count_time_ns: backtrack_count_time.as_nanos() as u64,
        grid_givens_count_time_ns: grid_givens_count_time.as_nanos() as u64,
        grid_direct_candidates_count_time_ns: grid_direct_candidates_count_time.as_nanos() as u64,
    })
}

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
    let metric_values: HashMap<String, Vec<EvaluatedGridMetric>> = [
        (
            "strategy_score",
            results.iter().map(|r| r.strategy_score).collect(),
        ),
        (
            "strategy_application_count",
            results
                .iter()
                .map(|r| r.strategy_application_count)
                .collect(),
        ),
        (
            "strategy_deduction_count",
            results.iter().map(|r| r.strategy_deduction_count).collect(),
        ),
        (
            "strategy_average_options",
            results.iter().map(|r| r.strategy_average_options).collect(),
        ),
        (
            "sat_step_count",
            results.iter().map(|r| r.sat_step_count).collect(),
        ),
        (
            "backtrack_count",
            results.iter().map(|r| r.backtrack_count).collect(),
        ),
        (
            "grid_givens_count",
            results.iter().map(|r| r.grid_givens_count).collect(),
        ),
        (
            "grid_direct_candidates_count",
            results
                .iter()
                .map(|r| r.grid_direct_candidates_count)
                .collect(),
        ),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect();

    // Calculate correlations
    let mut pearson_correlations = HashMap::new();
    for i in 0..metric_names.len() {
        for j in (i + 1)..metric_names.len() {
            let metric1 = &metric_names[i];
            let metric2 = &metric_names[j];

            let values1: Vec<f64> = metric_values[&metric1.to_string()]
                .iter()
                .map(|&v| v as f64)
                .collect();
            let values2: Vec<f64> = metric_values[&metric2.to_string()]
                .iter()
                .map(|&v| v as f64)
                .collect();

            let correlation = calculate_pearson_correlation(&values1, &values2);
            pearson_correlations.insert((metric1.to_string(), metric2.to_string()), correlation);
        }
    }

    // Calculate metric statistics
    let metric_stats: HashMap<String, MetricStats> = metric_values
        .into_iter()
        .map(|(name, values)| (name.clone(), calculate_metric_stats(&values)))
        .collect();

    // Calculate average computation times
    let computation_times: HashMap<String, Duration> = [
        (
            "strategy_score",
            results
                .iter()
                .map(|r| Duration::from_nanos(r.strategy_score_time_ns))
                .sum::<Duration>()
                / results.len() as u32,
        ),
        (
            "strategy_application_count",
            results
                .iter()
                .map(|r| Duration::from_nanos(r.strategy_application_count_time_ns))
                .sum::<Duration>()
                / results.len() as u32,
        ),
        (
            "strategy_deduction_count",
            results
                .iter()
                .map(|r| Duration::from_nanos(r.strategy_deduction_count_time_ns))
                .sum::<Duration>()
                / results.len() as u32,
        ),
        (
            "strategy_average_options",
            results
                .iter()
                .map(|r| Duration::from_nanos(r.strategy_average_options_time_ns))
                .sum::<Duration>()
                / results.len() as u32,
        ),
        (
            "sat_step_count",
            results
                .iter()
                .map(|r| Duration::from_nanos(r.sat_step_count_time_ns))
                .sum::<Duration>()
                / results.len() as u32,
        ),
        (
            "backtrack_count",
            results
                .iter()
                .map(|r| Duration::from_nanos(r.backtrack_count_time_ns))
                .sum::<Duration>()
                / results.len() as u32,
        ),
        (
            "grid_givens_count",
            results
                .iter()
                .map(|r| Duration::from_nanos(r.grid_givens_count_time_ns))
                .sum::<Duration>()
                / results.len() as u32,
        ),
        (
            "grid_direct_candidates_count",
            results
                .iter()
                .map(|r| Duration::from_nanos(r.grid_direct_candidates_count_time_ns))
                .sum::<Duration>()
                / results.len() as u32,
        ),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
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

    println!("\n💡 RECOMMENDATIONS:");
    println!(
        "   • Use fast metrics ({:?}) for real-time evaluation",
        fastest_metric.1
    );
    println!("   • Consider removing redundant metrics with high correlation");
    println!("   • Focus on uncorrelated metrics for diverse difficulty assessment");
    println!("   • Investigate human difficulty correlation with collected data");
}

fn write_csv_results(results: &[MetricResult], output_path: &str) -> Result<()> {
    let mut file = File::create(output_path)?;

    // Write CSV header
    writeln!(file, "grid_id,strategy_score,strategy_application_count,strategy_deduction_count,strategy_average_options,sat_step_count,backtrack_count,grid_givens_count,grid_direct_candidates_count,strategy_score_time_ns,strategy_application_count_time_ns,strategy_deduction_count_time_ns,strategy_average_options_time_ns,sat_step_count_time_ns,backtrack_count_time_ns,grid_givens_count_time_ns,grid_direct_candidates_count_time_ns")?;

    // Write data rows
    for result in results {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            result.grid_id,
            result.strategy_score,
            result.strategy_application_count,
            result.strategy_deduction_count,
            result.strategy_average_options,
            result.sat_step_count,
            result.backtrack_count,
            result.grid_givens_count,
            result.grid_direct_candidates_count,
            result.strategy_score_time_ns,
            result.strategy_application_count_time_ns,
            result.strategy_deduction_count_time_ns,
            result.strategy_average_options_time_ns,
            result.sat_step_count_time_ns,
            result.backtrack_count_time_ns,
            result.grid_givens_count_time_ns,
            result.grid_direct_candidates_count_time_ns
        )?;
    }

    Ok(())
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

    let pb = ProgressBar::new(args.samples as u64).with_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta}, {per_sec})")
            .context("Failed to create progress bar template")?
    );

    let results: Result<Vec<MetricResult>> = if args.parallel {
        (0..args.samples)
            .into_par_iter()
            .map(|i| {
                let generator_settings = if let Some(base_seed) = args.seed {
                    GeneratorSettings {
                        seed: Some(base_seed + (i as u64)),
                        ..generator_settings.clone()
                    }
                } else {
                    generator_settings.clone()
                };

                let generator = Generator::<Base>::with_settings(generator_settings);
                let grid = generator.generate()?;
                pb.inc(1);

                analyze_single_grid(i, grid, &strategies)
            })
            .collect()
    } else {
        (0..args.samples)
            .map(|i| {
                let generator_settings = if let Some(base_seed) = args.seed {
                    GeneratorSettings {
                        seed: Some(base_seed + (i as u64)),
                        ..generator_settings.clone()
                    }
                } else {
                    generator_settings.clone()
                };

                let generator = Generator::<Base>::with_settings(generator_settings);
                let grid = generator.generate()?;
                pb.inc(1);

                analyze_single_grid(i, grid, &strategies)
            })
            .collect()
    };

    pb.finish_with_message("Analysis complete!");

    let results = results?;

    // Perform statistical analysis
    let analysis = analyze_results(&results);
    print_analysis_summary(&analysis);

    // Write results to CSV
    write_csv_results(&results, &args.output)?;
    println!("\n📄 Results written to: {}", args.output);

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.base {
        2 => run_analysis::<Base2>(),
        3 => run_analysis::<Base3>(),
        4 => run_analysis::<Base4>(),
        _ => anyhow::bail!("Unsupported base: {}. Supported bases: 2, 3, 4", args.base),
    }
}
