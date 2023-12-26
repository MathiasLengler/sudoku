//! 21: 33                                                 
//! 22: 1215
//! 23: 18947
//! 24: 110189
//! 25: 273566
//! 26: 322286
//! 27: 195990
//! 28: 64256
//! 29: 12116
//! 30: 1301
//! 31: 97
//! 33: 4
//! Best grid #21:
//! ╔═════════════════╦═════════════════╦═════════════════╗
//! ║  2  │   3 │     ║     │     │     ║     │  2  │   3 ║
//! ║ 4   │ 45  │  1  ║ 456 │     │  56 ║  7  │ 4 6 │   6 ║
//! ║  89 │  89 │     ║   9 │   9 │   9 ║     │  89 │  8  ║
//! ║─────┼─────┼─────║─────┼─────┼─────║─────┼─────┼─────║
//! ║  2  │   3 │  23 ║ 1   │ 1   │ 1   ║ 123 │ 12  │ 1 3 ║
//! ║ 4   │ 45  │ 45  ║ 456 │     │  56 ║ 4 6 │ 4 6 │   6 ║
//! ║ 789 │ 789 │     ║ 7 9 │ 7 9 │ 7 9 ║  89 │  89 │  8  ║
//! ║─────┼─────┼─────║─────┼─────┼─────║─────┼─────┼─────║
//! ║     │     │     ║     │     │     ║ 1   │     │ 1   ║
//! ║  6  │ 4   │ 4   ║  3  │  8  │  2  ║ 4   │  5  │     ║
//! ║     │ 7 9 │     ║     │     │     ║   9 │     │     ║
//! ╠═════════════════╬═════════════════╬═════════════════╣
//! ║     │ 1   │     ║ 12  │     │ 1   ║ 12  │ 12  │     ║
//! ║  3  │  56 │  7  ║  5  │  4  │  5  ║  56 │   6 │  9  ║
//! ║     │  8  │     ║  8  │     │  8  ║  8  │  8  │     ║
//! ║─────┼─────┼─────║─────┼─────┼─────║─────┼─────┼─────║
//! ║ 1   │     │     ║ 1   │     │ 1 3 ║ 1   │ 1   │     ║
//! ║     │  2  │  5  ║  5  │  6  │  5  ║  5  │     │  4  ║
//! ║  89 │     │     ║ 789 │     │ 789 ║  8  │ 78  │     ║
//! ║─────┼─────┼─────║─────┼─────┼─────║─────┼─────┼─────║
//! ║ 1   │ 1   │     ║ 12  │ 12  │ 1   ║ 12  │     │ 1   ║
//! ║ 4   │ 456 │ 456 ║  5  │     │  5  ║  56 │  3  │  56 ║
//! ║  89 │  89 │     ║ 789 │ 7 9 │ 789 ║  8  │     │ 78  ║
//! ╠═════════════════╬═════════════════╬═════════════════╣
//! ║     │ 1 3 │     ║ 12  │ 123 │     ║ 1 3 │ 1   │ 1 3 ║
//! ║  5  │   6 │  9  ║   6 │     │  4  ║   6 │   6 │   6 ║
//! ║     │ 7   │     ║ 78  │ 7   │     ║  8  │ 78  │ 78  ║
//! ║─────┼─────┼─────║─────┼─────┼─────║─────┼─────┼─────║
//! ║ 1   │ 1 3 │   3 ║ 1   │ 1 3 │ 1 3 ║ 1 3 │ 1   │     ║
//! ║ 4   │ 4 6 │ 4 6 ║   6 │     │   6 ║ 456 │ 4 6 │  2  ║
//! ║ 7   │ 7   │     ║ 789 │ 7 9 │ 789 ║  89 │ 789 │     ║
//! ║─────┼─────┼─────║─────┼─────┼─────║─────┼─────┼─────║
//! ║ 12  │ 1 3 │     ║ 12  │     │ 1 3 ║ 1 3 │ 1   │ 1 3 ║
//! ║ 4   │ 4 6 │  8  ║   6 │  5  │   6 ║ 4 6 │ 4 6 │   6 ║
//! ║ 7   │ 7   │     ║ 7 9 │     │ 7 9 ║   9 │ 7 9 │ 7   ║
//! ╚═════════════════╩═════════════════╩═════════════════╝

#![allow(unused_imports)]

use std::sync::Mutex;

use hdrhistogram::Histogram;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
use rayon::prelude::*;

use sudoku::base::consts::*;
use sudoku::error::Result;
use sudoku::generator::{Generator, GeneratorSettings, GeneratorTarget};
use sudoku::solver::strategic::strategies::{HiddenSingles, NakedSingles};

type Base = Base2;

fn main() -> Result<()> {
    let mut hist = Histogram::<u64>::new_with_bounds(1, 100, 1)
        .unwrap()
        .into_sync();

    let best_grid = Mutex::new(None);

    const MAX: u64 = 100_000;
    let pb = ProgressBar::new(MAX).with_style(ProgressStyle::default_bar().template(
        "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] ({pos}/{len}, ETA {eta}, {per_sec})",
    )?);

    let generator = Generator::<Base>::with_settings(GeneratorSettings {
        target: GeneratorTarget::Minimal {
            set_all_direct_candidates: false,
        },
        // strategies: DynamicStrategy::default_solver_strategies(),
        strategies: vec![
            //
            NakedSingles.into(),
            HiddenSingles.into(),
            // Backtracking.into(),
        ],
        ..Default::default()
    });
    (0..MAX)
        .into_par_iter()
        .progress_with(pb)
        .for_each_with(hist.recorder(), |recorder, _i| {
            let grid = generator.generate().unwrap();

            let num_values = grid.all_value_positions().len();

            recorder.record(num_values.try_into().unwrap()).unwrap();

            let mut res = best_grid.lock().unwrap();

            match *res {
                Some((prev_num_values, _)) if prev_num_values > num_values => {
                    *res = Some((num_values, grid))
                }
                None => *res = Some((num_values, grid)),
                _ => {}
            }
        });

    hist.refresh();

    for v in hist.iter_recorded() {
        println!("{}: {}", v.value_iterated_to(), v.count_at_value());
    }

    if let Some((num_values, grid)) = best_grid.into_inner().unwrap() {
        println!("Best grid #{num_values}:\n{grid}");
    }

    Ok(())
}
