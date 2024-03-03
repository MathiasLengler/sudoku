use std::time::{Duration, Instant};

use env_logger::Env;

use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use itertools::Itertools;
use rayon::prelude::*;
use sudoku::base::consts::*;
use sudoku::error::Result;
use sudoku::generator::{Generator, GeneratorSettings, PruningSettings, PruningTarget};
use sudoku::grid::Grid;

type Base = Base4;

fn generate(start_from_near_minimal_grid: bool, seed: u64) -> (Grid<Base>, Duration) {
    let before = Instant::now();

    let grid = Generator::<Base4>::with_settings(GeneratorSettings {
        prune: Some(PruningSettings {
            target: PruningTarget::Minimal,
            set_all_direct_candidates: false,
            start_from_near_minimal_grid,
            ..Default::default()
        }),
        solution: None,
        seed: Some(seed),
    })
    .generate()
    .unwrap();
    let after = Instant::now();
    let total_time = after - before;
    (grid, total_time)
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info,varisat=warn"))
        .format_indent(Some(0))
        .init();

    const MAX: u64 = 2_000;
    let pb = ProgressBar::new(MAX).with_style(ProgressStyle::default_bar().template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta}, {per_sec})",
        )?);

    let (durations_false, durations_true): (Vec<_>, Vec<_>) = (0..MAX)
        .into_par_iter()
        .progress_with(pb)
        .map(|seed| {
            let (grid_false, dur_false) = generate(false, seed);
            let (grid_true, dur_true) = generate(true, seed);
            assert_eq!(grid_false, grid_true);
            (dur_false, dur_true)
        })
        .unzip();

    dbg!(&durations_false);
    dbg!(&durations_true);

    let list_false = durations_false.iter().map(|d| d.as_secs_f64()).join("\n");
    println!("list_false:\n{list_false}");
    let list_true = durations_true.iter().map(|d| d.as_secs_f64()).join("\n");
    println!("list_true:\n{list_true}");

    // println!("{grid}");
    // println!("{}", GivensLine.render(&grid));
    // dbg!(total_time);

    Ok(())
}
