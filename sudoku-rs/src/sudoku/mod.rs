use crate::cell::{Candidates, Value};
use crate::error::Result;
use crate::generator::multi_shot::{
    MultiShotGenerator, MultiShotGeneratorProgress, MultiShotGeneratorSettings,
};
use crate::generator::{Generator, GeneratorProgress, GeneratorSettings};
use crate::grid::Grid;
use crate::grid::dynamic::DynamicGrid;
use crate::grid::format::GridFormat;
use crate::grid::format::GridFormatEnum;
use crate::grid::solution_state::SolutionState;
use crate::position::{DynamicPosition, Position};
use crate::solver::strategic::deduction::Deductions;
use crate::solver::strategic::deduction::transport::TransportDeductions;
use crate::solver::strategic::strategies::StrategyEnum;
use crate::solver::strategic::{DynamicSolveStep, SolveStep, Solver as StrategicSolver};
use crate::{
    base::SudokuBase,
    generator::multi_shot::{EvaluatedGridMetric, GridMetric},
};
use crate::{
    cell::dynamic::{DynamicCandidates, DynamicValue},
    solver::strategic::strategies::selection::StrategySelection,
};
use history::History;
use log::info;
use serde::{Deserialize, Serialize};
use settings::Settings;
use std::fmt::{self, Display, Formatter};

pub use dynamic::{DynamicSudoku, DynamicSudokuActions};

mod dynamic;
mod history;
pub mod settings;
pub mod transport;

#[derive(Eq, PartialEq, Hash, Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "Base: SudokuBase")]
pub struct Sudoku<Base: SudokuBase> {
    grid: Grid<Base>,
    solution: SolutionState<Base>,
    history: History<Grid<Base>>,
    settings: Settings,
}

impl<Base: SudokuBase> Default for Sudoku<Base> {
    fn default() -> Self {
        Self::new()
    }
}

/// Constructors
impl<Base: SudokuBase> Sudoku<Base> {
    /// Creates a new empty Sudoku.
    pub fn new() -> Self {
        Self::with_grid(Grid::new())
    }

    pub fn with_grid(grid: Grid<Base>) -> Self {
        Self::with_grid_and_settings(grid, Settings::default())
    }

    pub fn with_grid_and_settings(grid: Grid<Base>, settings: Settings) -> Self {
        let solution = if settings.find_solution {
            grid.solution_state_for_fixed_values()
        } else {
            SolutionState::NoSolution
        };
        let history = History::with_limit(settings.history_limit);
        Self {
            grid,
            solution,
            history,
            settings,
        }
    }

    pub fn generate(
        generator_settings: GeneratorSettings<Base>,
        settings: Settings,
        on_progress: impl FnMut(GeneratorProgress) -> Result<()>,
    ) -> Result<Self> {
        info!("generator_settings {generator_settings:#?}");

        Ok(Self::with_grid_and_settings(
            Generator::with_settings(generator_settings).generate_with_progress(on_progress)?,
            settings,
        ))
    }

    pub fn generate_multi_shot(
        multi_shot_generator_settings: MultiShotGeneratorSettings<Base>,
        settings: Settings,
        on_progress: impl FnMut(MultiShotGeneratorProgress) -> Result<()>,
    ) -> Result<Self> {
        info!("multi_shot_generator_settings {multi_shot_generator_settings:#?}");

        let metric = multi_shot_generator_settings.metric;
        let generator = MultiShotGenerator::with_settings(multi_shot_generator_settings)?;

        let evaluated_grid = generator.generate_with_progress(on_progress)?;

        info!(
            "Grid generated with a final metric {:?} of {}",
            metric, evaluated_grid.evaluated_grid_metric
        );

        Ok(Self::with_grid_and_settings(evaluated_grid.grid, settings))
    }
}

impl<Base: SudokuBase> Sudoku<Base> {
    pub fn grid(&self) -> &Grid<Base> {
        &self.grid
    }
}

// base-specific implementations for `DynamicSudokuActions`
impl<Base: SudokuBase> Sudoku<Base> {
    pub fn set_value(&mut self, pos: Position<Base>, value: Value<Base>) {
        self.push_history();

        self.grid.get_mut(pos).set_value(value);

        if self.settings.update_candidates {
            self.grid.update_direct_candidates_for_new_value(pos, value);
        }
    }

    pub fn set_or_toggle_value(&mut self, pos: Position<Base>, value: Value<Base>) {
        self.push_history();

        let set_value = self.grid.get_mut(pos).set_or_toggle_value(value);

        if self.settings.update_candidates && set_value {
            self.grid.update_direct_candidates_for_new_value(pos, value);
        }
    }

    pub fn set_candidates(&mut self, pos: Position<Base>, candidates: Candidates<Base>) {
        self.push_history();

        self.grid.get_mut(pos).set_candidates(candidates);
    }

    pub fn toggle_candidate(&mut self, pos: Position<Base>, candidate: Value<Base>) {
        self.push_history();

        self.grid.get_mut(pos).toggle_candidate(candidate);
    }
    pub fn set_candidate(&mut self, pos: Position<Base>, candidate: Value<Base>) {
        self.push_history();

        self.grid.get_mut(pos).set_candidate(candidate);
    }
    pub fn delete_candidate(&mut self, pos: Position<Base>, candidate: Value<Base>) {
        self.push_history();

        self.grid.get_mut(pos).delete_candidate(candidate);
    }

    pub fn delete(&mut self, pos: Position<Base>) {
        self.push_history();

        self.grid.get_mut(pos).delete();
    }

    pub fn try_strategies(
        &mut self,
        strategies: Vec<StrategyEnum>,
    ) -> Result<Option<SolveStep<Base>>> {
        // Only create history entry if all candidates are empty.
        // If this is the case, StrategicSolver will mutate the grid by setting all direct candidates.
        if self.grid.are_all_candidates_empty() {
            self.push_history();
        }

        let solver = StrategicSolver::with_strategies(&mut self.grid, strategies);
        solver.try_strategies()
    }

    pub fn apply_deductions(&mut self, deductions: &Deductions<Base>) -> Result<()> {
        self.push_history();

        deductions.apply(&mut self.grid)
    }
}

impl<Base: SudokuBase> DynamicSudokuActions for Sudoku<Base> {
    // actions that handle base-dependend types
    // - convert runtime-base (`Dynamic*`) parameters to base-generic equivalents
    // - call base-generic implementation
    // - if return is base-generic, convert to its runtime-base eqivalent

    fn set_value(&mut self, pos: DynamicPosition, value: DynamicValue) -> Result<()> {
        let pos = pos.try_into()?;
        let value = value.try_into()?;

        self.set_value(pos, value);

        Ok(())
    }

    fn set_or_toggle_value(&mut self, pos: DynamicPosition, value: DynamicValue) -> Result<()> {
        let pos = pos.try_into()?;
        let value = value.try_into()?;

        self.set_or_toggle_value(pos, value);

        Ok(())
    }

    fn set_candidates(
        &mut self,
        pos: DynamicPosition,
        candidates: DynamicCandidates,
    ) -> Result<()> {
        let pos = pos.try_into()?;
        let candidates = candidates.try_into()?;

        self.set_candidates(pos, candidates);

        Ok(())
    }

    fn toggle_candidate(&mut self, pos: DynamicPosition, candidate: DynamicValue) -> Result<()> {
        let pos = pos.try_into()?;
        let candidate = candidate.try_into()?;

        self.toggle_candidate(pos, candidate);

        Ok(())
    }
    fn set_candidate(&mut self, pos: DynamicPosition, candidate: DynamicValue) -> Result<()> {
        let pos = pos.try_into()?;
        let candidate = candidate.try_into()?;

        self.set_candidate(pos, candidate);

        Ok(())
    }
    fn delete_candidate(&mut self, pos: DynamicPosition, candidate: DynamicValue) -> Result<()> {
        let pos = pos.try_into()?;
        let candidate = candidate.try_into()?;

        self.delete_candidate(pos, candidate);

        Ok(())
    }

    fn delete(&mut self, pos: DynamicPosition) -> Result<()> {
        let pos = pos.try_into()?;

        self.delete(pos);

        Ok(())
    }

    fn try_strategies(
        &mut self,
        strategies: Vec<StrategyEnum>,
    ) -> Result<Option<DynamicSolveStep>> {
        Ok(self.try_strategies(strategies)?.map(Into::into))
    }

    fn apply_deductions(&mut self, deductions: TransportDeductions) -> Result<()> {
        self.apply_deductions(&deductions.try_into()?)
    }

    // actions that don't depend on base

    fn set_all_direct_candidates(&mut self) {
        self.push_history();

        self.grid.set_all_direct_candidates();
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

    fn export(&self, format: GridFormatEnum) -> String {
        format.render(&self.grid)
    }

    fn to_dynamic_grid(&self) -> DynamicGrid {
        self.grid.clone().into()
    }

    fn evaluate_metric(
        &self,
        metric: GridMetric,
        strategies: impl StrategySelection + Copy,
    ) -> Result<EvaluatedGridMetric> {
        metric.evaluate(&self.grid, strategies)
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
