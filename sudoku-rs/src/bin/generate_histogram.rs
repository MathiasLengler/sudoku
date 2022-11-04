/// #20:3
/// #21:20
/// #22:344
/// #23:1690
/// #24:3382
/// #25:3074
/// #26:1212
/// #27:253
/// #28:21
/// #30:1
/// #20
///   .  .  .|  .  3  .|  .  4  .
///   .  5  .|  .  .  .|  7  .  .
///   .  .  7|  .  4  .|  8  .  6
/// ------------------------------
///   .  .  .|  .  .  .|  .  7  .
///   .  .  .|  .  .  .|  .  3  5
///   1  .  .|  .  2  8|  .  .  .
/// ------------------------------
///   4  .  .|  .  .  1|  .  .  .
///   9  .  3|  .  .  .|  .  .  .
///   .  .  .|  8  .  6|  .  .  .
use indexmap::IndexMap;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::sync::Mutex;
use sudoku::base::consts::U3;
use sudoku::error::Result;
use sudoku::generator::{Generator, GeneratorTarget};

type Base = U3;

fn main() -> Result<()> {
    let num_values_to_grid = Mutex::new(IndexMap::new());
    let best_grid = Mutex::new(None);

    const MAX: u64 = 100;
    let pb = ProgressBar::new(MAX).with_style(ProgressStyle::default_bar().template(
        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta})",
    ));
    (0..MAX).into_par_iter().progress_with(pb).for_each(|_i| {
        let grid = Generator::with_target(GeneratorTarget::Minimal).generate::<Base>();

        let num_values = grid.all_value_positions().len();

        *num_values_to_grid
            .lock()
            .unwrap()
            .entry(num_values)
            .or_insert(0) += 1;

        let mut res = best_grid.lock().unwrap();

        match *res {
            Some((prev_num_values, _)) if prev_num_values > num_values => {
                *res = Some((num_values, grid))
            }
            None => *res = Some((num_values, grid)),
            _ => {}
        }
    });

    let mut num_values_to_grid = num_values_to_grid.into_inner().unwrap();

    num_values_to_grid.sort_keys();

    for (num_values, grid) in num_values_to_grid.into_iter() {
        println!("#{}:{}", num_values, grid);
    }

    if let Some((num_values, grid)) = best_grid.into_inner().unwrap() {
        println!("#{}\n{}", num_values, grid);
    }

    Ok(())
}
