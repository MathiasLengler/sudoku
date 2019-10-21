use std::collections::HashSet;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::{Display, Formatter};

use failure::ensure;
// TODO: replace with bitvec
use fixedbitset::FixedBitSet;
use itertools::Itertools;
use ndarray::{Array2, Axis};
use typenum::Unsigned;

use crate::base::SudokuBase;
use crate::cell::view::CellView;
use crate::cell::Cell;
use crate::cell::SudokuCell;
use crate::error::{Error, Result};
use crate::grid::parser::{from_givens_grid, from_givens_line};
use crate::position::Position;

mod parser;

//use static_assertions as sa;
//sa::assert_obj_safe!(SudokuGrid);

// TODO: decide if this abstraction is useful
trait SudokuGrid {
    type Cell: SudokuCell;

    // Dimensions
    fn base() -> u8;
    fn side_length() -> u8;
    fn max_value() -> u8 {
        Self::side_length()
    }
    fn cell_count() -> usize;

    // Cell accessors
    fn get(&self, pos: Position) -> &Self::Cell;
    fn get_mut(&mut self, pos: Position) -> &mut Self::Cell;

    // Helper
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Grid<Base: SudokuBase> {
    cells: Array2<Cell<Base>>,
}

/// Public API
impl<Base: SudokuBase> Grid<Base> {
    pub fn set_all_direct_candidates(&mut self) {
        self.all_candidates_positions().into_iter().for_each(|pos| {
            let candidates = self.direct_candidates(pos);

            self.get_mut(pos).set_candidates(candidates);
        });
    }
    pub fn update_candidates(&mut self, pos: Position, value: u8) {
        if value == 0 {
            return;
        }

        self.neighbor_positions_with_duplicates(pos)
            .for_each(|pos| {
                if self.get(pos).candidates().is_some() {
                    let cell = self.get_mut(pos);

                    cell.delete_candidate(value);
                }
            });
    }
    pub fn direct_candidates(&self, pos: Position) -> Vec<u8> {
        // TODO: implement with bitvec (XOR?)
        // TODO: bitslice u8 index iterator

        let conflicting_values: FixedBitSet = self
            .neighbor_positions_with_duplicates(pos)
            .filter_map(|pos| self.get(pos).value().map(|value| value.into()))
            .collect();

        let values = self.value_range_bit_set();

        let mut candidates = Vec::with_capacity(Self::side_length().into());

        candidates.extend(values.difference(&conflicting_values));

        candidates
            .into_iter()
            .map(|candidate| candidate.try_into().unwrap())
            .collect()
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
    pub fn has_duplicate<'a>(&'a self, cells: impl Iterator<Item = &'a Cell<Base>>) -> bool {
        let mut unique = HashSet::with_capacity(Self::side_length() as usize);

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
impl<Base: SudokuBase> Grid<Base> {
    pub fn new() -> Self {
        let cells = vec![Cell::new(); Self::cell_count()];

        Self::with_cells(cells)
    }

    fn with_cells(cells: Vec<Cell<Base>>) -> Self {
        assert_eq!(cells.len(), Self::cell_count());

        let side_length = Self::side_length() as usize;

        Grid {
            cells: Array2::from_shape_vec((side_length, side_length), cells).unwrap(),
        }
    }

    pub fn get(&self, pos: Position) -> &Cell<Base> {
        Self::assert_position(pos);

        &self.cells[pos.index_tuple()]
    }

    pub fn get_mut(&mut self, pos: Position) -> &mut Cell<Base> {
        Self::assert_position(pos);

        &mut self.cells[pos.index_tuple()]
    }

    pub fn fix_all_values(&mut self) {
        for pos in self.all_value_positions() {
            self.get_mut(pos).fix();
        }
    }

    fn index_at(&self, pos: Position) -> usize {
        usize::from(pos.column) + usize::from(pos.row) * usize::from(Self::side_length())
    }

    #[allow(dead_code)]
    pub(super) fn value_range(&self) -> impl Iterator<Item = u8> {
        (1..=Self::side_length())
    }

    pub(super) fn value_range_bit_set(&self) -> FixedBitSet {
        let mut bit_set = FixedBitSet::with_capacity((Self::side_length() + 1).into());
        bit_set.set_range(1.., true);
        bit_set
    }
    pub fn base() -> u8 {
        Base::to_u8()
    }
    pub fn side_length() -> u8 {
        Base::SideLength::to_u8()
    }
    pub fn max_value() -> u8 {
        Base::MaxValue::to_u8()
    }
    pub fn cell_count() -> usize {
        Base::CellCount::to_usize()
    }
    pub fn base_usize() -> usize {
        Base::to_usize()
    }
    pub fn side_length_usize() -> usize {
        Base::SideLength::to_usize()
    }
    pub fn max_value_usize() -> usize {
        Base::MaxValue::to_usize()
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
impl<Base: SudokuBase> Grid<Base> {
    fn positions_to_cells(
        &self,
        positions: impl Iterator<Item = Position>,
    ) -> impl Iterator<Item = &Cell<Base>> {
        positions.map(move |pos| self.get(pos))
    }

    fn nested_positions_to_nested_cells(
        &self,
        nested_positions: impl Iterator<Item = impl Iterator<Item = Position>>,
    ) -> impl Iterator<Item = impl Iterator<Item = &Cell<Base>>> {
        nested_positions.map(move |row_pos| row_pos.map(move |pos| self.get(pos)))
    }

    pub fn row_cells(&self, row: u8) -> impl Iterator<Item = &Cell<Base>> {
        self.positions_to_cells(self.row_positions(row))
    }

    pub fn all_row_cells(&self) -> impl Iterator<Item = impl Iterator<Item = &Cell<Base>>> {
        self.nested_positions_to_nested_cells(self.all_row_positions())
    }

    pub fn column_cells(&self, column: u8) -> impl Iterator<Item = &Cell<Base>> {
        self.positions_to_cells(self.column_positions(column))
    }

    pub fn all_column_cells(&self) -> impl Iterator<Item = impl Iterator<Item = &Cell<Base>>> {
        self.nested_positions_to_nested_cells(self.all_column_positions())
    }

    pub fn block_cells(&self, pos: Position) -> impl Iterator<Item = &Cell<Base>> {
        self.positions_to_cells(self.block_positions(pos))
    }

    // TODO: exact chunks
    pub fn all_block_cells(&self) -> impl Iterator<Item = impl Iterator<Item = &Cell<Base>>> {
        self.nested_positions_to_nested_cells(self.all_block_positions())
    }
}

/// Position iterators
impl<Base: SudokuBase> Grid<Base> {
    pub fn all_positions(&self) -> impl Iterator<Item = Position> {
        self.all_row_positions().flatten()
    }

    pub fn row_positions(&self, row: u8) -> impl Iterator<Item = Position> {
        Self::assert_coordinate(row);

        (0..Self::side_length()).map(move |column| Position { column, row })
    }

    pub fn all_row_positions(&self) -> impl Iterator<Item = impl Iterator<Item = Position>> {
        (0..Self::side_length())
            .map(move |row_index| self.row_positions(row_index))
            .collect::<Vec<_>>()
            .into_iter()
    }

    pub fn column_positions(&self, column: u8) -> impl Iterator<Item = Position> {
        Self::assert_coordinate(column);

        (0..Self::side_length()).map(move |row| Position { column, row })
    }

    pub fn all_column_positions(&self) -> impl Iterator<Item = impl Iterator<Item = Position>> {
        (0..Self::side_length())
            .map(move |column| self.column_positions(column))
            .collect::<Vec<_>>()
            .into_iter()
    }

    pub fn block_positions(&self, pos: Position) -> impl Iterator<Item = Position> {
        Self::assert_position(pos);

        let base = Self::base();

        let Position {
            column: base_column,
            row: base_row,
        } = (pos / base) * base;

        (base_row..base_row + base).flat_map(move |row| {
            (base_column..base_column + base).map(move |column| Position { column, row })
        })
    }

    pub fn all_block_positions(&self) -> impl Iterator<Item = impl Iterator<Item = Position>> {
        let all_block_base_pos = (0..Self::base())
            .flat_map(move |row| (0..Self::base()).map(move |column| Position { column, row }))
            .map(move |pos| pos * Self::base());

        all_block_base_pos
            .map(|block_base_pos| self.block_positions(block_base_pos))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

/// Filtered position vec
impl<Base: SudokuBase> Grid<Base> {
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
impl<Base: SudokuBase> Grid<Base> {
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
impl<Base: SudokuBase> Grid<Base> {
    fn assert_coordinate(coordinate: u8) {
        assert!(coordinate < Self::side_length())
    }

    fn assert_position(pos: Position) {
        Self::assert_coordinate(pos.column);
        Self::assert_coordinate(pos.row);
    }
}

impl<Base: SudokuBase, CView: Into<CellView>> TryFrom<Vec<Vec<CView>>> for Grid<Base> {
    type Error = Error;

    fn try_from(nested_views: Vec<Vec<CView>>) -> Result<Self> {
        nested_views
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .try_into()
    }
}

impl<Base: SudokuBase, CView: Into<CellView>> TryFrom<Vec<CView>> for Grid<Base> {
    type Error = Error;

    fn try_from(views: Vec<CView>) -> Result<Self> {
        let cells = views
            .into_iter()
            .map(|view| view.into().into_cell())
            .collect();

        Ok(Self::with_cells(cells))
    }
}

impl<Base: SudokuBase> TryFrom<&str> for Grid<Base> {
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

impl<Base: SudokuBase> Display for Grid<Base> {
    // TODO: implement using prettytable-rs
    // TODO: show candidates (compare with exchange formats)
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        const PADDING: usize = 3;

        let horizontal_block_separator =
            "-".repeat(Self::base_usize() + (PADDING * Self::side_length_usize()));

        let output_string = self
            .cells
            .genrows()
            .into_iter()
            .map(|row| {
                row.axis_chunks_iter(Axis(0), Self::base_usize())
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
            .chunks(Self::base_usize())
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
    use typenum::consts::*;

    use crate::cell::Cell;
    use crate::samples;

    use super::*;

    #[test]
    fn test_value_range() {
        let grid = Grid::<U3>::new();

        let value_range: Vec<_> = grid.value_range().collect();

        assert_eq!(value_range, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_value_range_bit_set() {
        let grid = Grid::<Base>::new(3);

        let value_range_bit_set: Vec<_> = grid.value_range_bit_set().ones().collect();

        assert_eq!(value_range_bit_set, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_has_conflict() {
        let mut grid = Grid::<Base>::new(3);

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
            .map(|input| Grid::<Base>::try_from(*input))
            .collect::<Result<Vec<_>>>()?;

        Ok(())
    }
}
