use std::convert::TryInto;
use std::time::Instant;

use sudoku::cell::Cell;
use sudoku::error::Result;
use sudoku::solver::backtracking::BacktrackingSolver;
use sudoku::Sudoku;

fn main() -> Result<()> {
    let sudokus = vec![
        // 11 Star difficulty
        vec![
            vec![8, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 0, 3, 6, 0, 0, 0, 0, 0],
            vec![0, 7, 0, 0, 9, 0, 2, 0, 0],
            vec![0, 5, 0, 0, 0, 7, 0, 0, 0],
            vec![0, 0, 0, 0, 4, 5, 7, 0, 0],
            vec![0, 0, 0, 1, 0, 0, 0, 3, 0],
            vec![0, 0, 1, 0, 0, 0, 0, 6, 8],
            vec![0, 0, 8, 5, 0, 0, 0, 1, 0],
            vec![0, 9, 0, 0, 0, 0, 4, 0, 0],
        ],
    ]
    .into_iter()
    .map(TryInto::<Sudoku<Cell>>::try_into)
    .collect::<Result<Vec<_>>>()?;

    for (sudoku_index, sudoku) in sudokus.into_iter().enumerate() {
        eprintln!("sudoku_index = {:?}", sudoku_index);

        let mut solver = BacktrackingSolver::new(sudoku);

        let before = Instant::now();

        let solve_ret = solver.next();

        let after = Instant::now();

        eprintln!("time = {:?}", after - before);

        println!("{}", solve_ret.unwrap());
    }

    Ok(())
}
