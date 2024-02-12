use std::path::PathBuf;
use std::time::Instant;

use indicatif::ProgressIterator;

use sudoku::base::consts::Base3;
use sudoku::error::Result;
use sudoku::grid::deserialization::read_grids_from_file;
use sudoku::grid::Grid;
use sudoku::solver::strategic;
use sudoku::solver::{backtracking, FallibleSolver, InfallibleSolver};

#[allow(dead_code)]
enum SolverSelection {
    Strategic,
    BacktrackingBitset,
}

fn main() -> Result<()> {
    let solver_selection = SolverSelection::BacktrackingBitset;

    println!("Reading grids");
    let mut grids = read_grids_from_file::<Base3>(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/res/tdoku/puzzles1_unbiased"),
    )?;
    println!("Done");

    let before = Instant::now();

    let mut total_backtrack_count = 0;

    work(&mut grids, solver_selection, &mut total_backtrack_count);

    let after = Instant::now();
    let total_time = after - before;

    dbg!(total_time);
    dbg!(grids.len());

    println!(
        "Grids per second: {:.2}",
        grids.len() as f64 / total_time.as_secs_f64()
    );
    println!(
        "Avg time per grid: {:?}",
        total_time / u32::try_from(grids.len()).unwrap()
    );

    dbg!(total_backtrack_count);

    Ok(())
}

#[inline(never)]
fn work(
    grids: &mut [Grid<Base3>],
    solver_selection: SolverSelection,
    total_backtrack_count: &mut u64,
) {
    for grid in grids.iter_mut().progress() {
        match solver_selection {
            SolverSelection::Strategic => {
                assert!(strategic::Solver::new(grid).try_solve().unwrap().is_some());
            }
            SolverSelection::BacktrackingBitset => {
                let mut solver = backtracking::Solver::new(grid);
                assert!(solver.solve().is_some());
                *total_backtrack_count += solver.backtrack_count;
            }
        }
    }
}
