use std::convert::Infallible;

use log::debug;
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng, SeedableRng};
use rand_xoshiro::Xoshiro256StarStar;
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use ts_rs::TS;

use crate::base::SudokuBase;
use crate::cell::Value;
use crate::error::{Error, Result};
use crate::grid::Grid;
use crate::position::Position;
use crate::solver::backtracking;
use crate::solver::backtracking::CandidatesVisitOrder;
use crate::solver::strategic::strategies::{Backtracking, DynamicStrategy};

// TODO: strategic
//  target difficulty: sum of weighted strategy applications

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Serialize, Deserialize, Copy, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub enum GeneratorTarget {
    #[default]
    Filled,
    #[serde(rename_all = "camelCase")]
    FromFilled {
        distance_from_filled: usize,
        set_all_direct_candidates: bool,
    },
    #[serde(rename_all = "camelCase")]
    Minimal { set_all_direct_candidates: bool },
    #[serde(rename_all = "camelCase")]
    FromMinimal {
        distance: usize,
        set_all_direct_candidates: bool,
    },
}

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorSettings {
    pub target: GeneratorTarget,
    pub strategies: Vec<DynamicStrategy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
}

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicGeneratorSettings {
    pub base: u8,
    #[serde(flatten)]
    pub settings: GeneratorSettings,
}

#[derive(Debug)]
pub struct Generator {
    settings: GeneratorSettings,
}

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorProgress {
    position_index: usize,
    positions_count: usize,
    deleted_count: usize,
    is_position_required: bool,
}

impl Generator {
    pub fn with_target(target: GeneratorTarget) -> Self {
        Self::with_settings(GeneratorSettings {
            target,
            strategies: vec![Backtracking.into()],
            seed: None,
        })
    }

    pub fn with_settings(settings: GeneratorSettings) -> Self {
        Self { settings }
    }

    fn rng(&self) -> impl Rng {
        if let Some(seed) = self.settings.seed {
            Xoshiro256StarStar::seed_from_u64(seed)
        } else {
            Xoshiro256StarStar::from_rng(thread_rng()).unwrap()
        }
    }

    pub fn generate<Base: SudokuBase>(&self) -> Grid<Base> {
        match self.generate_with_progress(|_| Ok::<(), Infallible>(())) {
            Ok(grid) => grid,
            Err(infallible) => match infallible {},
        }
    }

    pub fn generate_with_progress<Base: SudokuBase, E: Into<Error>>(
        &self,
        on_progress: impl FnMut(GeneratorProgress) -> Result<(), E>,
    ) -> Result<Grid<Base>, E> {
        debug!("generate: {self:?}");

        let filled_grid = self.filled_grid();

        let (mut grid, set_all_direct_candidates) = match self.settings.target {
            GeneratorTarget::Filled => (filled_grid, false),
            GeneratorTarget::FromFilled {
                distance_from_filled,
                set_all_direct_candidates,
            } => (
                self.filled(filled_grid, distance_from_filled, on_progress)?,
                set_all_direct_candidates,
            ),
            GeneratorTarget::Minimal {
                set_all_direct_candidates,
            } => (
                self.minimal(filled_grid, 0, on_progress)?,
                set_all_direct_candidates,
            ),
            GeneratorTarget::FromMinimal {
                distance,
                set_all_direct_candidates,
            } => (
                self.minimal(filled_grid, distance, on_progress)?,
                set_all_direct_candidates,
            ),
        };

        grid.fix_all_values();

        if set_all_direct_candidates {
            grid.set_all_direct_candidates();
        }

        Ok(grid)
    }

    fn filled_grid<Base: SudokuBase>(&self) -> Grid<Base> {
        let mut grid = Grid::<Base>::new();

        let mut solver = backtracking::Solver::new_with_settings(
            &mut grid,
            backtracking::Settings {
                candidates_visit_order: if let Some(seed) = self.settings.seed {
                    CandidatesVisitOrder::RandomSeed(seed)
                } else {
                    CandidatesVisitOrder::Random
                },
                ..Default::default()
            },
        );

        solver.next().unwrap()
    }

    /// Try to delete a cell at specific position in a grid while preserving uniqueness of the grid solution.
    ///
    /// Returns the value of the deleted cell, if any.
    fn try_delete_cell_at_pos<Base: SudokuBase>(
        &self,
        grid: &mut Grid<Base>,
        pos: Position<Base>,
    ) -> Option<Value<Base>> {
        let cell = grid.get(pos);

        if let Some(value) = cell.value() {
            grid.get_mut(pos).delete();

            match grid.is_solvable_with_strategies(self.settings.strategies.clone()) {
                Ok(Some(_)) if grid.has_unique_solution() => {
                    // current position can be removed without losing uniqueness of the grid solution.
                    Some(value)
                }
                _ => {
                    // current position is necessary for unique solution
                    grid.get_mut(pos).set_value(value);
                    None
                }
            }
        } else {
            panic!("Expected value at {pos}, instead got: {cell:?}")
        }
    }

    fn filled<Base: SudokuBase, E: Into<Error>>(
        &self,
        mut grid: Grid<Base>,
        distance_from_filled: usize,
        mut on_progress: impl FnMut(GeneratorProgress) -> Result<(), E>,
    ) -> Result<Grid<Base>, E> {
        if distance_from_filled == 0 {
            return Ok(grid);
        }

        assert!(grid.is_solved());

        let mut all_positions: Vec<_> = Grid::<Base>::all_positions().collect();
        all_positions.shuffle(&mut self.rng());
        let positions_count = Grid::<Base>::cell_count_usize();

        let mut deleted_count = 0;
        for (i, pos) in all_positions.into_iter().enumerate() {
            let position_index = i + 1;

            if deleted_count >= distance_from_filled {
                break;
            }

            let is_position_required = if self.try_delete_cell_at_pos(&mut grid, pos).is_some() {
                deleted_count += 1;
                debug!("Position {position_index}/{positions_count} deleted, totaling {deleted_count}/{distance_from_filled} deleted positions");
                false
            } else {
                debug!(
                    "Position {position_index}/{positions_count} is required for unique solution"
                );
                true
            };

            on_progress(GeneratorProgress {
                position_index,
                positions_count,
                deleted_count,
                is_position_required,
            })?;
        }

        Ok(grid)
    }

    fn minimal<Base: SudokuBase, E: Into<Error>>(
        &self,
        mut grid: Grid<Base>,
        distance_from_minimal: usize,
        mut on_progress: impl FnMut(GeneratorProgress) -> Result<(), E>,
    ) -> Result<Grid<Base>, E> {
        // If the distance results in a filled sudoku, return it directly.
        if distance_from_minimal >= Grid::<Base>::cell_count_usize() {
            return Ok(grid);
        }

        assert!(grid.is_solved());

        let mut all_positions: Vec<_> = Grid::<Base>::all_positions().collect();
        all_positions.shuffle(&mut self.rng());
        let positions_count = Grid::<Base>::cell_count_usize();

        let mut deleted: Vec<(Position<Base>, Value<Base>)> = vec![];

        // Reduce grid to a minimal solution.
        for (i, pos) in all_positions.into_iter().enumerate() {
            let position_index = i + 1;

            let deleted_count = deleted.len();

            let is_position_required = if let Some(deleted_value) =
                self.try_delete_cell_at_pos(&mut grid, pos)
            {
                debug!("Position {i}/{positions_count} deleted, totaling {deleted_count} deleted positions");

                deleted.push((pos, deleted_value));
                false
            } else {
                debug!("Position {i}/{positions_count} is required for unique solution");
                true
            };

            on_progress(GeneratorProgress {
                position_index,
                positions_count,
                deleted_count,
                is_position_required,
            })?;
        }

        // Restore the required amount of values, specified by distance.
        for (deleted_pos, deleted_value) in deleted.into_iter().take(distance_from_minimal) {
            grid.get_mut(deleted_pos).set_value(deleted_value);
        }

        Ok(grid)
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::*;

    use super::*;

    fn is_minimal<Base: SudokuBase>(grid: &Grid<Base>) -> bool {
        let mut grid = grid.clone();

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

    #[test]
    fn test_minimal() {
        let grid = Generator::with_target(GeneratorTarget::Minimal {
            set_all_direct_candidates: false,
        })
        .generate::<Base2>();

        println!("{grid}");

        assert!(is_minimal(&grid));
    }

    #[test]
    fn test_filled() {
        let grid = Generator::with_target(GeneratorTarget::Filled).generate::<Base2>();

        assert!(grid.is_solved());
    }

    #[test]
    fn test_from_filled() {
        let grid = Generator::with_target(GeneratorTarget::FromFilled {
            distance_from_filled: 2,
            set_all_direct_candidates: false,
        })
        .generate::<Base2>();

        assert_eq!(grid.all_candidates_positions().len(), 2);

        assert!(grid.has_unique_solution());
    }

    #[test]
    fn test_seed() {
        let strategies = vec![Backtracking.into()];
        let generator_1 = Generator::with_settings(GeneratorSettings {
            seed: Some(1),
            target: GeneratorTarget::Filled,
            strategies: strategies.clone(),
        });
        assert_eq!(
            generator_1.generate::<Base3>(),
            generator_1.generate::<Base3>()
        );
        let generator_2 = Generator::with_settings(GeneratorSettings {
            seed: Some(2),
            target: GeneratorTarget::Filled,
            strategies,
        });
        assert_eq!(
            generator_2.generate::<Base3>(),
            generator_2.generate::<Base3>()
        );
        assert_ne!(
            generator_1.generate::<Base3>(),
            generator_2.generate::<Base3>()
        );
    }
}
