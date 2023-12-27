use sudoku::base::consts::*;
use sudoku::error::Result;
use sudoku::generator::{Generator, PruningSettings, PruningTarget};

fn main() -> Result<()> {
    let grid = Generator::<Base3>::with_pruning(PruningSettings {
        target: PruningTarget::Minimal,
        set_all_direct_candidates: true,
        ..Default::default()
    })
    .generate()
    .unwrap();

    println!("{grid}");

    Ok(())
}
