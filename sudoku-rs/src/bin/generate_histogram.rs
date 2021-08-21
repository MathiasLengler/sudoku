use sudoku::base::consts::*;
use sudoku::error::Result;
use sudoku::generator::backtracking::{Generator, Target};

fn main() -> Result<()> {
    let _ = Generator::with_target(Target::Minimal).generate::<U3>();

    Ok(())
}
