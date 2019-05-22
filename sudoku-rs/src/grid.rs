use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};

use failure::ensure;
use fixedbitset::FixedBitSet;

use crate::cell::SudokuCell;
use crate::error::{Error, Result};
use crate::position::Position;

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug)]
pub struct Grid<Cell: SudokuCell> {
    base: usize,
    cells: Vec<Cell>,
    fixed_cells: FixedBitSet,
}

// TODO: rethink indexing story (internal/cell position/block position)
//  => use Index/IndexMut with custom index type:
//     Cell, Row, Column, Block
impl<Cell: SudokuCell> Grid<Cell> {
    pub(super) fn new(base: usize) -> Self {
        let cell_count = Self::base_to_cell_count(base);

        let cells = vec![Cell::new(Self::base_to_max_value(base)); cell_count];

        Self::new_with_cells(base, cells)
    }

    fn new_with_cells(base: usize, cells: Vec<Cell>) -> Self {
        Grid {
            base,
            fixed_cells: Default::default(),
            cells,
        }
    }

    pub(super) fn get_pos(&self, pos: Position) -> &Cell {
        self.assert_position(pos);

        let index = self.index_at(pos);

        &self.cells[index]
    }

    pub(super) fn get_pos_mut(&mut self, pos: Position) -> &mut Cell {
        self.assert_position(pos);

        let index = self.index_at(pos);

        assert!(
            !self.fixed_cells[index],
            "Frozen cell at {} can't be modified",
            pos
        );

        &mut self.cells[index]
    }

    pub(super) fn fix_all_values(&mut self) {
        self.fixed_cells = self
            .all_positions()
            .filter_map(|pos| {
                if self.get_pos(pos).value().is_some() {
                    Some(self.index_at(pos))
                } else {
                    None
                }
            })
            .collect();
    }

    pub(super) fn unfix(&mut self) {
        self.fixed_cells = Default::default()
    }

    pub(super) fn is_fixed(&self, pos: Position) -> bool {
        let index = self.index_at(pos);

        self.fixed_cells[index]
    }

    fn index_at(&self, pos: Position) -> usize {
        pos.column + pos.row * self.side_length()
    }

    pub(super) fn value_range(&self) -> impl Iterator<Item = usize> {
        (1..=self.side_length())
    }

    pub(super) fn base(&self) -> usize {
        self.base
    }

    pub(super) fn side_length(&self) -> usize {
        Self::base_to_side_length(self.base)
    }

    pub(super) fn max_value(&self) -> usize {
        Self::base_to_max_value(self.base)
    }

    pub(super) fn cell_count(&self) -> usize {
        Self::base_to_cell_count(self.base)
    }

    fn base_to_side_length(base: usize) -> usize {
        base.pow(2)
    }

    fn base_to_max_value(base: usize) -> usize {
        Self::base_to_side_length(base)
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

/// Cell iterators
impl<Cell: SudokuCell> Grid<Cell> {
    fn positions_to_cells(
        &self,
        positions: impl Iterator<Item = Position>,
    ) -> impl Iterator<Item = &Cell> {
        positions.map(move |pos| self.get_pos(pos))
    }

    fn nested_positions_to_nested_cells(
        &self,
        nested_positions: impl Iterator<Item = impl Iterator<Item = Position>>,
    ) -> impl Iterator<Item = impl Iterator<Item = &Cell>> {
        nested_positions.map(move |row_pos| row_pos.map(move |pos| self.get_pos(pos)))
    }

    pub fn row_cells(&self, row: usize) -> impl Iterator<Item = &Cell> {
        self.positions_to_cells(self.row_positions(row))
    }

    pub fn all_row_cells(&self) -> impl Iterator<Item = impl Iterator<Item = &Cell>> {
        self.nested_positions_to_nested_cells(self.all_row_positions())
    }

    pub fn column_cells(&self, column: usize) -> impl Iterator<Item = &Cell> {
        self.positions_to_cells(self.column_positions(column))
    }

    pub fn all_column_cells(&self) -> impl Iterator<Item = impl Iterator<Item = &Cell>> {
        self.nested_positions_to_nested_cells(self.all_column_positions())
    }

    pub fn block_cells(&self, pos: Position) -> impl Iterator<Item = &Cell> {
        self.positions_to_cells(self.block_positions(pos))
    }

    pub fn all_block_cells(&self) -> impl Iterator<Item = impl Iterator<Item = &Cell>> {
        self.nested_positions_to_nested_cells(self.all_block_positions())
    }
}

/// Position iterators
impl<Cell: SudokuCell> Grid<Cell> {
    pub fn all_positions(&self) -> impl Iterator<Item = Position> {
        self.all_row_positions().flatten()
    }

    pub fn row_positions(&self, row: usize) -> impl Iterator<Item = Position> {
        self.assert_coordinate(row);

        (0..self.side_length()).map(move |column| Position { column, row })
    }

    pub fn all_row_positions(&self) -> impl Iterator<Item = impl Iterator<Item = Position>> {
        (0..self.side_length())
            .map(move |row_index| self.row_positions(row_index))
            .collect::<Vec<_>>()
            .into_iter()
    }

    pub fn column_positions(&self, column: usize) -> impl Iterator<Item = Position> {
        self.assert_coordinate(column);

        (0..self.side_length()).map(move |row| Position { column, row })
    }

    pub fn all_column_positions(&self) -> impl Iterator<Item = impl Iterator<Item = Position>> {
        (0..self.side_length())
            .map(move |column| self.column_positions(column))
            .collect::<Vec<_>>()
            .into_iter()
    }

    pub fn block_positions(&self, pos: Position) -> impl Iterator<Item = Position> {
        self.assert_position(pos);

        let base = self.base;

        let Position {
            column: base_column,
            row: base_row,
        } = (pos / base) * base;

        (base_row..base_row + base).flat_map(move |row| {
            (base_column..base_column + base).map(move |column| Position { column, row })
        })
    }

    pub fn all_block_positions(&self) -> impl Iterator<Item = impl Iterator<Item = Position>> {
        let all_block_base_pos = (0..self.base)
            .flat_map(move |row| (0..self.base).map(move |column| Position { column, row }))
            .map(move |pos| pos * self.base);

        all_block_base_pos
            .map(|block_base_pos| self.block_positions(block_base_pos))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

/// Filtered Position iterators
impl<Cell: SudokuCell> Grid<Cell> {
    pub fn all_empty_positions(&self) -> Vec<Position> {
        self.all_positions()
            .filter(|pos| self.get_pos(*pos).value().is_none())
            .collect()
    }

    pub fn all_value_positions(&self) -> Vec<Position> {
        self.all_positions()
            .filter(|pos| self.get_pos(*pos).value().is_some())
            .collect()
    }
}

/// Asserts
impl<Cell: SudokuCell> Grid<Cell> {
    fn assert_coordinate(&self, coordinate: usize) {
        assert!(coordinate < self.side_length())
    }

    fn assert_position(&self, pos: Position) {
        self.assert_coordinate(pos.column);
        self.assert_coordinate(pos.row);
    }
}

impl<Cell: SudokuCell> TryFrom<Vec<usize>> for Grid<Cell> {
    type Error = Error;

    fn try_from(values: Vec<usize>) -> Result<Self> {
        let base = Self::cell_count_to_base(values.len())?;

        let max = Self::base_to_side_length(base);

        let cells = values
            .into_iter()
            .map(|value| Cell::new_with_value(value, max))
            .collect();

        Ok(Self::new_with_cells(base, cells))
    }
}

impl<Cell: SudokuCell> Display for Grid<Cell> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use itertools::Itertools;

        const PADDING: usize = 3;

        let horizontal_block_separator = "-".repeat(self.base() + (PADDING * self.side_length()));

        let output_string = self
            .cells
            .chunks(self.side_length())
            .map(|row| {
                row.chunks(self.base())
                    .map(|block_row| {
                        block_row
                            .iter()
                            .map(|cell| {
                                format!("{:>PADDING$}", cell.to_string(), PADDING = PADDING)
                            })
                            .collect::<String>()
                    })
                    .collect::<Vec<_>>()
                    .join("|")
            })
            .collect::<Vec<String>>()
            .chunks(self.base())
            .intersperse(&[horizontal_block_separator])
            .flatten()
            .cloned()
            .collect::<Vec<String>>()
            .join("\n");

        f.write_str(&output_string)
    }
}
