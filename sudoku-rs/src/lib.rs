#![warn(missing_debug_implementations)]
#![warn(unsafe_code)]

use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::fmt::{Display, Formatter};

use cell::SudokuCell;

use crate::error::{Error, Result};
use crate::grid::Grid;
use crate::history::GridHistory;
use crate::position::Position;
use crate::settings::Settings;

pub mod cell;
pub mod error;
pub mod generator;
pub mod grid;
mod history;
pub mod parser;
pub mod position;
pub mod samples;
pub mod settings;
pub mod solver;
pub mod transport;

// TODO:
//  Sudoku can have a solved Grid for win state/incorrect values checking.
//  A Parser produces a Grid.
//  The grid provides an API where no invariants of the grid can be broken
//   (max_value of cells, get_pos_mut is private)

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Debug)]
pub struct Sudoku<Cell: SudokuCell> {
    grid: Grid<Cell>,
    history: GridHistory<Cell>,
    settings: Settings,
}

// TODO: make sudoku fully generic over cell
//  Sudoku<Cell<u8>>
//   find solution for repeated SudokuValue trait bound constraints
//  Alternative: sudoku decides cell type based on base at runtime
//   is this possible without dynamic dispatch?

// TODO: provide redo API
// TODO: return result in all asserts
//  sudoku::Error as JSValue (JS Exception)?
impl<Cell: SudokuCell> Sudoku<Cell> {
    pub fn new(base: usize) -> Self {
        Self::new_with_grid(Grid::new(base))
    }

    pub fn new_with_grid(grid: Grid<Cell>) -> Self {
        Sudoku {
            grid,
            history: Default::default(),
            settings: Default::default(),
        }
    }

    pub fn set_value(&mut self, pos: Position, value: usize) {
        self.push_history();

        self.grid.set_value(pos, value);

        if self.settings.update_candidates_on_set_value {
            self.grid.update_candidates(pos, value);
        }
    }

    pub fn set_or_toggle_value(&mut self, pos: Position, value: usize) {
        self.push_history();

        let set_value = self.grid.set_or_toggle_value(pos, value);

        if self.settings.update_candidates_on_set_value && set_value {
            self.grid.update_candidates(pos, value);
        }
    }

    pub fn set_candidates(&mut self, pos: Position, candidates: Vec<usize>) {
        self.push_history();

        self.grid.set_candidates(pos, candidates);
    }

    pub fn toggle_candidate(&mut self, pos: Position, candidate: usize) {
        self.push_history();

        self.grid.toggle_candidate(pos, candidate);
    }

    pub fn delete(&mut self, pos: Position) -> Cell {
        self.push_history();

        self.grid.delete(pos)
    }

    pub fn set_all_direct_candidates(&mut self) {
        self.push_history();

        self.grid.set_all_direct_candidates();
    }

    pub fn undo(&mut self) {
        if let Some(grid) = self.history.pop() {
            self.grid = grid;
        }
    }

    pub fn update_settings(&mut self, settings: Settings) {
        self.settings = settings;
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
        self.grid.get(pos)
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
    fn push_history(&mut self) {
        self.history
            .push(self.grid.clone(), self.settings.history_limit)
    }
}

impl<Cell: SudokuCell> TryFrom<Vec<usize>> for Sudoku<Cell> {
    type Error = Error;

    fn try_from(values: Vec<usize>) -> Result<Self> {
        Ok(Self::new_with_grid(values.try_into()?))
    }
}

impl<Cell: SudokuCell> Display for Sudoku<Cell> {
    // TODO: show history and settings
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.grid)
    }
}

#[cfg(test)]
mod tests {
    // TODO: test undo
    // TODO: test settings
}
