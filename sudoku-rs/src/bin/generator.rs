use env_logger::Env;
use std::sync::atomic::Ordering;
use std::time::Instant;
use sudoku::base::consts::*;
use sudoku::error::Result;
use sudoku::generator::{Generator, GeneratorSettings, PruningSettings, PruningTarget};
use sudoku::solver::backtracking::SPLIT_COUNT;

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
        .format_indent(Some(0))
        .init();

    let before = Instant::now();
    let grid = Generator::<Base4>::with_settings(GeneratorSettings {
        prune: Some(PruningSettings {
            target: PruningTarget::Minimal,
            set_all_direct_candidates: true,
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
    dbg!(total_time);

    dbg!(SPLIT_COUNT.load(Ordering::Acquire));

    Ok(())
}
