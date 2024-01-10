use std::sync::atomic::Ordering;
use std::time::Instant;

use sudoku::error::Result;
use sudoku::grid::Grid;
use sudoku::samples;
use sudoku::solver::backtracking::{Solver, SPLIT_COUNT};

fn main() -> Result<()> {
    let before = Instant::now();
    let grid = samples::base_4().into_iter().next().unwrap();
    let solver = Solver::builder(&grid)
        .availability_filter(Grid::new())
        .build();

    assert!(solver.has_any_solution());

    let after = Instant::now();
    let total_time = after - before;

    dbg!(total_time);

    dbg!(SPLIT_COUNT.load(Ordering::Acquire));

    Ok(())
}
