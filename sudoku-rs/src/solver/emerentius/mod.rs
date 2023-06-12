use sudoku_emerentius::Sudoku as SudokuEmerentius;

use crate::base::consts::Base3;
use crate::grid::Grid;

#[derive(Debug)]
pub struct Sudoku(SudokuEmerentius);

impl Sudoku {
    pub fn is_uniquely_solvable(&self) -> bool {
        self.0.is_uniquely_solvable()
    }
}

impl From<Grid<Base3>> for Sudoku {
    fn from(grid: Grid<Base3>) -> Self {
        let cells_vec = grid
            .all_cells()
            .map(|cell| {
                if let Some(value) = cell.value() {
                    value.get()
                } else {
                    0
                }
            })
            .collect::<Vec<_>>();
        let cells_arr: [u8; 81] = cells_vec.try_into().unwrap();

        Self(cells_arr.try_into().unwrap())
    }
}

impl From<Sudoku> for Grid<Base3> {
    fn from(Sudoku(sudoku_emerentius): Sudoku) -> Self {
        let cells_vec = sudoku_emerentius.to_bytes().to_vec();

        let mut grid: Self = cells_vec.try_into().unwrap();
        grid.fix_all_values();
        grid
    }
}

#[cfg(test)]
mod tests {
    use crate::samples;

    use super::*;

    #[test]
    fn test_roundtrip() {
        let mut grid = samples::base_3().pop().unwrap();
        let sudoku: Sudoku = grid.clone().try_into().unwrap();
        let grid_roundtrip: Grid<Base3> = sudoku.try_into().unwrap();

        assert_eq!(grid, grid_roundtrip);
    }

    #[test]
    fn test_is_uniquely_solvable() {
        let sudoku: Sudoku = Grid::<Base3>::new().try_into().unwrap();
        assert!(!sudoku.is_uniquely_solvable());

        let sudoku: Sudoku = samples::base_3().pop().unwrap().try_into().unwrap();
        assert!(sudoku.is_uniquely_solvable());
    }
}
