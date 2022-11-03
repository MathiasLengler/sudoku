use sudoku::base::consts::*;
use sudoku::error::Result;
use sudoku::generator::{Generator, Target};

fn main() -> Result<()> {
    let grid = Generator::with_target(Target::Minimal).generate::<Base3>();

    println!("{grid}");

    Ok(())
}
