use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};

use failure::format_err;

use crate::base::SudokuBase;
use crate::cell::SudokuCell;
use crate::error::Result;
use crate::generator::backtracking::RuntimeSettings as GeneratorSettings;
use crate::generator::backtracking::{Generator, Target};
use crate::grid::Grid;
use crate::history::History;
use crate::position::Position;
use crate::solver::backtracking::Solver as BacktrackingSolver;
use crate::solver::strategic::{
    strategies::{GroupReduction, SingleCandidate},
    Solver as StrategicSolver,
};
use crate::sudoku::dynamic::Game;

use self::settings::Settings;

pub mod dynamic;
pub mod settings;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Sudoku<Base: SudokuBase> {
    grid: Grid<Base>,
    solved_grid: Option<Grid<Base>>,
    history: History<Grid<Base>>,
    settings: Settings,
}

// TODO: provide redo API
// TODO: return result in all asserts
//  sudoku::Error as JSValue (JS Exception)?
impl<Base: SudokuBase> Sudoku<Base> {
    pub fn new() -> Self {
        Self::with_grid(Grid::new())
    }

    pub fn with_grid(grid: Grid<Base>) -> Self {
        Self::with_grid_and_settings(grid, Default::default())
    }

    pub fn with_grid_and_settings(mut grid: Grid<Base>, settings: Settings) -> Self {
        grid.fix_all_values();

        Sudoku {
            solved_grid: BacktrackingSolver::unique_solution(&grid),
            grid,
            settings,
            history: Default::default(),
        }
    }

    pub fn grid(&self) -> &Grid<Base> {
        &self.grid
    }

    pub fn solved_grid(&self) -> &Option<Grid<Base>> {
        &self.solved_grid
    }

    pub fn generate(&mut self, target: Target) -> Result<()> {
        let grid = Generator::with_target(target)
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
}

impl<Base: SudokuBase> Game for Sudoku<Base> {
    fn set_value(&mut self, pos: Position, value: u8) {
        self.push_history();

        self.grid.get_mut(pos).set_value(value);

        if self.settings.update_candidates_on_set_value {
            self.grid.update_candidates(pos, value);
        }
    }

    fn set_or_toggle_value(&mut self, pos: Position, value: u8) {
        self.push_history();

        let set_value = self.grid.get_mut(pos).set_or_toggle_value(value);

        if self.settings.update_candidates_on_set_value && set_value {
            self.grid.update_candidates(pos, value);
        }
    }

    fn set_candidates(&mut self, pos: Position, candidates: Vec<u8>) {
        self.push_history();

        self.grid.get_mut(pos).set_candidates(candidates);
    }

    fn toggle_candidate(&mut self, pos: Position, candidate: u8) {
        self.push_history();

        self.grid.get_mut(pos).toggle_candidate(candidate);
    }

    fn delete(&mut self, pos: Position) {
        self.push_history();

        self.grid.get_mut(pos).delete();
    }

    fn set_all_direct_candidates(&mut self) {
        self.push_history();

        self.grid.set_all_direct_candidates();
    }

    fn solve_single_candidates(&mut self) {
        self.push_history();

        let mut solver =
            StrategicSolver::new_with_strategies(&mut self.grid, vec![Box::new(SingleCandidate)]);

        solver.try_strategies();
    }

    fn group_reduction(&mut self) {
        self.push_history();

        let mut solver =
            StrategicSolver::new_with_strategies(&mut self.grid, vec![Box::new(GroupReduction)]);

        solver.try_strategies();
    }

    fn undo(&mut self) {
        if let Some(grid) = self.history.pop() {
            self.grid = grid;
        }
    }

    // TODO: wasm integration
    fn update_settings(&mut self, settings: Settings) {
        self.settings = settings;
    }

    // TODO: expose in UI (clipboard?)
    fn export(&self) -> String {
        self.grid.to_string()
    }
}

impl<Base: SudokuBase> Sudoku<Base> {
    fn push_history(&mut self) {
        self.history
            .push(self.grid.clone(), self.settings.history_limit)
    }

    fn replace_grid(&mut self, new_grid: Grid<Base>) {
        *self = Self::with_grid_and_settings(new_grid, self.settings);
    }
}

impl<Base: SudokuBase> Display for Sudoku<Base> {
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
