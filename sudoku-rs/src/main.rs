use sudoku::cell::Cell;
use sudoku::error::Result;
use sudoku::generator::backtracking::{
    BacktrackingGenerator, BacktrackingGeneratorSettings, BacktrackingGeneratorTarget,
};

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
