use indicatif::ParallelProgressIterator;
use rayon::prelude::*;

use sudoku::base::consts::*;
use sudoku::error::Result;
use sudoku::world::CellWorld;

fn main() -> Result<()> {
    let (tile_row_count, tile_col_count) = (100, 100);

    let overlap = 3;

    let world = (0..1_000_000u32)
        .into_par_iter()
        .progress()
        .filter_map(|seed| {
            let mut world = CellWorld::<Base3>::new((tile_row_count, tile_col_count), overlap);
            let world_generation_result = world.generate(Some(seed.into()));
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
