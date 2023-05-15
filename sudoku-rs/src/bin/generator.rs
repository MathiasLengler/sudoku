use sudoku::base::consts::*;
use sudoku::error::Result;
use sudoku::generator::{Generator, GeneratorTarget};

fn main() -> Result<()> {
    let grid = Generator::with_target(GeneratorTarget::FromFilled {
        distance_from_filled: 85,
        set_all_direct_candidates: true,
    })
    .generate::<Base3>();

    println!("{grid}");

    Ok(())
}
