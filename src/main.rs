use sudoku::cell::OptionCell;
use sudoku::error::Result;
use sudoku::solver::backtracking::BacktrackingSolver;
use sudoku::Sudoku;
use std::convert::TryInto;

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
        ]
    ]
        .into_iter()
        .map(TryInto::<Sudoku<OptionCell>>::try_into)
        .collect::<Result<Vec<_>>>()?;

    for (sudoku_index, sudoku) in sudokus.into_iter().enumerate() {
        eprintln!("sudoku_index = {:?}", sudoku_index);

        let mut solver = BacktrackingSolver::new_with_limit(sudoku, 0, true);

        let solve_ret = solver.solve();

        assert!(solve_ret);

        println!("{}", solver.sudoku());

        assert!(solver.sudoku().all_empty_positions().is_empty())
    }

    Ok(())
}
