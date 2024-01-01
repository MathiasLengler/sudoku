use sudoku::base::consts::*;
use sudoku::error::Result;
use sudoku::world::{CellWorld, TileDim};

fn main() -> Result<()> {
    let tile_dim = TileDim {
        row_count: 3,
        column_count: 3,
    };
    let seed = 1;
    let overlap = 1;

    let mut world = CellWorld::<Base3>::new(tile_dim, overlap);
    let world_generation_result = world.generate(Some(seed));
    dbg!(&world_generation_result);

    println!("solved world:\n{world}");
    assert!(world.is_solved());

    world.prune(Some(seed));
    println!("pruned world:\n{world}");

    for tile_index in world.all_tile_indexes() {
        let grid = world.to_grid_at(tile_index);
        println!("{tile_index:?}:\n{grid}\n",);
    }

    assert!(world.is_directly_consistent());

    Ok(())
}
