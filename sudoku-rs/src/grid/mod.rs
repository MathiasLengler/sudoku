use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::str::FromStr;

use anyhow::ensure;
use ndarray::{Array2, ArrayView2, ArrayViewMut2};

use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicCell;
use crate::cell::Candidates;
use crate::cell::Cell;
use crate::cell::CellState;
use crate::cell::Value;
use crate::error::{Error, Result};
use crate::grid::format::{CandidatesGridANSIStyled, DynamicGridFormat, GridFormat};
use crate::position::Coordinate;
use crate::position::Position;
use crate::solver::strategic::strategies::DynamicStrategy;
use crate::solver::{backtracking, introspective, strategic, FallibleSolver};
use crate::unsafe_utils::{get_unchecked, get_unchecked_mut};

pub mod deserialization;
pub mod format;

pub mod dynamic;

/// A square grid of cells with side length `Base::SIDE_LENGTH`.
///
/// By default, the cell type `T` is `Cell<Base>`.
/// Other cell types are supported, but with less functionality.
#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Grid<Base: SudokuBase, T = Cell<Base>> {
    /// The cells of this grid.
    ///
    /// # Safety invariants
    /// - `cells.len() == Base::CELL_COUNT`
    /// - `cells.is_standard_layout()`
    cells: Array2<T>,
    _base: PhantomData<Base>,
}

impl<Base: SudokuBase, T> AsRef<Grid<Base, T>> for Grid<Base, T> {
    fn as_ref(&self) -> &Grid<Base, T> {
        self
    }
}

impl<Base: SudokuBase, T> AsMut<Grid<Base, T>> for Grid<Base, T> {
    fn as_mut(&mut self) -> &mut Grid<Base, T> {
        self
    }
}

impl<Base: SudokuBase, T> Index<Position<Base>> for Grid<Base, T> {
    type Output = T;

    fn index(&self, pos: Position<Base>) -> &Self::Output {
        self.get(pos)
    }
}

impl<Base: SudokuBase, T> IndexMut<Position<Base>> for Grid<Base, T> {
    fn index_mut(&mut self, pos: Position<Base>) -> &mut Self::Output {
        self.get_mut(pos)
    }
}

/// Indexing
impl<Base: SudokuBase, T> Grid<Base, T> {
    // TODO: evaluate `cells: Box<[Cell<Base>; Base::CELL_COUNT]>`
    //  requires new associated type in `Base`, but could reduce the amount of unsafe code for slice conversions.
    fn cells_slice(&self) -> &[T] {
        self.debug_assert();

        // Safety: this is a unsafe fork of ndarray `ArrayBase::as_slice()`.
        // The `is_standard_layout` check is removed, since `Grid` guarantees that this is always the case.
        // The length is inlined, since `Grid` guarantees it.
        unsafe { std::slice::from_raw_parts(self.cells.as_ptr(), usize::from(Base::CELL_COUNT)) }
    }
    fn cells_slice_mut(&mut self) -> &mut [T] {
        self.debug_assert();

        // Safety: this is a unsafe fork of ndarray `ArrayBase::as_slice_mut()`.
        // The `is_standard_layout` check is removed, since `Grid` guarantees that this is always the case.
        // The call to `ensure_unique` is removed, since cells contain a `OwnedRepr`, where this call is a noop.
        // The length is inlined, since `Grid` guarantees it.
        unsafe {
            std::slice::from_raw_parts_mut(self.cells.as_mut_ptr(), usize::from(Base::CELL_COUNT))
        }
    }
}

/// Validation
impl<Base: SudokuBase, T> Grid<Base, T> {
    fn validate_cells(cells: &Array2<T>) -> Result<()> {
        ensure!(cells.len() == usize::from(Base::CELL_COUNT));
        ensure!(cells.is_standard_layout());

        Ok(())
    }

    fn validate(&self) -> Result<()> {
        Self::validate_cells(&self.cells)
    }

    fn assert(&self) {
        self.validate().unwrap();
    }

    fn debug_assert(&self) {
        debug_assert!({
            self.assert();
            true
        });
    }

    fn validate_vec_groups<U>(groups: &'_ [Vec<U>]) -> Result<()> {
        let side_length = usize::from(Base::SIDE_LENGTH);

        ensure!(
            groups.len() == side_length,
            "Invalid number of groups, expected {side_length}, instead got: {}",
            groups.len()
        );

        for (i, group) in groups.iter().enumerate() {
            ensure!(
                group.len() == side_length,
                "Invalid group size for group {i}, expected {side_length}, instead got: {}",
                group.len()
            );
        }

        Ok(())
    }
}

/// internal ndarray views for cells
impl<Base: SudokuBase, T> Grid<Base, T> {
    pub(crate) fn cells_view(&self) -> ArrayView2<T> {
        self.cells.view()
    }

    #[allow(dead_code)]
    pub(crate) fn cells_view_mut(&mut self) -> ArrayViewMut2<T> {
        self.cells.view_mut()
    }
}

/// Direct candidates
///
/// All candidates of a position which can't be removed by a group-adjacent value.
impl<Base: SudokuBase> Grid<Base> {
    /// Replace all candidates cells with the direct candidates for its position.
    pub fn set_all_direct_candidates(&mut self) {
        self.all_candidates_positions().into_iter().for_each(|pos| {
            let direct_candidates = self.direct_candidates(pos);

            self.get_mut(pos).set_candidates(direct_candidates);
        });
    }

    /// Update all candidates cells by removing group-adjacent values from the existing candidates.
    pub fn update_all_direct_candidates(&mut self) {
        self.all_candidates_positions().into_iter().for_each(|pos| {
            let existing_candidates = self.get(pos).candidates().unwrap();
            let direct_candidates = self.direct_candidates(pos);

            self.get_mut(pos)
                .set_candidates(existing_candidates.intersection(direct_candidates));
        });
    }

    pub fn set_all_direct_candidates_if_all_candidates_are_empty(&mut self) {
        let all_candidates_are_empty = self
            .all_candidates_positions()
            .into_iter()
            .all(|pos| self.get(pos).candidates().unwrap().is_empty());

        if all_candidates_are_empty {
            self.set_all_direct_candidates();
        }
    }

    pub fn update_direct_candidates_for_new_value(
        &mut self,
        pos: Position<Base>,
        value: Value<Base>,
    ) {
        Self::neighbor_positions_with_duplicates(pos).for_each(|pos| {
            let cell = self.get_mut(pos);
            if cell.has_candidates() {
                cell.delete_candidate(value);
            }
        });
    }

    pub fn direct_candidates(&self, pos: Position<Base>) -> Candidates<Base> {
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
    // Alternative: compare with solved grid

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

    pub fn validate_directly_consistent(&self) -> Result<()> {
        // Every candidate is directly consistent at its position
        for candidates_pos in self.all_candidates_positions() {
            ensure!(
                self.is_directly_consistent_at(candidates_pos),
                "Inconsistent candidates position {candidates_pos}"
            );
        }

        // Every group is directly consistent
        for (group_i, group) in (0..).zip(self.all_group_cells()) {
            ensure!(
                Self::is_group_directly_consistent(group),
                "Inconsistent group {} {}",
                match group_i / Base::SIDE_LENGTH {
                    0 => "row",
                    1 => "column",
                    2 => "block",
                    _ => "??",
                },
                group_i % Base::SIDE_LENGTH
            );
        }

        Ok(())
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
                    seen_candidates_or_values.set(value, true);
                }
                CellState::Candidates(candidates) => {
                    seen_candidates_or_values = seen_candidates_or_values.union(candidates);
                }
            }
        }

        // Every candidate must be contained in group.
        seen_candidates_or_values.is_full()
    }

    /// A cell with candidates is directly consistent, if its candidates:
    /// - are non-empty.
    /// - contain no candidate which is deletable based on a group-adjacent value.
    fn is_directly_consistent_at(&self, pos: Position<Base>) -> bool {
        let cell = self.get(pos);
        assert!(cell.has_candidates());
        let actual_candidates = cell.candidates().unwrap();
        // At least one candidate is required for a consistent grid.
        if actual_candidates.is_empty() {
            return false;
        }

        let direct_candidates = self.direct_candidates(pos);
        // No actual candidate is deletable via direct candidates.
        actual_candidates.without(direct_candidates).is_empty()
    }
}

/// Public Sudoku API
impl<Base: SudokuBase> Grid<Base> {
    // TODO: evaluate all_group_cells
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
                seen_values.insert(value);
                false
            }
        })
    }

    pub fn is_solved(&self) -> bool {
        self.all_candidates_positions().is_empty() && !self.has_value_conflict()
    }

    pub fn is_minimal(&self) -> bool {
        let mut grid = self.clone();

        grid.unfix_all_values();

        grid.has_unique_solution()
            && grid.all_value_positions().into_iter().all(|pos| {
                let cell = grid.get_mut(pos);
                let prev_value = cell.value().unwrap();
                cell.delete();
                let has_multiple_solutions = !grid.has_unique_solution();
                grid.get_mut(pos).set_value(prev_value);
                has_multiple_solutions
            })
    }

    pub fn has_unique_solution(&self) -> bool {
        self.unique_solution().is_some()
    }

    pub fn unique_solution(&self) -> Option<Self> {
        // FIXME: remove clone
        let mut solver = introspective::Solver::new(self.clone());

        match (solver.next(), solver.next()) {
            (Some(unique_solution), None) => Some(unique_solution),
            _ => None,
        }
    }

    pub fn unique_solution_for_fixed_values(&self) -> Option<Self> {
        let mut cloned_grid = self.clone();
        cloned_grid.delete_all_unfixed_values();

        cloned_grid.unique_solution()
    }

    pub fn solution_count(&self) -> usize {
        let solver = backtracking::Solver::new(self);
        solver.count()
    }

    pub fn is_solvable_with_strategies(
        &self,
        strategies: Vec<DynamicStrategy>,
    ) -> Result<Option<Self>> {
        let mut clone = self.clone();
        clone.fix_all_values();
        clone.set_all_direct_candidates();
        let mut solver = strategic::Solver::new_with_strategies(&mut clone, strategies);

        solver.try_solve()
    }
}

impl<Base: SudokuBase, T: Default + Clone> Default for Grid<Base, T> {
    fn default() -> Self {
        Grid::new()
    }
}

impl<Base: SudokuBase, T: Default + Clone> Grid<Base, T> {
    pub fn new() -> Self {
        Self::with(vec![T::default(); Base::CELL_COUNT.into()]).unwrap()
    }
}

/// Public Grid API
impl<Base: SudokuBase, T> Grid<Base, T> {
    pub fn with(cells: Vec<T>) -> Result<Self> {
        let cell_count = usize::from(Base::CELL_COUNT);

        ensure!(
            cells.len() == cell_count,
            "Invalid number of cells, expected {cell_count}, instead got: {}",
            cells.len()
        );

        let side_length = usize::from(Base::SIDE_LENGTH);

        // This is the only direct instantiation of Grid.
        let grid = Self {
            cells: Array2::from_shape_vec((side_length, side_length), cells)?,
            _base: PhantomData,
        };
        // Check for safety invariants in debug builds.
        grid.debug_assert();
        Ok(grid)
    }
}

impl<Base: SudokuBase> Grid<Base> {
    pub fn try_from_blocks(blocks: Vec<Vec<DynamicCell>>) -> Result<Self> {
        Self::validate_vec_groups(&blocks)?;

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
}
impl<Base: SudokuBase, T> Grid<Base, T> {
    pub fn get(&self, pos: Position<Base>) -> &T {
        // Debug validation
        pos.debug_assert();
        self.debug_assert();

        let cells_slice = self.cells_slice();

        let cell_index = pos.cell_index() as usize;

        // Safety:
        // - `cell_index < Base::CELL_COUNT` is guaranteed by `Position`
        // - `cells.len() == Base::CELL_COUNT` is guaranteed by `Grid`
        let cell = unsafe { get_unchecked(cells_slice, cell_index) };

        cell
    }

    pub fn get_mut(&mut self, pos: Position<Base>) -> &mut T {
        // Debug validation
        pos.debug_assert();
        self.debug_assert();

        let cells_slice = self.cells_slice_mut();

        let cell_index = pos.cell_index() as usize;

        // Safety:
        // - `cell_index < Base::CELL_COUNT` is guaranteed by `Position`
        // - `cells.len() == Base::CELL_COUNT` is guaranteed by `Grid`
        let cell = unsafe { get_unchecked_mut(cells_slice, cell_index) };

        cell
    }
}

impl<Base: SudokuBase> Grid<Base> {
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
}

/// Cell iterators
///
/// TODO: unify/expand iterator API:
///  Use-cases:
///  Iterator Item:
///  - Position
///  - &Cell
///  - &mut Cell
///  - Positioned<&Cell>
///  - Positioned<&mut Cell>
///  What is iterated:
///  - Position iterators
///    - all
///    - row i
///    - all rows
///    - column i
///    - all columns
///    - block i
///    - all blocks
///  - all groups (chained: all rows, all columns, all blocks)
///  - filtered cell state
///    - `all_value_positions`
///    - `all_unfixed_value_positions`
///    - `all_candidates_positions`
impl<Base: SudokuBase, T> Grid<Base, T> {
    fn positions_to_cells(
        &self,
        positions: impl Iterator<Item = Position<Base>>,
    ) -> impl Iterator<Item = &T> {
        positions.map(move |pos| self.get(pos))
    }

    fn nested_positions_to_nested_cells(
        &self,
        nested_positions: impl Iterator<Item = impl Iterator<Item = Position<Base>>>,
    ) -> impl Iterator<Item = impl Iterator<Item = &T>> {
        nested_positions.map(move |row_pos| row_pos.map(move |pos| self.get(pos)))
    }

    pub fn all_cells(&self) -> impl Iterator<Item = &T> {
        self.cells.iter()
    }

    pub fn row_cells(&self, row: Coordinate<Base>) -> impl Iterator<Item = &T> {
        self.cells.row(usize::from(row.get())).into_iter()
    }

    pub fn all_row_cells(&self) -> impl Iterator<Item = impl Iterator<Item = &T>> {
        self.cells.rows().into_iter().map(|row| row.into_iter())
    }

    pub fn column_cells(&self, column: Coordinate<Base>) -> impl Iterator<Item = &T> {
        self.cells.column(usize::from(column.get())).into_iter()
    }

    pub fn all_column_cells(&self) -> impl Iterator<Item = impl Iterator<Item = &T>> {
        self.cells
            .columns()
            .into_iter()
            .map(|column| column.into_iter())
    }

    pub fn block_cells(&self, block: Coordinate<Base>) -> impl Iterator<Item = &T> {
        self.positions_to_cells(Self::block_positions(block))
    }

    // TODO: exact chunks
    pub fn all_block_cells(&self) -> impl Iterator<Item = impl Iterator<Item = &T>> {
        self.nested_positions_to_nested_cells(Self::all_block_positions())
    }

    pub fn all_group_cells(&self) -> impl Iterator<Item = impl Iterator<Item = &T>> {
        self.nested_positions_to_nested_cells(Self::all_group_positions())
    }
}

/// Position iterators
impl<Base: SudokuBase, T> Grid<Base, T> {
    pub fn all_positions() -> impl Iterator<Item = Position<Base>> {
        Position::all()
    }

    pub fn row_positions(row: Coordinate<Base>) -> impl Iterator<Item = Position<Base>> {
        Position::row(row)
    }

    pub fn all_row_positions() -> impl Iterator<Item = impl Iterator<Item = Position<Base>>> {
        Position::all_rows()
    }

    pub fn column_positions(column: Coordinate<Base>) -> impl Iterator<Item = Position<Base>> {
        Position::column(column)
    }

    pub fn all_column_positions() -> impl Iterator<Item = impl Iterator<Item = Position<Base>>> {
        Position::all_columns()
    }

    pub fn block_positions(block: Coordinate<Base>) -> impl Iterator<Item = Position<Base>> {
        Position::block(block)
    }

    pub fn all_block_positions() -> impl Iterator<Item = impl Iterator<Item = Position<Base>>> {
        Position::all_blocks()
    }

    pub fn all_group_positions() -> impl Iterator<Item = impl Iterator<Item = Position<Base>>> {
        Position::all_groups()
    }
}

// TODO: return Vec<Positioned<Value | Candidates>>
/// Filtered position vec
impl<Base: SudokuBase> Grid<Base> {
    pub fn all_value_positions(&self) -> Vec<Position<Base>> {
        Self::all_positions()
            .filter(|pos| self.get(*pos).has_value())
            .collect()
    }

    pub fn all_unfixed_value_positions(&self) -> Vec<Position<Base>> {
        Self::all_positions()
            .filter(|pos| self.get(*pos).has_unfixed_value())
            .collect()
    }

    pub fn all_fixed_value_positions(&self) -> Vec<Position<Base>> {
        Self::all_positions()
            .filter(|pos| self.get(*pos).has_fixed_value())
            .collect()
    }

    pub fn all_candidates_positions(&self) -> Vec<Position<Base>> {
        Self::all_positions()
            .filter(|pos| self.get(*pos).has_candidates())
            .collect()
    }
}

/// Neighbor iterators
impl<Base: SudokuBase, T> Grid<Base, T> {
    // TODO: test
    fn neighbor_positions_with_duplicates(
        pos: Position<Base>,
    ) -> impl Iterator<Item = Position<Base>> {
        // TODO: reimplement without chain (VTune: bad speculation + unique version)
        Self::row_positions(pos.to_row())
            .chain(Self::column_positions(pos.to_column()))
            .chain(Self::block_positions(pos.to_block()))
    }

    #[allow(dead_code)]
    fn neighbor_positions(pos: Position<Base>) -> impl Iterator<Item = Position<Base>> {
        use itertools::Itertools;

        Self::neighbor_positions_with_duplicates(pos).unique()
    }
}

/// Convert nested rows into Grid<Base>
impl<Base: SudokuBase, IntoCell: Into<DynamicCell>> TryFrom<Vec<Vec<IntoCell>>> for Grid<Base> {
    type Error = Error;

    fn try_from(nested_views: Vec<Vec<IntoCell>>) -> Result<Self> {
        Self::validate_vec_groups(&nested_views)?;

        nested_views
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .try_into()
    }
}

/// Convert cells in row-major order into Grid<Base>
impl<Base: SudokuBase, IntoCell: Into<DynamicCell>> TryFrom<Vec<IntoCell>> for Grid<Base> {
    type Error = Error;

    fn try_from(views: Vec<IntoCell>) -> Result<Self> {
        let cells = views
            .into_iter()
            .map(|view| view.into().try_into())
            .collect::<Result<_>>()?;

        Self::with(cells)
    }
}

impl<Base: SudokuBase, T: Clone> TryFrom<ArrayView2<'_, T>> for Grid<Base, T> {
    type Error = Error;

    fn try_from(cells_array_view: ArrayView2<T>) -> Result<Self> {
        // TODO: assert square + side_length

        let cells_vec: Vec<_> = cells_array_view.iter().cloned().collect();

        Self::with(cells_vec)
    }
}

impl<Base: SudokuBase> FromStr for Grid<Base> {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        DynamicGridFormat::detect_and_parse(input)?.try_into()
    }
}

impl<Base: SudokuBase> Display for Grid<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&CandidatesGridANSIStyled.render(self))
    }
}

#[cfg(test)]
mod tests {
    use itertools::{assert_equal, Itertools};

    use crate::base::consts::*;
    use crate::position::DynamicPosition;
    use crate::samples;

    use super::*;

    #[test]
    fn test_has_conflict() -> Result<()> {
        let mut grid = Grid::<Base3>::new();
        assert!(!grid.has_value_conflict());

        grid.get_mut((0, 0).try_into().unwrap())
            .set_value(1.try_into()?);
        assert!(!grid.has_value_conflict());

        grid.get_mut((0, 1).try_into().unwrap())
            .set_value(1.try_into()?);
        assert!(grid.has_value_conflict());

        grid.get_mut((0, 1).try_into().unwrap()).delete();
        assert!(!grid.has_value_conflict());

        grid.get_mut((1, 0).try_into().unwrap())
            .set_value(1.try_into()?);
        assert!(grid.has_value_conflict());

        grid.get_mut((1, 0).try_into().unwrap()).delete();
        assert!(!grid.has_value_conflict());

        grid.get_mut((1, 1).try_into().unwrap())
            .set_value(1.try_into()?);
        assert!(grid.has_value_conflict());

        grid.get_mut((1, 1).try_into().unwrap()).delete();
        assert!(!grid.has_value_conflict());

        Ok(())
    }

    #[test]
    fn test_direct_candidates() -> Result<()> {
        let grid = samples::base_3().pop().unwrap();

        let direct_candidates = grid.direct_candidates((1, 1).try_into().unwrap());

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
                let pos = (3, 0).try_into().unwrap();
                grid.update_direct_candidates_for_new_value(pos, 1.try_into()?);
                grid
            },
            { grid.clone() }
        );

        assert_eq!(
            {
                let mut grid = grid.clone();
                let pos = (3, 0).try_into().unwrap();
                grid.update_direct_candidates_for_new_value(pos, 2.try_into()?);
                grid
            },
            {
                let mut grid = grid.clone();
                grid.get_mut((0, 0).try_into().unwrap()).delete();
                grid
            }
        );
        assert_eq!(
            {
                let mut grid = grid.clone();
                let pos = (3, 0).try_into().unwrap();
                grid.update_direct_candidates_for_new_value(pos, 4.try_into()?);
                grid
            },
            {
                let mut grid = grid.clone();
                grid.get_mut((2, 1).try_into().unwrap()).delete();
                grid.get_mut((3, 3).try_into().unwrap()).delete();
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
            .map(|pos| grid.get(pos.try_into().unwrap())),
        );
    }

    #[test]
    fn test_row_cells() {
        let grid = samples::base_2_candidates_coordinates();

        for row in 0..4 {
            assert_equal(
                grid.row_cells(row.try_into().unwrap()),
                vec![(row, 0), (row, 1), (row, 2), (row, 3)]
                    .into_iter()
                    .map(|pos| grid.get(pos.try_into().unwrap())),
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
                    expected_row
                        .into_iter()
                        .map(|pos| grid.get(pos.try_into().unwrap())),
                );
            });
    }
    #[test]
    fn test_column_cells() {
        let grid = samples::base_2_candidates_coordinates();

        for column in 0..4 {
            assert_equal(
                grid.column_cells(column.try_into().unwrap()),
                vec![(0, column), (1, column), (2, column), (3, column)]
                    .into_iter()
                    .map(|pos| grid.get(pos.try_into().unwrap())),
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
                    expected_row
                        .into_iter()
                        .map(|pos| grid.get(pos.try_into().unwrap())),
                );
            });
    }
    #[test]
    fn test_block_cells() {
        let grid = samples::base_2_candidates_coordinates();

        assert_equal(
            grid.block_cells(0.try_into().unwrap()),
            vec![(0, 0), (0, 1), (1, 0), (1, 1)]
                .into_iter()
                .map(|pos| grid.get(pos.try_into().unwrap())),
        );

        assert_equal(
            grid.block_cells(1.try_into().unwrap()),
            vec![(0, 2), (0, 3), (1, 2), (1, 3)]
                .into_iter()
                .map(|pos| grid.get(pos.try_into().unwrap())),
        );
        assert_equal(
            grid.block_cells(2.try_into().unwrap()),
            vec![(2, 0), (2, 1), (3, 0), (3, 1)]
                .into_iter()
                .map(|pos| grid.get(pos.try_into().unwrap())),
        );
        assert_equal(
            grid.block_cells(3.try_into().unwrap()),
            vec![(2, 2), (2, 3), (3, 2), (3, 3)]
                .into_iter()
                .map(|pos| grid.get(pos.try_into().unwrap())),
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
                    expected_row
                        .into_iter()
                        .map(|pos| grid.get(pos.try_into().unwrap())),
                );
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
                    expected_row
                        .into_iter()
                        .map(|pos| grid.get(pos.try_into().unwrap())),
                );
            });
    }

    #[test]
    fn test_has_duplicate_value() {
        let cells_with_no_duplicate_value = vec![
            DynamicCell::Value {
                value: 1.into(),
                fixed: false,
            }
            .try_into()
            .unwrap(),
            DynamicCell::Candidates {
                candidates: vec![1, 2, 3].into(),
            }
            .try_into()
            .unwrap(),
            DynamicCell::Candidates {
                candidates: vec![1, 2, 3].into(),
            }
            .try_into()
            .unwrap(),
            DynamicCell::Value {
                value: 2.into(),
                fixed: false,
            }
            .try_into()
            .unwrap(),
        ];

        assert!(!Grid::<Base2>::has_duplicate_value(
            cells_with_no_duplicate_value.iter()
        ));
        let cells_with_duplicate_value = vec![
            DynamicCell::Value {
                value: 1.into(),
                fixed: false,
            }
            .try_into()
            .unwrap(),
            DynamicCell::Candidates {
                candidates: vec![1, 2, 3].into(),
            }
            .try_into()
            .unwrap(),
            DynamicCell::Candidates {
                candidates: vec![1, 2, 3].into(),
            }
            .try_into()
            .unwrap(),
            DynamicCell::Value {
                value: 1.into(),
                fixed: false,
            }
            .try_into()
            .unwrap(),
        ];

        assert!(Grid::<Base2>::has_duplicate_value(
            cells_with_duplicate_value.iter()
        ));
    }

    #[test]
    fn test_unique_solution_for_fixed_values() {
        let mut grid = samples::base_2().into_iter().next().unwrap();

        grid.fix_all_values();

        assert!(grid.unique_solution_for_fixed_values().is_some());

        // Invalid unfixed value
        grid.get_mut((0, 0).try_into().unwrap())
            .set_value(1.try_into().unwrap());
        assert!(grid.unique_solution_for_fixed_values().is_some());

        // Invalid fixed value
        grid.get_mut((0, 0).try_into().unwrap()).fix();
        assert!(grid.unique_solution_for_fixed_values().is_none());
    }

    #[test]
    fn test_index() {
        let grid = samples::base_2_candidates_coordinates();

        let expected_cell = Cell::with_candidates(Candidates::new());
        assert_eq!(grid[Position::new(0).unwrap()], expected_cell);
        assert_eq!(grid[(0, 0).try_into().unwrap()], expected_cell);
        assert_eq!(
            grid[DynamicPosition { row: 0, column: 0 }.try_into().unwrap()],
            expected_cell
        );
        let expected_cell = Cell::with_candidates(Candidates::with_single(1.try_into().unwrap()));
        assert_eq!(grid[Position::new(1).unwrap()], expected_cell);
        assert_eq!(grid[(0, 1).try_into().unwrap()], expected_cell);
        assert_eq!(
            grid[DynamicPosition { row: 0, column: 1 }.try_into().unwrap()],
            expected_cell
        );
        let expected_cell = Cell::with_candidates(Candidates::all());
        assert_eq!(grid[Position::new(15).unwrap()], expected_cell);
        assert_eq!(grid[(3, 3).try_into().unwrap()], expected_cell);
        assert_eq!(
            grid[DynamicPosition { row: 3, column: 3 }.try_into().unwrap()],
            expected_cell
        );
    }

    #[test]
    fn test_index_mut() {
        let mut grid = samples::base_2_candidates_coordinates();

        let expected_cell = Cell::with_candidates(Candidates::new());
        assert_eq!(*grid.index_mut(Position::new(0).unwrap()), expected_cell);
        assert_eq!(*grid.index_mut((0, 0).try_into().unwrap()), expected_cell);
        assert_eq!(
            *grid.index_mut(DynamicPosition { row: 0, column: 0 }.try_into().unwrap()),
            expected_cell
        );
        let expected_cell = Cell::with_candidates(Candidates::with_single(1.try_into().unwrap()));
        assert_eq!(*grid.index_mut(Position::new(1).unwrap()), expected_cell);
        assert_eq!(*grid.index_mut((0, 1).try_into().unwrap()), expected_cell);
        assert_eq!(
            *grid.index_mut(DynamicPosition { row: 0, column: 1 }.try_into().unwrap()),
            expected_cell
        );
        let expected_cell = Cell::with_candidates(Candidates::all());
        assert_eq!(*grid.index_mut(Position::new(15).unwrap()), expected_cell);
        assert_eq!(*grid.index_mut((3, 3).try_into().unwrap()), expected_cell);
        assert_eq!(
            *grid.index_mut(DynamicPosition { row: 3, column: 3 }.try_into().unwrap()),
            expected_cell
        );
    }
}
