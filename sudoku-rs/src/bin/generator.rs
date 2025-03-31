use std::time::Instant;

use env_logger::Env;

use sudoku::base::consts::*;
use sudoku::error::Result;
use sudoku::generator::{
    Generator, GeneratorSettings, PruningOrder, PruningSettings, PruningTarget,
};
use sudoku::grid::format::{GridFormat, ValuesLine};
use sudoku::solver::strategic::strategies::*;

type Base = Base4;

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info,varisat=warn"))
        .format_indent(Some(0))
        .init();

    let before = Instant::now();

    let grid = Generator::<Base>::with_settings(GeneratorSettings {
        prune: Some(PruningSettings {
            set_all_direct_candidates: true,
            strategies: vec![
                NakedSingles.into(),
                HiddenSingles.into(),
                NakedPairs.into(),
                LockedSets.into(),
                GroupIntersectionBoth.into(),
            ],
            target: PruningTarget::MinClueCount(0),
            order: PruningOrder::Random,
            start_from_near_minimal_grid: false,
        }),
        solution: None,
        seed: None,
    })
    .generate()
    .unwrap();

    let after = Instant::now();
    let total_time = after - before;

    println!("{grid}");
    println!("{}", ValuesLine.render(&grid));
    dbg!(total_time);

    Ok(())
}
