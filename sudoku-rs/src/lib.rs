use std::collections::HashSet;
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
#[cfg(any(test, feature = "benchmark"))]
pub mod samples;
pub mod solver;
pub mod transport;

// TODO: deref(mut) to grid
//  check public API
//   can invariants be broken? (cell max_value)
//  grid seems to be a leaky abstraction if multiple wrapper methods are needed

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug)]
pub struct Sudoku<Cell: SudokuCell> {
    grid: Grid<Cell>,
    settings: Settings,
}

// TODO: add public settings API
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug)]
struct Settings {
    update_candidates_on_set_value: bool,
}

// TODO: make sudoku fully generic over cell
//  Sudoku<Cell<u8>>
//  Alternative: sudoku decides cell type based on base at runtime

// TODO: provide undo/redo API
// TODO: return result in all asserts
//  sudoku::Error as JSValue (JS Exception)?
impl<Cell: SudokuCell> Sudoku<Cell> {
    pub fn new(base: usize) -> Self {
        Self::new_with_grid(Grid::new(base))
    }

    fn new_with_grid(grid: Grid<Cell>) -> Self {
        Sudoku {
            grid,
            settings: Settings {
                update_candidates_on_set_value: true,
            },
        }
    }

    pub fn set_value(&mut self, pos: Position, value: usize) {
        let max_value = self.grid.max_value();

        self.grid.get_pos_mut(pos).set_value(value, max_value);
    }

    pub fn set_or_toggle_value(&mut self, pos: Position, value: usize) {
        let max_value = self.grid.max_value();

        let set_value = self
            .grid
            .get_pos_mut(pos)
            .set_or_toggle_value(value, max_value);

        if self.settings.update_candidates_on_set_value && set_value {
            self.update_candidates(pos, value);
        }
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

    pub fn delete(&mut self, pos: Position) -> Cell {
        let max_value = self.grid.max_value();

        self.grid.get_pos_mut(pos).delete(max_value)
    }

    pub fn set_all_direct_candidates(&mut self) {
        self.grid()
            .all_empty_positions()
            .into_iter()
            .for_each(|pos| {
                let candidates = self.direct_candidates(pos);

                self.set_candidates(pos, candidates);
            });
    }

    pub fn fix_all_values(&mut self) {
        self.grid.fix_all_values()
    }

    pub fn unfix(&mut self) {
        self.grid.unfix()
    }

    pub fn is_fixed(&self, pos: Position) -> bool {
        self.grid.is_fixed(pos)
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

    pub fn grid(&self) -> &Grid<Cell> {
        &self.grid
    }
}

impl<Cell: SudokuCell> Sudoku<Cell> {
    fn update_candidates(&mut self, pos: Position, value: usize) {
        let max = self.grid.max_value();

        self.grid
            .neighbor_positions_with_duplicates(pos)
            .for_each(|pos| {
                if self.grid.get_pos(pos).candidates().is_some() {
                    let cell = self.grid.get_pos_mut(pos);

                    cell.delete_candidate(value, max);
                }
            });
    }

    pub fn direct_candidates(&self, pos: Position) -> Vec<usize> {
        let conflicting_values: FixedBitSet = self
            .grid
            .neighbor_positions_with_duplicates(pos)
            .filter_map(|pos| self.get(pos).value())
            .collect();

        let values: FixedBitSet = self.grid.value_range().collect();

        values.difference(&conflicting_values).collect()
    }

    #[allow(dead_code)]
    pub(crate) fn has_conflict(&self) -> bool {
        self.grid.all_row_cells().any(|row| self.has_duplicate(row))
            || self
                .grid
                .all_column_cells()
                .any(|column| self.has_duplicate(column))
            || self
                .grid
                .all_block_cells()
                .any(|block| self.has_duplicate(block))
    }

    // TODO: optimize: is value in group?
    pub fn has_conflict_at(&self, pos: Position) -> bool {
        self.has_duplicate(self.grid.row_cells(pos.row))
            || self.has_duplicate(self.grid.column_cells(pos.column))
            || self.has_duplicate(self.grid.block_cells(pos))
    }

    // TODO: conflict location pairs
    pub fn has_duplicate<'a>(&'a self, cells: impl Iterator<Item = &'a Cell>) -> bool {
        let mut uniq = HashSet::new();

        cells
            .filter_map(|cell| cell.value())
            .any(move |x| !uniq.insert(x))
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
        Ok(Self::new_with_grid(values.try_into()?))
    }
}

impl<Cell: SudokuCell> Display for Sudoku<Cell> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.grid)
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

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

    #[test]
    fn test_direct_candidates() {
        let sudoku = samples::base_3().pop().unwrap();

        let direct_candidates = sudoku.direct_candidates(Position { column: 1, row: 1 });

        assert_eq!(direct_candidates, vec![1, 2, 4]);
    }

    #[test]
    fn test_set_or_toggle_value() {
        let mut sudoku: Sudoku<Cell<NonZeroUsize>> = samples::base_2().first().unwrap().clone();

        sudoku.set_all_direct_candidates();

        let sudoku = sudoku;

        assert_eq!(
            {
                let mut sudoku = sudoku.clone();
                sudoku.set_or_toggle_value(Position { column: 0, row: 3 }, 1);
                sudoku.set_candidates(Position { column: 0, row: 3 }, vec![3]);
                sudoku
            },
            { sudoku.clone() }
        );

        assert_eq!(
            {
                let mut sudoku = sudoku.clone();
                sudoku.set_or_toggle_value(Position { column: 0, row: 3 }, 2);
                sudoku.set_candidates(Position { column: 0, row: 3 }, vec![3]);
                sudoku
            },
            {
                let mut sudoku = sudoku.clone();
                sudoku.delete(Position { column: 0, row: 0 });
                sudoku
            }
        );
        assert_eq!(
            {
                let mut sudoku = sudoku.clone();
                sudoku.set_or_toggle_value(Position { column: 0, row: 3 }, 4);
                sudoku.set_candidates(Position { column: 0, row: 3 }, vec![3]);
                sudoku
            },
            {
                let mut sudoku = sudoku.clone();
                sudoku.delete(Position { column: 1, row: 2 });
                sudoku.delete(Position { column: 3, row: 3 });
                sudoku
            }
        );
    }
}
