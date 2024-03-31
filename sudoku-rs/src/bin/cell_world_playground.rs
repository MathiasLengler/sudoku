use sudoku::base::consts::*;
use sudoku::error::Result;
use sudoku::world::dynamic::DynamicCellWorldActions;
use sudoku::world::{CellWorld, WorldDim};

fn main() -> Result<()> {
    let grid_dim = WorldDim {
        row_count: 3.try_into().unwrap(),
        column_count: 3.try_into().unwrap(),
    };
    let seed = 1;
    let overlap = 1;

    let mut world = CellWorld::<Base3>::new(grid_dim, overlap);
    let world_generation_result = world.generate_solved(Some(seed));
    dbg!(&world_generation_result);

    println!("solved world:\n{world}");
    assert!(world.is_solved());

    world.prune(Some(seed))?;
    println!("pruned world:\n{world}");

    for grid_index in world.all_grid_indexes() {
        let grid = world.to_grid_at(grid_index)?;
        println!("{grid_index:?}:\n{grid}\n",);
    }

    assert!(world.is_directly_consistent());

    Ok(())
}
