use itertools::iproduct;
use rayon::prelude::*;

use sudoku::base::consts::*;
use sudoku::base::SudokuBase;
use sudoku::error::Result;
use sudoku::world::dynamic::DynamicCellWorldActions;
use sudoku::world::{CellWorld, TileDim};

fn main() -> Result<()> {
    fn gen_worlds_stats<Base: SudokuBase>() {
        for (overlap, tile_dim) in iproduct!(
            1..=Base::BASE,
            vec![
                (2, 2),
                (3, 3),
                (4, 4),
                (5, 5),
                (10, 10),
                (50, 50),
                (100, 100)
            ]
            .into_iter()
            .map(|(row_count, column_count)| TileDim {
                row_count: row_count.try_into().unwrap(),
                column_count: column_count.try_into().unwrap()
            })
        ) {
            let tile_count = u64::try_from(tile_dim.tile_count()).unwrap();
            let target_tile_count = 1_000_000;

            let total_seeds = target_tile_count / tile_count;
            let total_seeds_f64 = total_seeds as f64;

            let world_generation_results: Vec<_> = (0..total_seeds)
                .into_par_iter()
                .map(|seed| {
                    let mut world = CellWorld::<Base>::new(tile_dim, overlap);
                    world.generate_solved(Some(seed))
                })
                .collect();

            let total_success_count: u32 =
                world_generation_results.iter().flatten().map(|_| 1).sum();

            let backtrack_counts = world_generation_results
                .iter()
                .flatten()
                .map(|res| res.backtrack_count);
            let total_backtrack_count: u32 = backtrack_counts.clone().sum();
            let min_backtrack_count: u32 = backtrack_counts.clone().min().unwrap();
            let max_backtrack_count: u32 = backtrack_counts.max().unwrap();

            println!(
                "base {}, overlap {overlap}, tile_dim {tile_dim:?}:",
                Base::BASE
            );
            println!(
                "total_success_count {total_success_count} {:.2}%",
                (f64::from(total_success_count) / total_seeds_f64) * 100.
            );
            println!(
                "total_backtrack_count {total_backtrack_count} avg {:.2} min {min_backtrack_count} max {max_backtrack_count}",
                f64::from(total_backtrack_count) / total_seeds_f64
            );

            println!()
        }
    }

    gen_worlds_stats::<Base2>();
    gen_worlds_stats::<Base3>();

    Ok(())
}
