use std::path::Path;

use sudoku::base::consts::*;
use sudoku::error::Result;
use sudoku::grid::Grid;
use sudoku::solver::sat::Solver;

fn main() -> Result<()> {
    type Base = Base3;

    let grid = Grid::<Base>::new();
    println!("{grid}");
    let solver = Solver::new(&grid)?;

    solver.dump_cnf(Path::new("./sudoku-rs/out/cnf/base3_empty.cnf"));

    Ok(())
}
