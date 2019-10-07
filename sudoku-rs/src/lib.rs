#![warn(missing_debug_implementations)]
#![warn(unsafe_code)]

use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};

use failure::{bail, format_err};

use cell::SudokuCell;

use crate::error::Result;
use crate::generator::backtracking::Generator;
use crate::generator::backtracking::Settings as GeneratorSettings;
use crate::grid::Grid;
use crate::history::GridHistory;
use crate::position::Position;
use crate::settings::Settings;

pub mod cell;
pub mod error;
pub mod generator;
pub mod grid;
mod history;
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

    // TODO: wasm integration
    pub fn update_settings(&mut self, settings: Settings) {
        self.settings = settings;
    }

    pub fn generate(&mut self, generator_settings: GeneratorSettings) -> Result<()> {
        let grid = Generator::new(generator_settings)
            .generate()
            .ok_or(format_err!("Unable to generate grid"))?;

        self.replace_grid(grid);

        Ok(())
    }

    pub fn import(&mut self, input: &str) -> Result<()> {
        let grid = Grid::try_from(input)?;

        self.replace_grid(grid);

        Ok(())
    }

    pub fn export(&self) -> String {
        self.grid.to_string()
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

    fn replace_grid(&mut self, new_grid: Grid<Cell>) {
        self.grid = new_grid;
        self.grid.fix_all_values();
        self.history = Default::default();
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
