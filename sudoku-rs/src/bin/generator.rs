use sudoku::base::consts::*;
use sudoku::error::Result;
use sudoku::generator::backtracking::{Generator, Target};

fn main() -> Result<()> {
    for i in 0..20 {
        dbg!(i);
        Generator::with_target(Target::Minimal).generate::<Base3>();
    }

    Ok(())
}
