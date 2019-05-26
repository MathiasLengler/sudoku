//! Technique | Code | Cost for first use | Cost for subsequent uses
//! --- | --- | --- | ---
//! Single Candidate | sct | 100 | 100
//! Single Position | spt | 100 | 100
//! Candidate Lines | clt | 350 | 200
//! Double Pairs | dpt | 500 | 250
//! Multiple Lines | mlt | 700 | 400
//! Naked Pair | dj2 | 750 | 500
//! Hidden Pair | us2 | 1500 | 1200
//! Naked Triple | dj3 | 2000 | 1400
//! Hidden Triple | us3 | 2400 | 1600
//! X-Wing | xwg | 2800 | 1600
//! Forcing Chains | fct | 4200 | 2100
//! Naked Quad | dj4 | 5000 | 4000
//! Hidden Quad | us4 | 7000 | 5000
//! Swordfish | sf4 | 8000 | 6000

// TODO: single candidate in group (Single Position)

use crate::cell::SudokuCell;
use crate::position::Position;
use crate::solver::backtracking::BacktrackingSolver;
use crate::Sudoku;

pub fn strategies<Cell: SudokuCell>() -> [fn(&mut Sudoku<Cell>) -> Vec<Position>; 2] {
    [single_candidate, backtracking]
}

// TODO: bench
pub fn single_candidate<Cell: SudokuCell>(sudoku: &mut Sudoku<Cell>) -> Vec<Position> {
    sudoku
        .grid()
        .all_candidates_positions()
        .into_iter()
        .filter_map(|candidate_pos| {
            let candidates = sudoku.get(candidate_pos).candidates().unwrap();

            if candidates.len() == 1 {
                let single_candidate = candidates[0];

                sudoku.set_value(candidate_pos, single_candidate);

                Some(candidate_pos)
            } else {
                None
            }
        })
        .collect()
}

pub fn backtracking<Cell: SudokuCell>(sudoku: &mut Sudoku<Cell>) -> Vec<Position> {
    let mut solver = BacktrackingSolver::new(sudoku);

    if let Some(_) = solver.next() {
        solver.empty_positions().to_vec()
    } else {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use crate::cell::Cell;
    use crate::samples;

    use super::*;

    #[test]
    fn test_single_candidate() {
        let mut sudoku: Sudoku<Cell<NonZeroUsize>> = samples::base_2().first().unwrap().clone();

        sudoku.set_all_direct_candidates();
        sudoku.fix_all_values();

        let mut modified_positions = single_candidate(&mut sudoku);

        modified_positions.sort();

        assert_eq!(
            modified_positions,
            vec![
                Position { row: 0, column: 0 },
                Position { row: 0, column: 3 },
                Position { row: 1, column: 1 },
                Position { row: 1, column: 2 },
                Position { row: 2, column: 1 },
                Position { row: 2, column: 2 },
                Position { row: 3, column: 0 },
                Position { row: 3, column: 3 },
            ]
        );

        assert!(sudoku.is_solved());
    }

    #[test]
    fn test_backtracking() {
        let mut sudoku: Sudoku<Cell<NonZeroUsize>> = samples::base_3().first().unwrap().clone();

        let candidates_len = sudoku.grid().all_candidates_positions().len();

        sudoku.fix_all_values();

        let modified_positions = backtracking(&mut sudoku);

        assert!(sudoku.is_solved());

        assert_eq!(modified_positions.len(), modified_positions.len());
    }
}
