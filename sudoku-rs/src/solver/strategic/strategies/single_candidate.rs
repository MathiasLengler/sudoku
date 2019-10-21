use crate::cell::{SudokuBase, SudokuCell};
use crate::grid::Grid;
use crate::position::Position;

use super::Strategy;

#[derive(Debug)]
pub struct SingleCandidate;

impl<Base: SudokuBase> Strategy<Base> for SingleCandidate {
    fn execute(&self, grid: &mut Grid<Base>) -> Vec<Position> {
        grid.all_candidates_positions()
            .into_iter()
            .filter_map(|candidate_pos| {
                let candidates = grid.get(candidate_pos).candidates().unwrap();

                if candidates.len() == 1 {
                    let single_candidate = candidates[0];

                    grid.get_mut(candidate_pos).set_value(single_candidate);
                    grid.update_candidates(candidate_pos, single_candidate);

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
    use crate::samples;

    use super::*;

    #[test]
    fn test_single_candidate() {
        let mut grid = samples::base_2().first().unwrap().clone();

        grid.set_all_direct_candidates();
        grid.fix_all_values();

        let mut modified_positions = SingleCandidate.execute(&mut grid);

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

        assert!(grid.is_solved());
    }
}
