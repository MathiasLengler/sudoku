use anyhow::anyhow;
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::{Display, Formatter};

use ndarray::Array2;

use crate::base::SudokuBase;
use crate::cell::compact::candidates::Candidates;
use crate::cell::compact::cell_state::CellState;
use crate::cell::compact::value::Value;
use crate::cell::view::parser::parse_cells;
use crate::cell::view::CellView;
use crate::cell::Cell;
use crate::error::{Error, Result};
use crate::grid::serialization::GridFormat;
use crate::position::Position;
use crate::solver::strategic::deduction::{Deduction, DeductionKind};

pub mod deserialization;
pub mod serialization;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Grid<Base: SudokuBase> {
    cells: Array2<Cell<Base>>,
}

/// Direct Candidates
impl<Base: SudokuBase> Grid<Base> {
    pub fn set_all_direct_candidates(&mut self) {
        self.all_candidates_positions().into_iter().for_each(|pos| {
            let candidates = self.direct_candidates(pos);

            self.get_mut(pos).set_candidates(candidates);
        });
    }
    pub fn update_direct_candidates(&mut self, pos: Position, value: Value<Base>) {
        // TODO: assert params
        Self::neighbor_positions_with_duplicates(pos).for_each(|pos| {
            let cell = self.get_mut(pos);
            if cell.has_candidates() {
                cell.delete_candidate(value);
            }
        });
    }

    pub fn direct_candidates(&self, pos: Position) -> Candidates<Base> {
        assert!(self.get(pos).has_candidates());

        let mut candidates = Candidates::<Base>::all();

        for pos in Self::neighbor_positions_with_duplicates(pos) {
            if let Some(value) = self.get(pos).value() {
                candidates.delete(value);
            }
        }

        candidates
    }
}

// TODO: test
// TODO: bench
/// Consistency testing
impl<Base: SudokuBase> Grid<Base> {
    /// A grid is directly consistent, if:
    /// - No cell has empty candidates.
    /// - No candidate is deletable based on a group-adjacent value.
    /// - No group has duplicate values.
    /// - No group has a missing candidate, e.g. every group contains every value as either a value or at least one candidate.
    pub fn is_directly_consistent(&self) -> bool {
        // Every candidate is directly consistent at its position
        self.all_candidates_positions()
            .into_iter()
            .all(|pos| self.is_directly_consistent_at(pos))
            &&
            // Every group is directly consistent
            self
                .all_group_cells()
                .all(|group| Self::is_group_directly_consistent(group))
    }

    /// A group is directly consistent, if it:
    /// - has unique values.
    /// - has no missing candidate.
    fn is_group_directly_consistent<'a>(group_cells: impl Iterator<Item = &'a Cell<Base>>) -> bool
    where
        Base: 'a,
    {
        let mut seen_values = Candidates::new();
        let mut seen_candidates_or_values = Candidates::new();

        for cell in group_cells {
            match *cell.state() {
                CellState::Value(value) | CellState::FixedValue(value) => {
                    if seen_values.has(value) {
                        // Duplicate value in group.
                        return false;
                    }
                    seen_values.set(value, true);
                    seen_candidates_or_values.set(value, true)
                }
                CellState::Candidates(candidates) => {
                    seen_candidates_or_values = seen_candidates_or_values.union(&candidates)
                }
            }
        }

        // Every candidate must be contained in group.
        seen_candidates_or_values.is_full()
    }

    /// A cell with candidates is directly consistent, if its candidates:
    /// - are non-empty.
    /// - contain no candidate which is deletable based on a group-adjacent value.
    fn is_directly_consistent_at(&self, pos: Position) -> bool {
        let cell = self.get(pos);
        assert!(cell.has_candidates());
        let actual_candidates = cell.candidates().unwrap();
        // At least one candidate is required for a consistent grid.
        if actual_candidates.is_empty() {
            return false;
        }

        let direct_candidates = self.direct_candidates(pos);
        // No actual candidate is deletable via direct candidates.
        actual_candidates.without(&direct_candidates).is_empty()
    }
}

/// Public Sudoku API
impl<Base: SudokuBase> Grid<Base> {
    pub fn has_value_conflict(&self) -> bool {
        self.all_row_cells()
            .any(|row| Self::has_duplicate_value(row))
            || self
                .all_column_cells()
                .any(|column| Self::has_duplicate_value(column))
            || self
                .all_block_cells()
                .any(|block| Self::has_duplicate_value(block))
    }

    pub fn has_duplicate_value<'a>(cells: impl Iterator<Item = &'a Cell<Base>>) -> bool
    where
        Base: 'a,
    {
        let mut seen_values = Candidates::new();

        cells.filter_map(|cell| cell.value()).any(move |value| {
            if seen_values.has(value) {
                true
            } else {
                seen_values.set(value, true);
                false
            }
        })
    }

    pub fn is_solved(&self) -> bool {
        self.all_candidates_positions().is_empty() && !self.has_value_conflict()
    }
}

impl<Base: SudokuBase> Default for Grid<Base> {
    fn default() -> Self {
        Self::with_cells(vec![Cell::new(); Self::cell_count_usize()])
    }
}

// TODO: rethink indexing story (internal/cell position/block position)
//  => use Index/IndexMut with custom index type:
//     Cell, Row, Column, Block
/// Public Grid API
impl<Base: SudokuBase> Grid<Base> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_cells(cells: Vec<Cell<Base>>) -> Self {
        assert_eq!(cells.len(), Self::cell_count_usize());

        let side_length = Self::side_length() as usize;

        Grid {
            cells: Array2::from_shape_vec((side_length, side_length), cells).unwrap(),
        }
    }

    pub fn try_from_blocks(blocks: Vec<Vec<CellView>>) -> Result<Self> {
        assert_eq!(blocks.len(), Self::side_length() as usize);
        assert!(blocks
            .iter()
            .all(|block| block.len() == Self::side_length() as usize));

        let mut grid = Self::new();

        Self::all_block_positions()
            .zip(blocks)
            .try_for_each(|(block_positions, block)| {
                block_positions
                    .zip(block)
                    .try_for_each::<_, Result<()>>(|(pos, cell_view)| {
                        *grid.get_mut(pos) = cell_view.try_into()?;
                        Ok(())
                    })
            })?;

        Ok(grid)
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

    pub fn unfix_all_values(&mut self) {
        for pos in self.all_value_positions() {
            self.get_mut(pos).unfix();
        }
    }

    pub fn delete_all_unfixed_values(&mut self) {
        for pos in self.all_unfixed_value_positions() {
            self.get_mut(pos).delete();
        }
    }

    pub fn deduction_at<TryIntoKind: TryInto<DeductionKind<Base>>>(
        &self,
        pos: impl Into<Position>,
        kind: TryIntoKind,
    ) -> Result<Deduction<Base>>
    where
        Error: From<TryIntoKind::Error>,
    {
        let pos = pos.into();
        let kind = kind.try_into()?;

        Deduction::new(
            pos,
            self.get(pos).candidates().ok_or_else(|| {
                anyhow!("A deduction must be made for a position containing candidates")
            })?,
            kind,
        )
    }
}

/// Base constant accessors
impl<Base: SudokuBase> Grid<Base> {
    pub fn base() -> u8 {
        Base::BASE
    }
    pub fn side_length() -> u8 {
        Base::SIDE_LENGTH
    }
    pub fn max_value() -> u8 {
        Base::MAX_VALUE
    }
    pub fn cell_count() -> u16 {
        Base::CELL_COUNT
    }
    pub fn cell_count_usize() -> usize {
        Base::CELL_COUNT.into()
    }
    pub fn base_usize() -> usize {
        Base::BASE.into()
    }
    pub fn side_length_usize() -> usize {
        Base::SIDE_LENGTH.into()
    }
    pub fn max_value_usize() -> usize {
        Base::MAX_VALUE.into()
    }
}

// TODO: rewrite with ndarray slice
// TODO: zip position + cell
//  indexed_iter
//  https://docs.rs/ndarray/latest/ndarray/iter/struct.IndexedIter.html
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

    pub fn all_cells(&self) -> impl Iterator<Item = &Cell<Base>> {
        self.cells.iter()
    }

    pub fn row_cells(&self, row: u8) -> impl Iterator<Item = &Cell<Base>> {
        Self::assert_coordinate(row);

        self.cells.row(usize::from(row)).into_iter()
    }

    pub fn all_row_cells(&self) -> impl Iterator<Item = impl Iterator<Item = &Cell<Base>>> {
        self.cells.rows().into_iter().map(|row| row.into_iter())
    }

    pub fn column_cells(&self, column: u8) -> impl Iterator<Item = &Cell<Base>> {
        Self::assert_coordinate(column);

        self.cells.column(usize::from(column)).into_iter()
    }

    pub fn all_column_cells(&self) -> impl Iterator<Item = impl Iterator<Item = &Cell<Base>>> {
        self.cells
            .columns()
            .into_iter()
            .map(|column| column.into_iter())
    }

    pub fn block_cells(&self, pos: Position) -> impl Iterator<Item = &Cell<Base>> {
        self.positions_to_cells(Self::block_positions(pos))
    }

    // TODO: exact chunks
    pub fn all_block_cells(&self) -> impl Iterator<Item = impl Iterator<Item = &Cell<Base>>> {
        self.nested_positions_to_nested_cells(Self::all_block_positions())
    }

    pub fn all_group_cells(&self) -> impl Iterator<Item = impl Iterator<Item = &Cell<Base>>> {
        self.nested_positions_to_nested_cells(Self::all_group_positions())
    }
}

/// Position iterators
impl<Base: SudokuBase> Grid<Base> {
    pub fn all_positions() -> impl Iterator<Item = Position> {
        Self::all_row_positions().flatten()
    }

    pub fn row_positions(row: u8) -> impl Iterator<Item = Position> {
        Self::assert_coordinate(row);

        (0..Self::side_length()).map(move |column| Position { column, row })
    }

    pub fn all_row_positions() -> impl Iterator<Item = impl Iterator<Item = Position>> {
        (0..Self::side_length()).map(move |row_index| Self::row_positions(row_index))
    }

    pub fn column_positions(column: u8) -> impl Iterator<Item = Position> {
        Self::assert_coordinate(column);

        (0..Self::side_length()).map(move |row| Position { column, row })
    }

    pub fn all_column_positions() -> impl Iterator<Item = impl Iterator<Item = Position>> {
        (0..Self::side_length()).map(move |column| Self::column_positions(column))
    }

    pub fn block_positions(pos: Position) -> impl Iterator<Item = Position> {
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

    pub fn all_block_positions() -> impl Iterator<Item = impl Iterator<Item = Position>> {
        let all_block_base_pos = (0..Self::base())
            .flat_map(move |row| (0..Self::base()).map(move |column| Position { column, row }))
            .map(move |pos| pos * Self::base());

        all_block_base_pos.map(|block_base_pos| Self::block_positions(block_base_pos))
    }

    // TODO: optimize
    pub fn all_group_positions() -> impl Iterator<Item = impl Iterator<Item = Position>> {
        Self::all_row_positions()
            .map(|rows| rows.collect::<Vec<_>>().into_iter())
            .chain(
                Self::all_column_positions().map(|columns| columns.collect::<Vec<_>>().into_iter()),
            )
            .chain(Self::all_block_positions().map(|blocks| blocks.collect::<Vec<_>>().into_iter()))
    }
}

/// Filtered position vec
impl<Base: SudokuBase> Grid<Base> {
    pub fn all_value_positions(&self) -> Vec<Position> {
        Self::all_positions()
            .filter(|pos| self.get(*pos).has_value())
            .collect()
    }

    pub fn all_unfixed_value_positions(&self) -> Vec<Position> {
        Self::all_positions()
            .filter(|pos| self.get(*pos).has_unfixed_value())
            .collect()
    }

    pub fn all_candidates_positions(&self) -> Vec<Position> {
        Self::all_positions()
            .filter(|pos| self.get(*pos).has_candidates())
            .collect()
    }
}

/// Neighbor iterators
impl<Base: SudokuBase> Grid<Base> {
    fn neighbor_positions_with_duplicates(pos: Position) -> impl Iterator<Item = Position> {
        // TODO: reimplement without chain (VTune: bad speculation + unique version)
        Self::row_positions(pos.row)
            .chain(Self::column_positions(pos.column))
            .chain(Self::block_positions(pos))
    }

    #[allow(dead_code)]
    fn neighbor_positions(pos: Position) -> impl Iterator<Item = Position> {
        use itertools::Itertools;

        Self::neighbor_positions_with_duplicates(pos).unique()
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
            .map(|view| view.into().try_into())
            .collect::<Result<_>>()?;

        Ok(Self::with_cells(cells))
    }
}

impl<Base: SudokuBase> TryFrom<&str> for Grid<Base> {
    type Error = Error;

    fn try_from(input: &str) -> Result<Self> {
        parse_cells(input)?.try_into()
    }
}

impl<Base: SudokuBase> Display for Grid<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&GridFormat::CandidatesGrid.render(self))
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::*;
    use itertools::{assert_equal, Itertools};

    use crate::samples;

    use super::*;

    #[test]
    fn test_has_conflict() -> Result<()> {
        let mut grid = Grid::<U3>::new();
        assert!(!grid.has_value_conflict());

        grid.get_mut(Position { column: 0, row: 0 })
            .set_value(1.try_into()?);
        assert!(!grid.has_value_conflict());

        grid.get_mut(Position { column: 1, row: 0 })
            .set_value(1.try_into()?);
        assert!(grid.has_value_conflict());

        grid.get_mut(Position { column: 1, row: 0 }).delete();
        assert!(!grid.has_value_conflict());

        grid.get_mut(Position { column: 0, row: 1 })
            .set_value(1.try_into()?);
        assert!(grid.has_value_conflict());

        grid.get_mut(Position { column: 0, row: 1 }).delete();
        assert!(!grid.has_value_conflict());

        grid.get_mut(Position { column: 1, row: 1 })
            .set_value(1.try_into()?);
        assert!(grid.has_value_conflict());

        grid.get_mut(Position { column: 1, row: 1 }).delete();
        assert!(!grid.has_value_conflict());

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
                grid.update_direct_candidates(pos, 1.try_into()?);
                grid
            },
            { grid.clone() }
        );

        assert_eq!(
            {
                let mut grid = grid.clone();
                let pos = Position { column: 0, row: 3 };
                grid.update_direct_candidates(pos, 2.try_into()?);
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
                grid.update_direct_candidates(pos, 4.try_into()?);
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

    #[test]
    fn test_all_cells() {
        let grid = samples::base_2_candidates_coordinates();

        assert_equal(
            grid.all_cells(),
            vec![
                (0, 0),
                (0, 1),
                (0, 2),
                (0, 3),
                (1, 0),
                (1, 1),
                (1, 2),
                (1, 3),
                (2, 0),
                (2, 1),
                (2, 2),
                (2, 3),
                (3, 0),
                (3, 1),
                (3, 2),
                (3, 3),
            ]
            .into_iter()
            .map(|pos| grid.get(pos.into())),
        );
    }

    #[test]
    fn test_row_cells() {
        let grid = samples::base_2_candidates_coordinates();

        for row in 0..4 {
            assert_equal(
                grid.row_cells(row),
                vec![(row, 0), (row, 1), (row, 2), (row, 3)]
                    .into_iter()
                    .map(|pos| grid.get(pos.into())),
            );
        }
    }
    #[test]
    fn test_all_row_cells() {
        let grid = samples::base_2_candidates_coordinates();

        grid.all_row_cells()
            .zip_eq(vec![
                vec![(0, 0), (0, 1), (0, 2), (0, 3)],
                vec![(1, 0), (1, 1), (1, 2), (1, 3)],
                vec![(2, 0), (2, 1), (2, 2), (2, 3)],
                vec![(3, 0), (3, 1), (3, 2), (3, 3)],
            ])
            .for_each(|(actual_row, expected_row)| {
                assert_equal(
                    actual_row,
                    expected_row.into_iter().map(|pos| grid.get(pos.into())),
                )
            });
    }
    #[test]
    fn test_column_cells() {
        let grid = samples::base_2_candidates_coordinates();

        for column in 0..4 {
            assert_equal(
                grid.column_cells(column),
                vec![(0, column), (1, column), (2, column), (3, column)]
                    .into_iter()
                    .map(|pos| grid.get(pos.into())),
            );
        }
    }
    #[test]
    fn test_all_column_cells() {
        let grid = samples::base_2_candidates_coordinates();

        grid.all_column_cells()
            .zip_eq(vec![
                vec![(0, 0), (1, 0), (2, 0), (3, 0)],
                vec![(0, 1), (1, 1), (2, 1), (3, 1)],
                vec![(0, 2), (1, 2), (2, 2), (3, 2)],
                vec![(0, 3), (1, 3), (2, 3), (3, 3)],
            ])
            .for_each(|(actual_row, expected_row)| {
                assert_equal(
                    actual_row,
                    expected_row.into_iter().map(|pos| grid.get(pos.into())),
                )
            });
    }
    #[test]
    fn test_block_cells() {
        let grid = samples::base_2_candidates_coordinates();

        assert_equal(
            grid.block_cells((0, 0).into()),
            vec![(0, 0), (0, 1), (1, 0), (1, 1)]
                .into_iter()
                .map(|pos| grid.get(pos.into())),
        );

        assert_equal(
            grid.block_cells((0, 2).into()),
            vec![(0, 2), (0, 3), (1, 2), (1, 3)]
                .into_iter()
                .map(|pos| grid.get(pos.into())),
        );
        assert_equal(
            grid.block_cells((2, 0).into()),
            vec![(2, 0), (2, 1), (3, 0), (3, 1)]
                .into_iter()
                .map(|pos| grid.get(pos.into())),
        );
        assert_equal(
            grid.block_cells((2, 2).into()),
            vec![(2, 2), (2, 3), (3, 2), (3, 3)]
                .into_iter()
                .map(|pos| grid.get(pos.into())),
        );
    }
    #[test]
    fn test_all_block_cells() {
        let grid = samples::base_2_candidates_coordinates();

        grid.all_block_cells()
            .zip_eq(vec![
                vec![(0, 0), (0, 1), (1, 0), (1, 1)],
                vec![(0, 2), (0, 3), (1, 2), (1, 3)],
                vec![(2, 0), (2, 1), (3, 0), (3, 1)],
                vec![(2, 2), (2, 3), (3, 2), (3, 3)],
            ])
            .for_each(|(actual_row, expected_row)| {
                assert_equal(
                    actual_row,
                    expected_row.into_iter().map(|pos| grid.get(pos.into())),
                )
            });
    }
    #[test]
    fn test_all_group_cells() {
        let grid = samples::base_2_candidates_coordinates();

        grid.all_group_cells()
            .zip_eq(vec![
                // all_rows
                vec![(0, 0), (0, 1), (0, 2), (0, 3)],
                vec![(1, 0), (1, 1), (1, 2), (1, 3)],
                vec![(2, 0), (2, 1), (2, 2), (2, 3)],
                vec![(3, 0), (3, 1), (3, 2), (3, 3)],
                // all_columns
                vec![(0, 0), (1, 0), (2, 0), (3, 0)],
                vec![(0, 1), (1, 1), (2, 1), (3, 1)],
                vec![(0, 2), (1, 2), (2, 2), (3, 2)],
                vec![(0, 3), (1, 3), (2, 3), (3, 3)],
                // all_blocks
                vec![(0, 0), (0, 1), (1, 0), (1, 1)],
                vec![(0, 2), (0, 3), (1, 2), (1, 3)],
                vec![(2, 0), (2, 1), (3, 0), (3, 1)],
                vec![(2, 2), (2, 3), (3, 2), (3, 3)],
            ])
            .for_each(|(actual_row, expected_row)| {
                assert_equal(
                    actual_row,
                    expected_row.into_iter().map(|pos| grid.get(pos.into())),
                )
            });
    }

    #[test]
    fn test_has_duplicate_value() {
        let cells_with_no_duplicate_value = vec![
            CellView::Value {
                value: 1,
                fixed: false,
            }
            .try_into()
            .unwrap(),
            CellView::Candidates {
                candidates: vec![1, 2, 3],
            }
            .try_into()
            .unwrap(),
            CellView::Candidates {
                candidates: vec![1, 2, 3],
            }
            .try_into()
            .unwrap(),
            CellView::Value {
                value: 2,
                fixed: false,
            }
            .try_into()
            .unwrap(),
        ];

        assert!(!Grid::<U2>::has_duplicate_value(
            cells_with_no_duplicate_value.iter()
        ));
        let cells_with_duplicate_value = vec![
            CellView::Value {
                value: 1,
                fixed: false,
            }
            .try_into()
            .unwrap(),
            CellView::Candidates {
                candidates: vec![1, 2, 3],
            }
            .try_into()
            .unwrap(),
            CellView::Candidates {
                candidates: vec![1, 2, 3],
            }
            .try_into()
            .unwrap(),
            CellView::Value {
                value: 1,
                fixed: false,
            }
            .try_into()
            .unwrap(),
        ];

        assert!(Grid::<U2>::has_duplicate_value(
            cells_with_duplicate_value.iter()
        ));
    }
}
