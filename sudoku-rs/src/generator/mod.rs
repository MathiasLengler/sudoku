use anyhow::{bail, format_err};
use itertools::Itertools;
use log::debug;
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use ts_rs::TS;

pub use dynamic_settings::*;

use crate::base::SudokuBase;
use crate::cell::Value;
use crate::error::Result;
use crate::grid::Grid;
use crate::position::Position;
use crate::solver::backtracking;
use crate::solver::backtracking::CandidatesVisitOrder;
use crate::solver::strategic::strategies::{Backtracking, DynamicStrategy};
use crate::CrateRng;

// TODO: strategic
//  target difficulty: sum of weighted strategy applications

/*
Ideas:
- pruning with backtracking
- symmetrical/pair-wise or other pattern-based pruning
- early abort/skip config
- from minimal insertion order
 */

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PruningTarget {
    #[default]
    Minimal,
    MinimalPlusClueCunt(u16),
    MaxEmptyCellCount(u16),
    MinClueCount(u16),
}

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PruningGroupBehaviour {
    /// Never prune cells in this group
    Retain,
    /// Only prune cells in this group
    Exclusive,
    /// Prune cells in this group first, then cells outside this group.
    First,
    /// Prune cells in this group last, then cells inside this group.
    Last,
}

// TODO: introduce PruningSegmentation vs PruningVisitOrder
// TODO: test
/// Define the order in which cells should be pruned.
#[derive(Debug, Default, Clone)]
pub enum PruningOrder<Base: SudokuBase> {
    /// Prune all cells in a random order
    #[default]
    Random,
    /// Handle cells defined by a list of positions differently.
    Positions {
        /// The positions of the cells
        positions: Vec<Position<Base>>,
        /// How to handle those cells
        /// If pruning is allowed, the visit order will be defined by the list.
        behaviour: PruningGroupBehaviour,
    },
    /// Handle the set of values defined in `settings.solution.values_grid` differently.
    SolutionValues {
        /// How to handle those cells
        /// If pruning is allowed, the visit order will be random.
        behaviour: PruningGroupBehaviour,
    },
}

/// How to prune/delete clues from a solved sudoku, while preserving the uniqueness of the solution.
#[derive(Debug, Clone)]
pub struct PruningSettings<Base: SudokuBase> {
    /// Whether to set all direct candidates after pruning is done.
    pub set_all_direct_candidates: bool,
    /// With which strategies the sudoku should remain solvable for.
    pub strategies: Vec<DynamicStrategy>,
    /// How much to prune the solution.
    pub target: PruningTarget,
    /// Adjust order in which cells are pruned.
    pub order: PruningOrder<Base>,
}

impl<Base: SudokuBase> Default for PruningSettings<Base> {
    fn default() -> Self {
        Self {
            strategies: vec![Backtracking.into()],
            set_all_direct_candidates: false,
            target: PruningTarget::default(),
            order: PruningOrder::default(),
        }
    }
}

/// Influence the generated solution.
#[derive(Debug, Clone)]
pub struct SolutionSettings<Base: SudokuBase> {
    /// Every value cell in this grid will be included in the solution of the generated grid.
    pub values_grid: Grid<Base>,
}

#[derive(Debug, Default, Clone)]
pub struct GeneratorSettings<Base: SudokuBase> {
    /// How to prune the solution.
    pub prune: Option<PruningSettings<Base>>,
    /// How to generate the solution.
    pub solution: Option<SolutionSettings<Base>>,
    /// A seed for randomness in the generation process.
    ///
    /// If `Some`, generation of sudokus will be deterministic.
    ///
    /// If `None`, a new random seed will be chosen for each generated sudoku,
    /// making the generation non-deterministic.
    ///
    /// Influence of randomness when generating a sudoku:
    /// - The generated solution of the sudoku.
    /// - The order in which cells are pruned.
    pub seed: Option<u64>,
}

mod dynamic_settings {
    use anyhow::ensure;

    use crate::error::Error;
    use crate::grid::dynamic::DynamicGrid;
    use crate::position::DynamicPosition;

    use super::*;

    #[cfg_attr(feature = "wasm", derive(TS), ts(export))]
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum DynamicPruningOrder {
        Random,
        Positions {
            positions: Vec<DynamicPosition>,
            behaviour: PruningGroupBehaviour,
        },
        SolutionValues {
            behaviour: PruningGroupBehaviour,
        },
    }

    impl<Base: SudokuBase> TryFrom<DynamicPruningOrder> for PruningOrder<Base> {
        type Error = Error;

        fn try_from(dynamic_pruning_order: DynamicPruningOrder) -> Result<Self> {
            Ok(match dynamic_pruning_order {
                DynamicPruningOrder::Random => Self::Random,
                DynamicPruningOrder::Positions {
                    positions,
                    behaviour,
                } => Self::Positions {
                    positions: positions
                        .into_iter()
                        .map(TryInto::try_into)
                        .collect::<Result<_>>()?,
                    behaviour,
                },
                DynamicPruningOrder::SolutionValues { behaviour } => {
                    Self::SolutionValues { behaviour }
                }
            })
        }
    }

    #[cfg_attr(feature = "wasm", derive(TS), ts(export))]
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DynamicPruningSettings {
        pub set_all_direct_candidates: bool,
        pub strategies: Vec<DynamicStrategy>,
        pub target: PruningTarget,
        pub order: DynamicPruningOrder,
    }

    impl<Base: SudokuBase> TryFrom<DynamicPruningSettings> for PruningSettings<Base> {
        type Error = Error;

        fn try_from(dynamic_pruning_settings: DynamicPruningSettings) -> Result<Self> {
            let DynamicPruningSettings {
                set_all_direct_candidates,
                strategies,
                target,
                order,
            } = dynamic_pruning_settings;

            Ok(Self {
                set_all_direct_candidates,
                strategies,
                target,
                order: order.try_into()?,
            })
        }
    }

    #[cfg_attr(feature = "wasm", derive(TS), ts(export))]
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DynamicSolutionSettings {
        values_grid: DynamicGrid,
    }

    impl<Base: SudokuBase> TryFrom<DynamicSolutionSettings> for SolutionSettings<Base> {
        type Error = Error;

        fn try_from(dynamic_solution_settings: DynamicSolutionSettings) -> Result<Self> {
            let DynamicSolutionSettings { values_grid } = dynamic_solution_settings;
            Ok(Self {
                values_grid: values_grid.try_into()?,
            })
        }
    }

    #[cfg_attr(feature = "wasm", derive(TS), ts(export))]
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DynamicGeneratorSettings {
        pub base: u8,
        pub prune: Option<DynamicPruningSettings>,
        pub solution: Option<DynamicSolutionSettings>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub seed: Option<u64>,
    }

    impl<Base: SudokuBase> TryFrom<DynamicGeneratorSettings> for GeneratorSettings<Base> {
        type Error = Error;

        fn try_from(dynamic_generator_settings: DynamicGeneratorSettings) -> Result<Self> {
            let DynamicGeneratorSettings {
                base,
                prune,
                solution,
                seed,
            } = dynamic_generator_settings;

            ensure!(base == Base::BASE);

            Ok(Self {
                prune: if let Some(prune) = prune {
                    Some(prune.try_into()?)
                } else {
                    None
                },
                solution: if let Some(solution) = solution {
                    Some(solution.try_into()?)
                } else {
                    None
                },
                seed,
            })
        }
    }
}

// #[cfg_attr(feature = "wasm", derive(TS), ts(export))]
// #[derive(Debug, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct DynamicGeneratorSettings {
//     pub base: u8,
//     pub target: GeneratorTarget,
//     pub strategies: Vec<DynamicStrategy>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     pub seed: Option<u64>,
// }

#[derive(Debug, Default)]
pub struct Generator<Base: SudokuBase> {
    settings: GeneratorSettings<Base>,
}

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorProgress {
    pruning_position_index: usize,
    pruning_position_count: usize,
    deleted_count: u16,
    is_position_required: bool,
}

impl<Base: SudokuBase> Generator<Base> {
    pub fn with_target(target: PruningTarget) -> Self {
        Self::with_pruning(PruningSettings {
            target,
            ..Default::default()
        })
    }

    pub fn with_pruning(pruning_settings: PruningSettings<Base>) -> Self {
        Self::with_settings(GeneratorSettings {
            prune: Some(pruning_settings),
            ..Default::default()
        })
    }

    pub fn with_settings(settings: GeneratorSettings<Base>) -> Self {
        Self { settings }
    }

    fn rng(&self) -> impl Rng {
        if let Some(seed) = self.settings.seed {
            CrateRng::seed_from_u64(seed)
        } else {
            CrateRng::from_rng(thread_rng()).unwrap()
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

        let solved_grid = self.solved_grid()?;

        let Some(prune_settings) = &self.settings.prune else {
            return Ok(solved_grid);
        };

        self.prune(solved_grid, prune_settings, on_progress)
    }

    fn solved_grid(&self) -> Result<Grid<Base>> {
        let mut grid = if let Some(solution_settings) = &self.settings.solution {
            solution_settings.values_grid.clone()
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
            if self.settings.solution.is_some() {
                format_err!("'solution.values_grid' has no solution")
            } else {
                panic!("Expected empty grid to have at least one solution")
            }
        })
    }

    fn prune(
        &self,
        solved_grid: Grid<Base>,
        prune_settings: &PruningSettings<Base>,
        on_progress: impl FnMut(GeneratorProgress) -> Result<()>,
    ) -> Result<Grid<Base>> {
        let mut pruned_grid = match prune_settings.target {
            PruningTarget::Minimal => {
                self.prune_from_minimal(solved_grid, 0, prune_settings, on_progress)?
            }
            PruningTarget::MinimalPlusClueCunt(clue_count) => {
                self.prune_from_minimal(solved_grid, clue_count, prune_settings, on_progress)?
            }
            PruningTarget::MaxEmptyCellCount(empty_cell_count) => {
                self.prune_from_filled(solved_grid, empty_cell_count, prune_settings, on_progress)?
            }
            PruningTarget::MinClueCount(clue_count) => self.prune_from_filled(
                solved_grid,
                Base::CELL_COUNT - clue_count,
                prune_settings,
                on_progress,
            )?,
        };

        pruned_grid.fix_all_values();

        if prune_settings.set_all_direct_candidates {
            pruned_grid.set_all_direct_candidates();
        }

        Ok(pruned_grid)
    }

    /// Try to delete a cell at specific position in a grid while preserving uniqueness of the grid solution.
    ///
    /// Returns the value of the deleted cell, if any.
    fn try_delete_cell_at_pos(
        &self,
        grid: &mut Grid<Base>,
        pos: Position<Base>,
        prune_settings: &PruningSettings<Base>,
    ) -> Option<Value<Base>> {
        let cell = grid.get(pos);

        let Some(value) = cell.value() else {
            panic!("Expected value at {pos}, instead got: {cell:?}")
        };

        grid.get_mut(pos).delete();

        // FIXME: optimize
        //  is_solvable_with_strategies => strategic::Solver
        //  has_unique_solution => backtracking_bitset::Solver
        match grid.is_solvable_with_strategies(prune_settings.strategies.clone()) {
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
    }

    fn shuffle_vec<T>(rng: &mut impl Rng, mut vec: Vec<T>) -> Vec<T> {
        vec.shuffle(rng);
        vec
    }

    fn pruning_positions(
        &self,
        prune_settings: &PruningSettings<Base>,
    ) -> Result<Vec<Position<Base>>> {
        Ok(match &prune_settings.order {
            PruningOrder::Random => {
                let mut rng = self.rng();
                Self::shuffle_vec(&mut rng, Grid::<Base>::all_positions().collect_vec())
            }
            PruningOrder::Positions {
                positions,
                behaviour,
            } => {
                let other_positions = {
                    let mut rng = self.rng();

                    let sorted_positions = {
                        let mut positions = positions.clone();
                        positions.sort_unstable();
                        positions
                    };
                    Self::shuffle_vec(
                        &mut rng,
                        Grid::<Base>::all_positions()
                            // Remove positions contained in sorted_positions
                            .filter(|pos| sorted_positions.binary_search(pos).is_err())
                            .collect_vec(),
                    )
                };

                match behaviour {
                    PruningGroupBehaviour::Retain => other_positions,
                    PruningGroupBehaviour::Exclusive => positions.clone(),
                    PruningGroupBehaviour::First => {
                        let mut pruning_positions = positions.clone();
                        pruning_positions.extend(other_positions);
                        pruning_positions
                    }
                    PruningGroupBehaviour::Last => {
                        let mut pruning_positions = other_positions;
                        pruning_positions.extend(positions);
                        pruning_positions
                    }
                }
            }
            PruningOrder::SolutionValues { behaviour } => {
                let Some(SolutionSettings { values_grid }) = &self.settings.solution else {
                    bail!(
                        "'PruningOrder::SolutionValues' requires 'settings.solution.values_grid' to be defined"
                    )
                };

                let mut rng = self.rng();
                match behaviour {
                    PruningGroupBehaviour::Retain => {
                        Self::shuffle_vec(&mut rng, values_grid.all_candidates_positions())
                    }
                    PruningGroupBehaviour::Exclusive => {
                        Self::shuffle_vec(&mut rng, values_grid.all_value_positions())
                    }
                    PruningGroupBehaviour::First => {
                        let mut pruning_positions =
                            Self::shuffle_vec(&mut rng, values_grid.all_value_positions());
                        pruning_positions.extend(Self::shuffle_vec(
                            &mut rng,
                            values_grid.all_candidates_positions(),
                        ));
                        pruning_positions
                    }
                    PruningGroupBehaviour::Last => {
                        let mut pruning_positions =
                            Self::shuffle_vec(&mut rng, values_grid.all_candidates_positions());
                        pruning_positions.extend(Self::shuffle_vec(
                            &mut rng,
                            values_grid.all_value_positions(),
                        ));
                        pruning_positions
                    }
                }
            }
        })
    }

    fn prune_from_filled(
        &self,
        mut grid: Grid<Base>,
        distance_from_filled: u16,
        prune_settings: &PruningSettings<Base>,
        mut on_progress: impl FnMut(GeneratorProgress) -> Result<()>,
    ) -> Result<Grid<Base>> {
        debug_assert!(grid.is_solved());

        if distance_from_filled == 0 {
            return Ok(grid);
        }

        let pruning_positions: Vec<_> = self.pruning_positions(prune_settings)?;
        let pruning_position_count = pruning_positions.len();

        let mut deleted_count = 0;
        for (i, pos) in pruning_positions.into_iter().enumerate() {
            let pruning_position_index = i + 1;

            if deleted_count >= distance_from_filled {
                break;
            }

            let is_position_required = if self
                .try_delete_cell_at_pos(&mut grid, pos, prune_settings)
                .is_some()
            {
                deleted_count += 1;
                debug!("Position {pruning_position_index}/{pruning_position_count} deleted, totaling {deleted_count}/{distance_from_filled} deleted positions");
                false
            } else {
                debug!(
                    "Position {pruning_position_index}/{pruning_position_count} is required for unique solution"
                );
                true
            };

            on_progress(GeneratorProgress {
                pruning_position_index,
                pruning_position_count,
                deleted_count,
                is_position_required,
            })?;
        }

        Ok(grid)
    }

    fn prune_from_minimal(
        &self,
        mut grid: Grid<Base>,
        distance_from_minimal: u16,
        prune_settings: &PruningSettings<Base>,
        mut on_progress: impl FnMut(GeneratorProgress) -> Result<()>,
    ) -> Result<Grid<Base>> {
        debug_assert!(grid.is_solved());

        // If the distance results in a filled sudoku, return it directly.
        if distance_from_minimal >= Grid::<Base>::cell_count() {
            return Ok(grid);
        }

        let pruning_positions: Vec<_> = self.pruning_positions(prune_settings)?;
        let pruning_position_count = pruning_positions.len();

        let mut deleted: Vec<(Position<Base>, Value<Base>)> = vec![];

        // Reduce grid to a minimal solution.
        for (i, pos) in pruning_positions.into_iter().enumerate() {
            let pruning_position_index = i + 1;

            let deleted_count = u16::try_from(deleted.len()).unwrap();

            let is_position_required = if let Some(deleted_value) =
                self.try_delete_cell_at_pos(&mut grid, pos, prune_settings)
            {
                debug!("Position {pruning_position_index}/{pruning_position_count} deleted, totaling {deleted_count} deleted positions");

                deleted.push((pos, deleted_value));
                false
            } else {
                debug!(
                    "Position {pruning_position_index}/{pruning_position_count} is required for unique solution"
                );
                true
            };

            on_progress(GeneratorProgress {
                pruning_position_index,
                pruning_position_count,
                deleted_count,
                is_position_required,
            })?;
        }

        // Restore the required amount of values, specified by distance.
        for (deleted_pos, deleted_value) in deleted.into_iter().take(distance_from_minimal.into()) {
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
    fn test_solved() {
        let grid = Generator::<Base2>::default().generate().unwrap();

        assert!(grid.is_solved());
    }

    mod prune {
        use super::*;

        mod target {
            use super::*;

            #[test]
            fn test_minimal() {
                let grid = Generator::<Base2>::with_target(PruningTarget::Minimal)
                    .generate()
                    .unwrap();

                assert!(grid.is_minimal());
            }

            #[test]
            fn test_minimal_plus_clue_cunt() {
                let grid = Generator::<Base2>::with_target(PruningTarget::MinimalPlusClueCunt(1))
                    .generate()
                    .unwrap();

                assert!(grid.has_unique_solution());

                assert!(grid.all_value_positions().into_iter().any(|value_pos| {
                    let mut grid = grid.clone();
                    grid.unfix_all_values();
                    grid.get_mut(value_pos).delete();
                    grid.is_minimal()
                }));
            }

            #[test]
            fn test_max_empty_cell_count() {
                let grid = Generator::<Base2>::with_target(PruningTarget::MaxEmptyCellCount(2))
                    .generate()
                    .unwrap();

                assert_eq!(grid.all_candidates_positions().len(), 2);

                assert!(grid.has_unique_solution());
            }
            #[test]
            fn test_min_clue_count() {
                let grid = Generator::<Base2>::with_target(PruningTarget::MinClueCount(14))
                    .generate()
                    .unwrap();

                assert_eq!(grid.all_candidates_positions().len(), 2);

                assert!(grid.has_unique_solution());
            }
        }

        #[test]
        fn test_strategies() {
            use crate::solver::strategic::strategies::*;

            fn generate(target: PruningTarget, strategies: Vec<DynamicStrategy>) -> Grid<Base3> {
                Generator::<Base3>::with_pruning(PruningSettings {
                    strategies,
                    target,
                    ..Default::default()
                })
                .generate()
                .unwrap()
            }

            let targets = vec![PruningTarget::Minimal, PruningTarget::MaxEmptyCellCount(20)];

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
    }

    mod seed {
        use super::*;

        #[test]
        fn test_seed() {
            let pruning_settings_list = vec![
                None,
                Some(PruningSettings {
                    target: PruningTarget::Minimal,
                    ..Default::default()
                }),
                Some(PruningSettings {
                    target: PruningTarget::MinClueCount(0),
                    ..Default::default()
                }),
            ];

            for pruning_settings in pruning_settings_list {
                let gen_1 = Generator::<Base3>::with_settings(GeneratorSettings {
                    seed: Some(1),
                    prune: pruning_settings.clone(),
                    ..Default::default()
                });
                assert_eq!(gen_1.generate().unwrap(), gen_1.generate().unwrap());
                let gen_2 = Generator::<Base3>::with_settings(GeneratorSettings {
                    seed: Some(2),
                    prune: pruning_settings,
                    ..Default::default()
                });
                assert_eq!(gen_2.generate().unwrap(), gen_2.generate().unwrap());
                assert_ne!(gen_1.generate().unwrap(), gen_2.generate().unwrap());
            }
        }

        #[test]
        fn test_no_seed() {
            let gen = Generator::<Base3>::default();

            let grid1 = gen.generate().unwrap();
            let grid2 = gen.generate().unwrap();

            assert!(grid1.is_solved());
            assert!(grid2.is_solved());
            assert_ne!(grid1, grid2);
        }
    }

    mod solution {
        use super::*;

        #[test]
        fn test_values_grid_solved() {
            let values_grid: Grid<Base2> = "
  4  3  │  0  0  
  2  1  │  0  0  
────────┼────────
  0  0  │  0  0  
  0  0  │  0  0  "
                .parse()
                .unwrap();

            let grid = Generator::<Base2>::with_settings(GeneratorSettings {
                solution: Some(SolutionSettings {
                    values_grid: values_grid.clone(),
                }),
                ..Default::default()
            })
            .generate()
            .unwrap();

            // Top left block is unchanged
            itertools::assert_equal(
                values_grid.block_cells(Coordinate::default()),
                grid.block_cells(Coordinate::default()),
            );

            assert!(grid.is_solved());
        }

        #[test]
        fn test_values_grid_minimal() {
            let values_grid: Grid<Base2> = "
  4  3  │  0  0  
  2  1  │  0  0  
────────┼────────
  0  0  │  0  0  
  0  0  │  0  0  "
                .parse()
                .unwrap();

            let grid = Generator::<Base2>::with_settings(GeneratorSettings {
                solution: Some(SolutionSettings {
                    values_grid: values_grid.clone(),
                }),
                prune: Some(PruningSettings {
                    target: PruningTarget::Minimal,
                    order: PruningOrder::SolutionValues {
                        behaviour: PruningGroupBehaviour::Retain,
                    },
                    ..Default::default()
                }),
                ..Default::default()
            })
            .generate()
            .unwrap();

            // Top left block is unchanged
            itertools::assert_equal(
                values_grid.block_cells(Coordinate::default()),
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
}
