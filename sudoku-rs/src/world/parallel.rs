//! Parallel generation and pruning strategies for `CellWorld`.
//!
//! This module provides parallel implementations for generating and pruning
//! cell worlds. The key challenge is handling overlapping grids while still
//! enabling parallelism.
//!
//! # Island Segmentation
//!
//! The world is segmented into "islands" - groups of grids that can be
//! processed independently. Two islands don't overlap, so they can be
//! generated/pruned in parallel.
//!
//! # Generation Strategies
//!
//! - Sequential: Process all grids in row-major order (default)
//! - Parallel Islands: Divide world into independent islands, process each in parallel
//!
//! # Pruning Strategies
//!
//! - Sequential: Prune grids in row-major order (default)
//! - Parallel Islands: Prune islands in parallel, with careful handling of overlap cells

use crate::base::SudokuBase;
use crate::error::Result;
use crate::generator::{
    Generator, GeneratorSettings, PruningGroupBehaviour, PruningOrder, PruningSettings,
    PruningTarget, SolutionSettings,
};
use crate::position::Position;
use crate::rng::{CrateRng, new_crate_rng_with_seed};
use crate::solver::backtracking;
use crate::world::dynamic::DynamicCellWorldActions;
use crate::world::{
    CellWorld, ValidatedWorldGridPosition, WorldGenerationResult, WorldGridDim, WorldGridPosition,
};
use anyhow::bail;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

/// Status of a grid tile during generation/pruning.
#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum GridTileStatus {
    /// Grid is empty, not yet generated.
    #[default]
    Empty,
    /// Grid is currently being filled.
    Filling,
    /// Grid has been filled with a valid solution.
    Filled,
    /// Grid is currently being pruned.
    Pruning,
    /// Grid has been pruned to create a puzzle.
    Pruned,
}

/// Progress information for world generation.
#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldGenerationProgress {
    /// Status of each grid tile, in row-major order.
    pub tile_statuses: Vec<GridTileStatus>,
    /// Current stage of generation.
    pub stage: WorldGenerationStage,
    /// Number of grids that have been filled.
    pub filled_count: usize,
    /// Total number of grids.
    pub total_count: usize,
    /// Number of backtracks so far.
    pub backtrack_count: u32,
}

/// Stage of world generation.
#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WorldGenerationStage {
    /// World is being generated (filling grids).
    Generation,
    /// World is being pruned.
    Pruning,
    /// Generation/pruning is complete.
    Complete,
}

/// Represents an island of grids that can be processed independently.
///
/// Islands are non-overlapping regions of the world grid. Grids within an
/// island may overlap with each other but not with grids in other islands.
#[derive(Debug, Clone)]
pub struct GridIsland {
    /// Grid positions belonging to this island, in the order they should be processed.
    pub positions: Vec<WorldGridPosition>,
}

impl GridIsland {
    /// Create a new island with the given grid positions.
    pub fn new(positions: Vec<WorldGridPosition>) -> Self {
        Self { positions }
    }

    /// Returns the number of grids in this island.
    pub fn len(&self) -> usize {
        self.positions.len()
    }

    /// Returns true if this island has no grids.
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
}

/// Strategy for segmenting the world into islands.
#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IslandStrategy {
    /// No segmentation - treat entire world as one island.
    #[default]
    Single,
    /// Segment into rows - each row is an island.
    Rows,
    /// Segment into columns - each column is an island.
    Columns,
    /// Checkerboard pattern - alternating grids form two islands.
    /// Note: This requires special handling of diagonal constraints.
    Checkerboard,
}

impl IslandStrategy {
    /// Segment the world grid into islands based on this strategy.
    #[must_use]
    pub fn segment(self, grid_dim: WorldGridDim) -> Vec<GridIsland> {
        match self {
            IslandStrategy::Single => {
                vec![GridIsland::new(grid_dim.all_positions().collect())]
            }
            IslandStrategy::Rows => grid_dim
                .all_rows()
                .map(|row_positions| GridIsland::new(row_positions.collect()))
                .collect(),
            IslandStrategy::Columns => grid_dim
                .all_columns()
                .map(|col_positions| GridIsland::new(col_positions.collect()))
                .collect(),
            IslandStrategy::Checkerboard => {
                let mut even_island = Vec::new();
                let mut odd_island = Vec::new();

                for pos in grid_dim.all_positions() {
                    if (pos.row + pos.column) % 2 == 0 {
                        even_island.push(pos);
                    } else {
                        odd_island.push(pos);
                    }
                }

                vec![
                    GridIsland::new(even_island),
                    GridIsland::new(odd_island),
                ]
            }
        }
    }
}

/// Parallel generation and pruning implementation for `CellWorld`.
impl<Base: SudokuBase> CellWorld<Base> {
    /// Generate a solved world using parallel island processing.
    ///
    /// This method segments the world into islands based on the given strategy
    /// and processes each island. Note that islands must be processed sequentially
    /// if they share overlap regions.
    #[cfg(feature = "parallel")]
    pub fn generate_solved_parallel(
        &mut self,
        seed: Option<u64>,
        strategy: IslandStrategy,
        on_progress: impl Fn(WorldGenerationProgress) + Sync,
    ) -> Result<WorldGenerationResult> {
        use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

        let grid_positions: Vec<ValidatedWorldGridPosition> =
            self.all_validated_grid_positions().collect();
        let total_count = grid_positions.len();

        let filled_count = AtomicUsize::new(0);
        let backtrack_count = AtomicU32::new(0);

        // For now, use row-based parallelism where each row is generated
        // after the previous row is complete (to satisfy diagonal constraints).
        //
        // A more sophisticated approach would analyze the dependency graph
        // and parallelize grids that don't have pending dependencies.

        let islands = strategy.segment(self.grid_dim);

        // Process islands sequentially for now (ensuring diagonal constraints are met)
        // In the future, we could analyze which islands are truly independent
        for island in islands {
            let mut local_backtrack_count = 0;

            for grid_position in island.positions {
                let validated_pos = grid_position.validate(self.grid_dim)?;

                // Report progress
                on_progress(WorldGenerationProgress {
                    tile_statuses: self.get_tile_statuses(),
                    stage: WorldGenerationStage::Generation,
                    filled_count: filled_count.load(Ordering::Relaxed),
                    total_count,
                    backtrack_count: backtrack_count.load(Ordering::Relaxed),
                });

                local_backtrack_count +=
                    self.generate_single_grid_with_backtracking(validated_pos, seed)?;

                filled_count.fetch_add(1, Ordering::Relaxed);
            }

            backtrack_count.fetch_add(local_backtrack_count, Ordering::Relaxed);
        }

        Ok(WorldGenerationResult {
            backtrack_count: backtrack_count.load(Ordering::Relaxed),
        })
    }

    /// Generate a single grid with backtracking support.
    fn generate_single_grid_with_backtracking(
        &mut self,
        grid_position: ValidatedWorldGridPosition,
        seed: Option<u64>,
    ) -> Result<u32> {
        let mut rng = new_crate_rng_with_seed(seed);

        // Get diagonal constraints
        let denylist = self.combined_diagonal_denylist(grid_position);
        let grid = self.to_grid_at_validated(grid_position);

        let mut solver = backtracking::Solver::builder(grid)
            .rng(CrateRng::from_rng(&mut rng))
            .candidates_filter(denylist)
            .build();

        if let Some(solution) = solver.next() {
            self.set_grid_at_validated(&solution, grid_position);
            // Get actual backtrack count from the solver
            Ok(solver.backtrack_count.try_into().unwrap_or(u32::MAX))
        } else {
            bail!(
                "Failed to generate grid at {:?}: no solution found",
                grid_position.get()
            )
        }
    }

    /// Prune the world using parallel island processing.
    #[cfg(feature = "parallel")]
    pub fn prune_parallel(
        &mut self,
        seed: Option<u64>,
        strategy: IslandStrategy,
        on_progress: impl Fn(WorldGenerationProgress) + Sync,
    ) -> Result<()> {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let grid_positions: Vec<ValidatedWorldGridPosition> =
            self.all_validated_grid_positions().collect();
        let total_count = grid_positions.len();
        let pruned_count = AtomicUsize::new(0);

        assert!(self.is_solved());

        // Get positions that are in the middle (not in overlap regions)
        let middle_positions: Vec<_> = Position::<Base>::all()
            .filter(|pos| {
                let (row, column) = pos.to_row_and_column();
                let (row, column) = (row.get(), column.get());
                let middle_axis_range =
                    self.overlap.get()..(Base::SIDE_LENGTH - self.overlap.get());
                middle_axis_range.contains(&row) && middle_axis_range.contains(&column)
            })
            .collect();

        let islands = strategy.segment(self.grid_dim);

        // Process each island
        // For parallel pruning, we can actually parallelize within an island
        // if we're careful about overlap cells
        for island in islands {
            let mut rng = new_crate_rng_with_seed(seed);

            for grid_position in island.positions {
                let validated_pos = grid_position.validate(self.grid_dim)?;

                // Report progress
                on_progress(WorldGenerationProgress {
                    tile_statuses: self.get_tile_statuses(),
                    stage: WorldGenerationStage::Pruning,
                    filled_count: total_count,
                    total_count,
                    backtrack_count: 0,
                });

                let grid = self.to_grid_at_validated(validated_pos);

                let pruned_grid = Generator::with_settings(GeneratorSettings {
                    prune: Some(PruningSettings {
                        set_all_direct_candidates: true,
                        order: PruningOrder::Positions {
                            positions: middle_positions
                                .iter()
                                .filter(|pos| !grid.get(**pos).has_fixed_value())
                                .copied()
                                .collect(),
                            behaviour: PruningGroupBehaviour::Exclusive,
                        },
                        target: PruningTarget::Minimal,
                        ..Default::default()
                    }),
                    solution: Some(SolutionSettings { values_grid: grid }),
                    seed: Some(rng.random()),
                })
                .generate()?;

                self.set_grid_at_validated(&pruned_grid, validated_pos);
                pruned_count.fetch_add(1, Ordering::Relaxed);
            }
        }

        Ok(())
    }

    /// Get the current status of all grid tiles.
    fn get_tile_statuses(&self) -> Vec<GridTileStatus> {
        self.all_validated_grid_positions()
            .map(|pos| {
                let grid = self.to_grid_at_validated(pos);
                if grid.is_solved() {
                    GridTileStatus::Filled
                } else if grid
                    .cells_view()
                    .iter()
                    .all(|cell| !cell.has_value() && !cell.has_candidates())
                {
                    GridTileStatus::Empty
                } else {
                    GridTileStatus::Filling
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::consts::*;

    #[test]
    fn test_island_strategy_single() {
        let grid_dim = WorldGridDim::new(3, 3).unwrap();
        let islands = IslandStrategy::Single.segment(grid_dim);

        assert_eq!(islands.len(), 1);
        assert_eq!(islands[0].len(), 9);
    }

    #[test]
    fn test_island_strategy_rows() {
        let grid_dim = WorldGridDim::new(3, 3).unwrap();
        let islands = IslandStrategy::Rows.segment(grid_dim);

        assert_eq!(islands.len(), 3);
        for island in &islands {
            assert_eq!(island.len(), 3);
        }
    }

    #[test]
    fn test_island_strategy_columns() {
        let grid_dim = WorldGridDim::new(3, 3).unwrap();
        let islands = IslandStrategy::Columns.segment(grid_dim);

        assert_eq!(islands.len(), 3);
        for island in &islands {
            assert_eq!(island.len(), 3);
        }
    }

    #[test]
    fn test_island_strategy_checkerboard() {
        let grid_dim = WorldGridDim::new(3, 3).unwrap();
        let islands = IslandStrategy::Checkerboard.segment(grid_dim);

        assert_eq!(islands.len(), 2);
        // 3x3 grid: 5 even positions, 4 odd positions
        assert_eq!(islands[0].len(), 5); // even
        assert_eq!(islands[1].len(), 4); // odd
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn test_generate_solved_parallel() {
        let grid_dim = WorldGridDim::new(2, 2).unwrap();
        let overlap = 1.try_into().unwrap();
        let mut world = CellWorld::<Base2>::new(grid_dim, overlap);

        let result = world.generate_solved_parallel(
            Some(42),
            IslandStrategy::Single,
            |_progress| {},
        );

        assert!(result.is_ok());
        assert!(world.is_solved());
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn test_prune_parallel() {
        let grid_dim = WorldGridDim::new(2, 2).unwrap();
        let overlap = 1.try_into().unwrap();
        let mut world = CellWorld::<Base2>::new(grid_dim, overlap);

        world.generate_solved_parallel(
            Some(42),
            IslandStrategy::Single,
            |_progress| {},
        ).unwrap();

        assert!(world.is_solved());

        let result = world.prune_parallel(
            Some(42),
            IslandStrategy::Single,
            |_progress| {},
        );

        assert!(result.is_ok());
        assert!(world.is_directly_consistent());
    }
}
