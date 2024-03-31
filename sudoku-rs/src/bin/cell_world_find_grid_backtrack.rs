use indicatif::ParallelProgressIterator;
use rayon::prelude::*;

use sudoku::base::consts::*;
use sudoku::error::Result;
use sudoku::world::dynamic::DynamicCellWorldActions;
use sudoku::world::{CellWorld, WorldDim};

fn main() -> Result<()> {
    let grid_dim = WorldDim {
        row_count: 100.try_into().unwrap(),
        column_count: 100.try_into().unwrap(),
    };

    let overlap = 3;

    let world = (0..1_000_000u32)
        .into_par_iter()
        .progress()
        .filter_map(|seed| {
            let mut world = CellWorld::<Base3>::new(grid_dim, overlap);
            let world_generation_result = world.generate_solved(Some(seed.into())).unwrap();
            if world_generation_result.backtrack_count > 0 {
                dbg!(&world_generation_result);
                Some(world)
            } else {
                None
            }
        })
        .find_any(|_| true)
        .unwrap();

    println!("solved world:\n{world}");
    assert!(world.is_solved());

    // world.prune(Some(seed));
    // println!("pruned world:\n{world}");

    Ok(())
}
