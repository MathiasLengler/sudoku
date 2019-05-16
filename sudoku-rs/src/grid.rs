use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};

use failure::ensure;

use crate::cell::SudokuCell;
use crate::error::{Error, Result};
use crate::position::Position;

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug)]
pub(super) struct Grid<Cell: SudokuCell> {
    base: usize,
    cells: Vec<Cell>,
    // TODO: fixedCellIndices
    //  evaluate API for (un)fixing a cell
    //  check access
    //  propagate to transport sudoku
}

// TODO: rethink indexing story (internal/cell position/block position)
//  => position based indexing (performance should be no issue)
impl<Cell: SudokuCell> Grid<Cell> {
    pub(super) fn new(base: usize) -> Self {
        let mut grid = Grid {
            base,
            cells: vec![],
        };

        grid.cells = vec![Cell::new(Self::base_to_max_value(base)); grid.cell_count()];
        grid
    }

    pub(super) fn get_pos(&self, pos: Position) -> &Cell {
        self.assert_position(pos);

        let index = self.index_at(pos);

        &self.cells[index]
    }

    pub(super) fn get_pos_mut(&mut self, pos: Position) -> &mut Cell {
        self.assert_position(pos);

        let index = self.index_at(pos);

        &mut self.cells[index]
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

/// Utility iterators
impl<Cell: SudokuCell> Grid<Cell> {
    // TODO: change cell iters to be based on position iters and move to separate impl block
    pub(super) fn row(&self, row_index: usize) -> impl Iterator<Item = &Cell> {
        self.assert_coordinate(row_index);

        let starting_index = row_index * self.side_length();

        (starting_index..starting_index + self.side_length()).map(move |i| &self.cells[i])
    }

    pub(super) fn all_rows(&self) -> impl Iterator<Item = impl Iterator<Item = &Cell>> {
        (0..self.side_length()).map(move |row_index| self.row(row_index))
    }

    pub(super) fn column(&self, column_index: usize) -> impl Iterator<Item = &Cell> {
        self.assert_coordinate(column_index);

        (column_index..self.cell_count())
            .step_by(self.side_length())
            .map(move |i| &self.cells[i])
    }

    pub(super) fn all_columns(&self) -> impl Iterator<Item = impl Iterator<Item = &Cell>> {
        (0..self.side_length()).map(move |column_index| self.column(column_index))
    }

    pub(super) fn block(&self, pos: Position) -> impl Iterator<Item = &Cell> {
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

    pub(super) fn all_blocks(&self) -> impl Iterator<Item = impl Iterator<Item = &Cell>> {
        let all_block_base_pos = (0..self.base)
            .flat_map(move |block_y| {
                (0..self.base).map(move |block_x| Position {
                    column: block_x,
                    row: block_y,
                })
            })
            .map(move |pos| pos * self.base);

        all_block_base_pos.map(move |block_base_pos| self.block(block_base_pos))
    }

    pub(super) fn row_positions(&self, row: usize) -> impl Iterator<Item = Position> {
        self.assert_coordinate(row);

        (0..self.side_length()).map(move |column| Position { column, row })
    }

    pub(super) fn all_row_positions(&self) -> impl Iterator<Item = impl Iterator<Item = Position>> {
        (0..self.side_length())
            .map(move |row_index| self.row_positions(row_index))
            .collect::<Vec<_>>()
            .into_iter()
    }

    pub(super) fn column_positions(&self, column: usize) -> impl Iterator<Item = Position> {
        self.assert_coordinate(column);

        (0..self.side_length()).map(move |row| Position { column, row })
    }

    pub(super) fn all_column_positions(
        &self,
    ) -> impl Iterator<Item = impl Iterator<Item = Position>> {
        (0..self.side_length())
            .map(move |column| self.column_positions(column))
            .collect::<Vec<_>>()
            .into_iter()
    }

    pub(super) fn block_positions(&self, pos: Position) -> impl Iterator<Item = Position> {
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

    pub(super) fn all_block_positions(
        &self,
    ) -> impl Iterator<Item = impl Iterator<Item = Position>> {
        let all_block_base_pos = (0..self.base)
            .flat_map(move |row| (0..self.base).map(move |column| Position { column, row }))
            .map(move |pos| pos * self.base);

        all_block_base_pos
            .map(|block_base_pos| self.block_positions(block_base_pos))
            .collect::<Vec<_>>()
            .into_iter()
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

        Ok(Grid {
            base,
            cells: values
                .into_iter()
                .map(|value| Cell::new_with_value(value, max))
                .collect(),
        })
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
