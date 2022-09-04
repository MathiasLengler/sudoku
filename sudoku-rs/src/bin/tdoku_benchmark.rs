#![allow(unused)]

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use sudoku::base::consts::U3;
use sudoku::base::SudokuBase;
use sudoku::error::Result;
use sudoku::grid::deserialization::read_grids_from_file;
use sudoku::grid::Grid;
use sudoku::solver::strategic;
use sudoku::solver::{backtracking, backtracking_bitset};

enum SolverSelection {
    Backtracking,
    Strategic,
    BacktrackingBitset,
}

fn main() -> Result<()> {
    let solver_selection = SolverSelection::BacktrackingBitset;

    let grids = read_grids_from_file::<U3>("./sudoku-rs/tests/res/tdoku/puzzles0_kaggle")?;

    let before = Instant::now();

    for (i, mut grid) in grids.into_iter().enumerate() {
        match solver_selection {
            SolverSelection::Backtracking => {
                assert!(backtracking::Solver::new(&mut grid).next().is_some());
            }
            SolverSelection::Strategic => {
                grid.set_all_direct_candidates();

                assert!(strategic::Solver::new(&mut grid).try_solve()?);
            }
            SolverSelection::BacktrackingBitset => {
                assert!(backtracking_bitset::Solver::new(&grid)
                    .try_solve()
                    .is_some());
            }
        }

        if i % 1000 == 0 {
            println!("{i}");
        }
    }

    let after = Instant::now();
    let total_time = after - before;

    dbg!(total_time);
    Ok(())
}
