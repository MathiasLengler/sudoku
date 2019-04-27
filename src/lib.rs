use std::collections::btree_set::BTreeSet;
use std::collections::vec_deque::VecDeque;
use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};
use std::fmt;

use failure::ensure;
use itertools::Itertools;

use cell::SudokuCell;

use crate::error::{Error, Result};
use crate::position::Position;

pub mod cell;
pub mod position;
pub mod solver;
pub mod generator;
pub mod error;
pub mod transport;

// TODO: solve/verify incomplete sudoku
// TODO: generate valid incomplete sudoku

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sudoku<Cell: SudokuCell> {
    base: usize,
    cells: Vec<Cell>,
}

// TODO: rethink indexing story (internal/cell position/block position)
// TODO: separate sudoku and grid (model/controller) (efficiency?)
// TODO: move row, all_rows, column, all_column, all_cells into grid
// TODO: Cell with Position in iterators (PositionedCell?)
// TODO: Check value of cells
// TODO: Preset value/user filled cells
impl<Cell: SudokuCell> Sudoku<Cell> {
    pub fn new(base: usize) -> Self {
        let mut sudoku = Sudoku {
            base,
            cells: vec![],
        };

        sudoku.cells = vec![Default::default(); sudoku.cell_count()];
        sudoku
    }

    // TODO: return cell values
    pub fn candidates(&self, pos: Position) -> VecDeque<Cell> {
        let conflicting_cells = self.column(pos.column)
            .chain(self.row(pos.row))
            .chain(self.block(pos))
            .collect::<BTreeSet<&Cell>>();

        let value_range: Vec<Cell> = self.value_range().collect();

        let values = value_range.iter().collect::<BTreeSet<&Cell>>();

        values.difference(&conflicting_cells).map(|cell| (**cell).clone()).collect()
    }

    pub fn has_conflict(&self) -> bool {
        self.all_rows().any(|row| self.has_duplicate(row)) ||
            self.all_columns().any(|column| self.has_duplicate(column)) ||
            self.all_blocks().any(|block| self.has_duplicate(block))
    }

    pub fn has_conflict_at(&self, pos: Position) -> bool {
        self.has_duplicate(self.row(pos.row)) ||
            self.has_duplicate(self.column(pos.column)) ||
            self.has_duplicate(self.block(pos))
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

    pub fn all_positions(&self) -> impl Iterator<Item=Position> {
        let side_length = self.side_length();

        (0..side_length)
            .flat_map(move |row_index| (0..side_length).map(move |column_index| Position {
                column: column_index,
                row: row_index,
            }))
    }

    pub fn empty_positions(&self) -> Vec<Position> {
        self.all_positions().filter(|pos| !self.get(*pos).has_value()).collect()
    }

    // TODO: parameter row_index
    fn row(&self, row_index: usize) -> impl Iterator<Item=&Cell> {
        self.assert_coordinate(row_index);

        let starting_index = row_index * self.side_length();

        (starting_index..starting_index + self.side_length()).map(move |i| &self.cells[i])
    }

    fn all_rows(&self) -> impl Iterator<Item=impl Iterator<Item=&Cell>> {
        (0..self.side_length()).map(move |row_index| {
            self.row(row_index)
        })
    }

    // TODO: parameter column_index
    fn column(&self, column_index: usize) -> impl Iterator<Item=&Cell> {
        self.assert_coordinate(column_index);

        (column_index..self.cell_count()).step_by(self.side_length()).map(move |i| &self.cells[i])
    }


    fn all_columns(&self) -> impl Iterator<Item=impl Iterator<Item=&Cell>> {
        (0..self.side_length()).map(move |column_index| {
            self.column(column_index)
        })
    }

    fn block(&self, pos: Position) -> impl Iterator<Item=&Cell> {
        self.assert_position(pos);

        let block_base_pos = (pos / self.base) * self.base;

        let block_base_index = self.index_at(block_base_pos);

        (block_base_index..self.cell_count())
            .step_by(self.side_length())
            .take(self.base)
            .flat_map(move |block_row_start_index| {
                (block_row_start_index..block_row_start_index + self.base)
            })
            .map(move |i| &self.cells[i])
    }

    fn all_blocks(&self) -> impl Iterator<Item=impl Iterator<Item=&Cell>> {
        let all_block_base_pos =
            (0..self.base)
                .flat_map(
                    move |block_y| (0..self.base).map(move |block_x| Position {
                        column: block_x,
                        row: block_y,
                    })
                )
                .map(move |pos| pos * self.base);

        all_block_base_pos.map(move |block_base_pos| self.block(block_base_pos))
    }

    pub fn get(&self, pos: Position) -> &Cell {
        self.assert_position(pos);

        let index = self.index_at(pos);

        &self.cells[index]
    }


    pub fn set(&mut self, pos: Position, cell: Cell) -> Cell {
        use std::mem;

        self.assert_position(pos);

        let index = self.index_at(pos);

        let previous_value = mem::replace(&mut self.cells[index], cell);

        previous_value
    }

    fn assert_coordinate(&self, coordinate: usize) {
        assert!(coordinate < self.side_length())
    }

    fn assert_position(&self, pos: Position) {
        assert!(pos.column < self.side_length() && pos.row < self.side_length())
    }

    fn index_at(&self, pos: Position) -> usize {
        pos.column + pos.row * self.side_length()
    }

    #[allow(dead_code)]
    fn pos_at(&self, index: usize) -> Position {
        Position {
            column: index / self.side_length(),
            row: index % self.side_length(),
        }
    }

    fn value_range(&self) -> impl Iterator<Item=Cell> {
        (1..=self.side_length()).map(|value| Cell::new_with_value(value))
    }

    pub fn side_length(&self) -> usize {
        self.base.pow(2)
    }

    pub fn cell_count(&self) -> usize {
        Self::base_to_cell_count(self.base)
    }

    pub fn base(&self) -> usize {
        self.base
    }

    fn base_to_cell_count(base: usize) -> usize {
        base.pow(4)
    }

    fn cell_count_to_base(cell_count: usize) -> Result<usize> {
        let approx_base = (cell_count as f64).sqrt().sqrt().round() as usize;

        ensure!(
            Self::base_to_cell_count(approx_base) == cell_count,
            "Cell count {} has no valid sudoku base",
            cell_count
        );

        Ok(approx_base)
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
        let base = Self::cell_count_to_base(cells.len())?;

        Ok(Sudoku {
            base,
            cells,
        })
    }
}

impl<Cell: SudokuCell> TryFrom<Vec<Vec<usize>>> for Sudoku<Cell> {
    type Error = Error;

    fn try_from(nested_values: Vec<Vec<usize>>) -> Result<Self> {
        nested_values.into_iter().flatten().collect::<Vec<_>>().try_into()
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
        const PADDING: usize = 3;

        let horizontal_block_separator = "-".repeat(self.base + (PADDING * self.side_length()));

        let output_string = self.cells
            .chunks(self.side_length())
            .map(|row| row.chunks(self.base)
                .map(|block_row| block_row.iter()
                    .map(|cell| format!("{:>PADDING$}", cell.to_string(), PADDING = PADDING))
                    .collect::<String>()
                ).collect::<Vec<_>>().join("|")
            )
            .collect::<Vec<String>>()
            .chunks(self.base)
            .intersperse(&[horizontal_block_separator])
            .flatten()
            .cloned()
            .collect::<Vec<String>>().join("\n");


        f.write_str(&output_string)
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