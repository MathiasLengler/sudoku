use anyhow::format_err;
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

#[derive(Debug)]
pub struct GeneratorSettings<Base: SudokuBase> {
    pub target: GeneratorTarget,
    pub strategies: Vec<DynamicStrategy>,
    pub givens_grid: Option<Grid<Base>>,
    pub seed: Option<u64>,
}

impl<Base: SudokuBase> Default for GeneratorSettings<Base> {
    fn default() -> Self {
        Self {
            target: GeneratorTarget::default(),
            strategies: vec![Backtracking.into()],
            seed: None,
            givens_grid: None,
        }
    }
}

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicGeneratorSettings {
    pub base: u8,
    pub target: GeneratorTarget,
    pub strategies: Vec<DynamicStrategy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
}

#[derive(Debug)]
pub struct Generator<Base: SudokuBase> {
    settings: GeneratorSettings<Base>,
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

impl<Base: SudokuBase> Generator<Base> {
    pub fn with_target(target: GeneratorTarget) -> Self {
        Self::with_settings(GeneratorSettings {
            target,
            ..Default::default()
        })
    }

    pub fn with_settings(settings: GeneratorSettings<Base>) -> Self {
        Self { settings }
    }

    fn rng(&self) -> impl Rng {
        if let Some(seed) = self.settings.seed {
            Xoshiro256StarStar::seed_from_u64(seed)
        } else {
            Xoshiro256StarStar::from_rng(thread_rng()).unwrap()
        }
    }

    pub fn generate(&self) -> Result<Grid<Base>> {
        self.generate_with_progress(|_| Ok(()))
    }

    pub fn generate_with_progress(
        &self,
        on_progress: impl FnMut(GeneratorProgress) -> Result<()>,
    ) -> Result<Grid<Base>> {
        debug!("generate: {self:?}");

        let filled_grid = self.filled_grid()?;

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

    fn filled_grid(&self) -> Result<Grid<Base>> {
        let mut grid = if let Some(ref givens_grid) = self.settings.givens_grid {
            givens_grid.clone()
        } else {
            Grid::<Base>::new()
        };

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

        solver.next().ok_or_else(|| {
            if self.settings.givens_grid.is_some() {
                format_err!("Provided givens grid has no solution")
            } else {
                panic!("Expected empty grid to have at least one solution")
            }
        })
    }

    /// Try to delete a cell at specific position in a grid while preserving uniqueness of the grid solution.
    ///
    /// Returns the value of the deleted cell, if any.
    fn try_delete_cell_at_pos(
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

    fn deletable_positions(&self) -> Vec<Position<Base>> {
        let mut deletable_positions = if let Some(givens_grid) = &self.settings.givens_grid {
            givens_grid.all_candidates_positions()
        } else {
            Grid::<Base>::all_positions().collect()
        };
        deletable_positions.shuffle(&mut self.rng());
        deletable_positions
    }

    fn filled<E: Into<Error>>(
        &self,
        mut grid: Grid<Base>,
        distance_from_filled: usize,
        mut on_progress: impl FnMut(GeneratorProgress) -> Result<(), E>,
    ) -> Result<Grid<Base>, E> {
        debug_assert!(grid.is_solved());

        if distance_from_filled == 0 {
            return Ok(grid);
        }

        let deletable_positions: Vec<_> = self.deletable_positions();
        let positions_count = deletable_positions.len();

        let mut deleted_count = 0;
        for (i, pos) in deletable_positions.into_iter().enumerate() {
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

    fn minimal<E: Into<Error>>(
        &self,
        mut grid: Grid<Base>,
        distance_from_minimal: usize,
        mut on_progress: impl FnMut(GeneratorProgress) -> Result<(), E>,
    ) -> Result<Grid<Base>, E> {
        debug_assert!(grid.is_solved());

        // If the distance results in a filled sudoku, return it directly.
        if distance_from_minimal >= Grid::<Base>::cell_count_usize() {
            return Ok(grid);
        }

        let deletable_positions: Vec<_> = self.deletable_positions();
        let positions_count = deletable_positions.len();

        let mut deleted: Vec<(Position<Base>, Value<Base>)> = vec![];

        // Reduce grid to a minimal solution.
        for (i, pos) in deletable_positions.into_iter().enumerate() {
            let position_index = i + 1;

            let deleted_count = deleted.len();

            let is_position_required = if let Some(deleted_value) =
                self.try_delete_cell_at_pos(&mut grid, pos)
            {
                debug!("Position {position_index}/{positions_count} deleted, totaling {deleted_count} deleted positions");

                deleted.push((pos, deleted_value));
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

        // Restore the required amount of values, specified by distance.
        for (deleted_pos, deleted_value) in deleted.into_iter().take(distance_from_minimal) {
            grid.get_mut(deleted_pos).set_value(deleted_value);
        }

        Ok(grid)
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::base::consts::*;
    use crate::position::Coordinate;

    use super::*;

    #[test]
    fn test_minimal() {
        let grid = Generator::<Base2>::with_target(GeneratorTarget::Minimal {
            set_all_direct_candidates: false,
        })
        .generate()
        .unwrap();

        println!("{grid}");

        assert!(grid.is_minimal());
    }

    #[test]
    fn test_filled() {
        let grid = Generator::<Base2>::with_target(GeneratorTarget::Filled)
            .generate()
            .unwrap();

        assert!(grid.is_solved());
    }

    #[test]
    fn test_from_filled() {
        let grid = Generator::<Base2>::with_target(GeneratorTarget::FromFilled {
            distance_from_filled: 2,
            set_all_direct_candidates: false,
        })
        .generate()
        .unwrap();

        assert_eq!(grid.all_candidates_positions().len(), 2);

        assert!(grid.has_unique_solution());
    }

    #[test]
    fn test_seed_filled() {
        let generator_1 = Generator::<Base3>::with_settings(GeneratorSettings {
            seed: Some(1),
            ..Default::default()
        });
        assert_eq!(
            generator_1.generate().unwrap(),
            generator_1.generate().unwrap()
        );
        let generator_2 = Generator::<Base3>::with_settings(GeneratorSettings {
            seed: Some(2),
            ..Default::default()
        });
        assert_eq!(
            generator_2.generate().unwrap(),
            generator_2.generate().unwrap()
        );
        assert_ne!(
            generator_1.generate().unwrap(),
            generator_2.generate().unwrap()
        );
    }

    #[test]
    fn test_strategies() {
        use crate::solver::strategic::strategies::*;

        fn generate(target: GeneratorTarget, strategies: Vec<DynamicStrategy>) -> Grid<Base3> {
            Generator::<Base3>::with_settings(GeneratorSettings {
                strategies,
                target,
                ..Default::default()
            })
            .generate()
            .unwrap()
        }

        let targets = vec![
            GeneratorTarget::Minimal {
                set_all_direct_candidates: false,
            },
            GeneratorTarget::FromFilled {
                distance_from_filled: 20,
                set_all_direct_candidates: false,
            },
        ];

        for target in targets {
            let grid = generate(target, vec![]);
            assert!(grid.is_solved());

            let default_strategies = DynamicStrategy::default_solver_strategies();
            for i in 1..default_strategies.len() {
                let strategies = default_strategies.clone().into_iter().take(i).collect_vec();
                let grid = generate(target, strategies.clone());
                assert!(grid
                    .is_solvable_with_strategies(strategies)
                    .unwrap()
                    .is_some());
            }
        }
    }

    #[test]
    fn test_givens_grid_filled() {
        let givens_grid: Grid<Base2> = "
  4  3  │  0  0  
  2  1  │  0  0  
────────┼────────
  0  0  │  0  0  
  0  0  │  0  0  "
            .parse()
            .unwrap();

        let grid = Generator::<Base2>::with_settings(GeneratorSettings {
            givens_grid: Some(givens_grid.clone()),
            ..Default::default()
        })
        .generate()
        .unwrap();

        // Top left block is unchanged
        itertools::assert_equal(
            givens_grid.block_cells(Coordinate::default()),
            grid.block_cells(Coordinate::default()),
        );

        assert!(grid.is_solved());
    }

    #[test]
    fn test_givens_grid_minimal() {
        let givens_grid: Grid<Base2> = "
  4  3  │  0  0  
  2  1  │  0  0  
────────┼────────
  0  0  │  0  0  
  0  0  │  0  0  "
            .parse()
            .unwrap();

        let grid = Generator::<Base2>::with_settings(GeneratorSettings {
            givens_grid: Some(givens_grid.clone()),
            target: GeneratorTarget::Minimal {
                set_all_direct_candidates: false,
            },
            ..Default::default()
        })
        .generate()
        .unwrap();

        println!("{grid}");

        // Top left block is unchanged
        itertools::assert_equal(
            givens_grid.block_cells(Coordinate::default()),
            grid.block_cells(Coordinate::default()),
        );

        // Grid has unique solution
        assert!(grid.has_unique_solution());

        // Grid does not have a unique solution, if any value outside the top left is deleted.
        for non_top_left_block_pos in grid
            .all_value_positions()
            .into_iter()
            .filter(|pos| pos.to_block() != Coordinate::default())
        {
            let mut grid = grid.clone();
            grid.unfix_all_values();
            grid.get_mut(non_top_left_block_pos).delete();
            assert!(!grid.has_unique_solution());
        }
    }
}
