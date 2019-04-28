use std::collections::btree_set::BTreeSet;
use std::collections::vec_deque::VecDeque;
use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};
use std::fmt;

use cell::SudokuCell;

use crate::error::{Error, Result};
use crate::grid::Grid;
use crate::position::Position;

pub mod cell;
pub mod position;
pub mod solver;
pub mod generator;
pub mod error;
pub mod transport;
mod grid;

// TODO: solve/verify incomplete sudoku
// TODO: generate valid incomplete sudoku

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sudoku<Cell: SudokuCell> {
    grid: Grid<Cell>,
}

// TODO: Preset value/user filled cells
// TODO: provide undo/redo API
impl<Cell: SudokuCell> Sudoku<Cell> {
    pub fn new(base: usize) -> Self {
        Sudoku {
            grid: Grid::new(base)
        }
    }

    // TODO: return cell values
    pub fn direct_candidates(&self, pos: Position) -> VecDeque<Cell> {
        let conflicting_cells = self.grid.column(pos.column)
            .chain(self.grid.row(pos.row))
            .chain(self.grid.block(pos))
            .collect::<BTreeSet<&Cell>>();

        let value_range: Vec<Cell> = self.grid.value_range().collect();

        let values = value_range.iter().collect::<BTreeSet<&Cell>>();

        values.difference(&conflicting_cells).map(|cell| (**cell).clone()).collect()
    }

    pub fn has_conflict(&self) -> bool {
        self.grid.all_rows().any(|row| self.has_duplicate(row)) ||
            self.grid.all_columns().any(|column| self.has_duplicate(column)) ||
            self.grid.all_blocks().any(|block| self.has_duplicate(block))
    }

    pub fn has_conflict_at(&self, pos: Position) -> bool {
        self.has_duplicate(self.grid.row(pos.row)) ||
            self.has_duplicate(self.grid.column(pos.column)) ||
            self.has_duplicate(self.grid.block(pos))
    }

    // TODO: conflict location pairs
    fn has_duplicate<'a>(&'a self, cells: impl Iterator<Item=&'a Cell>) -> bool {
        let mut cells: Vec<_> = cells.filter(|cell| cell.has_value()).collect();

        cells.sort();

        let cell_count = cells.len();

        cells.dedup();

        let cell_count_dedup = cells.len();

        cell_count != cell_count_dedup
    }

    pub fn all_cell_positions(&self) -> impl Iterator<Item=Position> {
        let side_length = self.side_length();

        (0..side_length)
            .flat_map(move |row_index| (0..side_length).map(move |column_index| Position {
                column: column_index,
                row: row_index,
            }))
    }

    pub fn empty_positions(&self) -> Vec<Position> {
        self.all_cell_positions().filter(|pos| !self.get(*pos).has_value()).collect()
    }

    // TODO: replace
    pub fn get(&self, pos: Position) -> &Cell {
        self.grid.get_pos(pos)
    }

    // TODO: replace
    pub fn set(&mut self, pos: Position, cell: Cell) -> Cell {
        use std::mem;

        let previous_value = mem::replace(self.grid.get_pos_mut(pos), cell);

        previous_value
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

impl<Cell: SudokuCell> TryFrom<Vec<Vec<Cell>>> for Sudoku<Cell> {
    type Error = Error;

    fn try_from(nested_cells: Vec<Vec<Cell>>) -> Result<Self> {
        nested_cells.into_iter().flatten().collect::<Vec<_>>().try_into()
    }
}

impl<Cell: SudokuCell> TryFrom<Vec<Cell>> for Sudoku<Cell> {
    type Error = Error;

    fn try_from(cells: Vec<Cell>) -> Result<Self> {
        Ok(Sudoku {
            grid: cells.try_into()?
        })
    }
}

impl<Cell: SudokuCell> TryFrom<Vec<Vec<usize>>> for Sudoku<Cell> {
    type Error = Error;

    fn try_from(nested_values: Vec<Vec<usize>>) -> Result<Self> {
        nested_values.into_iter()
            .flatten()
            .map(|value| Cell::new_with_value(value))
            .collect::<Vec<_>>().try_into()
    }
}

impl<Cell: SudokuCell> TryFrom<Vec<usize>> for Sudoku<Cell> {
    type Error = Error;

    fn try_from(values: Vec<usize>) -> Result<Self> {
        values.into_iter()
            .map(|value| Cell::new_with_value(value))
            .collect::<Vec<_>>()
            .try_into()
    }
}

impl<Cell: SudokuCell> Display for Sudoku<Cell> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.grid)
    }
}

#[cfg(test)]
mod tests {
    use crate::cell::OptionCell;

    use super::*;

    #[test]
    fn test_has_conflict() {
        let mut sudoku = Sudoku::<OptionCell>::new(3);

        let mut debug_value = 1;
        for y in 0..sudoku.side_length() {
            for x in 0..sudoku.side_length() {
                sudoku.set(Position { column: x, row: y }, OptionCell::new_with_value(debug_value));
                debug_value += 1;
            }
        }

        assert!(!sudoku.has_conflict());

        let previous_cell = sudoku.set(Position { column: 2, row: 2 }, OptionCell::new_with_value(1));

        assert!(sudoku.has_conflict());

        sudoku.set(Position { column: 2, row: 2 }, previous_cell);

        assert!(!sudoku.has_conflict());
    }
}