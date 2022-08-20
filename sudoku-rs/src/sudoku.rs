use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Display, Formatter};

pub use dynamic::{DynamicSudoku, Game};

use crate::base::SudokuBase;
use crate::cell::compact::value::Value;
use crate::error::Result;
use crate::generator::backtracking::{Generator, Target};
use crate::grid::serialization::GridFormat;
use crate::grid::Grid;
use crate::history::History;
use crate::position::Position;
use crate::solver::backtracking::Solver as BacktrackingSolver;
use crate::solver::strategic::{
    strategies::{GroupReduction, SingleCandidate},
    Solver as StrategicSolver,
};

use self::settings::Settings;

mod dynamic;
pub mod settings;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Sudoku<Base: SudokuBase> {
    grid: Grid<Base>,
    solved_grid: Option<Grid<Base>>,
    history: History<Grid<Base>>,
    settings: Settings,
}

impl<Base: SudokuBase> Default for Sudoku<Base> {
    fn default() -> Self {
        Self::with_grid(Grid::new())
    }
}

// TODO: provide redo API
// TODO: return result in all asserts
impl<Base: SudokuBase> Sudoku<Base> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_grid(grid: Grid<Base>) -> Self {
        Self::with_grid_and_settings(grid, Default::default())
    }

    pub fn with_grid_and_settings(grid: Grid<Base>, settings: Settings) -> Self {
        Sudoku {
            solved_grid: if settings.solve_grid {
                BacktrackingSolver::unique_solution_for_fixed_values(&grid)
            } else {
                None
            },
            grid,
            settings,
            history: Default::default(),
        }
    }

    pub fn with_target_and_settings(target: Target, settings: Settings) -> Result<Self> {
        let grid = Generator::with_target(target).generate();

        Ok(Self::with_grid_and_settings(grid, settings))
    }

    pub fn grid(&self) -> &Grid<Base> {
        &self.grid
    }

    pub fn solved_grid(&self) -> &Option<Grid<Base>> {
        &self.solved_grid
    }

    pub fn import(&mut self, input: &str) -> Result<()> {
        let grid = Grid::try_from(input)?;

        self.replace_grid(grid);

        Ok(())
    }
}

impl<Base: SudokuBase> Game for Sudoku<Base> {
    fn set_value(&mut self, pos: Position, value: u8) -> Result<()> {
        self.push_history();

        let cell = self.grid.get_mut(pos);

        if let Some(value) = Value::new(value)? {
            cell.set_value(value);

            if self.settings.update_candidates {
                self.grid.update_direct_candidates(pos, value);
            }
        } else {
            cell.delete();
        }

        Ok(())
    }

    fn set_or_toggle_value(&mut self, pos: Position, value: u8) -> Result<()> {
        self.push_history();

        let cell = self.grid.get_mut(pos);

        if let Some(value) = Value::new(value)? {
            let set_value = cell.set_or_toggle_value(value);

            if self.settings.update_candidates && set_value {
                self.grid.update_direct_candidates(pos, value);
            }
        } else {
            cell.delete();
        }

        Ok(())
    }

    fn set_candidates(&mut self, pos: Position, candidates: Vec<u8>) -> Result<()> {
        self.push_history();

        self.grid
            .get_mut(pos)
            .set_candidates(candidates.try_into()?);

        Ok(())
    }

    fn toggle_candidate(&mut self, pos: Position, candidate: u8) -> Result<()> {
        self.push_history();

        self.grid
            .get_mut(pos)
            .toggle_candidate(candidate.try_into()?);

        Ok(())
    }

    fn delete(&mut self, pos: Position) {
        self.push_history();

        self.grid.get_mut(pos).delete();
    }

    fn set_all_direct_candidates(&mut self) {
        self.push_history();

        self.grid.set_all_direct_candidates();
    }

    // TODO: refactor Strategy API
    //  return deductions for visualization
    //  single method to try a specific strategy
    fn solve_single_candidates(&mut self) -> Result<()> {
        self.push_history();

        let mut solver =
            StrategicSolver::new_with_strategies(&mut self.grid, vec![Box::new(SingleCandidate)]);

        // TODO: apply deductions
        solver.try_strategies()?;

        Ok(())
    }

    fn group_reduction(&mut self) -> Result<()> {
        self.push_history();

        let mut solver =
            StrategicSolver::new_with_strategies(&mut self.grid, vec![Box::new(GroupReduction)]);

        // TODO: apply deductions
        solver.try_strategies()?;

        Ok(())
    }

    fn undo(&mut self) {
        if let Some(grid) = self.history.pop() {
            self.grid = grid;
        }
    }

    fn settings(&self) -> Settings {
        self.settings
    }

    // TODO: wasm integration
    fn update_settings(&mut self, settings: Settings) {
        self.settings = settings;
    }

    fn export(&self, format: &GridFormat) -> String {
        format.render(&self.grid)
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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.grid)
    }
}

#[cfg(test)]
mod tests {
    // TODO: test undo
    // TODO: test settings
}
