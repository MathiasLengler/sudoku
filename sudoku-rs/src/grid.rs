use std::collections::HashSet;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::{Display, Formatter};

use ndarray::{Array2, Axis};
use typenum::Unsigned;

use crate::base::SudokuBase;
use crate::cell::compact::candidates::Candidates;
use crate::cell::compact::value::Value;
use crate::cell::view::parser::parse_cells;
use crate::cell::view::CellView;
use crate::cell::Cell;
use crate::error::{Error, Result};
use crate::position::Position;

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
    pub fn update_candidates(&mut self, pos: Position, value: Value<Base>) {
        self.neighbor_positions_with_duplicates(pos)
            .for_each(|pos| {
                let cell = self.get_mut(pos);
                if cell.has_candidates() {
                    cell.delete_candidate(value);
                }
            });
    }

    pub fn direct_candidates(&self, pos: Position) -> Candidates<Base> {
        let mut candidates = Candidates::<Base>::all();

        {
            let mut candidates_mut = candidates.as_mut();

            for pos in self.neighbor_positions_with_duplicates(pos) {
                if let Some(value) = self.get(pos).value() {
                    candidates_mut.delete(value);
                }
            }
        }

        candidates
    }

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

    // TODO: bit_slice set optimization
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
            .filter(|pos| self.get(*pos).has_value())
            .collect()
    }

    pub fn all_candidates_positions(&self) -> Vec<Position> {
        self.all_positions()
            .filter(|pos| self.get(*pos).has_candidates())
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
        use itertools::Itertools;

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
            .map(|view| view.into().try_into_cell())
            .collect::<Result<_>>()?;

        Ok(Self::with_cells(cells))
    }
}

impl<Base: SudokuBase> TryFrom<&str> for Grid<Base> {
    type Error = Error;

    fn try_from(input: &str) -> Result<Self> {
        Ok(parse_cells(input)?.try_into()?)
    }
}

impl<Base: SudokuBase> Display for Grid<Base> {
    // TODO: implement using prettytable-rs
    // TODO: show candidates (compare with exchange formats)
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use itertools::Itertools;

        const PADDING: usize = 3;

        let horizontal_block_separator =
            "-".repeat(Self::base_usize() + (PADDING * Self::side_length_usize()));

        let output_string = Itertools::intersperse(
            self.cells
                .rows()
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
                .chunks(Self::base_usize()),
            &[horizontal_block_separator],
        )
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

    use crate::samples;

    use super::*;

    #[test]
    fn test_has_conflict() -> Result<()> {
        let mut grid = Grid::<U3>::new();
        assert!(!grid.has_conflict());

        grid.get_mut(Position { column: 0, row: 0 })
            .set_value(1.try_into()?);
        assert!(!grid.has_conflict());

        grid.get_mut(Position { column: 1, row: 0 })
            .set_value(1.try_into()?);
        assert!(grid.has_conflict());

        grid.get_mut(Position { column: 1, row: 0 }).delete();
        assert!(!grid.has_conflict());

        grid.get_mut(Position { column: 0, row: 1 })
            .set_value(1.try_into()?);
        assert!(grid.has_conflict());

        grid.get_mut(Position { column: 0, row: 1 }).delete();
        assert!(!grid.has_conflict());

        grid.get_mut(Position { column: 1, row: 1 })
            .set_value(1.try_into()?);
        assert!(grid.has_conflict());

        grid.get_mut(Position { column: 1, row: 1 }).delete();
        assert!(!grid.has_conflict());

        Ok(())
    }

    #[test]
    fn test_direct_candidates() -> Result<()> {
        let grid = samples::base_3().pop().unwrap();

        let direct_candidates = grid.direct_candidates(Position { column: 1, row: 1 });

        assert_eq!(direct_candidates, vec![1, 2, 4].try_into()?);

        Ok(())
    }

    #[test]
    fn test_update_candidates() -> Result<()> {
        let mut grid = samples::base_2().first().unwrap().clone();

        grid.set_all_direct_candidates();

        assert_eq!(
            {
                let mut grid = grid.clone();
                let pos = Position { column: 0, row: 3 };
                grid.update_candidates(pos, 1.try_into()?);
                grid
            },
            { grid.clone() }
        );

        assert_eq!(
            {
                let mut grid = grid.clone();
                let pos = Position { column: 0, row: 3 };
                grid.update_candidates(pos, 2.try_into()?);
                grid
            },
            {
                let mut grid = grid.clone();
                grid.get_mut(Position { column: 0, row: 0 }).delete();
                grid
            }
        );
        assert_eq!(
            {
                let mut grid = grid.clone();
                let pos = Position { column: 0, row: 3 };
                grid.update_candidates(pos, 4.try_into()?);
                grid
            },
            {
                let mut grid = grid.clone();
                grid.get_mut(Position { column: 1, row: 2 }).delete();
                grid.get_mut(Position { column: 3, row: 3 }).delete();
                grid
            }
        );

        Ok(())
    }
}
