//! World pruning settings with biome support.
//!
//! Biomes allow different regions of a `CellWorld` to be pruned with different settings,
//! enabling varied difficulty levels across the world.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::base::SudokuBase;
use crate::generator::{PruningOrder, PruningSettings, PruningTarget};
use crate::solver::strategic::strategies::selection::StrategySet;
use crate::solver::strategic::strategies::BruteForce;

use super::WorldGridPosition;

/// A unique identifier for a biome.
pub type BiomeId = u16;

/// A biome represents a region with specific pruning settings.
///
/// Multiple grid positions can belong to the same biome, allowing
/// areas of the world to share the same difficulty/pruning configuration.
#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Biome {
    /// Unique identifier for this biome.
    pub id: BiomeId,
    /// Whether to set all direct candidates after pruning is done.
    pub set_all_direct_candidates: bool,
    /// With which strategies the sudoku should remain solvable for.
    pub strategies: StrategySet,
    /// How much to prune the solution.
    pub target: PruningTarget,
}

impl Default for Biome {
    fn default() -> Self {
        Self {
            id: 0,
            set_all_direct_candidates: true,
            strategies: StrategySet::with_single(BruteForce.into()),
            target: PruningTarget::Minimal,
        }
    }
}

impl Biome {
    /// Creates a new biome with the given ID.
    pub fn new(id: BiomeId) -> Self {
        Self {
            id,
            ..Default::default()
        }
    }

    /// Creates a new biome with the given ID and pruning target.
    pub fn with_target(id: BiomeId, target: PruningTarget) -> Self {
        Self {
            id,
            target,
            ..Default::default()
        }
    }

    /// Creates a new biome with the given ID, target, and strategies.
    pub fn with_target_and_strategies(
        id: BiomeId,
        target: PruningTarget,
        strategies: StrategySet,
    ) -> Self {
        Self {
            id,
            strategies,
            target,
            ..Default::default()
        }
    }

    /// Converts this biome to pruning settings for a specific grid.
    pub(crate) fn to_pruning_settings<Base: SudokuBase>(
        &self,
        middle_positions_order: PruningOrder<Base>,
    ) -> PruningSettings<Base> {
        PruningSettings {
            set_all_direct_candidates: self.set_all_direct_candidates,
            strategies: self.strategies,
            target: self.target,
            order: middle_positions_order,
            ..Default::default()
        }
    }
}

/// Configuration for how a `CellWorld` should be pruned.
///
/// This allows specifying different pruning settings for different regions (biomes)
/// of the world, enabling varied difficulty across the puzzle.
#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldPruningSettings {
    /// Available biomes with their settings.
    /// If empty, default pruning settings are used for all grids.
    pub biomes: Vec<Biome>,

    /// Mapping from grid positions to biome IDs.
    /// Grid positions not in this map will use the default biome (ID 0).
    #[cfg_attr(
        feature = "wasm",
        ts(type = "Record<string, number>")
    )]
    pub grid_biome_assignments: HashMap<WorldGridPosition, BiomeId>,
}

impl WorldPruningSettings {
    /// Creates new world pruning settings with uniform difficulty (single biome for all grids).
    pub fn uniform(biome: Biome) -> Self {
        Self {
            biomes: vec![biome],
            grid_biome_assignments: HashMap::new(),
        }
    }

    /// Creates new world pruning settings with default minimal pruning.
    pub fn minimal() -> Self {
        Self::uniform(Biome::default())
    }

    /// Gets the biome for a specific grid position.
    ///
    /// Returns the assigned biome if found, otherwise returns the first biome
    /// (treated as default), or creates a default biome if none exist.
    pub fn get_biome_for_grid(&self, grid_position: WorldGridPosition) -> Biome {
        let biome_id = self
            .grid_biome_assignments
            .get(&grid_position)
            .copied()
            .unwrap_or(0);

        self.biomes
            .iter()
            .find(|b| b.id == biome_id)
            .cloned()
            .unwrap_or_else(|| {
                // If no biome with the ID exists, check for default (id 0)
                self.biomes
                    .first()
                    .cloned()
                    .unwrap_or_default()
            })
    }

    /// Assigns a biome ID to a grid position.
    pub fn assign_biome(&mut self, grid_position: WorldGridPosition, biome_id: BiomeId) {
        self.grid_biome_assignments.insert(grid_position, biome_id);
    }

    /// Adds a biome to the available biomes.
    pub fn add_biome(&mut self, biome: Biome) {
        // Remove existing biome with same ID if present
        self.biomes.retain(|b| b.id != biome.id);
        self.biomes.push(biome);
    }

    /// Returns true if all grids use the same biome settings.
    pub fn is_uniform(&self) -> bool {
        self.grid_biome_assignments.is_empty() || {
            let first_id = self.grid_biome_assignments.values().next();
            first_id.is_some() && self.grid_biome_assignments.values().all(|id| Some(id) == first_id)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biome_default() {
        let biome = Biome::default();
        assert_eq!(biome.id, 0);
        assert!(biome.set_all_direct_candidates);
        assert_eq!(biome.target, PruningTarget::Minimal);
    }

    #[test]
    fn test_biome_with_target() {
        let biome = Biome::with_target(1, PruningTarget::MinClueCount(30));
        assert_eq!(biome.id, 1);
        assert_eq!(biome.target, PruningTarget::MinClueCount(30));
    }

    #[test]
    fn test_world_pruning_settings_uniform() {
        let settings = WorldPruningSettings::uniform(Biome::with_target(0, PruningTarget::Minimal));
        assert!(settings.is_uniform());
        assert_eq!(settings.biomes.len(), 1);
    }

    #[test]
    fn test_world_pruning_settings_get_biome() {
        let mut settings = WorldPruningSettings::minimal();
        
        // Add a second biome
        let hard_biome = Biome::with_target(1, PruningTarget::MinClueCount(17));
        settings.add_biome(hard_biome);
        
        // Assign some grids to the hard biome
        let hard_position = WorldGridPosition::new(0, 0);
        settings.assign_biome(hard_position, 1);
        
        // Unassigned position should get default biome
        let default_position = WorldGridPosition::new(1, 1);
        let default_biome = settings.get_biome_for_grid(default_position);
        assert_eq!(default_biome.id, 0);
        
        // Assigned position should get the hard biome
        let assigned_biome = settings.get_biome_for_grid(hard_position);
        assert_eq!(assigned_biome.id, 1);
        assert_eq!(assigned_biome.target, PruningTarget::MinClueCount(17));
    }

    #[test]
    fn test_is_uniform() {
        let mut settings = WorldPruningSettings::minimal();
        assert!(settings.is_uniform());
        
        // Add assignments with the same biome ID
        settings.assign_biome(WorldGridPosition::new(0, 0), 0);
        settings.assign_biome(WorldGridPosition::new(0, 1), 0);
        assert!(settings.is_uniform());
        
        // Add a different biome ID
        settings.assign_biome(WorldGridPosition::new(1, 0), 1);
        assert!(!settings.is_uniform());
    }
}
