use std::convert::TryInto;

use crate::base::consts::*;
use crate::base::SudokuBase;
use crate::cell::Candidates;
use crate::cell::Cell;
use crate::error::Result;
use crate::generator::{Generator, PruningSettings, PruningTarget};
use crate::grid::Grid;

// TODO: rethink API (unwrap, clone for consumer of specific sudoku)
pub fn base_2() -> Vec<Grid<Base2>> {
    let mut grids = vec![
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
    .map(TryInto::<Grid<Base2>>::try_into)
    .collect::<Result<Vec<_>>>()
    .unwrap();

    for grid in &mut grids {
        grid.fix_all_values();
    }

    grids
}

pub fn base_2_solved() -> Grid<Base2> {
    Grid::<Base2>::try_from(vec![
        vec![2, 3, 4, 1],
        vec![4, 1, 3, 2],
        vec![1, 4, 2, 3],
        vec![3, 2, 1, 4],
    ])
    .unwrap()
}

pub fn base_2_candidates_coordinates() -> Grid<Base2> {
    Grid::<Base2>::with_cells(
        (0..u8::try_from(<Base2 as SudokuBase>::CELL_COUNT).unwrap())
            .map(|i| Cell::with_candidates(Candidates::with_integral(i)))
            .collect(),
    )
    .unwrap()
}

pub fn base_3() -> Vec<Grid<Base3>> {
    let mut grids = vec![
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
    .map(TryInto::<Grid<Base3>>::try_into)
    .collect::<Result<Vec<_>>>()
    .unwrap();

    for grid in &mut grids {
        grid.fix_all_values();
    }

    grids
}

pub fn minimal<Base: SudokuBase>() -> Grid<Base> {
    Generator::with_pruning(PruningSettings {
        target: PruningTarget::Minimal,
        set_all_direct_candidates: true,
        ..Default::default()
    })
    .generate()
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_2() {
        base_2();
    }

    #[test]
    fn test_base_2_solved() {
        assert!(base_2_solved().is_solved());
    }

    #[test]
    fn test_base_2_candidates_coordinates() {
        let grid = base_2_candidates_coordinates();

        let top_left_cell = grid.get((0, 0).try_into().unwrap());
        assert_eq!(*top_left_cell, Cell::with_candidates(Candidates::new()));

        let bottom_right = grid.get((3, 3).try_into().unwrap());
        assert_eq!(*bottom_right, Cell::with_candidates(Candidates::all()));
    }

    #[test]
    fn test_base_3() {
        base_3();
    }
}
