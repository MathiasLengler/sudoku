use std::convert::TryInto;

use typenum::consts::*;

use crate::base::SudokuBase;
use crate::cell::compact::candidates::Candidates;
use crate::cell::Cell;
use crate::error::Result;
use crate::generator::backtracking::{Generator, Target};
use crate::grid::Grid;

// TODO: rethink API (unwrap, clone for consumer of specific sudoku)
pub fn base_2() -> Vec<Grid<U2>> {
    vec![
        vec![
            vec![0, 3, 4, 0],
            vec![4, 0, 0, 2],
            vec![1, 0, 0, 3],
            vec![0, 2, 1, 0],
        ],
        vec![
            vec![1, 0, 4, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 1, 0, 2],
        ],
        vec![
            vec![0, 0, 1, 0],
            vec![4, 0, 0, 0],
            vec![0, 0, 0, 2],
            vec![0, 3, 0, 0],
        ],
    ]
    .into_iter()
    .map(TryInto::<Grid<U2>>::try_into)
    .collect::<Result<Vec<_>>>()
    .unwrap()
}

pub fn base_2_candidates_coordinates() -> Grid<U2> {
    Grid::<U2>::with_cells(
        (0..u8::try_from(<U2 as SudokuBase>::CELL_COUNT).unwrap())
            .map(|i| Cell::with_candidates(Candidates::with_arr([i])))
            .collect(),
    )
}

pub fn base_3() -> Vec<Grid<U3>> {
    vec![
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
    .map(TryInto::<Grid<U3>>::try_into)
    .collect::<Result<Vec<_>>>()
    .unwrap()
}

pub fn minimal<Base: SudokuBase>() -> Grid<Base> {
    Generator::with_target(Target::Minimal).generate()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::Position;

    #[test]
    fn test_base_2() {
        base_2();
    }
    #[test]
    fn test_base_2_candidates_coordinates() {
        let grid = base_2_candidates_coordinates();

        let top_left_cell = grid.get(Position { row: 0, column: 0 });
        assert_eq!(*top_left_cell, Cell::with_candidates(Candidates::new()));

        let bottom_right = grid.get(Position { row: 3, column: 3 });
        assert_eq!(*bottom_right, Cell::with_candidates(Candidates::all()));
    }

    #[test]
    fn test_base_3() {
        base_3();
    }
}
