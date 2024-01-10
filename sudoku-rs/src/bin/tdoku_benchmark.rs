use std::time::Instant;

use sudoku::base::consts::Base3;
use sudoku::error::Result;
use sudoku::grid::deserialization::read_grids_from_file;
use sudoku::solver::backtracking;
use sudoku::solver::strategic;

#[allow(dead_code)]
enum SolverSelection {
    Strategic,
    BacktrackingBitset,
}

fn main() -> Result<()> {
    let solver_selection = SolverSelection::BacktrackingBitset;

    println!("Reading grids");
    let mut grids = read_grids_from_file::<Base3>("./sudoku-rs/tests/res/tdoku/puzzles1_unbiased")?;
    println!("Done");

    let before = Instant::now();

    let mut total_backtrack_count = 0;

    for (i, grid) in grids.iter_mut().enumerate() {
        match solver_selection {
            SolverSelection::Strategic => {
                assert!(strategic::Solver::new(grid).try_solve().unwrap().is_some());
            }
            SolverSelection::BacktrackingBitset => {
                let mut solver = backtracking::Solver::new(grid);
                assert!(solver.try_solve().is_some());
                total_backtrack_count += solver.backtrack_count;
            }
        }

        if i % 10000 == 0 {
            println!("{i}");
        }
    }

    let after = Instant::now();
    let total_time = after - before;

    dbg!(total_time);
    dbg!(grids.len() as f64 / total_time.as_secs_f64());
    dbg!(total_backtrack_count);

    Ok(())
}
