use crate::cell::SudokuCell;
use crate::position::Position;
use crate::Sudoku;

use super::Strategy;

pub(in crate::solver::strategic) struct SingleCandidate;

impl<Cell: SudokuCell> Strategy<Cell> for SingleCandidate {
    fn name(&self) -> &'static str {
        "SingleCandidate"
    }

    fn execute(&self, sudoku: &mut Sudoku<Cell>) -> Vec<Position> {
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

        let mut modified_positions = SingleCandidate.execute(&mut sudoku);

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
}
