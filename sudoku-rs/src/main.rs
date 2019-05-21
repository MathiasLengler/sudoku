use std::convert::TryInto;
use std::time::Instant;

use sudoku::cell::Cell;
use sudoku::error::Result;
use sudoku::generator::backtracking::{
    BacktrackingGenerator, BacktrackingGeneratorSettings, BacktrackingGeneratorTarget,
};
use sudoku::solver::backtracking::BacktrackingSolver;
use sudoku::Sudoku;

fn main() -> Result<()> {
    for i in 0..20 {
        dbg!(i);
        BacktrackingGenerator::new(BacktrackingGeneratorSettings {
            base: 3,
            target: BacktrackingGeneratorTarget::Critical,
        })
        .generate::<Cell>()
        .unwrap();
    }

    Ok(())
}
