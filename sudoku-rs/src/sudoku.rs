use std::convert::TryFrom;
use std::fmt::{self, Display, Formatter};

use failure::format_err;

use crate::cell::SudokuCell;
use crate::error::Result;
use crate::generator::backtracking::Generator;
use crate::generator::backtracking::Settings as GeneratorSettings;
use crate::grid::Grid;
use crate::history::GridHistory;
use crate::position::Position;
use crate::settings::Settings;
use crate::solver::backtracking::Solver as BacktrackingSolver;
use crate::solver::strategic::{
    strategies::{GroupReduction, SingleCandidate},
    Solver as StrategicSolver,
};

// TODO: allow runtime update of base
//  <Base: SudokuBase> needs to be erased
//  Grid as a trait object?
//   Grid trait should not have generic parameters
//  hardcoded Grid instances?
//   how to interact with them dynamically?
//    enum for runtime decision?
#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Sudoku<Cell: SudokuCell> {
    grid: Grid<Cell>,
    solved_grid: Option<Grid<Cell>>,
    history: GridHistory<Cell>,
    settings: Settings,
}

// TODO: provide redo API
// TODO: return result in all asserts
//  sudoku::Error as JSValue (JS Exception)?
impl<Cell: SudokuCell> Sudoku<Cell> {
    pub fn new(base: usize) -> Self {
        Self::new_with_grid(Grid::new(base))
    }

    pub fn new_with_grid(grid: Grid<Cell>) -> Self {
        Self::new_with_grid_and_settings(grid, Default::default())
    }

    pub fn new_with_grid_and_settings(mut grid: Grid<Cell>, settings: Settings) -> Self {
        grid.fix_all_values();

        Sudoku {
            solved_grid: BacktrackingSolver::unique_solution(&grid),
            grid,
            settings,
            history: Default::default(),
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

    pub fn solve_single_candidates(&mut self) {
        self.push_history();

        let mut solver =
            StrategicSolver::new_with_strategies(&mut self.grid, vec![Box::new(SingleCandidate)]);

        solver.try_strategies();
    }

    pub fn group_reduction(&mut self) {
        self.push_history();

        let mut solver =
            StrategicSolver::new_with_strategies(&mut self.grid, vec![Box::new(GroupReduction)]);

        solver.try_strategies();
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

    // TODO: expose in UI (clipboard?)
    pub fn export(&self) -> String {
        self.grid.to_string()
    }

    pub fn grid(&self) -> &Grid<Cell> {
        &self.grid
    }

    pub fn solved_grid(&self) -> &Option<Grid<Cell>> {
        &self.solved_grid
    }
}

impl<Cell: SudokuCell> Sudoku<Cell> {
    fn push_history(&mut self) {
        self.history
            .push(self.grid.clone(), self.settings.history_limit)
    }

    fn replace_grid(&mut self, new_grid: Grid<Cell>) {
        *self = Self::new_with_grid_and_settings(new_grid, self.settings);
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
