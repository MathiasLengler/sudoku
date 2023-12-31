use std::fmt::Display;

use sudoku::base::consts::Base2;
use sudoku::base::consts::*;
use sudoku::error::Result;
use sudoku::world::CellWorld;

fn main() -> Result<()> {
    let (tile_row_count, tile_col_count) = (3, 3);
    let seed = 1;
    let overlap = 1;

    let mut world = CellWorld::<Base2>::new((tile_row_count, tile_col_count), overlap);
    let world_generation_result = world.generate(Some(seed));
    dbg!(&world_generation_result);

    println!("solved world:\n{world}");
    assert!(world.is_solved());

    world.prune(Some(seed));
    println!("pruned world:\n{world}");

    for grid in world.all_grids() {
        println!("{grid}\n",);
    }

    Ok(())
}
