use sudoku::base::consts::*;
use sudoku::error::Result;
use sudoku::generator::{Generator, GeneratorTarget};

fn main() -> Result<()> {
    let grid = Generator::<Base3>::with_target(GeneratorTarget::FromFilled {
        distance_from_filled: 85,
        set_all_direct_candidates: true,
    })
    .generate()
    .unwrap();

    println!("{grid}");

    Ok(())
}
