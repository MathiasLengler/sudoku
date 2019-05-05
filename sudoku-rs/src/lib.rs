use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::{Display, Formatter};

use fixedbitset::FixedBitSet;

use cell::SudokuCell;

use crate::error::{Error, Result};
use crate::grid::Grid;
use crate::position::Position;

pub mod cell;
pub mod error;
pub mod generator;
mod grid;
pub mod position;
pub mod solver;
pub mod transport;

// TODO: solve/verify incomplete sudoku
// TODO: generate valid incomplete sudoku

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sudoku<Cell: SudokuCell> {
    grid: Grid<Cell>,
}

// TODO: make sudoku fully generic over cell
//  struct T {
//      t: Sudoku<Cell<u8>>,
//  }

// TODO: clear candidates on set_value
// TODO: provide undo/redo API
// TODO: return result in all asserts
impl<Cell: SudokuCell> Sudoku<Cell> {
    pub fn new(base: usize) -> Self {
        Sudoku {
            grid: Grid::new(base),
        }
    }

    pub fn set_value(&mut self, pos: Position, value: usize) {
        let max_value = self.grid.max_value();

        self.grid.get_pos_mut(pos).set_value(value, max_value);
    }

    pub fn set_candidates(&mut self, pos: Position, candidates: Vec<usize>) {
        let max_value = self.grid.max_value();

        self.grid
            .get_pos_mut(pos)
            .set_candidates(candidates, max_value);
    }

    pub fn toggle_candidate(&mut self, pos: Position, candidate: usize) {
        let max_value = self.grid.max_value();

        self.grid
            .get_pos_mut(pos)
            .toggle_candidate(candidate, max_value);
    }

    pub fn set_all_direct_candidates(&mut self) {
        self.all_cell_positions().for_each(|pos| {
            let candidates = self.direct_candidates(pos);

            self.set_candidates(pos, candidates);
        });
    }

    pub fn get(&self, pos: Position) -> &Cell {
        self.grid.get_pos(pos)
    }

    pub fn side_length(&self) -> usize {
        self.grid.side_length()
    }

    pub fn cell_count(&self) -> usize {
        self.grid.cell_count()
    }

    pub fn base(&self) -> usize {
        self.grid.base()
    }
}

/// Utility iterators
impl<Cell: SudokuCell> Sudoku<Cell> {
    pub(crate) fn all_cell_positions(&self) -> impl Iterator<Item = Position> {
        let side_length = self.side_length();

        (0..side_length).flat_map(move |row_index| {
            (0..side_length).map(move |column_index| Position {
                column: column_index,
                row: row_index,
            })
        })
    }

    pub(crate) fn empty_positions(&self) -> Vec<Position> {
        self.all_cell_positions()
            .filter(|pos| !self.get(*pos).value().is_some())
            .collect()
    }
}

impl<Cell: SudokuCell> Sudoku<Cell> {
    // TODO: return cell values
    pub(crate) fn direct_candidates(&self, pos: Position) -> Vec<usize> {
        let conflicting_values = self
            .grid
            .column(pos.column)
            .chain(self.grid.row(pos.row))
            .chain(self.grid.block(pos))
            .filter_map(|cell| cell.value_as_usize())
            .collect::<FixedBitSet>();

        let values: FixedBitSet = self.grid.value_range().collect();

        values.difference(&conflicting_values).collect()
    }

    pub(crate) fn has_conflict(&self) -> bool {
        self.grid.all_rows().any(|row| self.has_duplicate(row))
            || self
                .grid
                .all_columns()
                .any(|column| self.has_duplicate(column))
            || self
                .grid
                .all_blocks()
                .any(|block| self.has_duplicate(block))
    }

    pub(crate) fn has_conflict_at(&self, pos: Position) -> bool {
        self.has_duplicate(self.grid.row(pos.row))
            || self.has_duplicate(self.grid.column(pos.column))
            || self.has_duplicate(self.grid.block(pos))
    }

    // TODO: conflict location pairs
    fn has_duplicate<'a>(&'a self, cells: impl Iterator<Item = &'a Cell>) -> bool {
        let mut cells: Vec<_> = cells.filter_map(|cell| cell.value_as_usize()).collect();

        cells.sort();

        let cell_count = cells.len();

        cells.dedup();

        let cell_count_dedup = cells.len();

        cell_count != cell_count_dedup
    }
}

impl<Cell: SudokuCell> TryFrom<Vec<Vec<usize>>> for Sudoku<Cell> {
    type Error = Error;

    fn try_from(nested_values: Vec<Vec<usize>>) -> Result<Self> {
        nested_values
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .try_into()
    }
}

impl<Cell: SudokuCell> TryFrom<Vec<usize>> for Sudoku<Cell> {
    type Error = Error;

    fn try_from(values: Vec<usize>) -> Result<Self> {
        Ok(Sudoku {
            grid: values.try_into()?,
        })
    }
}

impl<Cell: SudokuCell> Display for Sudoku<Cell> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.grid)
    }
}

#[cfg(test)]
mod tests {
    use crate::cell::Cell;

    use super::*;

    #[test]
    fn test_has_conflict() {
        let mut sudoku = Sudoku::<Cell>::new(3);

        assert!(!sudoku.has_conflict());

        sudoku.set_value(Position { column: 0, row: 0 }, 1);

        assert!(!sudoku.has_conflict());

        sudoku.set_value(Position { column: 1, row: 0 }, 1);

        assert!(sudoku.has_conflict());

        sudoku.set_value(Position { column: 1, row: 0 }, 0);

        assert!(!sudoku.has_conflict());

        sudoku.set_value(Position { column: 0, row: 1 }, 1);

        assert!(sudoku.has_conflict());

        sudoku.set_value(Position { column: 0, row: 1 }, 0);

        assert!(!sudoku.has_conflict());

        sudoku.set_value(Position { column: 1, row: 1 }, 1);

        assert!(sudoku.has_conflict());

        sudoku.set_value(Position { column: 1, row: 1 }, 0);

        assert!(!sudoku.has_conflict());
    }
}
