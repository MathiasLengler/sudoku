#![allow(unused)]

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, Instant};

use sudoku::base::consts::Base3;
use sudoku::base::SudokuBase;
use sudoku::error::Result;
use sudoku::grid::deserialization::read_grids_from_file;
use sudoku::grid::Grid;
use sudoku::solver::strategic::strategies::{GroupIntersectionBoth, Strategy};
use sudoku::solver::{backtracking, backtracking_bitset, emerentius, strategic};

enum SolverSelection {
    Backtracking,
    Strategic,
    BacktrackingBitset,
    Emerentius,
}

fn main() -> Result<()> {
    let solver_selection = SolverSelection::Emerentius;

    println!("Reading grids");
    let mut grids = read_grids_from_file::<Base3>("./sudoku-rs/tests/res/tdoku/puzzles1_unbiased")?;
    println!("Done");

    let before = Instant::now();

    let mut total_guess_count = 0;

    for (i, mut grid) in grids.into_iter().enumerate() {
        match solver_selection {
            SolverSelection::Backtracking => {
                assert!(backtracking::Solver::new(&mut grid).next().is_some());
            }
            SolverSelection::Strategic => {
                assert!(strategic::Solver::new(&mut grid)
                    .try_solve()
                    .unwrap()
                    .is_some());
            }
            SolverSelection::BacktrackingBitset => {
                let mut solver = backtracking_bitset::Solver::new(&grid);
                assert!(solver.try_solve().is_some());
                total_guess_count += solver.guess_count;
            }
            SolverSelection::Emerentius => {
                let sudoku: emerentius::Sudoku = grid.try_into().unwrap();
                assert!(sudoku.is_uniquely_solvable());
            }
        }

        if i % 10000 == 0 {
            println!("{i}");
        }
    }

    let after = Instant::now();
    let total_time = after - before;

    dbg!(total_time);
    dbg!(total_guess_count);

    Ok(())
}
