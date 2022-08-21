use std::fs::File;
use std::io::{BufRead, BufReader};
use sudoku::base::consts::U3;
use sudoku::error::Result;
use sudoku::grid::Grid;
use sudoku::solver::backtracking;
use sudoku::solver::strategic;

fn main() -> Result<()> {
    let file = File::open("./data/puzzles1_unbiased")?;
    let mut reader = BufReader::new(file);

    for (i, line) in reader.lines().enumerate() {
        let line = line?;
        if line.starts_with('#') {
            continue;
        }

        let mut grid: Grid<U3> = line.as_str().try_into()?;
        grid.fix_all_values();
        grid.set_all_direct_candidates();

        // assert!(backtracking::Solver::new(&mut grid).next().is_some());
        assert!(strategic::Solver::new(&mut grid).try_solve()?);

        if i % 1000 == 0 {
            println!("{i}");
        }
    }
    Ok(())
}
