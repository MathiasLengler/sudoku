use sudoku::base::consts::*;
use sudoku::error::Result;
use sudoku::world::dynamic::DynamicCellWorldActions;
use sudoku::world::{CellWorld, WorldGridDim};

fn main() -> Result<()> {
    let grid_dim = WorldGridDim::new(3, 3)?;
    let seed = 1;
    let overlap = 1.try_into().unwrap();

    let mut world = CellWorld::<Base3>::new(grid_dim, overlap);
    let world_generation_result = world.generate_solved(Some(seed));
    dbg!(&world_generation_result);

    println!("solved world:\n{world}");
    assert!(world.is_solved());

    world.prune(Some(seed))?;
    println!("pruned world:\n{world}");

    for grid_position in world.all_grid_positions() {
        let grid = world.to_grid_at(grid_position)?;
        println!("{grid_position:?}:\n{grid}\n",);
    }

    assert!(world.is_directly_consistent());

    Ok(())
}
