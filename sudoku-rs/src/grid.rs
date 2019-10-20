use std::collections::HashSet;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::{Display, Formatter};

use failure::ensure;
// TODO: replace with bitvec
use fixedbitset::FixedBitSet;
use itertools::Itertools;
use ndarray::{Array2, Axis};

use crate::cell::view::CellView;
use crate::cell::SudokuCell;
use crate::error::{Error, Result};
use crate::grid::parser::{from_givens_grid, from_givens_line};
use crate::position::Position;

mod parser;

// TODO: update:
//  Grid<Base: SudokuBase> {
//      cells: Array2<CompactCell<Base>>
//  }
#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Grid<Cell: SudokuCell> {
    base: usize,
    cells: Array2<Cell>,
    // TODO: move into value of cell
    fixed_cells: FixedBitSet,
}

/// Public API
impl<Cell: SudokuCell> Grid<Cell> {
    pub fn set_value(&mut self, pos: Position, value: usize) {
        let max_value = self.max_value();

        self.get_mut(pos).set_value(value, max_value);
    }

    /// Returns true if a new value has been set.
    pub fn set_or_toggle_value(&mut self, pos: Position, value: usize) -> bool {
        let max_value = self.max_value();

        self.get_mut(pos).set_or_toggle_value(value, max_value)
    }

    pub fn set_candidates(&mut self, pos: Position, candidates: Vec<usize>) {
        let max_value = self.max_value();

        self.get_mut(pos).set_candidates(candidates, max_value);
    }

    pub fn toggle_candidate(&mut self, pos: Position, candidate: usize) {
        let max_value = self.max_value();

        self.get_mut(pos).toggle_candidate(candidate, max_value);
    }

    pub fn delete(&mut self, pos: Position) -> Cell {
        let max_value = self.max_value();

        self.get_mut(pos).delete(max_value)
    }
    pub fn set_all_direct_candidates(&mut self) {
        self.all_candidates_positions().into_iter().for_each(|pos| {
            let candidates = self.direct_candidates(pos);

            self.set_candidates(pos, candidates);
        });
    }
    pub fn update_candidates(&mut self, pos: Position, value: usize) {
        if value == 0 {
            return;
        }

        let max = self.max_value();

        self.neighbor_positions_with_duplicates(pos)
            .for_each(|pos| {
                if self.get(pos).candidates().is_some() {
                    let cell = self.get_mut(pos);

                    cell.delete_candidate(value, max);
                }
            });
    }
    pub fn direct_candidates(&self, pos: Position) -> Vec<usize> {
        let conflicting_values: FixedBitSet = self
            .neighbor_positions_with_duplicates(pos)
            .filter_map(|pos| self.get(pos).value())
            .collect();

        let values = self.value_range_bit_set();

        let mut candidates = Vec::with_capacity(self.side_length());

        candidates.extend(values.difference(&conflicting_values));

        candidates
    }

    #[allow(dead_code)]
    pub(crate) fn has_conflict(&self) -> bool {
        self.all_row_cells().any(|row| self.has_duplicate(row))
            || self
                .all_column_cells()
                .any(|column| self.has_duplicate(column))
            || self
                .all_block_cells()
                .any(|block| self.has_duplicate(block))
    }

    // TODO: optimize: is value in group?
    pub fn has_conflict_at(&self, pos: Position) -> bool {
        self.has_duplicate(self.row_cells(pos.row))
            || self.has_duplicate(self.column_cells(pos.column))
            || self.has_duplicate(self.block_cells(pos))
    }

    // TODO: conflict location pairs
    pub fn has_duplicate<'a>(&'a self, cells: impl Iterator<Item = &'a Cell>) -> bool {
        let mut unique = HashSet::with_capacity(self.side_length());

        cells
            .filter_map(|cell| cell.value())
            .any(move |x| !unique.insert(x))
    }

    pub fn is_solved(&self) -> bool {
        self.all_candidates_positions().is_empty() && !self.has_conflict()
    }
}

// TODO: rethink indexing story (internal/cell position/block position)
//  => use Index/IndexMut with custom index type:
//     Cell, Row, Column, Block
impl<Cell: SudokuCell> Grid<Cell> {
    pub fn new(base: usize) -> Self {
        let cell_count = Self::base_to_cell_count(base);

        let cells = vec![Cell::new(Self::base_to_max_value(base)); cell_count];

        Self::new_with_cells(base, cells)
    }

    fn new_with_cells(base: usize, cells: Vec<Cell>) -> Self {
        let side_length = Self::base_to_side_length(base);

        Grid {
            base,
            fixed_cells: Default::default(),
            cells: Array2::from_shape_vec((side_length, side_length), cells).unwrap(),
        }
    }

    pub(super) fn get(&self, pos: Position) -> &Cell {
        self.assert_position(pos);

        &self.cells[pos.index_tuple()]
    }

    fn get_mut(&mut self, pos: Position) -> &mut Cell {
        self.assert_position(pos);

        let index = self.index_at(pos);

        assert!(
            !self.fixed_cells[index],
            "Fixed cell at {} can't be modified",
            pos
        );

        &mut self.cells[pos.index_tuple()]
    }

    pub fn fix_all_values(&mut self) {
        self.fixed_cells = self
            .all_positions()
            .filter_map(|pos| {
                if self.get(pos).value().is_some() {
                    Some(self.index_at(pos))
                } else {
                    None
                }
            })
            .collect();
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub(super) fn value_range(&self) -> impl Iterator<Item = usize> {
        (1..=self.side_length())
    }

    pub(super) fn value_range_bit_set(&self) -> FixedBitSet {
        let mut bit_set = FixedBitSet::with_capacity(self.side_length() + 1);
        bit_set.set_range(1.., true);
        bit_set
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

// TODO: rewrite with ndarray slice
// TODO: zip position + cell
//  => impl Iterator<Item = &mut Cell>

/// Cell iterators
impl<Cell: SudokuCell> Grid<Cell> {
    fn positions_to_cells(
        &self,
        positions: impl Iterator<Item = Position>,
    ) -> impl Iterator<Item = &Cell> {
        positions.map(move |pos| self.get(pos))
    }

    fn nested_positions_to_nested_cells(
        &self,
        nested_positions: impl Iterator<Item = impl Iterator<Item = Position>>,
    ) -> impl Iterator<Item = impl Iterator<Item = &Cell>> {
        nested_positions.map(move |row_pos| row_pos.map(move |pos| self.get(pos)))
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

    // TODO: exact chunks
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

/// Filtered position vec
impl<Cell: SudokuCell> Grid<Cell> {
    pub fn all_value_positions(&self) -> Vec<Position> {
        self.all_positions()
            .filter(|pos| self.get(*pos).value().is_some())
            .collect()
    }

    pub fn all_candidates_positions(&self) -> Vec<Position> {
        self.all_positions()
            .filter(|pos| self.get(*pos).candidates().is_some())
            .collect()
    }
}

/// Neighbor iterators
impl<Cell: SudokuCell> Grid<Cell> {
    pub fn neighbor_positions_with_duplicates(
        &self,
        pos: Position,
    ) -> impl Iterator<Item = Position> {
        // TODO: reimplement without chain (VTune: bad speculation + unique version)
        self.row_positions(pos.row)
            .chain(self.column_positions(pos.column))
            .chain(self.block_positions(pos))
    }

    pub fn neighbor_positions(&self, pos: Position) -> impl Iterator<Item = Position> {
        self.neighbor_positions_with_duplicates(pos).unique()
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

impl<Cell: SudokuCell, CView: Into<CellView>> TryFrom<Vec<Vec<CView>>> for Grid<Cell> {
    type Error = Error;

    fn try_from(nested_views: Vec<Vec<CView>>) -> Result<Self> {
        nested_views
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .try_into()
    }
}

impl<Cell: SudokuCell, CView: Into<CellView>> TryFrom<Vec<CView>> for Grid<Cell> {
    type Error = Error;

    fn try_from(views: Vec<CView>) -> Result<Self> {
        let base = Self::cell_count_to_base(views.len())?;

        let max = Self::base_to_max_value(base);

        let cells = views
            .into_iter()
            .map(|view| view.into().into_cell(max))
            .collect();

        Ok(Self::new_with_cells(base, cells))
    }
}

impl<Cell: SudokuCell> TryFrom<&str> for Grid<Cell> {
    type Error = Error;

    fn try_from(input: &str) -> Result<Self> {
        use crate::grid::parser::from_candidates;

        let input = input.trim();

        if input.contains('\n') {
            from_candidates(input).or_else(|_| from_givens_grid(input))
        } else {
            from_givens_line(input)
        }
    }
}

impl<Cell: SudokuCell> Display for Grid<Cell> {
    // TODO: implement using prettytable-rs
    // TODO: show candidates (compare with exchange formats)
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        const PADDING: usize = 3;

        let horizontal_block_separator = "-".repeat(self.base() + (PADDING * self.side_length()));

        let output_string = self
            .cells
            .genrows()
            .into_iter()
            .map(|row| {
                row.axis_chunks_iter(Axis(0), self.base())
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

#[cfg(test)]
mod tests {
    use crate::cell::Cell;
    use crate::samples;

    use super::*;

    #[test]
    fn test_value_range() {
        let grid = Grid::<Cell>::new(3);

        let value_range: Vec<_> = grid.value_range().collect();

        assert_eq!(value_range, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_value_range_bit_set() {
        let grid = Grid::<Cell>::new(3);

        let value_range_bit_set: Vec<_> = grid.value_range_bit_set().ones().collect();

        assert_eq!(value_range_bit_set, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_has_conflict() {
        let mut grid = Grid::<Cell>::new(3);

        assert!(!grid.has_conflict());

        grid.set_value(Position { column: 0, row: 0 }, 1);

        assert!(!grid.has_conflict());

        grid.set_value(Position { column: 1, row: 0 }, 1);

        assert!(grid.has_conflict());

        grid.set_value(Position { column: 1, row: 0 }, 0);

        assert!(!grid.has_conflict());

        grid.set_value(Position { column: 0, row: 1 }, 1);

        assert!(grid.has_conflict());

        grid.set_value(Position { column: 0, row: 1 }, 0);

        assert!(!grid.has_conflict());

        grid.set_value(Position { column: 1, row: 1 }, 1);

        assert!(grid.has_conflict());

        grid.set_value(Position { column: 1, row: 1 }, 0);

        assert!(!grid.has_conflict());
    }

    #[test]
    fn test_direct_candidates() {
        let grid = samples::base_3().pop().unwrap();

        let direct_candidates = grid.direct_candidates(Position { column: 1, row: 1 });

        assert_eq!(direct_candidates, vec![1, 2, 4]);
    }

    #[test]
    fn test_update_candidates() {
        let mut grid = samples::base_2().first().unwrap().clone();

        grid.set_all_direct_candidates();

        assert_eq!(
            {
                let mut grid = grid.clone();
                let pos = Position { column: 0, row: 3 };
                grid.update_candidates(pos, 1);
                grid
            },
            { grid.clone() }
        );

        assert_eq!(
            {
                let mut grid = grid.clone();
                let pos = Position { column: 0, row: 3 };
                grid.update_candidates(pos, 2);
                grid
            },
            {
                let mut grid = grid.clone();
                grid.delete(Position { column: 0, row: 0 });
                grid
            }
        );
        assert_eq!(
            {
                let mut grid = grid.clone();
                let pos = Position { column: 0, row: 3 };
                grid.update_candidates(pos, 4);
                grid
            },
            {
                let mut grid = grid.clone();
                grid.delete(Position { column: 1, row: 2 });
                grid.delete(Position { column: 3, row: 3 });
                grid
            }
        );
    }

    #[test]
    fn test_try_from_str() -> Result<()> {
        let inputs = [
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/res/candidates.txt"
            )),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/res/givens_line.txt"
            )),
        ];

        inputs
            .into_iter()
            .map(|input| Grid::<Cell>::try_from(*input))
            .collect::<Result<Vec<_>>>()?;

        Ok(())
    }
}
