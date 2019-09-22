use std::convert::TryInto;
use std::time::Instant;

use sudoku::cell::Cell;
use sudoku::error::Result;
use sudoku::samples::base_3;
use sudoku::solver::backtracking::Solver;
use sudoku::Sudoku;

// TODO: fix animation

fn main() -> Result<()> {
    let sudokus = base_3();

    for (sudoku_index, mut sudoku) in sudokus.into_iter().enumerate() {
        eprintln!("sudoku_index = {:?}", sudoku_index);

        let mut solver = Solver::new(&mut sudoku);

        let before = Instant::now();

        let solve_ret = solver.next();

        let after = Instant::now();

        eprintln!("time = {:?}", after - before);

        println!("{}", solve_ret.unwrap());
    }

    Ok(())
}
