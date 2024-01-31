use std::sync::atomic::Ordering;
use std::time::Instant;

use env_logger::Env;

use sudoku::base::consts::*;
use sudoku::error::Result;
use sudoku::generator::{Generator, GeneratorSettings, PruningSettings, PruningTarget};
use sudoku::grid::format::{GivensLine, GridFormat};
use sudoku::solver::backtracking::SPLIT_COUNT;

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or(
        "trace,sudoku::solver::backtracking=debug,sudoku::solver::strategic=debug",
    ))
    .format_indent(Some(0))
    .init();

    let before = Instant::now();
    let grid = Generator::<Base4>::with_settings(GeneratorSettings {
        prune: Some(PruningSettings {
            target: PruningTarget::Minimal,
            set_all_direct_candidates: false,
            ..Default::default()
        }),
        solution: None,
        seed: Some(2),
    })
    .generate()
    .unwrap();

    let after = Instant::now();
    let total_time = after - before;

    println!("{grid}");
    println!("{}", GivensLine.render(&grid));
    dbg!(total_time);

    dbg!(SPLIT_COUNT.load(Ordering::Acquire));

    Ok(())
}
