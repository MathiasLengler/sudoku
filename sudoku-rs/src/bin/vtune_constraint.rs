use sudoku::cell::Cell;
use sudoku::error::Result;
use sudoku::solver::constraint::Solver;

fn main() -> Result<()> {
    for i in 0..10 {
        dbg!(i);
        let mut sudoku = sudoku::samples::base_3()[0].clone();
        dbg!(Solver::new(&mut sudoku).try_solve());
    }

    Ok(())
}
