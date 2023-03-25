use std::convert::TryInto;
use std::fmt::{self, Display, Formatter};

pub use dynamic::{DynamicSudoku, Game};
use history::History;

use crate::base::SudokuBase;
use crate::cell::compact::value::Value;
use crate::error::Result;
use crate::generator::{Generator, GeneratorSettings};
use crate::grid::serialization::GridFormat;
use crate::grid::Grid;
use crate::position::DynamicPosition;
use crate::solver::strategic::strategies::DynamicStrategy;
use crate::solver::strategic::Solver as StrategicSolver;

use self::settings::Settings;

mod dynamic;
mod history;
pub mod settings;
pub mod transport;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct Sudoku<Base: SudokuBase> {
    grid: Grid<Base>,
    solved_grid: Option<Grid<Base>>,
    history: History<Grid<Base>>,
    settings: Settings,
}

impl<Base: SudokuBase> Default for Sudoku<Base> {
    fn default() -> Self {
        Self::new()
    }
}

// TODO: provide redo API
// TODO: return result in all asserts
impl<Base: SudokuBase> Sudoku<Base> {
    pub fn new() -> Self {
        Self::with_grid(Grid::new())
    }

    pub fn with_grid(grid: Grid<Base>) -> Self {
        Self::with_grid_and_settings(grid, Settings::default())
    }

    pub fn with_grid_and_settings(grid: Grid<Base>, settings: Settings) -> Self {
        Sudoku {
            solved_grid: if settings.solve_grid {
                grid.unique_solution_for_fixed_values()
            } else {
                None
            },
            grid,
            settings,
            history: History::with_limit(settings.history_limit),
        }
    }

    pub fn generate(generator_settings: GeneratorSettings, settings: Settings) -> Result<Self> {
        let grid = Generator::with_settings(generator_settings).generate();

        Ok(Self::with_grid_and_settings(grid, settings))
    }
}

impl<Base: SudokuBase> Sudoku<Base> {
    pub fn grid(&self) -> &Grid<Base> {
        &self.grid
    }

    pub fn solved_grid(&self) -> &Option<Grid<Base>> {
        &self.solved_grid
    }
}

impl<Base: SudokuBase> Game for Sudoku<Base> {
    fn set_value(&mut self, pos: DynamicPosition, value: u8) -> Result<()> {
        let pos = pos.try_into()?;
        let value = Value::new(value)?;

        self.push_history();

        let cell = self.grid.get_mut(pos);

        if let Some(value) = value {
            cell.set_value(value);

            if self.settings.update_candidates {
                self.grid.update_direct_candidates(pos, value);
            }
        } else {
            cell.delete();
        }

        Ok(())
    }

    fn set_or_toggle_value(&mut self, pos: DynamicPosition, value: u8) -> Result<()> {
        let pos = pos.try_into()?;
        let value = Value::new(value)?;

        self.push_history();

        let cell = self.grid.get_mut(pos);

        if let Some(value) = value {
            let set_value = cell.set_or_toggle_value(value);

            if self.settings.update_candidates && set_value {
                self.grid.update_direct_candidates(pos, value);
            }
        } else {
            cell.delete();
        }

        Ok(())
    }

    fn set_candidates(&mut self, pos: DynamicPosition, candidates: Vec<u8>) -> Result<()> {
        let pos = pos.try_into()?;
        let candidates = candidates.try_into()?;

        self.push_history();

        self.grid.get_mut(pos).set_candidates(candidates);

        Ok(())
    }

    fn toggle_candidate(&mut self, pos: DynamicPosition, candidate: u8) -> Result<()> {
        let pos = pos.try_into()?;
        let candidate = candidate.try_into()?;

        self.push_history();

        self.grid.get_mut(pos).toggle_candidate(candidate);

        Ok(())
    }
    fn set_candidate(&mut self, pos: DynamicPosition, candidate: u8) -> Result<()> {
        let pos = pos.try_into()?;
        let candidate = candidate.try_into()?;

        self.push_history();

        self.grid.get_mut(pos).set_candidate(candidate);

        Ok(())
    }
    fn delete_candidate(&mut self, pos: DynamicPosition, candidate: u8) -> Result<()> {
        let pos = pos.try_into()?;
        let candidate = candidate.try_into()?;

        self.push_history();

        self.grid.get_mut(pos).delete_candidate(candidate);

        Ok(())
    }

    fn delete(&mut self, pos: DynamicPosition) -> Result<()> {
        let pos = pos.try_into()?;

        self.push_history();

        self.grid.get_mut(pos).delete();

        Ok(())
    }

    fn set_all_direct_candidates(&mut self) {
        self.push_history();

        self.grid.set_all_direct_candidates();
    }

    // TODO: replace_with_direct_candidates(pos: Position)
    //  For which UI interactions could this operation make sense?

    fn try_strategy(&mut self, strategy: DynamicStrategy) -> Result<bool> {
        self.push_history();

        let mut solver = StrategicSolver::new_with_strategies(&mut self.grid, vec![strategy]);

        Ok(if let Some(deductions) = solver.try_strategies()? {
            deductions.apply(&mut self.grid)?;

            true
        } else {
            false
        })
    }

    fn undo(&mut self) {
        if let Some(grid) = self.history.go_back(&self.grid) {
            self.grid = grid;
        }
    }

    fn redo(&mut self) {
        if let Some(grid) = self.history.go_forward(&self.grid) {
            self.grid = grid;
        }
    }

    fn settings(&self) -> Settings {
        self.settings
    }

    // TODO: wasm integration
    fn update_settings(&mut self, settings: Settings) {
        self.settings = settings;
        self.history.set_limit(self.settings.history_limit);
    }

    fn export(&self, format: &GridFormat) -> String {
        format.render(&self.grid)
    }
}

impl<Base: SudokuBase> Sudoku<Base> {
    fn push_history(&mut self) {
        self.history.push(self.grid.clone());
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
